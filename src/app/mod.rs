pub mod assistance;
pub mod composeur;
pub mod epreuves;
mod fichiers;
mod journal;
pub mod message;
mod reglages;
pub mod state;
mod themes;

// Le parent garde la vue d'ensemble : les fonctions de ces deux modules sont
// appelées ici sans prefixe, comme si elles y etaient nees.
use assistance::*;
use journal::*;

use crate::app::state::{ChapterRename, Drag, LayoutState, PreviewPosition, Splitter};
pub use message::Message;
pub use state::State;

use iced::event::{Event, Status};
use iced::keyboard;
use iced::widget::image::Handle;
use iced::{Subscription, Task, time};

use std::path::PathBuf;

/// Intervalle de recompilation automatique après la dernière frappe.
const RENDER_INTERVAL_MS: u64 = 300;

/// Intervalle de sauvegarde automatique différée.
const SAVE_INTERVAL_MS: u64 = 1000;

/// Période du relevé de mémoire, en secondes.
const MEMORY_INTERVAL_S: u64 = 5;

impl State {
    /// État initial : le projet de la session est rouvert, ou le plus récent,
    /// ou un premier projet est créé — puis le chapitre à reprendre est ouvert.
    pub fn new() -> Self {
        // Reprise des documents des versions précédentes, qui vivaient à plat
        // dans le dossier de travail. Sans effet si un projet existe déjà.
        crate::project::migrate_flat_documents();

        let mut projects = crate::project::list_projects();

        // Aucun projet : c'est le premier démarrage, on en crée un.
        if projects.is_empty() {
            match crate::project::create(FIRST_PROJECT) {
                Ok(dir) => projects = vec![crate::project::load(&dir)],
                Err(error) => eprintln!("Création du premier projet : {error}"),
            }
        }

        let session = crate::project::load_session();

        // Le projet de la session s'il existe toujours, sinon le plus récent.
        let project = session
            .project
            .as_ref()
            .filter(|dir| dir.is_dir())
            .map(|dir| crate::project::load(dir))
            .or_else(|| projects.first().cloned())
            .unwrap_or_else(|| crate::project::load(&crate::project::projects_dir()));

        // La migration ne se fait pas en silence : ce qu'elle n'a pas
        // traduit se dit, à l'écran, dès le démarrage.
        let note_de_migration = match crate::project::migrate_to_document_avec_rapport(&project)
        {
            Ok((_, rapport)) => {
                if rapport.avertissements.is_empty() {
                    None
                } else {
                    Some(crate::app::state::Note::info(format!(
                        "Migration : {} bloc(s) laissé(s) en Typst brut — {}",
                        rapport.blocs_bruts,
                        rapport.avertissements.join(" ; ")
                    )))
                }
            }
            Err(error) => Some(crate::app::state::Note::erreur(format!(
                "Migration vers document.tw : {error}"
            ))),
        };

        let project = crate::project::load(&project.dir);

        // Un projet déjà lié se retrouve : sa bibliothèque est celle de
        // Zotero, lue — et les notices ne s'y écrivent pas.
        let zotero = crate::zotero::Etat {
            source: if project.zotero.is_some() {
                crate::zotero::Source::Zotero
            } else {
                crate::zotero::Source::Locale
            },
            lie: project.zotero.is_some(),
        };

        let mut state = Self {
            editor: crate::editor::EditorState::default(),
            export_status: None,
            bibliography: Vec::new(),
            project,
            projects,
            window_size: iced::Size::new(1180.0, 780.0),
            window: None,
            // L'écran se déclare dans les millisecondes qui suivent ; d'ici là,
            // un pixel par point, la densité la plus économique.
            scale_factor: 1.0,
            mise_en_page: None,
            renommage: None,
            composeur: None,
            edition_reference: None,
            zotero,
            renaming: None,
            jump: None,
            render: Default::default(),
            layout: Default::default(),
            lsp: Default::default(),
            system_theme: None,
            themes: Vec::new(),
            theme_problems: Vec::new(),
            // Le thème retenu est restauré **avant** la première lecture du
            // dossier : `refresh_themes` applique le thème, il doit donc savoir
            // lequel.
            chosen: session.themes.clone(),
            theme: session.theme_mode,
            resolved: crate::ui::theme::Resolved::of(&crate::theming::natives()[0]),
                    liaison: None,
            note: note_de_migration,
};

        // Le chapitre de la session, s'il appartient encore au projet ; sinon
        // le premier chapitre, sinon rien — le panneau proposera d'en créer un.
        //
        // L'appartenance doit être vérifiée, et non supposée : un document
        // retenu avant d'être un chapitre — ou dont le fichier a été déplacé
        // dans un autre projet — s'ouvrirait sinon dans l'éditeur sans jamais
        // être rendu, exactement le défaut que l'import répare.
        let wanted = session
            .last_document
            .filter(|path| path.is_file())
            .filter(|path| state.project.document.as_deref() == Some(path.as_path()));

        let document = wanted.or_else(|| state.project.document.clone());
        let chapter = document.or_else(|| {
            state
                .project
                .chapters
                .first()
                .map(|chapter| chapter.path.clone())
        });

        if let Some(path) = chapter
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            state.editor = crate::editor::EditorState::new_file(path, content);
            crate::app::assistance::actualiser_marques(&mut state);
        }

        state.refresh_context();
        // Les thèmes sont lus après le contexte : c'est le premier endroit où
        // l'état est complet, et la fenêtre doit s'ouvrir avec le thème retenu.
        crate::app::themes::refresh_themes(&mut state);

        state.save_session();

