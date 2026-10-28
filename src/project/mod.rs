//! Gestion du projet : les chapitres, la bibliographie, la session de travail.
//!
//! L'application travaille toujours sur des fichiers locaux : aucun dialogue
//! « où sauvegarder ? » n'intervient pendant l'édition — il ne sert qu'à
//! l'export.
//!
//! Un travail se présente comme un **projet** : un dossier dans
//! `~/.typst-writer/projets/`, avec un fichier par chapitre, sa bibliographie
//! et son assemblage. Voir [`model`] pour la structure exacte.

pub mod model;

pub use model::*;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Dossier de travail de l'application : les projets y vivent.
pub fn work_dir() -> PathBuf {
    let dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".typst-writer"),
        Err(_) => PathBuf::from(".typst-writer"),
    };

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Le dossier des suppressions, à côté des projets.
///
/// Une suppression n'est pas une destruction : le fichier y va, et le
/// chemin est rendu pour se raviser. C'est l'annulation du quotidien,
/// celle qui ne demande pas de dialogue.
pub fn corbeille() -> std::path::PathBuf {
    let dossier = work_dir().join("corbeille");
    let _ = std::fs::create_dir_all(&dossier);

    dossier
}

/// Déplace un fichier vers la corbeille, et rend où il est allé.
///
/// Le nom y devient unique : deux « document.tw » supprimés ne se
/// marchent pas dessus.
pub fn vers_la_corbeille(chemin: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let nom = chemin
        .file_name()
        .and_then(|nom| nom.to_str())
        .unwrap_or("fichier")
        .to_string();

    let mut destination = corbeille().join(&nom);
    let mut numero = 2;

    while destination.exists() {
        destination = corbeille().join(format!("{nom}-{numero}"));
        numero += 1;
    }

    std::fs::rename(chemin, &destination)?;
    Ok(destination)
}

/// Crée un projet au nom choisi — et jamais sur un dossier existant.
///
/// Le nom est celui du dossier, plié à une forme qui tient partout :
/// minuscules, sans espace ni barre. Un dossier existant n'est pas
/// réutilisé en silence : c'est peut-être le travail de quelqu'un.
pub fn creer_au_nom(parent: &Path, nom: &str) -> Result<PathBuf, String> {
    let plie = plier_nom(nom);

    if plie.is_empty() {
        return Err("Le nom du projet est vide.".to_string());
    }

    let dossier = parent.join(&plie);

    if dossier.exists() {
        return Err(format!(
            "« {plie} » existe déjà — un autre nom, ou l'ouvrir."
        ));
    }

    std::fs::create_dir_all(&dossier)
        .map_err(|erreur| format!("{} : {erreur}", dossier.display()))?;

    Ok(dossier)
}

/// Renomme un projet — le dossier suit, et tout ce qui y vit avec lui.
pub fn renommer(dir: &Path, nom: &str) -> Result<PathBuf, String> {
    let plie = plier_nom(nom);

    if plie.is_empty() {
        return Err("Le nom du projet est vide.".to_string());
    }

    let parent = dir
        .parent()
        .ok_or_else(|| "Pas de dossier parent.".to_string())?;
    let cible = parent.join(&plie);

    if cible == dir {
        return Ok(dir.to_path_buf());
    }

    if cible.exists() {
        return Err(format!("« {plie} » existe déjà — un autre nom."));
    }

    std::fs::rename(dir, &cible).map_err(|erreur| format!("{erreur}"))?;
    Ok(cible)
}

/// Supprime un projet — vers la corbeille, comme tout le reste.
pub fn supprimer(dir: &Path) -> Result<PathBuf, String> {
    vers_la_corbeille(dir).map_err(|erreur| format!("{erreur}"))
}

/// Un nom de dossier : minuscules, tirets, sans espace ni barre.
pub fn plier_nom(nom: &str) -> String {
    let plie: String = nom
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' | '_' => '-',
            '/' | '\\' | ':' => '-',
            autre => autre,
        })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    plie.trim_matches('-').to_string()
}

/// Le verrou d'un projet : une seule instance y travaille.
///
/// Deux instances s'écrasent leurs auto-sauvegardes — le texte de l'une
/// efface celui de l'autre, sans qu'on sache lequel a gagné. Le verrou est
/// un fichier qui porte le processus qui le tient : un mort le libère, un
/// vivant le garde.
#[derive(Debug)]
pub struct Verrou {
    chemin: std::path::PathBuf,
}

impl Verrou {
    /// Prend le verrou d'un projet. `Err` si une instance vivante le tient.
    pub fn prendre(dir: &Path) -> Result<Self, String> {
        let chemin = dir.join(".verrou");

        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&chemin)
        {
            Ok(mut fichier) => {
                use std::io::Write;
                let _ = writeln!(fichier, "{}", std::process::id());
                Ok(Self { chemin })
            }

            Err(_) => {
                // Il existe : qui le tient ? Un mort l'a laissé, on le reprend.
                let contenu = std::fs::read_to_string(&chemin).unwrap_or_default();
                let pid: i32 = contenu.trim().parse().unwrap_or(0);

                if pid > 0 && process_vivant(pid) {
                    return Err(format!(
                        "Une autre instance travaille sur ce projet (processus {pid})."
                    ));
                }

                std::fs::write(&chemin, format!("{}\n", std::process::id()))
                    .map_err(|erreur| format!("{} : {erreur}", chemin.display()))?;

                Ok(Self { chemin })
            }
        }
    }
}

impl Drop for Verrou {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.chemin);
    }
}

