//! Le client : le processus du serveur, ses deux fils, et l'appariement des
//! requêtes.
//!
//! Le protocole veut qu'un client tienne un **dialogue**, et non une suite
//! d'appels : les réponses arrivent dans l'ordre où le serveur veut bien les
//! donner, et certaines lui sont adressées (il demande la configuration de
//! l'espace de travail, il crée des tâches de progression, il signale des
//! capacités). Un client qui n'y répond pas laisse le serveur attendre.
//!
//! D'où deux fils, et un seul point de passage :
//!
//! - un fil **d'écriture** prend des messages JSON et les cadre sur l'entrée du
//!   serveur — jamais l'interface, qui ne doit pas pouvoir se bloquer sur un
//!   tuyau plein ;
//! - un fil de **lecture** décadre la sortie du serveur, reconnaît les réponses
//!   par leur numéro, et pousse tout le reste vers l'application sous forme
//!   d'[`Event`].
//!
//! Ce qui vient du serveur est du texte que nous n'avons pas écrit : chaque
//! message est donc traité comme une entrée non fiable. Un cadre illisible fait
//! s'arrêter la lecture et prévenir l'application, jamais paniquer le client.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use super::messages::{CompletionItem, Diagnostic, Position, Range};

/// La commande du serveur, quand l'environnement ne dit rien d'autre.
pub const DEFAULT_COMMAND: &str = "tinymist";

/// Variable d'environnement qui remplace la commande : un chemin, ou un nom à
/// chercher dans le `PATH`.
pub const COMMAND_ENV: &str = "TINYMIST";

/// Variable d'environnement qui fait passer le journal du serveur sur notre
/// sortie d'erreur. Absente, il est jeté : le serveur est bavard, et ses lignes
/// se mêleraient au journal de l'application.
pub const VERBOSE_ENV: &str = "TYPST_WRITER_LSP_VERBOSE";

/// Un emplacement de définition, tel que le protocole le transmet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

impl<'de> serde::Deserialize<'de> for Location {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            uri: String,
            range: Range,
        }

        let raw = Raw::deserialize(deserializer)?;

        Ok(Self {
            uri: raw.uri,
            range: raw.range,
        })
    }
}

/// Ce que le serveur peut nous apprendre, du point de vue de l'application.
///
/// `Clone` parce qu'un message d'`iced` doit pouvoir l'être ; les événements
/// sont de petites choses, et leur clonage ne pèse rien.
#[derive(Debug, Clone)]
pub enum Event {
    /// Le serveur a répondu à l'initialisation : il est prêt.
    ///
    /// Les capacités sont transmises telles quelles : c'est à l'application de
    /// décider ce qu'elle en fait, et de ne pas demander ce qui n'est pas
    /// annoncé.
    Ready { capabilities: Value },

    /// Les diagnostics d'un document. Ils **remplacent** les précédents : c'est
    /// ce que veut dire `publishDiagnostics`.
    Diagnostics { uri: String, items: Vec<Diagnostic> },

    /// La réponse à une demande de complétion, portant l'étiquette fournie.
    Completions {
        token: u64,
        items: Vec<CompletionItem>,
    },

    /// La réponse à une demande de survol. `None` : rien à cet endroit.
    Hover { token: u64, text: Option<String> },

    /// La réponse à une demande de renommage : les fichiers à modifier.
    Rename {
        token: u64,
        changes: Vec<crate::lsp::messages::FileEdit>,
    },

    /// La réponse à une demande de définition.
    Definitions {
        token: u64,
        locations: Vec<Location>,
    },

    /// Une commande du serveur est passée, ou a échoué.
    CommandFinished { token: u64, error: Option<String> },

    /// Le serveur s'est arrêté, ou sa sortie est devenue illisible.
    ///
    /// L'application n'a rien à en faire d'autre que de continuer sans lui :
    /// l'éditeur et l'aperçu ne dépendent pas du serveur.
    Gone { reason: String },
}

/// Ce qu'on attend, une fois la requête envoyée.
///
/// C'est ce qui permet de reconnaître une réponse : le protocole ne répond
/// qu'avec un numéro, et un numéro ne dit pas ce qu'on avait demandé.
#[derive(Debug, Clone, Copy)]
enum Pending {
    Initialize,
    Completion(u64),
    Hover(u64),
    Definition(u64),
    Rename(u64),
    Command(u64),
}

/// Ce qui déclenche une demande de complétion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Demandée à la main — Ctrl+Espace.
    Invoked,

    /// Une touche vient d'être tapée, et c'est elle qui la déclenche.
    Character(char),
}

/// Une session avec le serveur de langage.
pub struct Session {
    events: Receiver<Event>,
    outgoing: Sender<Value>,
    pending: Arc<Mutex<HashMap<i64, Pending>>>,
    child: Child,