        state
    }

    /// Rescanne le projet et recharge sa bibliographie.
    ///
    /// C'est ce qui met à jour le panneau de travail : les comptes de mots, les
    /// chapitres ajoutés ou retirés, les entrées de bibliographie.
    /// Le thème actuellement appliqué.
    ///
    /// Recalculé à la demande plutôt que retenu : la liste des thèmes change
    /// quand on en charge ou en supprime un, et une référence gardée désignerait
    /// alors un thème qui n'existe plus.
    pub fn current_theme(&self) -> Option<&crate::theming::Theme> {
        let appearance = self.theme.appearance(self.system_theme);

        let index = crate::theming::select(&self.themes, appearance, &self.chosen);

        self.themes.get(index)
    }

    fn refresh_context(&mut self) {
        self.projects = crate::project::list_projects();

        // Le projet est relu depuis le disque : ses chapitres ont pu être
        // renommés, ajoutés ou supprimés depuis le dernier scan.
        self.project = crate::project::load(&self.project.dir);

        self.bibliography = self
            .project
            .bibliography
            .as_ref()
            .map(|bib| crate::project::parse_bibliography(bib))
            .unwrap_or_default();
    }

    /// Charge un chapitre dans l'éditeur, rendu vierge.
    ///
    /// Les images du chapitre précédent ne doivent pas survivre au changement :
    /// les `Handle` sont relâchés pour que le backend libère les textures.
    fn open_document(&mut self, path: PathBuf, content: String) {
        self.editor = crate::editor::EditorState::new_file(path, content);
        // Le chapitre neuf reçoit le thème courant : sans cela il garderait
        // celui du widget de la bibliothèque jusqu'au prochain changement.
        self.editor.retheme(&self.resolved);
        crate::app::assistance::actualiser_marques(self);
        self.export_status = None;
        self.render.reset();
        // Le compte de mots du document qu'on ouvre, tout de suite : la barre
        // d'état et le panneau le lisent, et attendre le prochain tic
        // afficherait un zéro entre-temps.
        self.render.words = Some(self.content_word_count(&self.editor.content()));
        self.refresh_context();
        self.save_session();
    }

    /// Met le serveur de langage au service du document que l'éditeur tient.
    ///
    /// Appelée à chaque tic, et **idempotente** : elle ne fait rien tant que le
    /// document ouvert est déjà celui que le serveur connaît. C'est ce qui la
    /// rend sûre comme unique point d'accroche — il n'y a pas de chemin
    /// d'ouverture à oublier, puisqu'on ne l'appelle depuis aucun.
    ///
    /// Le processus est cherché au premier document, une seule fois : quelques
    /// mégaoctets et une seconde de démarrage, ce qui n'a pas à être payé à
    /// chaque changement de chapitre — on lui ouvre donc le nouveau document, et
    /// on lui ferme l'ancien.
    ///
    /// Son absence n'est pas une panne : c'est un état, que l'état retient pour
    /// ne pas reposer la question.
    fn follow_with_lsp(&mut self) {
        if self.lsp.unavailable {
            return;
        }

        let Some(path) = self.editor.path.clone() else {
            return;
        };

        if path.extension().is_some_and(|extension| extension == "tw") {
            // Le serveur ne s'applique pas au langage intermédiaire : ce qu'il
            // a pu laisser n'a plus lieu d'être. La complétion de citation, elle,
            // reste la nôtre — `purge_server` ne la touche pas.
            self.lsp.purge_server();

            return;
        }

        let wanted = crate::lsp::client::Session::uri_for(&path);

        if self.lsp.document.as_deref() == Some(wanted.as_str()) {
            return;
        }

        let text = self.editor.content();

        if let Some(session) = self.lsp.session.as_mut() {
            session.open_document(&path, &text);
            self.lsp.document = Some(session.uri().to_string());
            self.lsp.vu = text;

            // Les diagnostics du document précédent ne restent pas : ses
            // boutons sauteraient aux anciennes lignes du nouveau.
            self.lsp.diagnostics.clear();
            self.lsp.reported.clear();

            return;
        }

        // Une panne n'est pas une disgrâce : changer de document tente de
        // nouveau, et un serveur relancé est repris.
        self.lsp.unavailable = false;

        if !crate::lsp::client::Session::is_available() {
            self.lsp.unavailable = true;

            return;
        }

        // L'assemblage est donné comme point d'entrée : c'est lui qui porte
        // l'en-tête et la bibliographie, et c'est lui que l'aperçu compile. Le
        // serveur et le rendu parlent donc du même document.
        match crate::lsp::client::Session::start(
            &self.project.dir,
            Some(&self.project.main),
            &path,
            &text,
        ) {
            Ok(session) => {
                self.lsp.document = Some(session.uri().to_string());
                self.lsp.session = Some(session);
                self.lsp.ready = false;
            }
            Err(error) => {
                eprintln!("Le serveur de langage n'a pas pu démarrer : {error}");

                self.lsp.unavailable = true;
            }
        }
    }

    /// Retient le projet et le chapitre ouverts pour le prochain démarrage.
    fn save_session(&self) {
        crate::project::save_session(&crate::project::Session {
            project: Some(self.project.dir.clone()),
            last_document: self.editor.path.clone(),
            theme_mode: self.theme,
            themes: self.chosen.clone(),
        });
    }

    /// Ce qu'il faut compiler pour refléter l'état réel du document.
    ///
    /// L'entrée est l'**assemblage** du projet, et non le chapitre ouvert : c'est
    /// la seule façon que l'en-tête Typst (`#set`, `#show`, réglages de page)
    /// soit toujours appliqué, avec la pagination et les compteurs qui en
    /// découlent.
    ///
    /// Le fichier ouvert est fourni en mémoire, car l'assemblage l'`#include`
    /// depuis le disque : sans cet apport, l'aperçu aurait jusqu'à une seconde
    /// de retard sur la frappe, le temps de la sauvegarde différée.
    fn compilation_unit(&self) -> (String, Vec<(PathBuf, String)>) {
        let open = self.editor.path.clone();
        let buffer = self.editor.content();

        let assembly_is_open = open.as_deref() == Some(self.project.main.as_path());

        let entry = if assembly_is_open {
            buffer.clone()
        } else {
            std::fs::read_to_string(&self.project.main).unwrap_or_default()
        };

        // `document.tw` est la source d'écriture unique même quand l'utilisateur
        // édite `main.typ` : les anciens includes de chapitres sont écartés et le
        // corps structuré est réinjecté en mémoire avant la bibliographie.
        let is_intermediate = open
            .as_deref()
            .and_then(|path| path.extension())
            .is_some_and(|extension| extension == "tw");
        let writing_document = if is_intermediate {
            self.editor.document.clone()
        } else if assembly_is_open {
            self.project
                .document
                .as_deref()
                .and_then(|path| std::fs::read_to_string(path).ok())
                .map(|source| crate::langage::parse(&source))
        } else {
            None
        };

        if let Some(document) = writing_document {
            let entry = crate::project::assembly_without_chapters(&self.project, &entry);

            return crate::langage::pipeline::compilation_unit_document(
                &self.project.dir,
                &entry,
                &document,
            );
        }

        if is_intermediate {
            let entry = crate::project::assembly_without_chapters(&self.project, &entry);

            return crate::langage::pipeline::compilation_unit(&self.project.dir, &entry, &buffer);
        }

        let overrides = match open {
            // L'assemblage est l'entrée : il n'a pas à figurer aussi en mémoire.
            Some(_) if assembly_is_open => Vec::new(),
            Some(path) => vec![(path, buffer)],
            None => Vec::new(),
        };

        (entry, overrides)
    }

    /// Empreinte de ce qui serait compilé : sert à ne relancer que si quelque
    /// chose a changé.
    pub fn compilation_fingerprint(&self) -> String {
        let (entry, overrides) = self.compilation_unit();

        // La **même** fonction que celle dont `compile` tire l'empreinte qu'il
        // rapporte, et c'est obligatoire.
        //
        // NE JAMAIS recalculer cette empreinte ici, par exemple avec
        // `render::fingerprint(&entry, &overrides)`. L'entrée compilée porte un
        // repère devant le chapitre en cours, et l'entrée brute non : les deux
        // calculs ne se rencontrent alors jamais. L'aperçu croyait avoir affaire
        // à un texte nouveau à chaque tick — plus de trois fois par seconde —
        // et recompilait sans fin, même sans la moindre frappe. Aucun délai de
        // repos ne peut corriger cela : le texte semblait toujours modifié.
        crate::typst::render::compilation_fingerprint(&self.document_dir(), &entry, &overrides)
    }

    /// Dossier résolu pour les imports et la bibliographie.
    ///
    /// C'est la racine du projet, et non le dossier du chapitre : un chapitre
    /// écrit `#include` ou `#bibliography("references.bib")` par rapport à
    /// l'assemblage, pas par rapport à lui-même.
    fn document_dir(&self) -> PathBuf {
        self.project.dir.clone()
    }

    /// Écrit le contenu courant et enregistre la session.
    fn persist(&mut self) {
        if self.editor.path.is_some() {
            if let Err(error) = self.editor.save() {
                self.note = Some(crate::app::state::Note::erreur(format!(
                    "Sauvegarde impossible : {error}"
                )));
            }

            self.save_session();
        }
    }
}