/// Un processus existe-t-il encore ? Un mort ne tient pas de verrou.
fn process_vivant(pid: i32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// Écrit un fichier d'un seul tenant : temporaire, puis renommage.
///
/// Une écriture directe tronque le fichier si la machine s'arrête au
/// milieu ; celle-ci n'écrit jamais qu'un temporaire, et l'ancien contenu
/// tient jusqu'au bout. C'est la même voie que prennent déjà les
/// migrations — un document d'écriture ne mérite pas moins.
pub fn ecrire_atomique(chemin: &std::path::Path, contenu: &str) -> std::io::Result<()> {
    let nom = chemin
        .file_name()
        .and_then(|nom| nom.to_str())
        .unwrap_or("fichier");
    let temporaire = chemin.with_file_name(format!(".{nom}.tmp"));

    std::fs::write(&temporaire, contenu)?;
    std::fs::rename(&temporaire, chemin)
}

/// La même voie, pour des octets qui ne sont pas du texte (un import).
pub fn ecrire_atomique_octets(chemin: &std::path::Path, octets: &[u8]) -> std::io::Result<()> {
    let nom = chemin
        .file_name()
        .and_then(|nom| nom.to_str())
        .unwrap_or("fichier");
    let temporaire = chemin.with_file_name(format!(".{nom}.tmp"));

    std::fs::write(&temporaire, octets)?;
    std::fs::rename(&temporaire, chemin)
}

/// Session de travail : ce qu'il faut rouvrir au démarrage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Dossier du projet ouvert.
    #[serde(default)]
    pub project: Option<PathBuf>,

    /// Chemin du chapitre ouvert dans ce projet.
    #[serde(default)]
    pub last_document: Option<PathBuf>,

    /// Le mode d'apparence suivi : système, clair ou sombre.
    ///
    /// À défaut, `default` : le mode système, et les thèmes natifs. Une session
    /// enregistrée avant les thèmes se relit donc sans rien perdre.
    #[serde(default)]
    pub theme_mode: crate::ui::theme::ThemeMode,

    /// Les thèmes retenus pour chaque apparence.
    ///
    /// On enregistre le **nom**, jamais une position : la liste change quand on
    /// charge ou supprime un thème, et une position finirait par désigner un
    /// autre thème que celui qu'on avait choisi. Un nom devenu introuvable est
    /// traité par la sélection elle-même, qui retombe sur un thème de la même
    /// apparence.
    #[serde(default)]
    pub themes: crate::theming::Chosen,
}

fn session_path() -> PathBuf {
    work_dir().join("session.json")
}

/// Charge la session enregistrée.
pub fn load_session() -> Session {
    load_session_from(&session_path()).unwrap_or_default()
}

/// Charge une session depuis un fichier donné.
///
/// Une session absente, tronquée ou illisible donne la session **vide** : un
/// fichier de configuration cassé ne doit jamais empêcher l'application de
/// démarrer, et repartir des valeurs par défaut est toujours possible.
///
/// Le chemin est un paramètre plutôt qu'une constante pour que la persistance
/// soit vérifiable sur un fichier réel, et non seulement en mémoire.
/// Relit la session. **Une session absente est normale** ; une session
/// illisible est un problème, et elle le dit — jamais de repli muet qui
/// laisserait croire que tout va bien.
pub fn load_session_from(path: &Path) -> Result<Session, String> {
    let texte = match std::fs::read_to_string(path) {
        Ok(texte) => texte,
        Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Session::default());
        }
        Err(erreur) => return Err(format!("{} : {erreur}", path.display())),
    };

    serde_json::from_str(&texte).map_err(|erreur| format!("{} : {erreur}", path.display()))
}

/// Enregistre la session.
pub fn save_session(session: &Session) {
    save_session_to(&session_path(), session);
}

/// Enregistre une session dans un fichier donné.
///
/// Un échec d'écriture est signalé sans interrompre : la session est une
/// commodité, pas une donnée dont dépend la suite du travail.
pub fn save_session_to(path: &Path, session: &Session) {
    let text = match serde_json::to_string_pretty(session) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Sérialisation de la session : {error}");
            return;
        }
    };

    if let Err(error) = ecrire_atomique(path, &text) {
        eprintln!("Enregistrement de la session : {error}");
    }
}

/// Une entrée de bibliographie (fichier `.bib`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BibliographyEntry {
    /// La clé de citation, celle qui suit `@type{`.
    pub key: String,

    /// Le type BibTeX (`article`, `book`, `inproceedings`…).
    pub kind: String,

    /// Les auteurs, `(nom, prénom)` dans l'ordre du fichier.
    ///
    /// BibTeX écrit « Dupont, Jean » ou « Jean Dupont » : les deux formes
    /// sont ramenées au même couple, nom d'abord.
    pub auteurs: Vec<(String, String)>,

    /// Le titre de l'œuvre.
    pub titre: String,

    /// L'année de parution, si le fichier la donne.
    pub annee: Option<String>,

    /// La maison d'édition, `publisher` chez BibTeX.
    pub editeur: Option<String>,

    /// Le lieu de parution, `location` ou `address`.
    pub lieu: Option<String>,

    /// Le titre court, `shorttitle` : celui des notes serrées.
    pub titre_court: Option<String>,
}

impl BibliographyEntry {
    /// La ligne de liste : « Dupont, Jean — Le titre… ».
    ///
    /// L'affichage la compose lui-même en deux registres ; cette ligne
    /// unique reste celle des tests, du modèle, et de qui préfère un trait.
    #[allow(dead_code)]
    pub fn libelle(&self) -> String {
        let auteur = match self.auteurs.first() {
            Some((nom, prenom)) if !prenom.is_empty() => format!("{nom}, {prenom}"),
            Some((nom, _)) => nom.clone(),
            None => String::new(),
        };

        let corps = if self.titre.is_empty() {
            self.key.clone()
        } else {
            self.titre.clone()
        };

        if auteur.is_empty() {
            corps
        } else {
            format!("{auteur} — {corps}")
        }
    }
}

/// Lit les entrées d'un fichier `.bib`.
///
/// Parseur volontairement tolérant : il cherche les entrées (`@type{clé,`),
/// puis les champs qu'elles portent — auteur, titre, année. Un fichier
/// malformé renvoie les entrées reconnues jusqu'au problème.
pub fn parse_bibliography(path: &Path) -> Vec<BibliographyEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    parse_bibliography_text(&content)
}

/// Les champs d'une notice, tels que le formulaire les édite.
pub struct ReferenceChamps {
    /// La clé de citation — celle que les documents citent.
    pub cle: String,

    /// Le type de document : `book`, `article`, `inproceedings`…
    pub kind: String,

    /// Le nom de l'auteur.
    pub nom: String,

    /// Son prénom.
    pub prenom: String,

    /// Le titre de l'œuvre.
    pub titre: String,

    /// Le titre court, celui des notes serrées.
    pub titre_court: String,

    /// L'année de parution.
    pub annee: String,

    /// La maison d'édition.
    pub editeur: String,

    /// Le lieu de parution.
    pub lieu: String,
}