    /// L'adresse du document ouvert, telle que le protocole la demande.
    uri: String,

    /// Numéro de version du document : le protocole exige qu'il avance, et le
    /// serveur refuse une modification qui ne l'a pas fait avancer.
    version: i32,

    /// Le dernier URI a-t-il été annoncé par `didOpen` et reste-t-il ouvert ?
    document_open: bool,

    next_id: i64,
}

impl Session {
    /// Le serveur est-il installé et exécutable ?
    ///
    /// On le demande au binaire lui-même plutôt que de chercher un fichier : un
    /// fichier présent mais inexécutable, ou d'une version trop ancienne pour
    /// comprendre notre dialogue, ne doit pas être annoncé comme disponible.
    pub fn is_available() -> bool {
        Command::new(command())
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// La commande employée, telle que l'environnement la donne.
    pub fn command() -> String {
        command()
    }

    /// Le chemin d'une adresse du protocole. L'inverse de `uri_for`.
    ///
    /// Un serveur peut désigner un fichier que nous ne connaissons pas encore :
    /// c'est le cas d'une définition qui vit dans un autre chapitre. Le chemin
    /// est rendu tel quel, et c'est le projet qui décidera s'il lui appartient.
    pub fn path_from_uri(uri: &str) -> Option<PathBuf> {
        Some(PathBuf::from(percent_decode(uri.strip_prefix("file://")?)))
    }

    /// L'adresse du protocole pour un chemin.
    pub fn uri_for(path: &Path) -> String {
        let absolute = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

        format!("file://{}", percent_encode(&absolute.to_string_lossy()))
    }

    /// Démarre le serveur et ouvre le document.
    ///
    /// L'initialisation n'est pas attendue ici : elle est **asynchrone**, comme
    /// tout le reste, et l'application apprend que le serveur est prêt en
    /// recevant [`Event::Ready`]. Bloquer dessus ferait attendre l'interface
    /// plusieurs secondes au démarrage, pour une fonction dont elle peut très
    /// bien se passer.
    pub fn start(
        root: &Path,
        entry: Option<&Path>,
        path: &Path,
        text: &str,
    ) -> std::io::Result<Self> {
        let mut child = Command::new(command())
            .arg("lsp")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if verbose() {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .spawn()?;

        let stdin = child.stdin.take().expect("entrée du serveur");
        let stdout = child.stdout.take().expect("sortie du serveur");

        let (outgoing, outgoing_requests) = channel::<Value>();
        let (event_sender, events) = channel::<Event>();
        let pending = Arc::new(Mutex::new(HashMap::<i64, Pending>::new()));

        writer_loop(stdin, outgoing_requests);
        reader_loop(stdout, Arc::clone(&pending), event_sender, outgoing.clone());

        let uri = Self::uri_for(path);

        let mut session = Self {
            events,
            outgoing,
            pending,
            child,
            uri,
            version: 1,
            document_open: false,
            next_id: 1,
        };

        session.initialize(root, entry);
        session.did_open(text);

        Ok(session)
    }

    /// Les événements du serveur.
    pub fn events(&self) -> &Receiver<Event> {
        &self.events
    }

    /// Le document ouvert, tel que le protocole le désigne.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Le processus du serveur tourne-t-il encore ?
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Demande les complétions possibles à une position.
    ///
    /// `token` est à nous : le serveur le renvoie tel quel, ce qui permet de
    /// reconnaître une réponse devenue inutile — celle d'une frappe précédente,
    /// arrivée après la suivante.
    ///
    /// La **déclencheuse** compte : tinymist ne propose pas les mêmes choses
    /// selon qu'on lui demande à la main ou qu'une touche vient d'être tapée. La
    /// complétion des citations, en particulier, ne vient que si la demande dit
    /// `@` — demandée autrement, elle revient vide, sans que rien ne l'explique.
    pub fn completion(&mut self, position: Position, token: u64, trigger: Trigger) {
        let context = match trigger {
            Trigger::Invoked => json!({ "triggerKind": 1 }),
            Trigger::Character(character) => json!({
                "triggerKind": 2,
                "triggerCharacter": character.to_string(),
            }),
        };

        self.request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": self.uri },
                "position": position,
                "context": context,
            }),
            Pending::Completion(token),
        );
    }

    /// Demande le survol d'une position.
    pub fn hover(&mut self, position: Position, token: u64) {
        self.request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": self.uri },
                "position": position,
            }),
            Pending::Hover(token),
        );
    }

    /// Demande où est défini ce qui se trouve à une position.
    pub fn definition(&mut self, position: Position, token: u64) {
        self.request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": self.uri },
                "position": position,
            }),
            Pending::Definition(token),
        );
    }

    /// Demande comment renommer ce qui se trouve à une position.
    ///
    /// Le serveur répond par des modifications réparties sur les fichiers ;
    /// c'est à l'application de les **montrer** avant de les appliquer.
    pub fn rename(&mut self, position: Position, new_name: &str, token: u64) {
        self.request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": self.uri },
                "position": position,
                "newName": new_name,
            }),
            Pending::Rename(token),
        );
    }

    /// Ouvre un autre document auprès du serveur.
    ///
    /// Le protocole demande de **fermer** celui qu'on quittait : un serveur qui
    /// garde ouvert un document dont on ne lui parle plus continue de le
    /// compiler, de le diagnostiquer, et de consommer pour rien.
    pub fn open_document(&mut self, path: &Path, text: &str) {
        self.close_document();

        self.uri = Self::uri_for(path);
        self.version = 1;

        self.did_open(text);
    }

    /// Ferme le document actif sans arrêter la session de serveur.
    pub fn close_document(&mut self) {
        if !self.document_open {
            return;
        }

        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": self.uri } }),
        );
        self.document_open = false;
    }

    /// Demande au serveur d'exécuter une de ses commandes.
    ///
    /// C'est le canal par lequel tinymist expose ce qui n'est pas du protocole
    /// ordinaire : épingler le fichier principal d'un projet, exporter une page,
    /// demander les mesures d'un document. Les noms sont les siens ; il refuse
    /// ceux qu'il ne connaît pas, et le dit.
    pub fn execute_command(&mut self, command: &str, arguments: Value, token: u64) {
        self.request(
            "workspace/executeCommand",
            json!({ "command": command, "arguments": arguments }),
            Pending::Command(token),
        );
    }

    /// Signale que le document a changé.
    ///
    /// Le texte entier est renvoyé, sans intervalle : le protocole autorise un
    /// envoi complet, et pour un document de quelques dizaines de milliers
    /// d'octets c'est plus sûr que de calculer des différences — une différence
    /// calculée de travers désynchronise le serveur, et rien ne le dit.
    pub fn did_change(&mut self, text: &str) {
        self.version += 1;

        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": self.uri, "version": self.version },
                "contentChanges": [ { "text": text } ],
            }),
        );
    }

    /// Signale que le document a été écrit sur le disque.
    pub fn did_save(&mut self) {
        self.notify(
            "textDocument/didSave",
            json!({ "textDocument": { "uri": self.uri } }),
        );
    }

    /// Termine le serveur, proprement.
    ///
    /// Le protocole demande de lui dire au revoir, puis de lui demander de
    /// sortir. On n'attend pas longtemps : un serveur qui ne répond pas à cela
    /// n'a pas à retenir l'application, et il est tué.
    pub fn shutdown(&mut self) {
        let _ = self.outgoing.send(json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "shutdown",
            "params": null,
        }));

        self.next_id += 1;

        let _ = self.outgoing.send(json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null,
        }));

        for _ in 0..20 {
            if !self.is_alive() {
                return;
            }

            std::thread::sleep(Duration::from_millis(25));
        }

        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// La salutation initiale, telle que le protocole la décrit.
    fn initialize(&mut self, root: &Path, entry: Option<&Path>) {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "clientInfo": { "name": "Typst Writer", "version": env!("CARGO_PKG_VERSION") },
                "rootUri": format!("file://{}", percent_encode(&root.to_string_lossy())),
                "workspaceFolders": [ {
                    "uri": format!("file://{}", percent_encode(&root.to_string_lossy())),
                    "name": "Projet",
                } ],
                // **Le point d'entrée du projet.** Sans lui, tinymist résout le
                // projet « fichier par fichier » : il traite le document ouvert
                // comme un document isolé, ignore l'en-tête de l'assemblage et
                // sa bibliographie. Un chapitre paraît alors plein d'erreurs qui
                // n'en sont pas — une macro définie dans l'assemblage serait
                // « inconnue » — et les citations ne se complètent pas.
                //
                // C'est **le même point d'entrée que celui de l'aperçu** : les
                // diagnostics du serveur et notre rendu parlent donc du même
                // document, et non de deux documents qui se ressemblent.
                "initializationOptions": {
                    "projectResolution": "lockDatabase",
                    "entry": entry.map(|path| path.to_string_lossy().to_string()),
                },
                "capabilities": {
                    "textDocument": {
                        "synchronization": { "dynamicRegistration": false },
                        "completion": {
                            "contextSupport": true,
                            "completionItem": {
                                // Pas d'expansion de gabarits : nous insérons du
                                // texte, et un gabarit inséré tel quel laisserait
                                // ses marqueurs dans le document.
                                "snippetSupport": false,
                                "documentationFormat": [ "markdown", "plaintext" ],
                                "insertReplaceSupport": true,
                                "labelDetailsSupport": false,
                            },
                        },
                        "hover": { "contentFormat": [ "markdown", "plaintext" ], "dynamicRegistration": false },
                        "definition": { "linkSupport": false },
                        "publishDiagnostics": { "relatedInformation": true, "versionSupport": true },
                    },
                    "workspace": {
                        "workspaceFolders": true,
                        "configuration": true,
                    },
                },
            },
        });

        if let Ok(mut pending) = self.pending.lock() {
            pending.insert(self.next_id, Pending::Initialize);
        }

        let _ = self.outgoing.send(request);

        self.next_id += 1;

        self.notify("initialized", json!({}));
    }

    /// Ouvre le document auprès du serveur.
    fn did_open(&mut self, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": self.uri,
                    // L'identifiant de langage est celui que le serveur attend :
                    // il ne le devine pas de l'extension.
                    "languageId": "typst",
                    "version": self.version,
                    "text": text,
                },
            }),
        );
        self.document_open = true;
    }

    fn notify(&self, method: &str, params: Value) {
        let _ = self.outgoing.send(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }));
    }

    fn request(&mut self, method: &str, params: Value, expectation: Pending) {
        let id = self.next_id;

        self.next_id += 1;

        if let Ok(mut pending) = self.pending.lock() {
            pending.insert(id, expectation);
        }

        let _ = self.outgoing.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }));
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Écrit les messages sur l'entrée du serveur, cadrés comme le protocole le veut.
fn writer_loop(mut stdin: std::process::ChildStdin, outgoing: Receiver<Value>) {
    std::thread::spawn(move || {
        while let Ok(message) = outgoing.recv() {
            let body = message.to_string();

            if write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).is_err() {
                break;
            }

            if stdin.flush().is_err() {
                break;
            }
        }
    });
}