/// Nom du projet créé au premier démarrage.
const FIRST_PROJECT: &str = "mon-travail";

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // Le domaine des réglages du document a son module.
        Message::MiseEnPageOuvrir
        | Message::MiseEnPageFermer
        | Message::MiseEnPageChamp(_, _)
        | Message::MiseEnPageBascule(_)
        | Message::MiseEnPagePanneau(_)
        | Message::MiseEnPageTypographieDefaut(_)
        | Message::MiseEnPageAccrocher(_)
        | Message::MiseEnPageAppliquer
        | Message::MiseEnPageImporter
        | Message::MiseEnPageImportee(_)
        | Message::MiseEnPageExporter
        | Message::MiseEnPageNote(_) => {
            crate::app::reglages::update(state, message).unwrap_or_else(Task::none)
        }

        // Le domaine des fichiers et du projet a son module.
        Message::NewFile
        | Message::OpenFile
        | Message::FileOpened(_)
        | Message::OpenChapter(_)
        | Message::ChapterOpened(_, _)
        | Message::NewChapter
        | Message::NewProject
        | Message::OpenProject(_)
        | Message::RefreshProject
        | Message::DeleteChapter(_)
        | Message::StartRename(_)
        | Message::RenameChanged(_)
        | Message::RenameCommit
        | Message::RenameCancel
        | Message::InsertCitation(_)
        | Message::SaveFile
        | Message::SaveAs
        | Message::FileSaved(_)
        | Message::ExportPdf
        | Message::SymbolRenameStart
        | Message::SymbolRenameChanged(_)
        | Message::SymbolRenameAsk
        | Message::SymbolRenameApply
        | Message::SymbolRenameCancel
        | Message::PdfExported(_) => crate::app::fichiers::update(state, message),

        // L'édition d'une référence : une fenêtre, des champs, et une
        // bibliothèque locale à écrire — jamais une notice Zotero.
        Message::ModifierReference(cle) => {
            let entree = state
                .bibliography
                .iter()
                .find(|entree| entree.key == cle)
                .cloned();

            let Some(entree) = entree else {
                return Task::none();
            };

            let (nom, prenom) = entree
                .auteurs
                .first()
                .cloned()
                .unwrap_or((String::new(), String::new()));

            // La bibliographie vient-elle de Zotero ? Seule une bibliothèque
            // locale s'écrit : la règle tient dans le champ lui-même.
            let locale = state.zotero.source == crate::zotero::Source::Locale;

            state.edition_reference = Some(crate::app::state::EditionReference {
                cle: entree.key,
                kind: entree.kind,
                nom,
                prenom,
                titre: entree.titre,
                titre_court: entree.titre_court.unwrap_or_default(),
                annee: entree.annee.unwrap_or_default(),
                editeur: entree.editeur.unwrap_or_default(),
                lieu: entree.lieu.unwrap_or_default(),
                creation: false,
                locale,
            });

            // La fiche s'ouvre sur son premier champ : le clavier a déjà
            // quelqu'un à qui parler.
            iced::widget::operation::focus(crate::ui::reference::id_premier_champ())
        }
        Message::EditionReferenceChamp(champ, valeur) => {
            if let Some(edition) = state.edition_reference.as_mut() {
                match champ.as_str() {
                    "cle" => edition.cle = valeur,
                    "kind" => edition.kind = valeur,
                    "nom" => edition.nom = valeur,
                    "prenom" => edition.prenom = valeur,
                    "titre" => edition.titre = valeur,
                    "titre_court" => edition.titre_court = valeur,
                    "annee" => edition.annee = valeur,
                    "editeur" => edition.editeur = valeur,
                    "lieu" => edition.lieu = valeur,
                    _ => {}
                }
            }

            Task::none()
        }
        Message::ZoteroLierOuvrir
        | Message::ZoteroCollections(_)
        | Message::ZoteroChoisir(_, _)
        | Message::ZoteroExport(_, _, _)
        | Message::ZoteroDelier
        | Message::ZoteroLiaisonFermer => liaison_zotero(state, message),

        Message::EditionReferenceNouveau => {
            // Seule une bibliothèque locale s'écrit : la création suppose
            // d'écrire, donc un fichier à soi.
            let locale = state.zotero.source == crate::zotero::Source::Locale;

            if !locale {
                return Task::none();
            }

            state.edition_reference = Some(crate::app::state::EditionReference {
                cle: String::new(),
                kind: "book".to_string(),
                nom: String::new(),
                prenom: String::new(),
                titre: String::new(),
                titre_court: String::new(),
                annee: String::new(),
                editeur: String::new(),
                lieu: String::new(),
                creation: true,
                locale,
            });

            iced::widget::operation::focus(crate::ui::reference::id_premier_champ())
        }
        Message::EditionReferenceFermer => {
            state.edition_reference = None;

            Task::none()
        }
        Message::EditionReferenceEnregistrer => {
            let Some(edition) = state.edition_reference.clone() else {
                return Task::none();
            };

            if !edition.locale {
                // Une notice Zotero est à Zotero : on ne l'écrit pas ici.
                state.edition_reference = None;

                return Task::none();
            }

            let Some(chemin) = state.project.bibliography.clone() else {
                state.edition_reference = None;

                return Task::none();
            };

            let champs = crate::project::ReferenceChamps {
                cle: edition.cle.clone(),
                kind: edition.kind.clone(),
                nom: edition.nom.clone(),
                prenom: edition.prenom.clone(),
                titre: edition.titre.clone(),
                titre_court: edition.titre_court.clone(),
                annee: edition.annee.clone(),
                editeur: edition.editeur.clone(),
                lieu: edition.lieu.clone(),
            };

            let ecrit: Result<(), String> = if edition.creation {
                crate::project::nouvelle_reference(&chemin, &champs)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            } else {
                crate::project::maj_reference(&chemin, &edition.cle, &champs)
                    .map_err(|e| e.to_string())
            };

            if ecrit.is_ok() {
                // La clé retenue se voit aussitôt : elle vient d'être
                // citable, et c'est celle-là qu'on insérera.
                state.bibliography = crate::project::parse_bibliography(&chemin);
            }

            match ecrit {
                Err(erreur) => {
                    // La saisie reste : un formulaire qui se ferme en
                    // échec jette le travail de l'utilisateur.
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement de la référence impossible : {erreur}"
                    )));
                }
                Ok(()) => {
                    state.edition_reference = None;
                    state.bibliography = crate::project::parse_bibliography(&chemin);
                    state.note = Some(crate::app::state::Note::info("Référence enregistrée."));
                }
            }

            Task::none()
        }

        // Le domaine des thèmes a son module. Le motif ne **rien** lier : c'est
        // ce qui permet de repasser le message entier, sans le dupliquer.
        Message::SystemTheme(_)
        | Message::CycleTheme
        | Message::RefreshThemes
        | Message::OpenThemesFolder
        | Message::SelectTheme(_)
        | Message::LoadTheme
        | Message::ThemeLoaded(_)
        | Message::DeleteTheme(_) => crate::app::themes::update(state, message),

        Message::Editor(message) => {
            let content_changed = crate::editor::EditorState::message_may_change_content(&message);

            // Un `@` qui vient d'être tapé est le déclencheur du composeur ;
            // les autres frappes peuvent être l'édition d'une citation.
            let declencheur_tape =
                matches!(message, iced_code_editor::Message::CharacterInput('@'));

            // Un clic dans le texte a le dernier mot sur le composeur.
            let clic_dans_le_texte = matches!(
                message,
                iced_code_editor::Message::MouseClick(_)
                    | iced_code_editor::Message::DoubleClick(_)
                    | iced_code_editor::Message::TripleClick(_)
            );

            // Le texte a peut-être changé : le compte de mots est à refaire, au
            // prochain tic, et non à chaque image.
            state.render.words_pending = true;

            // L'aperçu suit ce qu'on lit — mais seulement quand le suivi est
            // demandé : le bouton de la barre d'outils en décide, et écrire
            // ailleurs dans l'assemblage ne doit pas arracher l'aperçu.
            let suivre = matches!(message, iced_code_editor::Message::Scrolled(_))
                && state.layout.follow_preview;

            let task = state.editor.update(message).map(Message::Editor);

            if clic_dans_le_texte {
                if state.composeur.is_some() {
                    // La main est rendue au document, et la note en cours ne
                    // s'écrit pas — le même geste qu'Échap.
                    return Task::batch([task, fermer_composeur(state, None)]);
                }

                // **Le clic sur un glyphe est la seule porte d'édition d'une
                // note.** Une citation ouvre son composeur ; un pied de page se
                // déplie sous le curseur. Ailleurs, rien ne bouge — et surtout
                // plus jamais l'édition d'une citation au fil de la frappe.
                let contenu = state.editor.content();
                let notes = crate::langage::marques::notes(&contenu);

                if let Some(rang) = state.editor.note_au_clic(&notes) {
                    let note = notes[rang].clone();

                    return match note.nature {
                        crate::langage::marques::Nature::Appel => {
                            let ouverture = crate::langage::parse::appel_autour(
                                &contenu,
                                note.debut + 1,
                            )
                            .map(|(references, debut, fin)| {
                                ouvrir_edition(state, references, debut, fin)
                            })
                            .unwrap_or_else(Task::none);

                            Task::batch([task, ouverture])
                        }
                        crate::langage::marques::Nature::PiedDePage => Task::batch([
                            task,
                            state
                                .editor
                                .set_cursor_offset(note.debut + 1)
                                .map(Message::Editor),
                        ]),
                    };
                }
            }

            let suivi = if suivre {
                follow_editor_scroll(state)
            } else {
                Task::none()
            };

            // La frappe filtre la liste ; la navigation au curseur la ferme.
            // Le défilement, lui, la garde ouverte et met à jour son ancre.
            let completion = request_completion(state, content_changed, declencheur_tape);

            if crate::app::epreuves::actif("lsp") {
                report_lsp(state);
            }

            Task::batch(vec![task, completion, suivi])
        }

        Message::Format(action) => {
            if !state.editor.is_intermediate() {
                return Task::none();
            }

            // Le bouton « citation » est un `@` tapé par la main de l'outil.
            let declencheur_tape =
                matches!(action, crate::app::message::FormatAction::Citation);

            state.render.words_pending = true;
            let edit = state.editor.apply_format(action).map(Message::Editor);
            let completion = request_completion(state, true, declencheur_tape);

            Task::batch([edit, completion])
        }

        Message::CompletionMove(delta) => {
            if let Some(open) = state.lsp.completion.as_mut() {
                open.move_selection(delta);
            }

            Task::none()
        }

        Message::CompletionAccept => accept_completion(state),

        Message::CompletionPick(index) => {
            if let Some(open) = state.lsp.completion.as_mut()
                && index < open.shown.len()
            {
                open.selected = index;
            }

            accept_completion(state)
        }

        Message::CompletionDismiss => {
            close_completion(state);

            Task::none()
        }

        Message::ComposeurSaisie(_)
        | Message::ComposeurAvancer
        | Message::ComposeurTerminer
        | Message::ComposeurFermer
        | Message::ComposeurDeplacer(_)
        | Message::ComposeurPrendre(_)
        | Message::ComposeurChoisir(_) => composeur_update(state, message),

        Message::ScaleFactor(scale_factor) => {
            // La densité du rendu en découle, et elle décide de la mémoire d'une
            // page : une A4 pèse 8 Mo de pixels à 2 px/pt, dont le backend fait
            // une texture d'environ 14 Mio.
            if (scale_factor - state.scale_factor).abs() < 0.001 {
                return Task::none();
            }

            state.scale_factor = scale_factor;

            // Les images en cache appartiennent à l'ancienne densité : leur
            // empreinte ne veut plus rien dire, et toutes les pages sont à
            // refaire. Elles ne sont pas jetées pour autant — une image à
            // l'ancienne densité reste juste à l'écran, simplement remise à
            // l'échelle, jusqu'à l'arrivée de la nouvelle. Changer d'écran ne
            // doit pas faire disparaître le document.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            state.render.invalidate_rasterization(total);

            rasterize_visible(state)
        }

        Message::LspPoll => {
            let events: Vec<crate::lsp::client::Event> = match state.lsp.session.as_ref() {
                Some(session) => session.events().try_iter().collect(),
                None => Vec::new(),
            };

            // Un message par événement, plutôt qu'un appel direct : tout ce qui
            // change l'état passe par la même porte, et c'est cette porte qu'un
            // test peut franchir pour éprouver l'application.
            Task::batch(
                events
                    .into_iter()
                    .map(|event| Task::done(Message::Lsp(event))),
            )
        }

        Message::Lsp(event) => apply_lsp(state, event),

        Message::MemoryTick => {
            report_memory(state);
            report_lsp(state);

            Task::none()
        }

        Message::RenderTick => {
            // Les épreuves peuvent piloter l'éditeur elles-mêmes : un pas par
            // tic, et elles rendent la main aussitôt.
            if let Some(tache) = crate::app::epreuves::driver(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_renommage(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_reglages(state) {
                return tache;
            }

            // Une attente qui dure est une attente morte : un serveur lent
            // ou muet ne doit pas tenir l'application en haleine ni
            // empêcher la liste de se fermer.
            if state.lsp.awaiting
                && state
                    .lsp
                    .depuis
                    .is_some_and(|depuis| depuis.elapsed() > std::time::Duration::from_secs(5))
            {
                state.lsp.awaiting = false;
                state.lsp.depuis = None;

                return update(state, Message::CompletionDismiss);
            }

            if let Some(tache) = crate::app::epreuves::driver_composeur(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_glyphe(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_annulation(state) {
                return tache;
            }

            // Le serveur de langage suit **le document que l'éditeur tient**,
            // quel que soit le chemin par lequel on y est arrivé. S'accrocher
            // aux fonctions qui *ouvrent* un document laissait passer le
            // démarrage, qui ouvre le premier d'une autre façon : le serveur
            // n'était alors jamais lancé, et rien ne le disait.
            state.follow_with_lsp();

            // Le compte de mots est une donnée de la vue, et la vue n'a pas le
            // droit de modifier l'état : il se calcule donc ici, au rythme des
            // tics, et seulement quand le texte a changé. Le calculer chez elle,
            // c'était cloner le document entier à chaque image — deux fois,
            // puisque la barre d'état et le panneau le demandent chacun.
            if state.render.words_pending {
                let content = state.editor.content();

                state.render.words = Some(state.content_word_count(&content));
                state.render.words_pending = false;

                // C'est **le** moment où l'on sait que le texte a changé, et le
                // seul endroit où l'on en a la preuve : le serveur de langage
                // voit donc le document au même rythme. Le synchroniser à
                // chaque frappe lui renverrait tout le fichier à chaque touche.
                if let Some(session) = state.lsp.session.as_mut() {
                    session.did_change(&content);
                }
            }

            if state.render.pending {
                // Une seule compilation à la fois ; si le contenu a changé
                // pendant ce temps, le tick suivant relancera.
                return Task::none();
            }

            // L'empreinte couvre l'assemblage et le fichier ouvert : comparer le
            // seul chapitre rendrait l'aperçu sourd à toute frappe, puisque
            // l'assemblage ne change pas quand on écrit dans un chapitre.
            let fingerprint = state.compilation_fingerprint();
            let newlines = state.editor.content().matches('\n').count();

            if !state
                .render
                .compile_due(&fingerprint, newlines, std::time::Instant::now())
            {
                return Task::none();
            }

            state.render.pending = true;

            compile_task(state)
        }

        Message::RenderDone(outcome) => {
            state.render.pending = false;
            state.render.done = true;
            let chapter_page = outcome.chapter_page;

            state.render.content = outcome.content;
            state.render.errors = outcome.errors;
            state.render.warnings = outcome.warnings;

            // Un échec ne vide pas l'aperçu : la dernière image bonne
            // reste affichée sous les erreurs, et ses textures gardent
            // leur place jusqu'à ce qu'une nouvelle compilation les
            // remplace. C'était la promesse du double tampon, tenue.
            if outcome.document.is_some() {
                state.render.document = outcome.document;
            }

            // Nouveau document : toute image en cache est dorénavant suspecte.
            // On ne les supprime pas pour autant — elles restent affichées
            // (double tampon) — mais elles sont marquées périmées, et seules
            // les pages visibles seront rasterisées à nouveau.
            state.render.revision += 1;

            // Les tâches de l'ancienne révision peuvent encore finir, mais
            // chacune porte sa propre clé et ne réserve plus de place pour la
            // nouvelle. Le cache et les pages périmées, eux, sont bornés au
            // nombre de pages actuel.
            state.render.rendering.clear();

            if let Some(document) = state.render.document.as_deref() {
                let total = crate::typst::render::page_count(document);

                state.render.page_cache.retain(|index, _| *index < total);
                state.render.stale.retain(|index| *index < total);
            }


            state.render.chapter_page = chapter_page;
            state.render.outline = source_outline(state, outcome.outline);

            // Toutes les pages sont périmées, et pas seulement les visibles :
            // l'image d'une page hors écran appartient elle aussi à la
            // révision précédente, et se retrouverait affichée telle quelle en
            // y revenant. Le coût reste borné, `rasterize_visible` ne rendant
            // que les pages visibles.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            for index in 0..total {
                state.render.stale.insert(index);
            }

            // Rendre les pages visibles, et amener l'aperçu sur le chapitre
            // qu'on écrit s'il vient de changer.
            Task::batch([rasterize_visible(state), follow_chapter(state)])
        }

        Message::PageRendered(revision, index, image) => {
            state.render.finish_page_render(revision, index);

            // Une compilation est arrivée pendant le rendu : cette image
            // appartient à un document dépassé, on la jette plutôt que de
            // faire régresser l'affichage.
            if state.render.image_perimee(revision, index) {
                return rasterize_visible(state);
            }

            state.render.stale.remove(&index);

            if let Some(image) = image {
                // Le papier se lit sur le premier pixel de la première page
                // rendue : c'est le coin supérieur gauche, donc le fond du
                // document. Une seule fois — il ne change pas d'une page à
                // l'autre — et il sert ensuite à remplir les cadres de page
                // avant que leur texture n'existe.
                if state.render.paper.is_none() {
                    state.render.paper = paper_colour(&image);
                }

                // Pixels identiques à ceux déjà en place : la page n'a pas
                // changé. On garde le **même** `Handle`, donc la même texture,
                // et le compositeur n'a rien à refaire. Remplacer l'image
                // faisait naître une texture par compilation et par page ; à
                // plusieurs compilations par seconde, c'est ce qui se voyait
                // comme un clignotement.
                if state.render.image_hashes.get(&index) == Some(&image.hash) {
                    return Task::none();
                }

                state.render.image_hashes.insert(index, image.hash);

                // Un seul `Handle` par page, créé ici et réutilisé tel quel
                // par `view()` : le backend retrouve sa texture et ne la
                // retéléverse pas. L'ancienne image reste en place jusqu'à
                // ce que celle-ci la remplace — l'écran ne passe jamais par
                // le vide.
                state.render.page_cache.insert(
                    index,
                    Handle::from_rgba(image.width, image.height, image.pixels),
                );

                // Une image peut arriver après qu'on a défilé ailleurs :
                // la refuser tout de suite, plutôt que de la garder pour rien.
                state.render.bound_page_cache();
            }

            // Une place s'est libérée : ce qui attendait part.
            rasterize_visible(state)
        }

        Message::ViewportChanged(first, last, bounds) => {
            // « Aucune page » envoie (0, 0) : ce n'est pas une plage,
            // c'est l'absence d'information. Un document tenu dans la
            // fenêtre n'en reçoit jamais d'autre, et ses pages 2+ ne
            // venaient qu'après un défilement manuel.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            let (first, last) = if (first, last) == (0, 0) && total > 1 {
                (0, 2.min(total - 1))
            } else {
                (first, last)
            };

            let unchanged = state.render.visible == Some((first, last))
                && state.render.viewport == Some(bounds);

            state.render.visible = Some((first, last));

            // La place d'une page a changé : la taille d'affichage change, donc
            // la densité dont le rendu a besoin. Les images en cache
            // appartiennent à l'ancienne — leur empreinte ne veut plus rien
            // dire — mais elles restent affichées, simplement remises à
            // l'échelle, jusqu'à l'arrivée des nouvelles.
            if state.render.viewport != Some(bounds) {
                state.render.viewport = Some(bounds);

                let total = state
                    .render
                    .document
                    .as_deref()
                    .map_or(0, crate::typst::render::page_count);

                // Une rasterisation lancée à l'ancienne taille ne satisfait
                // pas le nouveau viewport, même si elle porte sur la même page.
                state.render.invalidate_rasterization(total);
            }

            // **Rien de neuf : ne rien faire.** C'est la garde qui empêche un
            // cycle. Un viewport identique — même place, mêmes pages — signifie
            // que rien n'a bougé ; le traiter relançait des rasterisations, qui
            // produisaient des images neuves, qui reconstruisaient la vue, qui
            // signalait le même viewport : un tour de plus, plusieurs mégaoctets
            // à chaque fois.
            if unchanged {
                return Task::none();
            }

            // Le cache d'images est borné aux pages visibles et à leur
            // voisinage : c'est ici, où l'on sait ce qu'on voit, qu'on retire
            // le reste. Les `Handle` relâchés libèrent leurs textures.
            state.render.bound_page_cache();

            // Une page qui entre dans le viewport sans image valide doit
            // être rasterisée ; une page déjà en cache et à jour ne l'est
            // plus (c'est ce qui rendait chaque défilement coûteux).
            for index in first..=last {
                if !state.render.page_cache.contains_key(&index) {
                    state.render.stale.insert(index);
                }
            }

            rasterize_visible(state)
        }

        Message::PreviewZoom(zoom) => set_preview_zoom(state, zoom),

        Message::PreviewZoomStep(step) => {
            let box_size = state
                .render
                .viewport
                .unwrap_or_else(crate::ui::preview::default_viewport);
            let page = state
                .render
                .document
                .as_deref()
                .and_then(|document| {
                    let index = state.render.visible.map_or(0, |(first, _)| first);
                    crate::typst::render::page_size(document, index)
                })
                .unwrap_or((595.3, 841.9));
            let zoom = crate::ui::preview::stepped_zoom(state.render.zoom, step, box_size, page);

            set_preview_zoom(state, zoom)
        }

        Message::SaveTick => {
            // Écriture différée : une seule par tick, et seulement si le
            // document a réellement changé. **La compilation ne décide de
            // rien** : un document en erreur Typst s'écrit quand même, et
            // c'est même là qu'il en a le plus besoin — une coupure ne
            // doit jamais emporter ce qui a été tapé.
            if state.editor.is_modified() {
                state.persist();
                state.refresh_context();

                // Le serveur suit les écritures : c'est ainsi qu'il sait
                // recompiler le projet, et que les diagnostics des *autres*
                // chapitres se mettent à jour.
                if let Some(session) = state.lsp.session.as_mut() {
                    session.did_save();
                }
            }

            Task::none()
        }

        Message::ToggleSidebar => {
            state.layout.sidebar_visible = !state.layout.sidebar_visible;

            Task::none()
        }

        Message::TogglePreview => {
            state.layout.preview_visible = !state.layout.preview_visible;

            Task::none()
        }

        Message::TogglePreviewSync => {
            state.layout.follow_preview = !state.layout.follow_preview;

            if crate::app::epreuves::actif("lsp") {
                println!(
                    "[vue] suivi de l'aperçu : {}",
                    if state.layout.follow_preview {
                        "actif"
                    } else {
                        "coupé"
                    }
                );
            }

            Task::none()
        }

        Message::SetPreviewPosition(position) => {
            state.layout.preview_position = position;

            Task::none()
        }

        Message::SplitterPressed(which) => {
            // L'origine est la dernière position connue du curseur : elle est
            // tenue à jour à chaque mouvement de souris, y compris hors
            // glissement. Le premier mouvement pendant le glissement sert donc
            // à déplacer, pas à repérer l'origine — un glissement court ne
            // reste plus sans effet.
            let vertical = state.layout.preview_position == PreviewPosition::Bottom
                && which == Splitter::Preview;

            let origin = if vertical {
                state.layout.cursor.y
            } else {
                state.layout.cursor.x
            };

            // L'étendue de départ est mémorisée ici, une fois pour toutes : les
            // mouvements suivants sont des deltas depuis cette origine, et les
            // ajouter à la valeur courante les cumulerait.
            let extent = if vertical {
                state.layout.preview_height
            } else {
                match which {
                    Splitter::Sidebar => state.layout.sidebar_width,
                    Splitter::Preview => state.layout.preview_width,
                }
            };

            state.layout.dragging = Some(Drag {
                which,
                origin,
                extent,
            });

            Task::none()
        }

        Message::MouseMoved(at) => {
            let Some(drag) = state.layout.dragging else {
                // Aucun séparateur tenu : on retient seulement la position du
                // curseur, pour que la prochaine saisie parte du bon endroit.
                state.layout.cursor = at;

                return Task::none();
            };

            // L'axe dépend de la disposition : en colonnes, l'aperçu se règle
            // en hauteur.
            let vertical = state.layout.preview_position == PreviewPosition::Bottom
                && drag.which == Splitter::Preview;

            let delta = if vertical {
                at.y - drag.origin
            } else {
                at.x - drag.origin
            };

            state.layout.cursor = at;

            // Tout part de l'étendue de départ, jamais de la valeur courante :
            // le delta est mesuré depuis l'origine.
            let dragged = LayoutState::dragged_extent(drag.which, vertical, drag.extent, delta);

            if vertical {
                state.layout.preview_height = dragged;
            } else {
                match drag.which {
                    Splitter::Sidebar => state.layout.sidebar_width = dragged,
                    Splitter::Preview => state.layout.preview_width = dragged,
                }
            }

            Task::none()
        }

        Message::SplitterReleased => {
            state.layout.dragging = None;

            // **Le relâchement va aussi à l'éditeur.** Si le bouton a été
            // relâché ailleurs que sur lui — sur une ligne de la liste de
            // complétion, par exemple —, il ne le voit jamais, et reste en
            // glissement : ensuite, chaque mouvement de souris déplace le caret
            // et la vue se met à le suivre. Le message est sans effet s'il n'y
            // avait rien à relâcher.
            Task::done(Message::Editor(iced_code_editor::Message::MouseRelease))
        }

        Message::GoToDefinition => request_definition(state),

        Message::ShowHover => request_hover(state),

        Message::BlocMonter => state
            .editor
            .deplacer_le_bloc(true)
            .map(Message::Editor),
        Message::BlocDescendre => state
            .editor
            .deplacer_le_bloc(false)
            .map(Message::Editor),

        Message::TitreSuivant => sauter_titre(state, 1),
        Message::TitrePrecedent => sauter_titre(state, -1),

        Message::GotoHeading(index) => {
            let Some(entry) = state.render.outline.get(index).cloned() else {
                return Task::none();
            };

            // Le sommaire sert à se repérer dans **les deux** vues : le curseur
            // va à la ligne, l'aperçu à la page.
            state.render.followed_page = Some(entry.page);

            let aller = match entry.file {
                Some(path) => {
                    crate::app::assistance::jump_to(state, &path, entry.line.saturating_sub(1), 0)
                }
                None => Task::none(),
            };

            Task::batch([aller, scroll_preview_to_page(state, entry.page)])
        }

        Message::WindowResized(size) => {
            state.window_size = size;

            Task::none()
        }

        Message::WindowOpened(id) => {
            state.window = Some(id);

            // L'échelle de l'écran est redemandée ici : une fenêtre peut
            // s'ouvrir sur un écran dont l'échelle diffère de celle des autres,
            // et c'est elle qui règle la densité du rendu.
            iced::window::scale_factor(id).map(Message::ScaleFactor)
        }

        Message::ToggleFocusMode => {
            state.layout.focus_mode = !state.layout.focus_mode;

            // Le mode focus est un plein écran : masquer les panneaux ne suffit
            // pas, la barre de titre et le reste du bureau resteraient dans le
            // champ de vision.
            let mode = if state.layout.focus_mode {
                iced::window::Mode::Fullscreen
            } else {
                iced::window::Mode::Windowed
            };

            match state.window {
                Some(id) => iced::window::set_mode(id, mode),
                // La fenêtre n'est pas encore connue : on la demande, puis on
                // appliquera le mode. Sans cela, la première bascule au tout
                // début du démarrage resterait sans effet.
                None => iced::window::latest().and_then(move |id| iced::window::set_mode(id, mode)),
            }
        }

        Message::GotoError(line, column) => {
            // Les positions de typst sont 1-based ; l'éditeur attend du
            // 0-based.
            let _ = state.editor.update(iced_code_editor::Message::GotoPosition(
                line.saturating_sub(1),
                column.saturating_sub(1),
            ));

            Task::none()
        }

        Message::RawKey(key, modifiers) => handle_shortcut(state, key, modifiers),
    }
}