/// Met à jour une référence dans un fichier `.bib`.
///
/// Les champs connus sont remplacés ou ajoutés **dans le bloc de l'entrée** ;
/// tout le reste du fichier ne bouge pas, et la clé reste la sienne : elle
/// est citée dans les documents.
pub fn maj_reference(
    chemin: &std::path::Path,
    cle: &str,
    champs: &ReferenceChamps,
) -> std::io::Result<()> {
    let contenu = std::fs::read_to_string(chemin)?;
    let octets = contenu.as_bytes();
    let mut sortie = String::with_capacity(contenu.len() + 64);
    let mut curseur = 0usize;

    while curseur < contenu.len() {
        let Some(relatif) = contenu[curseur..].find('@') else {
            sortie.push_str(&contenu[curseur..]);
            break;
        };

        let debut = curseur + relatif;
        sortie.push_str(&contenu[curseur..debut]);

        let fin = fin_d_entree(octets, debut).unwrap_or(contenu.len());
        let bloc = &contenu[debut..fin];

        if entree_de(bloc).is_some_and(|entree| entree.key == cle) {
            let auteur = if champs.prenom.is_empty() {
                champs.nom.clone()
            } else {
                format!("{}, {}", champs.nom, champs.prenom)
            };

            let mut maj = bloc.to_string();
            maj = remplacer_type(&maj, &champs.kind);
            maj = remplacer_champ(&maj, "author", &auteur);
            maj = remplacer_champ(&maj, "title", &champs.titre);
            maj = option(&maj, "year", &champs.annee);
            maj = option(&maj, "publisher", &champs.editeur);
            maj = option(&maj, "location", &champs.lieu);
            maj = option(&maj, "shorttitle", &champs.titre_court);

            sortie.push_str(&maj);
        } else {
            sortie.push_str(bloc);
        }

        curseur = fin;
    }

    ecrire_atomique(chemin, &sortie)
}

/// Un champ optionnel : écrit quand il a une valeur, retiré sinon.
fn option(entree: &str, champ: &str, valeur: &str) -> String {
    if valeur.is_empty() {
        retirer_champ(entree, champ)
    } else {
        remplacer_champ(entree, champ, valeur)
    }
}

/// Le type d'une entrée : `@book{…` devient `@article{…`.
///
/// La clé, elle, ne bouge pas : elle est citée dans les documents.
fn remplacer_type(entree: &str, kind: &str) -> String {
    let Some(ouverture) = entree.find('{') else {
        return entree.to_string();
    };

    format!("@{}{}", kind.trim(), &entree[ouverture..])
}

/// Remplace la valeur d'un champ dans un bloc d'entrée, ou l'y ajoute.
fn remplacer_champ(bloc: &str, nom: &str, valeur: &str) -> String {
    let trouve = champ_plage(bloc, nom);

    match trouve {
        Some((debut, fin)) => {
            let mut sortie = String::with_capacity(bloc.len() + valeur.len());
            sortie.push_str(&bloc[..debut]);
            sortie.push_str(valeur);
            sortie.push_str(&bloc[fin..]);

            sortie
        }
        None => {
            // Avant l'accolade fermante : la dernière du bloc.
            let Some(fin) = bloc.rfind('}') else {
                return bloc.to_string();
            };

            // La virgule sépare les champs — jamais deux, et toujours une
            // entre le dernier champ et le nouveau. Une virgule orpheline,
            // et c'est BibLaTeX qui refuse le fichier entier.
            // La virgule sépare les champs — jamais deux, et toujours une
            // entre le dernier champ et le nouveau. Une virgule orpheline,
            // et c'est BibLaTeX qui refuse le fichier entier.
            let tete = bloc[..fin].trim_end();
            let separateur = if tete.ends_with(',') { "" } else { "," };

            let mut sortie = String::with_capacity(bloc.len() + valeur.len() + 16);
            sortie.push_str(tete);
            sortie.push_str(separateur);
            sortie.push_str(&format!("\n  {nom} = {{{valeur}}}\n"));
            sortie.push_str(&bloc[fin..]);

            sortie
        }
    }
}

/// Retire un champ d'un bloc d'entrée, s'il y est.
fn retirer_champ(bloc: &str, nom: &str) -> String {
    let Some((debut_valeur, fin_valeur)) = champ_plage(bloc, nom) else {
        return bloc.to_string();
    };

    // Le champ **entier** : depuis la fin du champ précédent — sa virgule
    // ou l'ouverture de l'entrée — jusqu'après le délimiteur fermant de la
    // valeur et sa virgule. Ne rien laisser : une accolade orpheline
    // referme l'entrée avant l'heure et chasse le reste des champs, et une
    // virgule oubliée en double corrompt le fichier aux yeux de Typst.
    let debut = bloc[..debut_valeur]
        .rfind(',')
        .map(|index| index + 1)
        .or_else(|| bloc.find('{').map(|index| index + 1))
        .unwrap_or(0);

    // Le délimiteur fermant est juste après la valeur : accolade ou
    // guillemet, puis la virgule s'il y en a une.
    let mut fin = (fin_valeur + 1).min(bloc.len());

    let suite = bloc[fin..].chars().next();
    if suite == Some(',') {
        fin += 1;
    }

    let mut sortie = String::with_capacity(bloc.len());
    sortie.push_str(&bloc[..debut]);
    sortie.push_str(&bloc[fin..]);

    sortie
}