/// Lit la sortie du serveur et la distribue.
fn reader_loop(
    stdout: ChildStdout,
    pending: Arc<Mutex<HashMap<i64, Pending>>>,
    events: Sender<Event>,
    responses: Sender<Value>,
) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);

        while let Some(message) = read_message(&mut reader) {
            if !dispatch(message, &pending, &events, &responses) {
                break;
            }
        }

        let _ = events.send(Event::Gone {
            reason: "le serveur a fermé sa sortie".to_string(),
        });
    });
}

/// Décadre un message du protocole : en-têtes, puis charge utile.
///
/// `None` signifie la fin de la sortie, ou un cadre que nous ne savons pas lire
/// — dans les deux cas, il n'y a plus de dialogue possible.
fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    let mut length: Option<usize> = None;

    loop {
        let mut header = String::new();

        if reader.read_line(&mut header).ok()? == 0 {
            return None;
        }

        let header = header.trim_end();

        if header.is_empty() {
            break;
        }

        if let Some(value) = header.strip_prefix("Content-Length:") {
            length = value.trim().parse().ok();
        }
    }

    let length = length?;
    let mut body = vec![0u8; length];

    reader.read_exact(&mut body).ok()?;

    serde_json::from_slice(&body).ok()
}

/// Traite un message. Renvoie `false` s'il faut cesser de lire.
fn dispatch(
    message: Value,
    pending: &Arc<Mutex<HashMap<i64, Pending>>>,
    events: &Sender<Event>,
    responses: &Sender<Value>,
) -> bool {
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id");

    match (method, id) {
        // Le serveur nous demande quelque chose. Sans réponse, il attend — et
        // certaines de ses demandes conditionnent la suite du dialogue, comme la
        // configuration de l'espace de travail.
        (Some(method), Some(id)) => {
            let result = match method {
                // Aucune de nos réglages n'est configurable pour l'instant : une
                // valeur nulle par entrée demandée est la réponse honnête.
                "workspace/configuration" => {
                    let count = message
                        .get("params")
                        .and_then(|params| params.get("items"))
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);

                    Value::Array(vec![Value::Null; count])
                }
                _ => Value::Null,
            };

            // La réponse repart par le **même** fil d'écriture que nos
            // requêtes : l'entrée du serveur n'a qu'un écrivain, sans quoi deux
            // fils entrelaceraient leurs cadres.
            let _ = responses.send(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result,
            }));

            true
        }

        // Une notification : personne n'attend de réponse.
        (Some("textDocument/publishDiagnostics"), None) => {
            let params = message.get("params").cloned().unwrap_or(Value::Null);

            let uri = params
                .get("uri")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            let items = params
                .get("diagnostics")
                .cloned()
                .and_then(|value| serde_json::from_value::<Vec<Diagnostic>>(value).ok())
                .unwrap_or_default();

            let _ = events.send(Event::Diagnostics { uri, items });

            true
        }

        (Some(_), None) => true,

        // Une réponse : à nous de la reconnaître par son numéro.
        (None, Some(id)) => {
            let id = id.as_i64().unwrap_or_default();

            let expectation = pending
                .lock()
                .ok()
                .and_then(|mut pending| pending.remove(&id));

            let Some(expectation) = expectation else {
                return true;
            };

            // Une réponse porte soit un résultat, soit une erreur. Traiter
            // l'erreur comme un résultat vide ferait passer un échec pour une
            // réponse : c'est exactement ainsi qu'une fonctionnalité semble
            // branchée sans l'être.
            let failure = message
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .map(str::to_string);

            if let Some(reason) = &failure {
                eprintln!("Le serveur a refusé une demande : {reason}");
            }

            let result = message.get("result").cloned().unwrap_or(Value::Null);

            match expectation {
                Pending::Initialize => {
                    let capabilities = result.get("capabilities").cloned().unwrap_or(Value::Null);

                    let _ = events.send(Event::Ready { capabilities });
                }

                Pending::Completion(token) => {
                    let items = completion_items(&result);

                    let _ = events.send(Event::Completions { token, items });
                }

                Pending::Hover(token) => {
                    let text = hover_text(&result);

                    let _ = events.send(Event::Hover { token, text });
                }

                Pending::Definition(token) => {
                    let locations = definition_locations(&result);

                    let _ = events.send(Event::Definitions { token, locations });
                }

                Pending::Rename(token) => {
                    let changes = workspace_edits(&result);

                    let _ = events.send(Event::Rename { token, changes });
                }

                Pending::Command(token) => {
                    let _ = events.send(Event::CommandFinished {
                        token,
                        error: failure,
                    });
                }
            }

            true
        }

        (None, None) => true,
    }
}