/// Titre par défaut d'un nouveau chapitre.
///
/// « Chapitre N » avec N après le dernier numéro utilisé : le titre est une
/// amorce à réécrire, pas une décision à prendre avant d'écrire.
fn default_chapter_title(project: &crate::project::Project) -> String {
    // Le numéro vient du disque, comme celui du nom de fichier : sans cela, le
    // titre proposé et le nom réel pourraient annoncer deux numéros différents.
    format!("Chapitre {}", crate::project::next_chapter_order(project))
}

/// Nom libre pour un nouveau projet.
///
/// « projet », puis « projet-2 », etc. : un nom doit exister avant d'être
/// choisi, sinon créer un projet écraserait le précédent.
fn next_project_name(projects: &[crate::project::Project]) -> String {
    for n in 1..=1000 {
        let name = if n == 1 {
            "projet".to_string()
        } else {
            format!("projet-{n}")
        };

        if !projects.iter().any(|project| project.name == name) {
            return name;
        }
    }

    "projet-final".to_string()
}

/// Raccourcis globaux : Ctrl+S/N/O/E.
///
/// Ils n'agissent que sur des événements qu'aucun widget n'a consommés, donc
/// jamais en conflit avec les raccourcis propres de l'éditeur.
/// Le panneau de liaison à Zotero : ouvrir, lire, choisir, et ce qui cloche.
fn liaison_zotero(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ZoteroLierOuvrir => {
            // Seule une bibliothèque locale se lie : une notice Zotero
            // déjà pointée n'a pas de deuxième chef.
            if state.zotero.source != crate::zotero::Source::Locale {
                return Task::none();
            }

            state.liaison = Some(crate::app::state::LiaisonZotero::Chargement);

            Task::perform(
                async { crate::zotero::client::collections() },
                |resultat| Message::ZoteroCollections(resultat.map_err(|e| e.to_string())),
            )
        }

        Message::ZoteroCollections(resultat) => match resultat {
            Ok(collections) => {
                state.liaison = Some(crate::app::state::LiaisonZotero::Choix(collections));
                Task::none()
            }
            Err(erreur) => {
                state.liaison = Some(crate::app::state::LiaisonZotero::Erreur(erreur));
                Task::none()
            }
        },

        Message::ZoteroChoisir(cle, nom) => {
            // L'export vient de Zotero : on le demande avant d'écrire quoi
            // que ce soit au projet.
            state.liaison = Some(crate::app::state::LiaisonZotero::Chargement);

            let (demande, rendu) = (cle.clone(), (cle, nom));

            Task::perform(
                async move {
                    crate::zotero::client::bibtex(&demande).map_err(|e| e.to_string())
                },
                move |resultat| Message::ZoteroExport(rendu.0, rendu.1, resultat),
            )
        }

        Message::ZoteroExport(cle, nom, resultat) => match resultat {
            Ok(bibtex) => {
                let dir = state.project.dir.clone();

                if let Err(erreur) =
                    crate::project::ecrire_atomique(
                        &dir.join("references-zotero.bib"),
                        &bibtex,
                    )
                {
                    state.liaison = Some(crate::app::state::LiaisonZotero::Erreur(format!(
                        "La bibliothèque n'a pas pu être écrite : {erreur}"
                    )));

                    return Task::none();
                }

                if let Err(erreur) =
                    crate::zotero::lier(&dir, &crate::zotero::Lien { cle, nom })
                {
                    state.liaison = Some(crate::app::state::LiaisonZotero::Erreur(format!(
                        "La liaison n'a pas pu être gardée : {erreur}"
                    )));

                    return Task::none();
                }

                // La liaison est faite : le projet la porte, et la
                // bibliothèque montrée devient celle de Zotero — lue, jamais
                // écrite.
                state.project = crate::project::load(&dir);
                state.bibliography = state
                    .project
                    .bibliography
                    .as_ref()
                    .map(|path| crate::project::parse_bibliography(path))
                    .unwrap_or_default();
                state.zotero = crate::zotero::Etat {
                    source: crate::zotero::Source::Zotero,
                    lie: true,
                };
                state.liaison = None;

                Task::none()
            }
            Err(erreur) => {
                state.liaison = Some(crate::app::state::LiaisonZotero::Erreur(erreur));
                Task::none()
            }
        },

        Message::ZoteroDelier => {
            let dir = state.project.dir.clone();
            let destination = crate::zotero::delier(&dir);

            if let Some(destination) = destination {
                state.note = Some(crate::app::state::Note::info(format!(
                    "Liaison déplacée vers la corbeille : {}",
                    destination.display()
                )));
            }

            state.project = crate::project::load(&dir);
            state.bibliography = state
                .project
                .bibliography
                .as_ref()
                .map(|path| crate::project::parse_bibliography(path))
                .unwrap_or_default();
            state.zotero = crate::zotero::Etat::default();
            state.liaison = None;

            Task::none()
        }

        Message::ZoteroLiaisonFermer => {
            state.liaison = None;
            Task::none()
        }

        _ => Task::none(),
    }
}