/// La plage de la **valeur** d'un champ : entre accolades, sans elles.
fn champ_plage(bloc: &str, nom: &str) -> Option<(usize, usize)> {
    let mut base = 0usize;

    while let Some(relatif) = bloc[base..].find('=') {
        let egal = base + relatif;
        let debut_valeur = egal + 1 + bloc[egal + 1..].len() - bloc[egal + 1..].trim_start().len();
        let avant = bloc[..egal].trim();
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            let valeur = &bloc[debut_valeur..];

            if let Some(ouvrante) = valeur.find('{') {
                let mut profondeur = 1usize;

                for (relatif, caractere) in valeur[ouvrante + 1..].char_indices() {
                    match caractere {
                        '{' => profondeur += 1,
                        '}' => {
                            profondeur -= 1;

                            if profondeur == 0 {
                                return Some((
                                    debut_valeur + ouvrante + 1,
                                    debut_valeur + ouvrante + 1 + relatif,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }

            if let Some(ouvrante) = valeur.find('"')
                && let Some(fermante) = valeur[ouvrante + 1..].find('"')
            {
                return Some((
                    debut_valeur + ouvrante + 1,
                    debut_valeur + ouvrante + 1 + fermante,
                ));
            }

            // Valeur nue : jusqu'à la virgule ou l'accolade.
            let fin = valeur
                .find([',', '}'])
                .map_or(valeur.len(), |index| debut_valeur + index);

            return Some((debut_valeur, fin));
        }

        base = egal + 1;
    }

    None
}

/// Les entrées d'un texte BibTeX.
/// Supprime une notice de la bibliothèque locale — vers la corbeille.
///
/// Une notice n'est pas un fichier : elle part **avec** son entrée, et le
/// fichier reste propre. La suppression ne touche jamais à une bibliothèque
/// Zotero : elle, se lit seulement.
pub fn supprimer_reference(path: &Path, cle: &str) -> Result<(), String> {
    let contenu = std::fs::read_to_string(path).map_err(|e| format!("{e}"))?;
    let (ouverture, fermeture) =
        plage_de_notice(&contenu, cle).ok_or_else(|| format!("« {cle} » introuvable."))?;

    // L'entière : sa tête, ses champs, et la ligne qui les sépare du reste.
    let mut fin = fermeture;

    while fin < contenu.len() && contenu[fin..].starts_with('\n') {
        fin += 1;
    }

    let mut reste = contenu[..ouverture].to_string();
    reste.push_str(&contenu[fin..]);

    ecrire_atomique(path, &reste).map_err(|e| format!("{e}"))
}

/// L'étendue d'une notice : sa tête `@type{clé,` et son accolade fermante.
fn plage_de_notice(contenu: &str, cle: &str) -> Option<(usize, usize)> {
    let motif = format!("{{{cle},");
    // Jamais de `find - 1` : un motif en tête de fichier soustrairait un
    // à zéro. La tête commence au `@` qui précède.
    let tete = contenu.find(&motif)?;
    let ouverture = contenu[..tete].rfind('@')?;

    let mut profondeur = 0usize;

    for (position, caractere) in contenu[ouverture..].char_indices() {
        match caractere {
            '{' => profondeur += 1,
            '}' => {
                profondeur = profondeur.saturating_sub(1);

                if profondeur == 0 {
                    return Some((ouverture, ouverture + position + 1));
                }
            }
            _ => {}
        }
    }

    None
}

pub fn parse_bibliography_text(content: &str) -> Vec<BibliographyEntry> {
    let mut entrees = Vec::new();
    let octets = content.as_bytes();
    let mut curseur = 0usize;

    while let Some(debut) = content[curseur..].find('@') {
        let debut = curseur + debut;

        let Some(fin) = fin_d_entree(octets, debut) else {
            break;
        };

        if let Some(entree) = entree_de(&content[debut..fin]) {
            entrees.push(entree);
        }

        curseur = fin;
    }

    entrees
}

/// Ajoute une référence au fichier `.bib` et rend la clé retenue.
///
/// Une clé vide prend un nom généré ; une clé déjà citée est dédoublonnée
/// d'un numéro — jamais deux documents ne partagent une clé de citation.
pub fn nouvelle_reference(
    chemin: &std::path::Path,
    champs: &ReferenceChamps,
) -> std::io::Result<String> {
    let contenu = std::fs::read_to_string(chemin).unwrap_or_default();
    let existantes = parse_bibliography_text(&contenu);

    let mut cle = champs.cle.trim().to_string();
    if cle.is_empty() {
        cle = "reference".to_string();
    }

    let racine = cle.clone();
    let mut numero = 2;
    while existantes.iter().any(|entree| entree.key == cle) {
        cle = format!("{racine}{numero}");
        numero += 1;
    }

    let auteur = if champs.prenom.is_empty() {
        champs.nom.trim().to_string()
    } else {
        format!("{}, {}", champs.nom.trim(), champs.prenom.trim())
    };

    let mut entree = format!("@{}{{{},\n", champs.kind.trim(), cle);

    for (champ, valeur) in [
        ("author", auteur.as_str()),
        ("title", champs.titre.trim()),
        ("shorttitle", champs.titre_court.trim()),
        ("year", champs.annee.trim()),
        ("publisher", champs.editeur.trim()),
        ("location", champs.lieu.trim()),
    ] {
        if !valeur.is_empty() {
            entree.push_str(&format!("  {champ} = {{{valeur}}},\n"));
        }
    }

    entree.push_str("}\n\n");

    let mut suite = contenu.trim_end().to_string();
    if !suite.is_empty() {
        suite.push('\n');
        suite.push('\n');
    }
    suite.push_str(&entree);

    ecrire_atomique(chemin, &suite)?;
    Ok(cle)
}

/// La fin d'une entrée : l'accolade qui referme celle ouverte en `debut`.
fn fin_d_entree(octets: &[u8], debut: usize) -> Option<usize> {
    let mut profondeur = 0usize;

    for (relatif, octet) in octets[debut..].iter().enumerate() {
        match octet {
            b'{' => profondeur += 1,
            b'}' => {
                profondeur = profondeur.saturating_sub(1);

                if profondeur == 0 {
                    return Some(debut + relatif + 1);
                }
            }
            _ => {}
        }
    }

    None
}

/// Une entrée, lue dans son bloc entier : `@type{clé, champs… }`.
fn entree_de(bloc: &str) -> Option<BibliographyEntry> {
    let reste = bloc.trim().strip_prefix('@')?;
    let (kind, reste) = reste.split_once('{')?;

    let (cle, champs) = reste.split_once(',').unwrap_or((reste, ""));
    let key = cle.split(['}', ',']).next()?.trim().to_string();

    if key.is_empty() {
        return None;
    }

    let lieu = champ(champs, "location").or_else(|| champ(champs, "address"));

    Some(BibliographyEntry {
        kind: kind.trim().to_string(),
        key,
        auteurs: auteurs_de(champ(champs, "author")),
        titre: champ(champs, "title").unwrap_or_default(),
        annee: champ(champs, "year"),
        editeur: champ(champs, "publisher"),
        lieu,
        titre_court: champ(champs, "shorttitle"),
    })
}

/// La valeur d'un champ BibTeX, sans ses accolades ni ses guillemets.
fn champ(champs: &str, nom: &str) -> Option<String> {
    let mut reste = champs;

    while let Some(egal) = reste.find('=') {
        let avant = reste[..egal].trim();
        let apres = &reste[egal + 1..];

        // Le nom du champ est ce qui suit la virgule précédente.
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            return Some(valeur_de(apres));
        }

        reste = apres;
    }

    None
}

/// La valeur qui suit un `=` : entre accolades, entre guillemets, ou jusqu'à
/// la virgule suivante.
fn valeur_de(texte: &str) -> String {
    let texte = texte.trim_start();

    if let Some(reste) = texte.strip_prefix('{') {
        let mut profondeur = 1usize;

        for (index, caractere) in reste.char_indices() {
            match caractere {
                '{' => profondeur += 1,
                '}' => {
                    profondeur -= 1;

                    if profondeur == 0 {
                        return reste[..index].trim().to_string();
                    }
                }
                _ => {}
            }
        }

        return reste.trim().to_string();
    }

    if let Some(reste) = texte.strip_prefix('"') {
        return reste
            .split_once('"')
            .map(|(valeur, _)| valeur.trim().to_string())
            .unwrap_or_else(|| reste.trim().to_string());
    }

    texte.split(',').next().unwrap_or(texte).trim().to_string()
}

/// Les auteurs d'un champ `author` : « et » sépare, la virgule dit le nom.
fn auteurs_de(champ: Option<String>) -> Vec<(String, String)> {
    let Some(texte) = champ else {
        return Vec::new();
    };

    texte
        .replace(" AND ", " and ")
        .split(" and ")
        .filter(|auteur| !auteur.trim().is_empty())
        .map(|auteur| {
            let auteur = auteur.trim();

            if let Some((nom, prenom)) = auteur.split_once(',') {
                (nom.trim().to_string(), prenom.trim().to_string())
            } else {
                // « Jean Dupont » : le nom est le dernier mot.
                let mut mots = auteur.rsplitn(2, ' ');
                let nom = mots.next().unwrap_or(auteur).to_string();
                let prenom = mots.next().unwrap_or("").to_string();

                (nom, prenom)
            }
        })
        .collect()
}

/// Le nom du projet créé lors de la reprise des documents existants.
const MIGRATED_PROJECT: &str = "mon-travail";

/// Reprend les documents à plat dans un premier projet.
///
/// Les versions précédentes rangeaient les documents directement à la racine de
/// `~/.typst-writer/`. Puisqu'un travail est désormais un projet, ces fichiers
/// deviennent les chapitres d'un projet « mon-travail ».
///
/// Ne fait rien si un projet existe déjà, ou s'il n'y a rien à reprendre :
/// la migration ne s'exécute qu'une fois, sur un dossier vierge de projets.
///
/// Renvoie le dossier du projet créé, le cas échéant.
pub fn migrate_flat_documents() -> Option<PathBuf> {
    let work = work_dir();
    let flat = locate_flat_documents(&work);

    if flat.is_empty() {
        return None;
    }

    let dir = model::create(MIGRATED_PROJECT).ok()?;

    // Une bibliographie restée à la racine doit rejoindre le projet : sans
    // cela, un chapitre qui la cite perdrait sa référence.
    adopt_stray_bibliography(&work, &dir);

    let chapters_dir = dir.join("chapitres");

    let session = load_session();
    let mut moved_session = session.project.is_none();

    for (index, path) in flat.iter().enumerate() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("chapitre.typ");

        let destination = chapters_dir.join(format!("{:02}-{}", index + 1, name));

        if std::fs::rename(path, &destination).is_err() {
            // Un fichier qu'on n'a pas pu déplacer est laissé sur place plutôt
            // que perdu : les `.typ` encore à la racine font reprendre la
            // migration au prochain démarrage.
            continue;
        }

        let _ = model::add_chapter_to_assembly(&dir.join("main.typ"), &destination);

        if session.last_document.as_deref() == Some(path.as_path()) {
            moved_session = true;
        }
    }

    // La session pointait vers un fichier à plat : elle pointe maintenant vers
    // le projet, sinon le démarrage suivant chercherait un document disparu.
    if moved_session {
        save_session(&Session {
            project: Some(dir.clone()),
            last_document: None,
            // Le thème retenu ne regarde pas la migration des documents à plat :
            // il survit tel quel, et le reste de la session avec lui.
            ..session.clone()
        });
    }

    Some(dir)
}

/// Fait rejoindre au projet une bibliographie laissée à la racine.
///
/// Le fichier créé par `model::create` est un modèle vide : on ne le remplace
/// que s'il est effectivement vide, pour ne jamais écraser des références
/// existantes par un exemple commenté.
fn adopt_stray_bibliography(work: &Path, project_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(work) else {
        return;
    };

    let Some(stray) = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().map(|ext| ext == "bib").unwrap_or(false))
    else {
        return;
    };

    let destination = project_dir.join("references.bib");

    // Le modèle **intact**, et pas « sans `@` » : ses commentaires en
    // portent un, et cette vérification-là ne pouvait jamais être vraie —
    // la bibliothèque errante n'était donc jamais adoptée.
    let template_is_untouched = std::fs::read_to_string(&destination)
        .map(|content| content.trim() == crate::project::model::BIBLIOGRAPHY_TEMPLATE.trim())
        .unwrap_or(false);

    if template_is_untouched {
        let _ = std::fs::rename(&stray, &destination);
    }
}

/// Les documents `.typ` restés à la racine du dossier de travail.
fn locate_flat_documents(work: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(work) else {
        return Vec::new();
    };

    let mut documents: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().map(|ext| ext == "typ").unwrap_or(false))
        .collect();

    // L'ordre de lecture du dossier n'est pas garanti : on trie pour que la
    // numérotation des chapitres soit reproductible.
    documents.sort();

    documents
}

#[cfg(test)]
mod tests {
    /// Un projet se crée au nom choisi, et jamais sur un autre.
    #[test]
    fn un_projet_se_cree_au_nom_choisi() {
        let parent = std::env::temp_dir().join(format!("tw-projets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "Ma Thèse").expect("création");
        assert_eq!(dossier.file_name().unwrap(), "ma-thèse", "le nom suit");

        // Un dossier existant n'est pas réutilisé en silence.
        let probleme = creer_au_nom(&parent, "Ma Thèse").expect_err("refus");
        assert!(probleme.contains("existe"), "vu : {probleme}");

        let _ = std::fs::remove_dir_all(&parent);
    }

    /// Un projet se renomme — le dossier suit.
    #[test]
    fn un_projet_se_renomme() {
        let parent = std::env::temp_dir().join(format!("tw-projets-2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "ancien").expect("création");
        let renomme = renommer(&dossier, "nouveau").expect("renommage");

        assert_eq!(renomme.file_name().unwrap(), "nouveau");
        assert!(renomme.exists());

        let _ = std::fs::remove_dir_all(&parent);
    }

    /// Un projet supprimé part à la corbeille — jamais dans le vide.
    #[test]
    fn un_projet_supprime_part_a_la_corbeille() {
        let parent = std::env::temp_dir().join(format!("tw-projets-3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "a-supprimer").expect("création");
        let corbeille = supprimer(&dossier).expect("suppression");

        assert!(!dossier.exists());
        assert!(corbeille.exists(), "la corbeille le garde");

        let _ = std::fs::remove_dir_all(&parent);
        let _ = std::fs::remove_file(&corbeille);
    }

    /// Une seule instance tient le verrou ; un mort le libère.
    #[test]
    fn le_verrou_garde_le_projet() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        let fichier = dir.join(".verrou");
        let premier = Verrou::prendre(&dir).expect("premier verrou");

        // Un mort l'a laissé : le verrou se reprend.
        std::fs::write(&fichier, "999999999\n").expect("un mort");
        drop(premier);

        let repris = Verrou::prendre(&dir).expect("un mort libère");
        drop(repris);

        // Et parti, le fichier n'est plus.
        assert!(!fichier.exists(), "le verrou part avec l'instance");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une instance vivante est refusée — et le refus nomme le processus.
    #[test]
    fn une_instance_vivante_est_refusee() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        // Un processus vivant : celui du test lui-même.
        std::fs::write(dir.join(".verrou"), format!("{}\n", std::process::id())).expect("verrou");

        let probleme = Verrou::prendre(&dir).expect_err("refusé");
        assert!(probleme.contains("autre instance"), "vu : {probleme}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// La bibliothèque errante est adoptée quand le modèle est intact —
    /// et le modèle contient un `@` dans ses commentaires, ce que
    /// l'ancienne vérification ne pardonnait pas.
    #[test]
    fn la_bibliotheque_errante_est_adoptee() {
        let work = std::env::temp_dir().join(format!("tw-adopt-{}", std::process::id()));
        let projet = work.join("mon-travail");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&projet).expect("dossier");

        // Le modèle intact, celui que `create` écrit.
        std::fs::write(
            projet.join("references.bib"),
            crate::project::model::BIBLIOGRAPHY_TEMPLATE,
        )
        .expect("modèle");

        // La bibliothèque errante, à la racine du dossier de travail.
        std::fs::write(
            work.join("references.bib"),
            "@book{deja,\n  title = {Déjà},\n}\n",
        )
        .expect("errante");

        super::adopt_stray_bibliography(&work, &projet);

        let contenu = std::fs::read_to_string(projet.join("references.bib")).expect("lu");
        assert!(
            contenu.contains("Déjà"),
            "la bibliothèque errante rejoint le projet : {contenu}"
        );

        let _ = std::fs::remove_dir_all(&work);
    }

    /// Un modèle déjà travaillé n'est pas écrasé.
    #[test]
    fn un_modele_travaille_n_est_pas_ecrase() {
        let work = std::env::temp_dir().join(format!("tw-adopt-2-{}", std::process::id()));
        let projet = work.join("mon-travail");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&projet).expect("dossier");

        std::fs::write(
            projet.join("references.bib"),
            "@book{mien,\n  title = {Le mien},\n}\n",
        )
        .expect("travail");
        std::fs::write(
            work.join("references.bib"),
            "@book{autre,\n  title = {Autre},\n}\n",
        )
        .expect("errante");

        super::adopt_stray_bibliography(&work, &projet);

        let contenu = std::fs::read_to_string(projet.join("references.bib")).expect("lu");
        assert!(contenu.contains("Le mien"), "le travail reste : {contenu}");

        let _ = std::fs::remove_dir_all(&work);
    }

    /// Une suppression n'est pas une destruction : le fichier va à la
    /// corbeille, le chemin est rendu, et rien ne se marche dessus.
    #[test]
    fn une_suppression_va_a_la_corbeille() {
        let dir = std::env::temp_dir().join(format!("tw-corbeille-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        let fichier = dir.join("document.tw");
        std::fs::write(&fichier, "le travail du jour").expect("fichier");

        let ou = vers_la_corbeille(&fichier).expect("déplacé");
        assert!(!fichier.exists(), "il a quitté sa place");
        assert!(ou.exists(), "il est quelque part");
        assert_eq!(
            std::fs::read_to_string(&ou).expect("relu"),
            "le travail du jour",
            "rien n'est perdu"
        );

        // Un second du même nom ne se marche pas dessus.
        std::fs::write(&fichier, "le lendemain").expect("fichier");
        let ou_deux = vers_la_corbeille(&fichier).expect("déplacé");
        assert_ne!(ou, ou_deux, "deux places, pas une");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Vider un champ présent retire le champ entier — jamais son
    /// accolade fermante, qui refermerait l'entrée et chasserait le reste.
    #[test]
    fn vider_un_champ_present_le_retire_entier() {
        let entree = "@book{cle,\n  author = {Nom},\n  title = {Titre},\n  year = {2014},\n}\n";

        let sans_annee = super::retirer_champ(entree, "year");
        assert_eq!(
            sans_annee, "@book{cle,\n  author = {Nom},\n  title = {Titre},\n}\n",
            "vu : {sans_annee}"
        );

        // Et le fichier reste lisible : le parseur y retrouve ses champs.
        let entrees = parse_bibliography_text(&sans_annee);
        assert_eq!(entrees.len(), 1);
        assert_eq!(entrees[0].titre, "Titre");

        // Sur une entrée d'une seule ligne aussi — la virgule de la clé
        // reste, et c'est bon : `@book{cle,}` se relit comme `@book{cle}`.
        let sans_auteur = super::retirer_champ("@book{cle, author = {Doe}}", "author");
        assert_eq!(sans_auteur, "@book{cle,}");
        assert_eq!(parse_bibliography_text(&sans_auteur).len(), 1);
    }

    /// Un champ retiré puis vidé à nouveau ne change plus rien.
    #[test]
    fn retirer_un_champ_absent_ne_change_rien() {
        let entree = "@book{cle,\n  title = {Titre},\n}\n";
        assert_eq!(super::retirer_champ(entree, "year"), entree);
    }

    /// Une notice neuve s'écrit avec ses champs — type, éditeur, lieu, titre
    /// court — et le fichier reste lisible par Typst.
    #[test]
    fn une_notice_neuve_s_ecrit_et_se_relit() {
        let dir = std::env::temp_dir().join(format!("tw-nouvelle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let chemin = dir.join("references.bib");
        std::fs::write(&chemin, "@book{deja,\n  title = {Déjà},\n}\n").unwrap();

        let cle = nouvelle_reference(
            &chemin,
            &ReferenceChamps {
                cle: "dupont2024".to_string(),
                kind: "article".to_string(),
                nom: "Dupont".to_string(),
                prenom: "Jean".to_string(),
                titre: "Un titre".to_string(),
                titre_court: "Titre court".to_string(),
                annee: "2024".to_string(),
                editeur: "Gallimard".to_string(),
                lieu: "Paris".to_string(),
            },
        )
        .unwrap();

        assert_eq!(cle, "dupont2024");

        let entrees = parse_bibliography(&chemin);
        assert_eq!(entrees.len(), 2);
        let neuve = &entrees[1];
        assert_eq!(neuve.key, "dupont2024");
        assert_eq!(neuve.kind, "article");
        assert_eq!(neuve.editeur.as_deref(), Some("Gallimard"));
        assert_eq!(neuve.lieu.as_deref(), Some("Paris"));
        assert_eq!(neuve.titre_court.as_deref(), Some("Titre court"));

        // Le vrai juge : le parseur BibLaTeX de Typst.
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        let root = std::path::Path::new("/projet");
        let entry = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n";
        let overrides = vec![(root.join("references.bib"), contenu)];
        let outcome = crate::typst::render::compile(root, entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "la notice neuve doit être valide : {:?}",
            outcome.errors
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une clé déjà citée n'est jamais réutilisée : elle prend un numéro.
    #[test]
    fn une_cle_citee_ne_se_double_pas() {
        let dir = std::env::temp_dir().join(format!("tw-cle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let chemin = dir.join("references.bib");
        std::fs::write(&chemin, "@book{dupont,\n  title = {Déjà},\n}\n").unwrap();

        let champs = ReferenceChamps {
            cle: "dupont".to_string(),
            kind: "book".to_string(),
            nom: "Dupont".to_string(),
            prenom: String::new(),
            titre: "Autre".to_string(),
            titre_court: String::new(),
            annee: String::new(),
            editeur: String::new(),
            lieu: String::new(),
        };

        assert_eq!(nouvelle_reference(&chemin, &champs).unwrap(), "dupont2");
        assert_eq!(nouvelle_reference(&chemin, &champs).unwrap(), "dupont3");

        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::*;

    /// Les entrées se lisent avec leurs auteurs, leur titre, leur année —
    /// c'est ce que l'onglet des références montrera à la place des clés.
    #[test]
    fn la_bibliographie_lit_les_auteurs_et_les_titres() {
        let texte = r#"
@article{dupont2023,
  author = {Dupont, Jean and Curie, Marie},
  title = {Une {étude} des notes},
  year = {2023},
}
@book{martin2020,
  author = "Paul Martin",
  title = "Le grand traité",
  year = "2020"
}
"#;
        let entrees = parse_bibliography_text(texte);

        assert_eq!(entrees.len(), 2, "deux entrées, même sur plusieurs lignes");
        assert_eq!(entrees[0].key, "dupont2023");
        assert_eq!(
            entrees[0].auteurs,
            vec![
                ("Dupont".to_string(), "Jean".to_string()),
                ("Curie".to_string(), "Marie".to_string())
            ],
            "« et » sépare les auteurs, la virgule dit le nom"
        );
        assert_eq!(entrees[0].titre, "Une {étude} des notes");
        assert_eq!(entrees[0].annee.as_deref(), Some("2023"));

        assert_eq!(
            entrees[1].libelle(),
            "Martin, Paul — Le grand traité",
            "la ligne de liste se lit : nom, prénom, titre"
        );
    }

    /// Sans auteur, la ligne montre le titre seul ; sans titre, la clé.
    #[test]
    fn le_libelle_tient_debout_sans_auteur() {
        let entree = BibliographyEntry {
            key: "anonyme".to_string(),
            kind: "book".to_string(),
            titre: "L'œuvre anonyme".to_string(),
            ..Default::default()
        };

        assert_eq!(entree.libelle(), "L'œuvre anonyme");
    }

    #[test]
    fn parses_bib_entries() {
        let dir = std::env::temp_dir().join("typst-writer-bib");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{smith2020,\n  title = {Un titre},\n}\n@book{doe2019, author = {Doe}}\n",
        )
        .unwrap();

        let entries = parse_bibliography(&bib);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "smith2020");
        assert_eq!(entries[0].kind, "article");
        assert_eq!(entries[1].key, "doe2019");
        assert_eq!(entries[1].kind, "book");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une entrée sans `@` ou sans clé est ignorée, pas une erreur ; une
    /// entrée jamais refermée est le « problème » où s'arrête la lecture.
    #[test]
    fn ignores_malformed_bib_lines() {
        assert!(parse_bibliography_text("ceci n'est pas une entrée").is_empty());
        assert!(parse_bibliography_text("@article{,}").is_empty());
        assert!(parse_bibliography_text("@article{clé,").is_empty());
        assert_eq!(parse_bibliography_text("@article{clé,}").len(), 1);
    }

    /// Modifier une référence réécrit ses champs, sans toucher au reste du
    /// Écrire une référence dans une entrée BibLaTeX donne un fichier que
    /// Typst accepte — la virgule de séparation n'est jamais deux.
    ///
    /// C'est la régression du rendu cassé : un champ ajouté après une ligne
    /// finissant déjà par une virgule écrivait une virgule orpheline, et le
    /// parseur BibLaTeX de Typst refusait le fichier entier.
    #[test]
    fn ecrire_une_reference_laisse_un_bib_que_typst_accepte() {
        let dir = std::env::temp_dir().join(format!("tw-maj-typst-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let chemin = dir.join("references.bib");
        let entree = "@book{cle,\n\tauthor = {Nom},\n\ttitle = {Titre},\n\tdate = {2023},\n}\n";
        std::fs::write(&chemin, entree).unwrap();

        // L'entrée n'a pas de champ `year` : c'est l'ajout qui corrompait.
        super::maj_reference(
            &chemin,
            "cle",
            &ReferenceChamps {
                cle: "cle".to_string(),
                kind: "book".to_string(),
                nom: "Nom".to_string(),
                prenom: "Prénom".to_string(),
                titre: "Titre".to_string(),
                titre_court: String::new(),
                annee: "2014".to_string(),
                editeur: String::new(),
                lieu: String::new(),
            },
        )
        .unwrap();

        let corrige = std::fs::read_to_string(&chemin).unwrap();
        assert!(
            !corrige.contains(",\n,"),
            "jamais deux virgules : {corrige}"
        );

        // Le vrai juge : le parseur BibLaTeX de Typst.
        let root = std::path::Path::new("/projet");
        let entry = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n";
        let overrides = vec![(root.join("references.bib"), corrige)];
        let outcome = crate::typst::render::compile(root, entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "le .bib doit rester valide : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// fichier — et la clé ne bouge pas, elle est citée dans les documents.
    #[test]
    fn modifier_une_reference_ne_reecrit_que_ses_champs() {
        let dir = std::env::temp_dir().join(format!("tw-maj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{dupont2023,\n  author = {Dupont, Jean},\n  title = {Un titre},\n  journal = {Revue},\n}\n@book{martin2020,\n  author = {Martin},\n  title = {L'autre},\n}\n",
        )
        .unwrap();

        maj_reference(
            &bib,
            "dupont2023",
            &ReferenceChamps {
                cle: "dupont2023".to_string(),
                kind: "article".to_string(),
                nom: "Curie".to_string(),
                prenom: "Marie".to_string(),
                titre: "Un autre titre".to_string(),
                titre_court: String::new(),
                annee: "2024".to_string(),
                editeur: String::new(),
                lieu: String::new(),
            },
        )
        .unwrap();

        let entrees = parse_bibliography(&bib);

        assert_eq!(entrees.len(), 2, "les deux entrées sont là");
        assert_eq!(entrees[0].key, "dupont2023", "la clé ne bouge pas");
        assert_eq!(
            entrees[0].auteurs,
            vec![("Curie".to_string(), "Marie".to_string())]
        );
        assert_eq!(entrees[0].titre, "Un autre titre");
        assert_eq!(entrees[0].annee.as_deref(), Some("2024"));
        assert_eq!(
            entrees[1].titre, "L'autre",
            "la seconde entrée n'a pas bougé"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une entrée sans virgule après la clé (ex. `@article{clé}`) donne la
    /// clé sans son accolade fermante.
    #[test]
    fn une_entree_sans_virgule_garde_sa_cle_nue() {
        let entry = entree_de("@article{clé}");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().key, "clé");

        let entry = entree_de("@book{smith2020 }");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().key, "smith2020");
    }

    /// Une session d'une version précédente, qui ne connaissait que
    /// `last_document`, doit se lire sans erreur.
    #[test]
    fn an_older_session_still_loads() {
        let session: Session =
            serde_json::from_str(r#"{"last_document": "/tmp/ancien.typ"}"#).unwrap();

        assert_eq!(
            session.last_document,
            Some(PathBuf::from("/tmp/ancien.typ"))
        );
        assert!(session.project.is_none());

        // Les champs de thème, absents, retombent sur le mode système et les
        // thèmes natifs : une session écrite avant les thèmes se relit donc
        // sans rien perdre, et sans démarrer sur un thème inexistant.
        assert_eq!(session.theme_mode, crate::ui::theme::ThemeMode::System);
        assert_eq!(session.themes, crate::theming::Chosen::default());
    }

    /// Le thème retenu survit à un redémarrage.
    ///
    /// C'est la raison d'être de ces champs : sans eux, l'application repartait
    /// sur les thèmes natifs à chaque lancement, et choisir un thème revenait à
    /// le rechoisir tous les matins.
    #[test]
    fn a_session_keeps_the_chosen_theme() {
        let session = Session {
            project: Some(PathBuf::from("/tmp/projet")),
            last_document: None,
            theme_mode: crate::ui::theme::ThemeMode::Light,
            themes: crate::theming::Chosen {
                light: "Moka clair".to_string(),
                dark: "Moka sombre".to_string(),
            },
        };

        let text = serde_json::to_string_pretty(&session).unwrap();

        // Le fichier reste lisible : c'est ce qu'on veut pouvoir inspecter et
        // corriger à la main quand quelque chose ne se rouvre pas comme prévu.
        assert!(text.contains("\"light\""), "session écrite :\n{text}");

        let restored: Session = serde_json::from_str(&text).unwrap();

        assert_eq!(restored.theme_mode, crate::ui::theme::ThemeMode::Light);
        assert_eq!(restored.themes.light, "Moka clair");
        assert_eq!(restored.themes.dark, "Moka sombre");
    }

    /// Le thème retenu survit à un aller-retour par un vrai fichier.
    ///
    /// C'est la chaîne complète du redémarrage : écrire la session, la relire,
    /// retrouver le thème qu'elle nomme dans la liste, et le sélectionner. Les
    /// trois maillons sont ici, dans l'ordre où l'application les exécute.
    #[test]
    fn the_chosen_theme_survives_a_real_restart() {
        let dir = std::env::temp_dir().join("typst-writer-session-theme");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("session.json");

        // 1. L'utilisateur choisit un thème clair et un thème sombre.
        save_session_to(
            &path,
            &Session {
                project: None,
                last_document: None,
                theme_mode: crate::ui::theme::ThemeMode::Dark,
                themes: crate::theming::Chosen {
                    light: "Clair".to_string(),
                    dark: "Mon thème".to_string(),
                },
            },
        );

        // 2. L'application redémarre et relit la session.
        let restored = load_session_from(&path).expect("session lisible");

        assert_eq!(restored.theme_mode, crate::ui::theme::ThemeMode::Dark);

        // 3. Les thèmes disponibles sont relus du dossier, et le choix retrouvé.
        let mut themes = crate::theming::natives();

        let file = crate::theming::ThemeFile::parse(crate::theming::EXAMPLE, None).unwrap();
        let name = file.meta.name.clone();

        themes.push(crate::theming::Theme {
            name: name.clone(),
            appearance: file.meta.appearance,
            file: Some(file),
        });

        let index =
            crate::theming::select(&themes, crate::theming::Appearance::Dark, &restored.themes);

        assert_eq!(
            themes[index].name, "Mon thème",
            "le thème retenu avant le redémarrage doit être celui qu'on applique après"
        );

        // 4. Une session absente retombe sur les défauts, sans échouer.
        let missing =
            load_session_from(&dir.join("absent.json")).expect("absente, et c'est normal");

        assert_eq!(missing.theme_mode, crate::ui::theme::ThemeMode::System);
        assert_eq!(missing.themes, crate::theming::Chosen::default());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un mode de thème s'écrit en minuscules et se relit tel quel.
    #[test]
    fn the_theme_mode_is_written_in_lowercase() {
        for (mode, expected) in [
            (crate::ui::theme::ThemeMode::System, "\"system\""),
            (crate::ui::theme::ThemeMode::Light, "\"light\""),
            (crate::ui::theme::ThemeMode::Dark, "\"dark\""),
        ] {
            let text = serde_json::to_string(&mode).unwrap();

            assert_eq!(text, expected, "mode {mode:?}");
        }
    }
}