/// Les éléments de complétion d'une réponse, quel que soit leur emballage.
///
/// Le protocole autorise une liste, mais aussi une liste accompagnée d'un
/// marqueur d'incomplétude : les deux formes sont acceptées.
fn completion_items(result: &Value) -> Vec<CompletionItem> {
    let items = result
        .get("items")
        .or_else(|| result.as_array().map(|_| result))
        .cloned()
        .unwrap_or(Value::Null);

    serde_json::from_value(items).unwrap_or_default()
}

/// Le texte d'un survol, quelle que soit la forme que le serveur lui donne.
fn hover_text(result: &Value) -> Option<String> {
    if result.is_null() {
        return None;
    }

    let contents = result.get("contents")?;

    let text = match contents {
        Value::String(text) => text.clone(),
        Value::Object(object) => object
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| match part {
                Value::String(text) => Some(text.clone()),
                Value::Object(object) => object
                    .get("value")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };

    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Les modifications d'un `WorkspaceEdit`, réparties par fichier.
///
/// Le protocole en donne deux formes — `changes`, une carte uri → remplacements,
/// et `documentChanges`, une liste d'éditions de document — et tinymist use de
/// la seconde. Les deux sont lues : un renommage ne doit pas dépendre de la
/// forme choisie par le serveur.
fn workspace_edits(result: &Value) -> Vec<crate::lsp::messages::FileEdit> {
    let mut out = Vec::new();

    if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            let Some(path) = Session::path_from_uri(uri) else {
                continue;
            };

            let edits =
                serde_json::from_value::<Vec<crate::lsp::messages::TextEdit>>(edits.clone())
                    .unwrap_or_default();

            out.push(crate::lsp::messages::FileEdit { path, edits });
        }
    }

    if let Some(changes) = result.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            let Some(uri) = change
                .get("textDocument")
                .and_then(|d| d.get("uri"))
                .and_then(Value::as_str)
            else {
                continue;
            };

            let Some(path) = Session::path_from_uri(uri) else {
                continue;
            };

            let edits = change
                .get("edits")
                .cloned()
                .and_then(|edits| serde_json::from_value(edits).ok())
                .unwrap_or_default();

            out.push(crate::lsp::messages::FileEdit { path, edits });
        }
    }

    out
}