fn handle_shortcut(
    state: &mut State,
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    // La fiche de référence, quand elle est ouverte : un formulaire se
    // conduit au clavier — `Tabulation` circule dans les champs, `Entrée`
    // enregistre, `Échap` ferme. Ses touches lui appartiennent.
    if state.liaison.is_some() {
        use iced::keyboard::{Key, key::Named};

        if let Key::Named(Named::Escape) = key {
            return update(state, Message::ZoteroLiaisonFermer);
        }
    }

    if state.edition_reference.is_some() {
        use iced::keyboard::{Key, key::Named};

        match key {
            Key::Named(Named::Tab) if modifiers.shift() => return iced::widget::operation::focus_previous(),
            Key::Named(Named::Tab) => return iced::widget::operation::focus_next(),
            Key::Named(Named::Enter) => return update(state, Message::EditionReferenceEnregistrer),
            Key::Named(Named::Escape) => return update(state, Message::EditionReferenceFermer),
            _ => {}
        }
    }

    // La liste de complétion, quand elle est ouverte, prend les touches dont
    // elle a besoin. L'éditeur les a **déclinées** (patch 4 du crate) : elles
    // arrivent donc jusqu'ici, et c'est ici qu'on décide de ce qu'elles font.
    if let Some(task) = completion_shortcut(state, &key, modifiers) {
        return task;
    }

    handle_shortcut_inner(state, key, modifiers)
}

/// Les touches de la liste de complétion, quand elle est ouverte.
///
/// `None` quand la liste est fermée, ou quand la touche ne la concerne pas : la
/// frappe continue alors son chemin, et la liste se filtrera à la réponse du
/// serveur — c'est ce qui permet d'écrire `figure` en voyant la liste se
/// réduire.
fn completion_shortcut(
    state: &mut State,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Task<Message>> {
    use iced::keyboard::{Key, key::Named};

    // Le composeur de notes, quand il est ouvert, prend ses touches — et la
    // liste de complétion, elle, n'a plus rien à faire.
    if state.composeur.is_some() {
        return composeur_touche(state, key, modifiers);
    }

    state.lsp.completion.as_ref()?;

    // Ctrl+Espace accepte : c'est le geste qu'on a dans les doigts quand la
    // liste est ouverte, et il ne sert à rien d'autre ici.
    if modifiers.control() && matches!(key, Key::Character(text) if text.as_str() == " ") {
        return Some(update(state, Message::CompletionAccept));
    }

    match key {
        Key::Named(Named::ArrowUp) => Some(update(state, Message::CompletionMove(-1))),
        Key::Named(Named::ArrowDown) => Some(update(state, Message::CompletionMove(1))),
        Key::Named(Named::Enter) | Key::Named(Named::Tab) => {
            Some(update(state, Message::CompletionAccept))
        }
        Key::Named(Named::Escape) => Some(update(state, Message::CompletionDismiss)),
        _ => None,
    }
}

fn handle_shortcut_inner(
    state: &mut State,
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    // F11 bascule le mode focus sans modificateur : c'est la touche consacrée
    // au plein écran, et le geste doit rester d'une seule main.
    if let iced::keyboard::Key::Named(iced::keyboard::key::Named::F11) = key {
        return update(state, Message::ToggleFocusMode);
    }

    // Échap en sort : en mode focus il n'y a plus de barre d'outils, donc plus
    // de bouton à cliquer — sans cette sortie, l'utilisateur serait enfermé.
    // F12 va à la définition : c'est la touche consacrée, et elle ne demande
    // aucune combinaison — le geste reste d'une seule main.
    if let iced::keyboard::Key::Named(iced::keyboard::key::Named::F12) = key {
        return update(state, Message::GoToDefinition);
    }

    // Le bloc entier se déplace d'un bloc : `Ctrl+Maj+Haut/Bas`. Les
    // flèches sont prises par le curseur et les lignes — celles-ci
    // ajoutent la majuscule que rien d'autre ne réclame.
    if modifiers.control()
        && modifiers.shift()
        && matches!(
            key,
            iced::keyboard::Key::Named(
                iced::keyboard::key::Named::ArrowUp | iced::keyboard::key::Named::ArrowDown
            )
        )
    {
        let vers_le_haut = matches!(
            key,
            iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp)
        );

        return update(state, if vers_le_haut { Message::BlocMonter } else { Message::BlocDescendre });
    }

    if let iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) = key {
        // Ce qui s'est ouvert en dernier se ferme en premier : la liste, puis le
        // survol, et le mode focus en dernier — l'utilisateur ne doit jamais
        // avoir à chercher par où sortir.
        if state.lsp.completion.take().is_some() {
            state.editor.editor.set_reserved_keys(Vec::new());

            return Task::none();
        }

        if state.lsp.hover.take().is_some() {
            return Task::none();
        }

        if state.layout.focus_mode {
            return update(state, Message::ToggleFocusMode);
        }

        return Task::none();
    }

    if !modifiers.control() || modifiers.alt() || modifiers.logo() {
        return Task::none();
    }

    let iced::keyboard::Key::Character(character) = &key else {
        return Task::none();
    };

    match character.to_lowercase().as_str() {
        "s" => {
            state.persist();
            state.refresh_context();

            Task::none()
        }
        "i" => update(state, Message::ShowHover),
        "n" => update(state, Message::NewFile),
        "o" => update(state, Message::OpenFile),
        "e" => update(state, Message::ExportPdf),
        // Le titre suivant, ou le précédent avec Maj : la table des matières
        // se parcourt sans quitter le clavier.
        "t" => {
            if modifiers.shift() {
                update(state, Message::TitrePrecedent)
            } else {
                update(state, Message::TitreSuivant)
            }
        }
        _ => Task::none(),
    }
}