/// Les emplacements d'une définition, quel que soit leur emballage.
fn definition_locations(result: &Value) -> Vec<Location> {
    let values = match result {
        Value::Array(items) => items.clone(),
        Value::Object(_) => vec![result.clone()],
        _ => Vec::new(),
    };

    values
        .into_iter()
        .filter_map(|value| {
            // Une définition peut être un emplacement, ou un lien vers un
            // emplacement : on ne garde que ce qui a une plage.
            serde_json::from_value::<Location>(value.clone())
                .ok()
                .or_else(|| {
                    let target = value.get("targetUri")?.clone();
                    let range = value.get("targetRange")?.clone();

                    let uri = target.as_str()?.to_string();
                    let range = serde_json::from_value::<Range>(range).ok()?;

                    Some(Location { uri, range })
                })
        })
        .collect()
}

/// L'encodage d'un chemin en adresse du protocole.
///
/// Un espace ou un accent dans un nom de dossier ferait autre chose qu'une
/// adresse : c'est le genre de détail qui ne se voit que chez l'utilisateur.
fn percent_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());

    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

/// Le texte d'une chaîne pourcentage-encodée. L'inverse de `percent_encode`.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2]))
        {
            decoded.push(high * 16 + low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

/// La valeur d'un chiffre hexadécimal.
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// La commande du serveur, d'après l'environnement.
fn command() -> String {
    std::env::var(COMMAND_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_COMMAND.to_string())
}

/// Le journal du serveur doit-il passer sur notre sortie d'erreur ?
fn verbose() -> bool {
    std::env::var_os(VERBOSE_ENV).is_some()
}

/// Le dossier de travail d'un document : la racine du projet.
pub fn root_of(path: &Path) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une adresse se relit, accents compris.
    #[test]
    fn a_path_is_read_back_from_its_address() {
        assert_eq!(
            Session::path_from_uri("file:///tmp/caf%C3%A9%20du%20matin.typ"),
            Some(PathBuf::from("/tmp/café du matin.typ"))
        );

        // Ce qui n'est pas une adresse de fichier n'en donne pas le chemin.
        assert_eq!(Session::path_from_uri("https://exemple.test/a.typ"), None);

        // Et l'aller-retour rend exactement le chemin canonique.
        let path = Path::new("Cargo.toml");

        assert_eq!(
            Session::path_from_uri(&Session::uri_for(path)),
            Some(fs::canonicalize(path).unwrap())
        );
    }
    use std::fs;

    /// Un projet minimal, choisi pour ce qu'on veut éprouver : une citation avec
    /// sa bibliographie, une commande à compléter, et un fichier à diagnostiquer.
    fn project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("typst-writer-lsp-{name}"));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).expect("dossier du projet");

        fs::write(
            dir.join("main.typ"),
            "#set page(paper: \"a4\")\n#set text(lang: \"fr\")\n#include \"chapitres/01-intro.typ\"\n#bibliography(\"biblio.bib\")\n",
        )
        .expect("assemblage");

        fs::write(
            dir.join("chapitres/01-intro.typ"),
            "= Introduction\n\nUn texte avec une citation @clef.\n\n#fig\n",
        )
        .expect("chapitre");

        fs::write(
            dir.join("biblio.bib"),
            "@book{clef,\n  title = {Un titre},\n  author = {Une autrice},\n  year = {2026},\n}\n",
        )
        .expect("bibliographie");

        dir
    }

    /// Où mènent les endroits : une définition, et un survol.
    ///
    /// C'est l'épreuve du vrai serveur pour la navigation — celle qui dit si un
    /// saut a quelque part où aller, et si le chemin rendu est bien un chapitre
    /// du projet.
    ///
    ///     cargo test --release -- --ignored tinymist_gives_locations --nocapture
    #[test]
    #[ignore]
    fn tinymist_gives_locations() {
        if !Session::is_available() {
            println!("tinymist absent : rien à éprouver");
            return;
        }

        let dir = project("endroits");
        let first = dir.join("chapitres/01-intro.typ");
        let second = dir.join("chapitres/02-suite.typ");

        // Les deux chapitres sont inclus par l'assemblage : c'est le seul moyen
        // que la référence puisse mener ailleurs que dans son propre fichier.
        fs::write(
            dir.join("main.typ"),
            "#set page(paper: \"a4\")\n#set text(lang: \"fr\")\n#include \"chapitres/01-intro.typ\"\n#include \"chapitres/02-suite.typ\"\n#bibliography(\"biblio.bib\")\n",
        )
        .expect("assemblage");

        fs::write(&first, "= Introduction <intro>\n\nLe début.\n").expect("premier chapitre");

        fs::write(
            &second,
            "= Suite\n\nVoir @intro pour commencer.\n\n#let repere = 1\n\nLe repere vaut #repere.\n",
        )
        .expect("second chapitre");

        let text = fs::read_to_string(&second).expect("chapitre lisible");
        let entry = dir.join("main.typ");

        let mut session =
            Session::start(&dir, Some(&entry), &second, &text).expect("démarrage du serveur");

        wait_for(
            &session,
            |event| matches!(event, Event::Ready { .. }),
            Duration::from_secs(40),
        );

        // 1. Une référence d'un chapitre vers un autre.
        let offset = text.find("@intro").expect("le chapitre cite") + 1;
        let position = crate::lsp::messages::position_of(&text, offset);

        session.definition(position, 21);

        match wait_for(
            &session,
            |event| matches!(event, Event::Definitions { token, .. } if *token == 21),
            Duration::from_secs(30),
        ) {
            Some(Event::Definitions { locations, .. }) => {
                println!("définition de `@intro` : {} endroit(s)", locations.len());

                for location in &locations {
                    let path = Session::path_from_uri(&location.uri);
                    let known = path
                        .as_ref()
                        .is_some_and(|path| crate::project::same_file(path, &first));

                    println!(
                        "  ligne {} | chapitre du projet : {known} | {}",
                        location.range.start.line + 1,
                        location.uri
                    );
                }
            }
            _ => println!("définition de `@intro` : AUCUNE"),
        }

        // 2. Le survol d'une variable.
        let offset = text.rfind("#repere").expect("le chapitre utilise") + 3;
        let position = crate::lsp::messages::position_of(&text, offset);

        session.hover(position, 22);

        match wait_for(
            &session,
            |event| matches!(event, Event::Hover { token, .. } if *token == 22),
            Duration::from_secs(30),
        ) {
            Some(Event::Hover { text, .. }) => println!(
                "survol de `#repere` : {:?}",
                text.map(|text| text.lines().take(4).collect::<Vec<_>>().join(" / "))
            ),
            _ => println!("survol de `#repere` : AUCUNE RÉPONSE"),
        }

        // 3. Sa définition, dans le même chapitre.
        session.definition(position, 23);

        match wait_for(
            &session,
            |event| matches!(event, Event::Definitions { token, .. } if *token == 23),
            Duration::from_secs(30),
        ) {
            Some(Event::Definitions { locations, .. }) => println!(
                "définition de `#repere` : {} endroit(s), ligne {:?}",
                locations.len(),
                locations
                    .first()
                    .map(|location| location.range.start.line + 1)
            ),
            _ => println!("définition de `#repere` : AUCUNE"),
        }
    }

    /// Attend un événement, en écartant les autres.
    ///
    /// Le serveur parle quand il veut : il envoie ses diagnostics, ses
    /// progressions et ses journaux sans qu'on les demande. Un test qui prend le
    /// premier message venu éprouve l'ordre d'arrivée, pas ce qu'il cherche.
    fn wait_for(
        session: &Session,
        wanted: impl Fn(&Event) -> bool,
        timeout: Duration,
    ) -> Option<Event> {
        let deadline = std::time::Instant::now() + timeout;

        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());

            if left.is_zero() {
                return None;
            }

            match session.events().recv_timeout(left) {
                Ok(event) if wanted(&event) => return Some(event),
                Ok(_) => continue,
                Err(_) => return None,
            }
        }
    }

    /// Le dialogue entier, contre le vrai serveur : il se présente, il signale
    /// ce qui ne va pas, il complète une commande — et il complète une citation
    /// depuis la bibliographie du projet.
    ///
    ///     cargo test --release -- --ignored tinymist_answers --nocapture
    #[test]
    #[ignore]
    fn tinymist_answers_a_real_project() {
        if !Session::is_available() {
            println!("tinymist absent : rien à éprouver");
            return;
        }

        let dir = project("dialogue");
        let chapter = dir.join("chapitres/01-intro.typ");
        let text = fs::read_to_string(&chapter).expect("chapitre lisible");

        // Le point d'entrée est l'assemblage : c'est lui qui porte la
        // bibliographie, et c'est lui que l'aperçu compile.
        let entry = dir.join("main.typ");

        // Tinymist ne sait pas quel fichier est l'entrée d'un projet sans le
        // lui dire : sa documentation est explicite — la « résolution par
        // verrou » s'appuie sur un fichier `tinymist.lock`, écrit par la ligne
        // de commande, les commandes LSP correspondantes n'étant *pas*
        // implémentées. C'est donc à l'application de créer ce verrou.
        let lock = std::process::Command::new(Session::command())
            .args(["compile", "--save-lock", "main.typ"])
            .current_dir(&dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("compilation avec verrou");

        // C'est le **fichier** qui compte, et non le code de sortie : tinymist
        // écrit son verrou même quand le document contient une erreur — et le
        // projet de ce test en contient une exprès, puisqu'il sert à éprouver la
        // complétion après un nom de commande incomplet.
        assert!(
            dir.join("tinymist.lock").is_file(),
            "le verrou du projet doit être écrit (compilation sortie en {lock:?})"
        );

        let mut session =
            Session::start(&dir, Some(&entry), &chapter, &text).expect("démarrage du serveur");

        // 1. Il se présente.
        match wait_for(
            &session,
            |e| matches!(e, Event::Ready { .. }),
            Duration::from_secs(40),
        ) {
            Some(Event::Ready { capabilities }) => {
                println!(
                    "initialisation : reçue | complétion annoncée : {}",
                    capabilities.get("completionProvider").is_some()
                );
            }
            _ => println!("initialisation : AUCUNE"),
        }

        // 2. Les diagnostics arrivent d'eux-mêmes.
        match wait_for(
            &session,
            |e| matches!(e, Event::Diagnostics { .. }),
            Duration::from_secs(30),
        ) {
            Some(Event::Diagnostics { items, .. }) => {
                println!("diagnostics : {}", items.len());

                for item in items.iter().take(3) {
                    println!(
                        "  {:?} ligne {} : {}",
                        item.severity,
                        item.range.start.line + 1,
                        item.message.trim()
                    );
                }
            }
            _ => println!("diagnostics : aucun"),
        }

        // 3. Une commande, juste après `#fig`.
        let offset = text.find("#fig").expect("le chapitre contient #fig") + 4;
        let position = crate::lsp::messages::position_of(&text, offset);

        session.completion(position, 7, Trigger::Character('#'));

        match wait_for(
            &session,
            |e| matches!(e, Event::Completions { token, .. } if *token == 7),
            Duration::from_secs(30),
        ) {
            Some(Event::Completions { items, .. }) => {
                println!("complétion après `#fig` : {} éléments", items.len());

                for item in items
                    .iter()
                    .filter(|item| item.label.contains("figure"))
                    .take(3)
                {
                    println!(
                        "  {} ({:?}) insère {:?} en remplacement de {:?}",
                        item.label,
                        item.kind,
                        item.insertion(),
                        item.replacement()
                    );
                }
            }
            _ => println!("complétion après `#fig` : AUCUNE"),
        }

        // 3 bis. Le fichier principal du projet est épinglé sur l'assemblage :
        // c'est ce qui doit donner au serveur la vue du **projet**, et non d'un
        // fichier isolé — donc l'en-tête, et la bibliographie.
        session.execute_command("tinymist.pinMainToCurrent", serde_json::json!([]), 10);

        match wait_for(
            &session,
            |e| matches!(e, Event::CommandFinished { token, .. } if *token == 10),
            Duration::from_secs(30),
        ) {
            Some(Event::CommandFinished { error, .. }) => {
                println!("épinglage : {}", error.as_deref().unwrap_or("accepté"))
            }
            _ => println!("épinglage : AUCUNE RÉPONSE"),
        }

        // Le serveur a besoin d'un instant pour reprendre la compilation sous
        // sa nouvelle entrée ; on lui laisse le temps de publier.
        std::thread::sleep(Duration::from_secs(3));

        // 4. Une citation, juste après `@c`, **déclarée comme telle**.
        let offset = text.find("@clef").expect("le chapitre cite") + 2;
        let position = crate::lsp::messages::position_of(&text, offset);

        session.completion(position, 8, Trigger::Character('@'));

        match wait_for(
            &session,
            |e| matches!(e, Event::Completions { token, .. } if *token == 8),
            Duration::from_secs(30),
        ) {
            Some(Event::Completions { items, .. }) => {
                println!("complétion après `@c` : {} éléments", items.len());

                for item in items.iter().take(5) {
                    println!("  {} → insère {:?}", item.label, item.insertion());
                }
            }
            _ => println!("complétion après `@c` : AUCUNE"),
        }

        // 5. Le survol d'une citation : c'est là qu'il a quelque chose à dire.
        let offset = text.find("@clef").expect("le chapitre cite") + 1;
        session.hover(crate::lsp::messages::position_of(&text, offset), 9);

        match wait_for(
            &session,
            |e| matches!(e, Event::Hover { token, .. } if *token == 9),
            Duration::from_secs(30),
        ) {
            Some(Event::Hover { text, .. }) => {
                println!(
                    "survol : {:?}",
                    text.as_deref()
                        .unwrap_or("rien")
                        .chars()
                        .take(90)
                        .collect::<String>()
                )
            }
            _ => println!("survol : AUCUNE"),
        }

        session.shutdown();
    }

    /// Un aller-retour d'adresses : un dossier accentué, un espace.
    #[test]
    fn a_path_becomes_an_address() {
        assert_eq!(
            percent_encode("/home/mat/un projet"),
            "/home/mat/un%20projet"
        );
        assert_eq!(percent_encode("/home/mat/café"), "/home/mat/caf%C3%A9");
    }

    /// Une réponse sans `items` emballés, et une réponse emballée.
    #[test]
    fn completion_items_accepts_both_shapes() {
        let plain = serde_json::json!([ { "label": "figure" } ]);
        let wrapped =
            serde_json::json!({ "isIncomplete": false, "items": [ { "label": "figure" } ] });

        assert_eq!(completion_items(&plain).len(), 1);
        assert_eq!(completion_items(&wrapped).len(), 1);
        assert!(completion_items(&serde_json::Value::Null).is_empty());
    }

    /// Le texte inséré vient de la modification, quand le serveur en propose une.
    #[test]
    fn an_item_uses_its_text_edit_first() {
        let item: CompletionItem = serde_json::from_value(serde_json::json!({
            "label": "#figure(…)",
            "insertText": "figure",
            "textEdit": {
                "range": { "start": { "line": 4, "character": 0 }, "end": { "line": 4, "character": 4 } },
                "newText": "figure(",
            },
        }))
        .expect("élément");

        assert_eq!(item.insertion(), Some("figure("));
        assert_eq!(
            item.replacement().map(|range| range.start.character),
            Some(0)
        );
    }
}