/// Va au titre suivant ou précédent : le curseur y va, l'aperçu suit —
/// exactement le geste d'un clic au sommaire, sans quitter le clavier.
fn sauter_titre(state: &mut State, direction: i8) -> Task<Message> {
    let Some(path) = state.editor.path.clone() else {
        return Task::none();
    };

    // Les lignes du sommaire sont celles du fichier : la ligne du curseur
    // les rejoint à l'unité près.
    let (ligne, _) = state.editor.editor.cursor_position();
    let ligne = ligne + 1;

    let index = if direction > 0 {
        crate::typst::render::entry_suivant(&state.render.outline, &path, ligne)
    } else {
        crate::typst::render::entry_precedent(&state.render.outline, &path, ligne)
    };

    match index {
        Some(index) => update(state, Message::GotoHeading(index)),
        None => Task::none(),
    }
}

/// Lance les rasterisations nécessaires, et rien de plus.
///
/// Une page n'est rasterisée que si elle est visible, marquée périmée, et
/// qu'aucun rendu n'est déjà en cours pour elle. Un document de 5500 mots ne
/// coûte donc qu'une compilation (~15 ms) + 1-2 pages rasterisées (~30 ms
/// chacune) ; un défilement sur des images déjà à jour ne déclenche aucun
/// travail, et un document intact n'est jamais recompilé.
fn rasterize_visible(state: &mut State) -> Task<Message> {
    let Some(document) = state.render.document.clone() else {
        return Task::none();
    };

    // Tant qu'aucun viewport n'a été signalé, les pages du haut sont celles
    // qu'on voit. `iced` n'en signale d'ailleurs aucun tant que le contenu
    // tient dans la fenêtre — auquel cas tout est visible d'un coup, et trois
    // pages suffisent à couvrir ce cas comme le précédent.
    let (first, last) = state.render.visible.unwrap_or((0, 2));
    let total = crate::typst::render::page_count(&document);
    let revision = state.render.revision;
    let box_size = state
        .render
        .viewport
        .unwrap_or_else(crate::ui::preview::default_viewport);
    let scale_factor = state.scale_factor;

    let candidates: Vec<usize> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .collect();

    // Réserver avant de construire les tâches : deux changements de viewport
    // qui arrivent avant la réponse ne doivent pas rasteriser deux fois la
    // même page. Chaque tâche conserve ainsi un seul tampon RGBA en mémoire.
    let wanted: Vec<usize> = candidates
        .into_iter()
        .filter(|index| state.render.begin_page_render(revision, *index))
        .collect();

    state.render.requested += wanted.len() as u64;

    let tasks: Vec<Task<Message>> = wanted
        .into_iter()
        .map(|index| {
            let document = document.clone();

            // La densité est celle de **l'affichage** : la page est montrée
            // réduite — le panneau est plus étroit qu'une page — et le rendu
            // n'a besoin que des pixels que cette réduction consomme. Elle se
            // calcule page par page, parce qu'une page en paysage n'a pas le
            // même rapport qu'une page en portrait.
            let density = crate::typst::render::pixel_per_pt(
                crate::ui::preview::display_ratio_for_zoom(
                    box_size,
                    crate::typst::render::page_size(&document, index).unwrap_or((595.3, 841.9)),
                    state.render.zoom,
                ),
                scale_factor,
            );

            Task::perform(
                async move { crate::typst::render::render_page(&document, index, density) },
                move |image| Message::PageRendered(revision, index, image),
            )
        })
        .collect();

    Task::batch(tasks)
}

/// L'aperçu suit la ligne qu'on lit.
///
/// Typst ne relie pas une ligne à une page ; il relie chaque **titre** à la
/// sienne. La ligne du haut de la zone de rédaction désigne donc le titre qui la
/// domine, et c'est sa page qu'on montre : exacte aux titres près, et c'est la
/// seule correspondance que la composition donne.
fn follow_editor_scroll(state: &mut State) -> Task<Message> {
    let Some(path) = state.editor.path.clone() else {
        return Task::none();
    };

    let hauteur = f64::from(state.editor.editor.line_height()).max(1.0);
    let ligne = (f64::from(state.editor.editor.viewport_scroll()) / hauteur).ceil() as usize + 1;

    let Some(index) = crate::typst::render::entry_covering(&state.render.outline, &path, ligne)
    else {
        return Task::none();
    };

    let page = state.render.outline[index].page;

    // Déjà là : ne rien faire. Sans cette garde, chaque défilement renverrait la
    // même opération à l'aperçu, soixante fois par seconde.
    if state.render.followed_page == Some(page) {
        return Task::none();
    }

    state.render.followed_page = Some(page);

    if crate::app::epreuves::actif("vue") {
        println!("[vue] suivi : ligne {ligne} → page {}", page + 1);
    }

    scroll_preview_to_page(state, page)
}

/// Fait défiler l'aperçu jusqu'à une page.
///
/// L'opération est paramétrée par le type de message qu'elle produirait : elle
/// n'en produit aucun, mais c'est ainsi qu'elle se raccorde à la boucle de
/// messages.
fn scroll_preview_to_page(state: &State, page: usize) -> Task<Message> {
    let Some(document) = state.render.document.clone() else {
        return Task::none();
    };

    let sizes = crate::ui::preview::sizes_of_with_zoom(
        &document,
        state
            .render
            .viewport
            .unwrap_or_else(crate::ui::preview::default_viewport),
        state.render.zoom,
    );

    let offset = crate::ui::preview::scroll_offset_for_page(&sizes, page);

    iced::advanced::widget::operate(iced::advanced::widget::operation::scrollable::scroll_to::<
        Message,
    >(
        crate::ui::preview::SCROLL_ID.into(),
        iced::advanced::widget::operation::scrollable::AbsoluteOffset {
            x: None,
            y: Some(offset),
        },
    ))
}

/// Amène l'aperçu sur le chapitre en cours d'écriture, une fois par chapitre.
///
/// Sans cela l'aperçu resterait en tête de l'assemblage : écrire le chapitre 7
/// revenait à regarder le chapitre 1, et l'on pouvait croire que les chapitres
/// suivants n'étaient pas rendus.
///
/// Le déclenchement a lieu à l'ouverture d'un chapitre, jamais à chaque
/// compilation : la page d'un chapitre se décale à chaque frappe, et s'accrocher
/// au numéro arracherait l'aperçu sous les yeux du rédacteur au milieu d'une
/// phrase.
fn follow_chapter(state: &mut State) -> Task<Message> {
    // Le suivi est un réglage, et non une fatalité : suivre le chapitre évite
    // de croire qu'il n'est pas rendu, mais arrache l'aperçu sous les yeux dès
    // qu'on écrit ailleurs dans l'assemblage. La barre d'outils porte
    // l'interrupteur ; ici, on l'écoute.
    if !state.layout.follow_preview
        || !state
            .render
            .should_follow_chapter(state.editor.path.as_deref())
    {
        return Task::none();
    }

    let Some(page) = state.render.chapter_page else {
        return Task::none();
    };

    // Le chapitre est marqué comme suivi **avant** de calculer le défilement :
    // si le calcul échouait, on ne redéfilerait pas indéfiniment.
    state.render.followed_chapter = state.editor.path.clone();

    scroll_preview_to_page(state, page)
}

/// Applique un mode de zoom et invalide les images rasterisées à l'ancienne
/// échelle, sans élargir le cache au-delà du voisinage visible.
fn set_preview_zoom(state: &mut State, zoom: crate::app::state::PreviewZoomMode) -> Task<Message> {
    let zoom = crate::ui::preview::normalize_zoom(zoom);

    if state.render.zoom == zoom {
        return Task::none();
    }

    state.render.zoom = zoom;
    let total = state
        .render
        .document
        .as_deref()
        .map_or(0, crate::typst::render::page_count);
    state.render.invalidate_rasterization(total);
    state.render.bound_page_cache();

    rasterize_visible(state)
}

/// Relie les titres composés par Typst aux lignes du `document.tw` canonique.
///
/// L'assemblage peut aussi être ouvert pour régler ses normes : les destinations
/// du sommaire restent alors le document unique, pas les anciens chapitres
/// inclus par l'archive Typst. Les titres bruts sans équivalent dans l'AST sont
/// écartés puisqu'ils n'ont pas de ligne `.tw` à ouvrir.
fn source_outline(
    state: &State,
    outline: Vec<crate::typst::render::OutlineEntry>,
) -> Vec<crate::typst::render::OutlineEntry> {
    if state.editor.is_intermediate() {
        let Some(path) = state.editor.path.clone() else {
            return Vec::new();
        };
        let Some(document) = state.editor.document.as_ref() else {
            return Vec::new();
        };

        return map_outline_to_source(path, &state.editor.content(), document, outline);
    }

    let assembly_is_open = state
        .editor
        .path
        .as_deref()
        .is_some_and(|path| crate::project::same_file(path, &state.project.main));
    if !assembly_is_open {
        return outline;
    }

    let Some(path) = state.project.document.clone() else {
        return outline;
    };
    let Ok(source) = std::fs::read_to_string(&path) else {
        return outline;
    };
    let document = crate::langage::parse(&source);

    map_outline_to_source(path, &source, &document, outline)
}

fn map_outline_to_source(
    path: std::path::PathBuf,
    source: &str,
    document: &crate::langage::Document,
    outline: Vec<crate::typst::render::OutlineEntry>,
) -> Vec<crate::typst::render::OutlineEntry> {
    let source_headings = crate::langage::parse::titres_source(source, document);
    let mut next_heading = 0;

    outline
        .into_iter()
        .filter_map(|mut entry| {
            let found =
                source_headings
                    .iter()
                    .enumerate()
                    .skip(next_heading)
                    .find(|(_, heading)| {
                        heading.level as usize == entry.level
                            && heading
                                .text
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                == entry.text.split_whitespace().collect::<Vec<_>>().join(" ")
                    });
            let (index, heading) = found?;
            next_heading = index + 1;
            entry.file = Some(path.clone());
            entry.line = heading.line;
            Some(entry)
        })
        .collect()
}

/// Lance la compilation du document (moteur Typst en bibliothèque).
fn compile_task(state: &State) -> Task<Message> {
    let dir = state.document_dir();
    let (entry, overrides) = state.compilation_unit();

    Task::perform(
        async move { crate::typst::render::compile(&dir, &entry, &overrides) },
        Message::RenderDone,
    )
}

/// Recompile le document périodiquement : le moteur n'est relancé que si le
/// contenu a changé depuis la dernière compilation (debounce par inactivité).
fn subscription_timer() -> Subscription<Message> {
    time::every(time::milliseconds(RENDER_INTERVAL_MS)).map(|_| Message::RenderTick)
}

/// Écrit le document sur le disque à intervalle régulier, et seulement s'il
/// y a quelque chose à écrire.
///
/// Écrire à chaque compilation réussie produisait une écriture disque toutes
/// les 300 ms pendant la frappe ; la sauvegarde est maintenant différée et
/// regroupée.
fn subscription_save() -> Subscription<Message> {
    time::every(time::milliseconds(SAVE_INTERVAL_MS)).map(|_| Message::SaveTick)
}

/// Lit le thème du système au démarrage, puis suit ses changements.
///
/// `iced` détecte la préférence de couleur du système (par `mundy` sous
/// Linux) : il suffit de la demander une fois, puis de s'abonner.
fn subscription_system_theme() -> Subscription<Message> {
    iced::system::theme_changes().map(|mode| Message::SystemTheme(Some(mode)))
}

/// Suit le curseur pendant qu'un séparateur est tenu.
///
/// `mouse_area::on_move` ne sert à rien ici : il cesse de émettre dès que le
/// curseur quitte la bande de 6 px, ce qui arrive à chaque glissement un peu
/// vif. Il faut donc écouter la souris au niveau de la fenêtre, et ne rien
/// faire tant qu'aucun séparateur n'est saisi.
fn subscription_drag() -> Subscription<Message> {
    iced::event::listen_with(|event, status, _window| match status {
        // Un glissement de séparateur est une interaction exclusive : on prend
        // les événements même si un widget les a capturés.
        Status::Captured | Status::Ignored => match event {
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Some(Message::MouseMoved(position))
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                Some(Message::SplitterReleased)
            }
            _ => None,
        },
    })
}

/// Écoute des événements clavier au niveau application.
///
/// Les raccourcis globaux (Ctrl+S, Ctrl+N, Ctrl+O, Ctrl+E) n'agissent que
/// lorsque l'événement n'a pas été consommé par un widget — l'éditeur garde
/// donc la priorité sur ses propres raccourcis.
/// Retient l'identifiant de la fenêtre dès son ouverture.
fn subscription_window() -> Subscription<Message> {
    Subscription::batch(vec![
        iced::window::open_events().map(Message::WindowOpened),
        // La taille de la fenêtre : c'est elle qui dit où il y a de la place
        // pour la liste de complétion.
        iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
    ])
}

fn subscription_keys() -> Subscription<Message> {
    iced::event::listen_with(|event, status, _window| match status {
        Status::Ignored => match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::RawKey(key, modifiers))
            }
            _ => None,
        },
        Status::Captured => None,
    })
}

/// Les touches du composeur que son champ de saisie capture.
///
/// Le champ prend la frappe entière — c'est sa fonction — mais il attrape
/// aussi Échap, qu'il se contente de défiler du focus. Le composeur, lui,
/// veut cette touche : on relaie les siennes même capturées, et jamais une
/// autre. L'abonnement n'existe que tant que le composeur est ouvert.
fn subscription_composeur(state: &State) -> Subscription<Message> {
    if state.composeur.is_none() {
        return Subscription::none();
    }

    iced::event::listen_with(|event, status, _window| match status {
        Status::Captured => match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if est_touche_composeur(&key) =>
            {
                Some(Message::RawKey(key, modifiers))
            }
            _ => None,
        },
        Status::Ignored => None,
    })
}

/// Les touches qui font avancer le composeur, et que rien d'autre ne garde.
fn est_touche_composeur(key: &iced::keyboard::Key) -> bool {
    use iced::keyboard::{Key, key::Named};

    matches!(
        key,
        Key::Named(Named::Tab | Named::Enter | Named::Escape | Named::ArrowUp | Named::ArrowDown)
    )
}

pub fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch(vec![
        subscription_timer(),
        subscription_save(),
        subscription_system_theme(),
        subscription_window(),
        subscription_drag(),
        subscription_keys(),
        subscription_composeur(_state),
        subscription_lsp(_state),
        subscription_memory(),
    ])
}

/// L'écoute du serveur de langage.
///
/// Deux rythmes, parce qu'un seul ne peut pas convenir aux deux moments : quand
/// une réponse est attendue, l'attente doit être courte — une liste de
/// complétion qui arrive un quart de seconde après la frappe est une liste qu'on
/// a déjà cessé d'attendre ; au repos, un réveil fréquent ne ferait que
/// reconstruire la vue vingt fois par seconde pour rien.
///
/// Sans serveur, il n'y a rien à écouter : l'abonnement disparaît, et
/// l'application ne se réveille plus pour lui.
fn subscription_lsp(state: &State) -> Subscription<Message> {
    if state.lsp.session.is_none() {
        return Subscription::none();
    }

    let interval = if state.lsp.awaiting { 16 } else { 250 };

    time::every(std::time::Duration::from_millis(interval)).map(|_| Message::LspPoll)
}

/// La couleur du papier, lue sur le coin supérieur gauche d'une page rendue.
///
/// Le fond d'une page est en haut à gauche : c'est la seule couleur qui
/// permette de remplir un cadre de page avant que sa texture n'existe, sans
/// jamais laisser voir le fond de l'application.
fn paper_colour(image: &crate::typst::render::PageImage) -> Option<iced::Color> {
    let pixel = image.pixels.first_chunk::<4>()?;

    Some(iced::Color::from_rgba8(
        pixel[0],
        pixel[1],
        pixel[2],
        f32::from(pixel[3]) / 255.0,
    ))
}

/// Tâche de démarrage : lire le thème du système avant le premier rendu, pour
/// que la fenêtre n'apparaisse pas une frame en sombre avant de passer au
/// clair (ou l'inverse).
pub fn startup() -> Task<Message> {
    Task::batch(vec![
        iced::system::theme().map(|mode| Message::SystemTheme(Some(mode))),
        // La densité du rendu vient de l'écran : elle doit être connue avant la
        // première rasterisation, sinon la première page se paie à la densité de
        // repli, et il faut la refaire.
        iced::window::latest()
            .and_then(iced::window::scale_factor)
            .map(Message::ScaleFactor),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le sommaire Typst pointe vers la ligne source `.tw`, pas vers le fichier
    /// généré en mémoire ; les titres sans équivalent source sont écartés.
    #[test]
    fn intermediate_outline_targets_editable_source_lines() {
        let source = "# Introduction\n\n## Section\n";
        let document = crate::langage::parse(source);
        let generated = std::path::PathBuf::from("document.generated.typ");
        let outline = vec![
            crate::typst::render::OutlineEntry {
                level: 1,
                text: "Introduction".to_string(),
                file: Some(generated.clone()),
                line: 1,
                page: 0,
            },
            crate::typst::render::OutlineEntry {
                level: 1,
                text: "Titre Typst brut".to_string(),
                file: Some(generated.clone()),
                line: 4,
                page: 1,
            },
            crate::typst::render::OutlineEntry {
                level: 2,
                text: "Section".to_string(),
                file: Some(generated),
                line: 8,
                page: 2,
            },
        ];

        let path = std::path::PathBuf::from("document.tw");
        let mapped = map_outline_to_source(path.clone(), source, &document, outline);

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0].file.as_deref(), Some(path.as_path()));
        assert_eq!(mapped[0].line, 1);
        assert_eq!(mapped[0].page, 0);
        assert_eq!(mapped[1].line, 3);
        assert_eq!(mapped[1].page, 2);
    }
}
