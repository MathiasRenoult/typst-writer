pub mod assistance;
pub mod composeur;
pub mod epreuves;
pub mod espace;
mod espace_runtime;
mod fichiers;
mod journal;
pub mod message;
pub mod recherche;
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
const RENDER_INTERVAL_MS: u64 = 100;

/// Combien de temps le geste de zoom attend avant le rendu net : la
/// molette dix fois ne relance pas dix rendus, mais un seul (bug-report-9).
const DUREE_GESTE_ZOOM: std::time::Duration = std::time::Duration::from_millis(150);

/// La marge de rendu autour de la fenêtre visible, en fraction de fenêtre :
/// défiler dans la marge ne coûte aucun rendu.
const MARGE_RENDU: f32 = 0.5;

/// L'identifiant du scrollable de l'aperçu : il porte le nom de la vue
/// qui le montre, et l'identifiant nu en mode focus (bug-report-9 :
/// cibler l'identifiant nu en vue normale ne trouvait personne — le
/// zoom s'ancre en haut à gauche, le défilement horizontal ne suit pas).
fn id_du_scrollable(state: &State) -> iced::widget::Id {
    if state.layout.focus_mode {
        crate::ui::preview::SCROLL_ID.into()
    } else {
        format!("{}-{}", crate::ui::preview::SCROLL_ID, state.espace.active).into()
    }
}

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

        // Une création interrompue laisse un temporaire caché : son auteur
        // n'existe plus, il part à la corbeille plutôt que d'encombrer.
        crate::project::nettoyer_temporaires_de_creation(&crate::project::projects_dir());

        let mut projects = crate::project::list_projects();

        // Aucun projet : c'est le premier démarrage, on en crée un.
        if projects.is_empty() {
            match crate::project::create(FIRST_PROJECT) {
                Ok(dir) => projects = vec![crate::project::load(&dir)],
                Err(error) => {
                    eprintln!("Création du premier projet : {error}");
                    // Une autre instance a pu créer le projet entre les deux
                    // lectures : ne pas ouvrir le dossier racine à sa place.
                    projects = crate::project::list_projects();
                }
            }
        }

        let mut note_au_demarrage: Option<crate::app::state::Note> = None;
        let session = crate::project::load_session();

        // Le projet de la session s'il existe toujours, sinon le plus récent.
        let project = session
            .project
            .as_ref()
            .filter(|dir| dir.is_dir())
            .map(|dir| crate::project::load(dir))
            .or_else(|| projects.first().cloned())
            .unwrap_or_else(|| crate::project::load(&crate::project::projects_dir()));

        // Le verrou doit être acquis avant toute migration : une seconde
        // instance n'a pas le droit de modifier le projet qu'elle consulte.
        let verrou = match crate::project::Verrou::prendre(&project.dir) {
            Ok(verrou) => Some(verrou),
            Err(probleme) => {
                note_au_demarrage = Some(crate::app::state::Note::erreur(format!(
                    "{probleme} Ce projet est ouvert en lecture seule."
                )));
                None
            }
        };

        // La migration ne se fait pas en silence : ce qu'elle n'a pas
        // traduit se dit, à l'écran, dès le démarrage. Elle ne touche jamais
        // un projet dont le verrou n'a pas été obtenu.
        let note_de_migration = if verrou.is_some() {
            match crate::project::migrate_to_document_avec_rapport(&project) {
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
            }
        } else {
            None
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
            espace: Default::default(),
            police_combo: iced::widget::combo_box::State::new(
                crate::typst::fonts::familles_pour_choisir(),
            ),
            typst_genere: String::new(),
            projet_dialogue: None,
            controle_tenu: false,
            maj_tenu: false,
            references_repliees: false,
            references_filtre: String::new(),
            tableau: None,
            projets_vue: false,
            mode: crate::app::state::ModeVue::default(),
            vignettes: Vec::new(),
            drag_projet: None,
            drag_etagere: None,
            editeur_theme: None,
            editeur_theme_textes: std::collections::HashMap::new(),
            survol_reference: None,
            survol_onglet: None,
            survol_projet: None,
            zotero_recherche: String::new(),
            onglets: Vec::new(),
            etagere_brouillon: String::new(),
            projets_tous: false,
            projets_recherche: String::new(),
            references_recherche_visible: false,
            projets_recherche_visible: false,
            survol_sommaire: None,
            projets_etage: "Mon étagère".to_string(),
            note_dialogue: None,
            extension: None,
            geste_confirme: false,
            conflit_refuse: None,
            importation: None,
            derniere_notice_supprimee: None,
            style_dependant_en_attente: None,
            confirmation: None,
            verrou,
            aide: false,
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
            note_overlay: None,
            edition_reference: None,
            glossaire: Vec::new(),
            glossaire_replie: false,
            glossaire_recherche_visible: false,
            glossaire_filtre: String::new(),
            survol_glossaire: None,
            glossaire_fenetre: false,
            edition_glossaire: None,
            zotero,
            renaming: None,
            jump: None,
            rendus_partages: Default::default(),
            render: Default::default(),
            layout: Default::default(),
            lsp: Default::default(),
            system_theme: None,
            themes: Vec::new(),
            derniere_couleur: None,
            picker_couleur: None,
            theme_problems: Vec::new(),
            // Le thème retenu est restauré **avant** la première lecture du
            // dossier : `refresh_themes` applique le thème, il doit donc savoir
            // lequel.
            chosen: session.themes.clone(),
            theme: session.theme_mode,
            resolved: crate::ui::theme::Resolved::of(&crate::theming::natives()[0]),
            liaison: None,
            recherche: None,
            note: note_de_migration,
        };

        // Ce qui n'a pas pu se lire au chargement se dit — jamais avale.
        // Un fichier corrompu n'est pas « pas de liaison », et l'utilisateur
        // doit le savoir avant de croire que tout va bien.
        let problemes: Vec<String> = state
            .projects
            .iter()
            .flat_map(|projet| projet.problemes.iter().cloned())
            .collect();

        if let Some(premier) = problemes.into_iter().next() {
            state.note = Some(crate::app::state::Note::erreur(premier));
        }

        if state.note.is_none() {
            state.note = note_au_demarrage;
        }

        // Le chapitre de la session, s'il appartient encore au projet ; sinon
        // le premier chapitre, sinon rien — le panneau proposera d'en créer un.
        //
        // L'appartenance doit être vérifiée, et non supposée : un document
        // retenu avant d'être un chapitre — ou dont le fichier a été déplacé
        // dans un autre projet — s'ouvrirait sinon dans l'éditeur sans jamais
        // être rendu, exactement le défaut que l'import répare.
        // Le document, ou un chapitre : tout ce qui appartient au projet.
        // N'accepter que `document.tw` laissait un chapitre ouvert se perdre
        // au redémarrage — exactement ce que la session promet de garder.
        let wanted = session.last_document.filter(|path| {
            path.is_file()
                && (state.project.document.as_deref() == Some(path.as_path())
                    || state.project.chapters.iter().any(|c| c.path == *path))
        });

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

        state.initialiser_espace(session.espace);
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

        self.glossaire = crate::project::glossaire::Glossaire::charger(&self.project.dir).entrees;
    }

    /// Charge un chapitre dans l'éditeur, rendu vierge.
    ///
    /// Les images du chapitre précédent ne doivent pas survivre au changement :
    /// les `Handle` sont relâchés pour que le backend libère les textures.
    fn open_document(&mut self, path: PathBuf, content: String, nouvel_onglet: bool) {
        // Une écriture interrompue a pu laisser un temporaire qui porte la
        // dernière version du document : il devient récupérable, à condition
        // de détenir le verrou — un projet en lecture seule ne s'écrit pas.
        if self.projet_verrouille(&self.project.dir) {
            crate::project::recuperer_temporaires_perdus(&self.project.dir, &path);
        }
        if self.espace.initialise {
            let ouverture = if nouvel_onglet {
                crate::app::espace::Ouverture::Nouvel
            } else {
                crate::app::espace::Ouverture::Remplacer
            };
            self.ouvrir_vue(self.project.clone(), path, content, ouverture);
            return;
        }
        self.editor = crate::editor::EditorState::new_file(path.clone(), content.clone());
        // Ce que le tampon vient de recevoir est ce que le disque contenait :
        // c'est la référence dont tout changement extérieur s'écartera.
        self.espace
            .disques
            .entry((self.project.dir.clone(), path))
            .or_insert(crate::project::empreinte_contenu(content.as_bytes()));
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
            espace: self.disposition_complete(),
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
        let (entree, substitutions, _) = self.compilation_unit_avec_diagnostics();
        (entree, substitutions)
    }

    /// Comme [`State::compilation_unit`], mais dit les citations neutralisées
    /// pendant la préparation : elles deviennent des avertissements, au même
    /// titre que ceux du compilateur.
    fn compilation_unit_avec_diagnostics(
        &self,
    ) -> (
        String,
        Vec<(PathBuf, String)>,
        Vec<crate::typst::render::Diagnostic>,
    ) {
        let open = self.editor.path.clone();
        let buffer = self.editor.content();

        let assembly_is_open = open.as_deref() == Some(self.project.main.as_path());

        let entry = if assembly_is_open {
            buffer.clone()
        } else {
            self.espace
                .documents
                .get(&(self.project.dir.clone(), self.project.main.clone()))
                .map(|d| d.source.contenu())
                .unwrap_or_else(|| std::fs::read_to_string(&self.project.main).unwrap_or_default())
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
                .and_then(|path| {
                    self.espace
                        .documents
                        .get(&(self.project.dir.clone(), path.to_path_buf()))
                        .map(|d| d.source.contenu())
                        .or_else(|| std::fs::read_to_string(path).ok())
                })
                .map(|source| crate::langage::parse(&source))
        } else {
            None
        };

        if let Some(document) = writing_document {
            let entry = crate::project::assembly_without_chapters(&self.project, &entry);

            return crate::langage::pipeline::compilation_unit_document_avec_diagnostics(
                &self.project.dir,
                &entry,
                &document,
            );
        }

        if is_intermediate {
            let entry = crate::project::assembly_without_chapters(&self.project, &entry);

            return crate::langage::pipeline::compilation_unit_avec_diagnostics(
                &self.project.dir,
                &entry,
                &buffer,
            );
        }

        let overrides = match open {
            // L'assemblage est l'entrée : il n'a pas à figurer aussi en mémoire.
            Some(_) if assembly_is_open => Vec::new(),
            Some(path) => vec![(path, buffer)],
            None => Vec::new(),
        };

        (entry, overrides, Vec::new())
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

    /// Un projet n'est écrit que par l'instance qui en tient le verrou.
    ///
    /// Avant l'initialisation de l'espace, le verrou courant est celui du
    /// projet ouvert ; ensuite, chaque projet possède le sien.
    fn projet_verrouille(&self, dir: &std::path::Path) -> bool {
        (self.verrou.is_some() && dir == self.project.dir) || self.espace.verrous.contains_key(dir)
    }

    /// Écrit le contenu courant et enregistre la session.
    fn persist(&mut self) -> bool {
        // Sans verrou, rien ne s'écrit : deux instances ne se dévorent
        // jamais leur texte. La raison a été dite au démarrage.
        if self.verrou.is_none() && !self.espace.verrous.contains_key(&self.project.dir) {
            return false;
        }

        let Some(chemin) = self.editor.path.clone() else {
            return false;
        };

        match self.sauvegarder_document(&chemin, false) {
            Ok(()) => {
                self.save_session();
                true
            }
            Err(SauvegardeRefusee::Conflit(empreinte)) => {
                self.poser_la_question_du_conflit(empreinte);
                false
            }
            Err(SauvegardeRefusee::Echec(probleme)) => {
                self.note = Some(crate::app::state::Note::erreur(format!(
                    "Sauvegarde impossible : {probleme}"
                )));
                false
            }
        }
    }

    /// Écrit le document courant, en conservant d'abord la version qui va
    /// l'être remplacée.
    ///
    /// Un changement extérieur n'est jamais écrasé en silence : sans `force`,
    /// la sauvegarde est refusée et dit son remède. La copie de la version
    /// précédente précède toute écriture — si elle ne peut être conservée,
    /// rien ne se remplace.
    fn sauvegarder_document(
        &mut self,
        chemin: &std::path::Path,
        force: bool,
    ) -> Result<(), SauvegardeRefusee> {
        let contenu = self.editor.content();
        let cle = (self.project.dir.clone(), chemin.to_path_buf());
        let sur_disque = match std::fs::read(chemin) {
            Ok(octets) => Some(octets),
            Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => None,
            Err(erreur) => {
                return Err(SauvegardeRefusee::Echec(format!(
                    "Lire la version du disque : {erreur}"
                )));
            }
        };

        let mut changement_externe = false;
        if let Some(octets) = &sur_disque {
            let empreinte = crate::project::empreinte_contenu(octets);
            if self
                .espace
                .disques
                .get(&cle)
                .is_some_and(|attendue| *attendue != empreinte)
            {
                if !force {
                    return Err(SauvegardeRefusee::Conflit(empreinte));
                }
                changement_externe = true;
            }
        }

        if let Some(ancien) = sur_disque.as_deref() {
            crate::project::sauvegarder_version(
                &self.project.dir,
                chemin,
                &String::from_utf8_lossy(ancien),
                changement_externe,
            )
            .map_err(|erreur| {
                SauvegardeRefusee::Echec(format!("Conserver la version précédente : {erreur}"))
            })?;
        }

        self.editor
            .save()
            .map_err(|erreur| SauvegardeRefusee::Echec(erreur.to_string()))?;
        self.espace
            .disques
            .insert(cle, crate::project::empreinte_contenu(contenu.as_bytes()));
        self.conflit_refuse = None;
        Ok(())
    }

    /// Un fichier a changé hors de l'application : le dire, et demander une
    /// fois — jamais à chaque sauvegarde, jamais deux fois pour le même
    /// changement refusé.
    fn poser_la_question_du_conflit(&mut self, empreinte: u64) {
        let nom = self
            .editor
            .path
            .as_ref()
            .and_then(|chemin| chemin.file_name())
            .map(|nom| nom.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Le document".to_string());
        self.note = Some(crate::app::state::Note::erreur(format!(
            "Sauvegarde en attente : {nom} a été modifié hors de l'application. Vos frappes restent en mémoire."
        )));
        if self.conflit_refuse == Some(empreinte) || self.confirmation.is_some() {
            return;
        }
        self.confirmation = Some(crate::app::state::Confirmation {
            question: format!(
                "{nom} a été modifié hors de l'application. Remplacer sa version par votre texte ? Elle restera dans « sauvegardes/ »."
            ),
            action: Message::SauvegardeForcee(empreinte),
        });
    }
}

/// Ce qui empêche une sauvegarde d'aboutir.
enum SauvegardeRefusee {
    /// Le fichier a changé hors de l'application, avec l'empreinte de ce
    /// changement : elle distingue les conflits entre eux.
    Conflit(u64),

    /// Une erreur d'entrée-sortie, à dire telle quelle.
    Echec(String),
}

/// Nom du projet créé au premier démarrage.
const FIRST_PROJECT: &str = "mon-travail";

/// Un champ de fiche rempli devient une valeur ; le vide devient l'absence.
fn champ_renseigne(valeur: &str) -> Option<String> {
    let valeur = valeur.trim();
    if valeur.is_empty() {
        None
    } else {
        Some(valeur.to_string())
    }
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    espace_runtime::update(state, message)
}

fn update_interne(state: &mut State, message: Message) -> Task<Message> {
    // Toutes les actions qui écrivent dans le projet doivent passer par le
    // même contrôle, y compris les messages issus d'une confirmation.
    let modifie_le_projet = match &message {
        Message::ReferenceSupprimer(_)
        | Message::ReferenceRestaurer(_)
        | Message::BibliographieCreer
        | Message::EditionReferenceEnregistrer
        | Message::GlossaireSupprimer(_)
        | Message::EditionGlossaireEnregistrer
        | Message::ImageChoisie(Some(_))
        | Message::StyleCslChoisi(_)
        | Message::StyleZoteroChoisi(_)
        | Message::StyleParentTelecharger
        | Message::BibImporte(Some(_))
        | Message::MiseEnPageAppliquer
        | Message::NewFile
        | Message::NewChapter
        | Message::DeleteChapter(_)
        | Message::RenameCommit
        | Message::SymbolRenameApply
        | Message::ZoteroDelier
        | Message::ZoteroExport(_, _, _, Ok(_))
        | Message::Format(_)
        | Message::NoteInserer
        | Message::NoteTexte(_)
        | Message::NoteValider
        | Message::NoteAnnuler
        | Message::FigureInserer
        | Message::EtiquetteInserer
        | Message::RenvoiInserer
        | Message::TitreInserer(_)
        | Message::InsertCitation(_)
        | Message::SommaireInserer
        | Message::BibliographieSectionInserer
        | Message::GlossaireSectionInserer
        | Message::GlossaireInserer(_)
        | Message::TableauInserer
        | Message::BlocMonter
        | Message::BlocDescendre
        | Message::RechercheRemplacer
        | Message::RechercheRemplacerTout
        | Message::CompletionAccept
        | Message::CompletionPick(_)
        | Message::ComposeurTerminer
        | Message::SauvegardeForcee(_)
        | Message::ImportAppliquer
        | Message::NoteOverlayEnregistrer => true,
        Message::Editor(message) => crate::editor::EditorState::message_may_change_content(message),
        Message::FileOpened(Some((chemin, _))) => {
            !crate::project::same_file(chemin, &state.project.main)
                && !state
                    .project
                    .document
                    .as_ref()
                    .is_some_and(|document| crate::project::same_file(chemin, document))
        }
        _ => false,
    };
    if modifie_le_projet
        && state.verrou.is_none()
        && !state.espace.verrous.contains_key(&state.project.dir)
    {
        state.geste_confirme = false;
        state.note = Some(crate::app::state::Note::erreur(
            "Projet en lecture seule : fermez l'autre instance avant de modifier ses fichiers.",
        ));
        return Task::none();
    }
    match message {
        Message::FermetureDemandee => unreachable!("fermeture traitée par l’espace"),
        Message::RetourDocument(..)
        | Message::VersVue(..)
        | Message::RetourVue(..)
        | Message::VueChoisir(..)
        | Message::VueFermer(..)
        | Message::VueDupliquer
        | Message::Scinder(..)
        | Message::OngletSuivant
        | Message::PanneauSuivant
        | Message::PanneauChoisir(..)
        | Message::PanneauRedimensionner(..)
        | Message::OngletPrendre(..)
        | Message::OngletCible(..)
        | Message::OngletPoser => unreachable!("messages traités par l’espace"),
        // Une notice locale se supprime — vers la corbeille, avec son mot.
        Message::ReferenceSupprimer(cle) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!("Supprimer la notice « {cle} » ? Elle part à la corbeille."),
                    action: Message::ReferenceSupprimer(cle),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            // Une notice Zotero ne se supprime pas ici : elle est à Zotero.
            if state.zotero.source == crate::zotero::Source::Zotero {
                state.note = Some(crate::app::state::Note::erreur(
                    "Une notice Zotero se supprime dans Zotero.".to_string(),
                ));
                return Task::none();
            }

            let Some(bibliotheque) = state.project.bibliography.clone() else {
                return Task::none();
            };

            match crate::project::supprimer_reference(&bibliotheque, &cle) {
                Ok(copie) => {
                    state.bibliography = crate::project::parse_bibliography(&bibliotheque);
                    state.edition_reference = None;
                    state.derniere_notice_supprimee = Some(copie);
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La notice « {cle} » part à la corbeille."
                    )));
                    state.refresh_context();
                    Task::none()
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        Message::ReferenceRestaurer(copie) => {
            let Some(bibliotheque) = state.project.bibliography.clone() else {
                return Task::none();
            };

            match crate::project::restaurer_reference(&bibliotheque, &copie) {
                Ok(cle) => {
                    state.bibliography = crate::project::parse_bibliography(&bibliotheque);
                    state.derniere_notice_supprimee = None;
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La notice « {cle} » est de retour."
                    )));
                    state.refresh_context();
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }
            Task::none()
        }

        // Les projets : le nom en frappe, puis les trois gestes.
        Message::ProjetNom(nom) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                dialogue.nom = nom;
            }

            Task::none()
        }

        // Les métadonnées choisies à la création (bug-report-7, point 9).
        Message::ProjetMeta(champ, valeur) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                match champ.as_str() {
                    "titre" => dialogue.titre = valeur,
                    "auteur" => dialogue.auteur = valeur,
                    _ => {}
                }
            }

            Task::none()
        }

        // Le dialogue s'ouvre sur le vide, ou sur le nom du projet visé.
        // L'éditeur de thème : une copie de travail, et le fichier ne
        // bouge qu'à l'enregistrement — la règle de tout dialogue.
        Message::ThemeEditeurOuvrir => {
            // Le thème actif d'abord — éditer un autre que celui qu'on
            // porte serait un contresens.
            let actif = state.current_theme().map(|t| t.name.clone());
            let premier = actif
                .and_then(|nom| {
                    state
                        .themes
                        .iter()
                        .find(|t| t.name == nom && t.file.is_some())
                        .and_then(|t| t.file.clone())
                })
                .or_else(|| {
                    state
                        .themes
                        .iter()
                        .find(|theme| theme.file.is_some())
                        .and_then(|theme| theme.file.clone())
                });

            match premier {
                Some(fichier) => {
                    state.editeur_theme = Some(fichier);
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(
                        "Aucun thème à éditer : les thèmes natifs n'ont pas de fichier."
                            .to_string(),
                    ));
                }
            }

            Task::none()
        }

        Message::ThemeEditeurChoisir(nom) => {
            let fichier = state
                .themes
                .iter()
                .find(|theme| theme.file.as_ref().is_some_and(|f| f.meta.name == nom))
                .and_then(|theme| theme.file.clone());

            if let Some(fichier) = fichier {
                state.editeur_theme = Some(fichier);
                state.editeur_theme_textes.clear();
            }

            Task::none()
        }

        Message::ThemeEditeurColorPicker(champ) => {
            // La même roue que pour le texte, visée sur le champ du thème
            // (bug-report-7, point 11). Elle s'ouvre par-dessus l'éditeur.
            let valeur = state.editeur_theme_textes.get(champ).cloned();
            let (teinte, saturation, valeur_v) = valeur
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 0.0, 0.5));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur: valeur_v,
                hex_saisi: valeur,
                champ: Some(champ),
                reglage: None,
            });
            Task::none()
        }

        Message::ThemeEditeurChamp(champ, valeur) => {
            // La saisie brute reste dans le champ, même incomplète : c'est
            // elle que l'utilisateur voit. La couleur n'est mise à jour
            // que si la valeur se parse.
            state
                .editeur_theme_textes
                .insert(champ.to_string(), valeur.clone());

            if let Some(fichier) = state.editeur_theme.as_mut() {
                let Ok(valeur) = crate::theming::HexColor::parse(valeur.trim()) else {
                    return Task::none();
                };

                match champ {
                    "background" => fichier.colors.background = valeur,
                    "surface" => fichier.colors.surface = valeur,
                    "text" => fichier.colors.text = valeur,
                    "accent" => fichier.colors.accent = valeur,
                    "text_muted" => fichier.colors.text_muted = Some(valeur),
                    "border" => fichier.colors.border = Some(valeur),
                    "danger" => fichier.colors.danger = Some(valeur),
                    "warning" => fichier.colors.warning = Some(valeur),
                    "success" => fichier.colors.success = Some(valeur),
                    _ => {}
                }
            }

            // Le thème s'applique en direct : l'éditeur suit le fond de
            // l'application, même pendant l'édition du thème.
            crate::app::themes::retheme(state);

            Task::none()
        }

        Message::ThemeEditeurReset => {
            // Recharge le fichier depuis le disque : les changements
            // non enregistrés sont abandonnés.
            let nom = state.editeur_theme.as_ref().map(|f| f.meta.name.clone());
            if let Some(nom) = nom
                && let Some(theme) = state
                    .themes
                    .iter()
                    .find(|t| t.name == nom && t.file.is_some())
                    .and_then(|t| t.file.clone())
            {
                state.editeur_theme = Some(theme);
                state.editeur_theme_textes.clear();
            }
            Task::none()
        }

        Message::ThemeEditeurFermer => {
            state.editeur_theme = None;
            state.editeur_theme_textes.clear();
            Task::none()
        }

        Message::ThemeEditeurEnregistrer => {
            let Some(fichier) = state.editeur_theme.clone() else {
                return Task::none();
            };

            match crate::theming::enregistrer(&fichier) {
                Ok(avertissements) => {
                    state.editeur_theme = None;
                    // Le thème enregistré s'applique aussitôt : recharger
                    // la liste et re-résoudre le thème courant.
                    crate::app::themes::refresh_themes(state);
                    crate::app::themes::retheme(state);
                    let message = if avertissements.is_empty() {
                        "Le thème est enregistré et appliqué.".to_string()
                    } else {
                        format!(
                            "Le thème est enregistré et appliqué. Avertissement : {}",
                            avertissements.join(" ; ")
                        )
                    };
                    state.note = Some(crate::app::state::Note::info(message));
                    Task::none()
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                    Task::none()
                }
            }
        }

        // Les onglets : choisir un projet, c'est garder le monde de
        // chacun — la session se persiste avant de partir, et le suivant
        // se charge tout entier.
        Message::OngletChoisir(dir) => {
            if state.project.dir == dir {
                return Task::none();
            }

            state.persist();

            update(state, Message::OpenProject(dir))
        }

        Message::OngletFermer(dir) => {
            state.onglets.retain(|onglet| *onglet != dir);

            if state.project.dir == dir {
                match state.onglets.first().cloned() {
                    Some(suivant) => {
                        state.persist();
                        return update(state, Message::OpenProject(suivant));
                    }
                    None => {
                        state.note = Some(crate::app::state::Note::info(
                            "Dernier onglet fermé.".to_string(),
                        ));
                    }
                }
            }

            Task::none()
        }

        Message::ZoteroRecherche(filtre) => {
            state.zotero_recherche = filtre;
            Task::none()
        }

        // La bibliographie locale : un projet qui n'en a pas la voit
        // naître d'un geste, au modèle.
        Message::BibliographieCreer => {
            let dir = state.project.dir.clone();
            let chemin = dir.join("references.bib");

            if chemin.is_file() {
                state.note = Some(crate::app::state::Note::info(
                    "Le projet a déjà sa bibliographie.".to_string(),
                ));
                return Task::none();
            }

            match crate::project::ecrire_atomique(
                &chemin,
                crate::project::model::BIBLIOGRAPHY_TEMPLATE,
            ) {
                Ok(()) => {
                    state.project = crate::project::load(&dir);
                    state.bibliography = crate::project::parse_bibliography(&chemin);
                    state.note = Some(crate::app::state::Note::info(
                        "La bibliographie locale est créée — references.bib.".to_string(),
                    ));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "La bibliographie n'a pas pu naître : {erreur}"
                    )));
                }
            }

            Task::none()
        }

        // L'alternative sans plugin : Zotero sait exporter un `.bib` tout
        // seul, et ce fichier rejoint la bibliothèque locale sans rien
        // écraser.
        Message::BibliographieImporter => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Bibliographie BibLaTeX", &["bib"])
                    .pick_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::BibImporte,
        ),

        Message::BibImporte(result) => {
            let Some(source) = result else {
                return Task::none();
            };

            match crate::project::importer_bib(&state.project.dir, &source) {
                Ok(rapport) => {
                    let dir = state.project.dir.clone();
                    let chemin = dir.join("references.bib");
                    state.project = crate::project::load(&dir);
                    state.bibliography = crate::project::parse_bibliography(&chemin);

                    // Le rapport se lit entièrement : ce qui est entré, ce
                    // qui était déjà là, ce qui s'est heurté.
                    let mut parties = Vec::new();
                    if !rapport.ajoutees.is_empty() {
                        parties.push(format!(
                            "{} entrée(s) : {}",
                            rapport.ajoutees.len(),
                            rapport.ajoutees.join(", ")
                        ));
                    }
                    if !rapport.deja_la.is_empty() {
                        parties.push(format!(
                            "{} déjà là : {}",
                            rapport.deja_la.len(),
                            rapport.deja_la.join(", ")
                        ));
                    }
                    if !rapport.conflits.is_empty() {
                        parties.push(format!(
                            "{} conflit(s), la première tient : {}",
                            rapport.conflits.len(),
                            rapport.conflits.join(", ")
                        ));
                    }

                    let message = format!("Import BibLaTeX — {}.", parties.join(" ; "));
                    state.note = Some(if rapport.conflits.is_empty() {
                        crate::app::state::Note::info(message)
                    } else {
                        crate::app::state::Note::erreur(message)
                    });
                }
                Err(probleme) => {
                    // Rien n'a changé — la notice le dit déjà.
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }

            Task::none()
        }

        Message::SurvolReference(cle) => {
            state.survol_reference = cle;
            Task::none()
        }

        Message::SurvolOnglet(id) => {
            state.survol_onglet = id;
            Task::none()
        }

        Message::SurvolProjet(dir) => {
            state.survol_projet = dir;
            Task::none()
        }

        // Les vignettes se rendent en arrière-plan : la vue les attend
        // sans jamais bloquer l'écran.
        // Le mode de la fenêtre : édition, rendu, Typst. `Ctrl+e`
        // bascule rendu/édition, `Ctrl+Maj+e` ouvre le Typst — et le
        // Typst ne sert que quand quelque chose refuse de se composer.
        Message::ModeBasculer => {
            let ancien = state.mode;
            state.mode = match state.mode {
                crate::app::state::ModeVue::Edition => crate::app::state::ModeVue::Rendu,
                crate::app::state::ModeVue::Rendu => crate::app::state::ModeVue::Edition,
                crate::app::state::ModeVue::Typst => crate::app::state::ModeVue::Edition,
            };

            // La bascule vers la vue rendu régénère systématiquement le PDF
            // (bug-report-9) : l'aperçu affiche le fichier livré, il doit
            // donc être neuf — jamais l'export d'une session précédente.
            if state.mode == crate::app::state::ModeVue::Rendu
                && ancien != crate::app::state::ModeVue::Rendu
            {
                let pdf = generer_le_pdf_du_rendu(state);

                // Le texte affiché suit l'édition : au basculement, on
                // compile d'entrée si le rendu est périmé — jamais l'image
                // d'une frappe d'avant (bug-report-10, 6.2).
                let frache =
                    state.render.pending || state.render.content == state.compilation_fingerprint();

                return if frache {
                    pdf
                } else {
                    state.render.pending = true;
                    Task::batch([pdf, compile_task(state)])
                };
            }

            Task::none()
        }

        Message::ModeTypst => {
            state.mode = if state.mode == crate::app::state::ModeVue::Typst {
                crate::app::state::ModeVue::Edition
            } else {
                crate::app::state::ModeVue::Typst
            };

            Task::none()
        }

        Message::ProjetsVue => {
            state.projets_vue = !state.projets_vue;

            if state.projets_vue {
                let dirs: Vec<std::path::PathBuf> = state
                    .projects
                    .iter()
                    .map(|projet| projet.dir.clone())
                    .collect();

                return Task::perform(
                    async move { crate::project::vignettes(&dirs) },
                    Message::VignettesFaites,
                );
            }

            Task::none()
        }

        Message::VignettesFaites(vignettes) => {
            state.vignettes = vignettes;
            Task::none()
        }

        // Le glisser-déposer : prendre un projet, survoler une étagère,
        // poser. Le geste dit où il en est — une ombre suit le projet.
        Message::DragCommencer(nom) => {
            state.drag_projet = Some(nom);
            Task::none()
        }

        // Les deux recherches ne se montrent que sur leur icône, et
        // l'étagère se choisit au menu (bug-report 2026-11-03).
        Message::SommaireSurvol(rang) => {
            state.survol_sommaire = rang;
            Task::none()
        }

        Message::ReferencesRechercheBascule => {
            state.references_recherche_visible = !state.references_recherche_visible;
            Task::none()
        }

        Message::ProjetsRechercheBascule => {
            state.projets_recherche_visible = !state.projets_recherche_visible;
            Task::none()
        }

        Message::ProjetsEtageChoisie(choix) => {
            state.projets_etage = choix;
            Task::none()
        }

        // La liste des projets : étagère courante par défaut, tous sur
        // demande — et la recherche filtre par nom (bug-report-7, 18).
        Message::ProjetsFiltreBascule => {
            state.projets_tous = !state.projets_tous;
            Task::none()
        }

        Message::ProjetsRecherche(valeur) => {
            state.projets_recherche = valeur;
            Task::none()
        }

        Message::DragSurvol(etagere) => {
            state.drag_etagere = Some(etagere);
            Task::none()
        }

        Message::DragPoser(etagere) => {
            if let Some(projet) = state.drag_projet.take() {
                state.drag_etagere = None;

                match crate::project::definir_etagere(&projet, &etagere) {
                    Ok(()) => {
                        state.note = Some(crate::app::state::Note::info(format!(
                            "« {projet} » rejoint « {etagere} »."
                        )));
                    }
                    Err(message) => {
                        state.note = Some(crate::app::state::Note::erreur(message));
                    }
                }
            }

            Task::none()
        }

        Message::EtagereBrouillon(valeur) => {
            state.etagere_brouillon = valeur;
            Task::none()
        }

        Message::EtagereDefinir(projet, etagere) => {
            match crate::project::definir_etagere(&projet, &etagere) {
                Ok(()) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "« {projet} » rejoint « {etagere} »."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // La création depuis une étagère : le dialogue s'ouvre déjà
        // classé (bug 3 du rapport 2).
        Message::ProjetDepuisEtagere(etagere) => {
            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                norme: String::new(),
                etagere: etagere.clone(),
                nom: String::new(),
                titre: String::new(),
                auteur: String::new(),
                cible: None,
            });

            state.note = Some(crate::app::state::Note::info(format!(
                "Le projet rejoindra « {etagere} »."
            )));

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }

        Message::ProjetNorme(nom) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                dialogue.norme = nom;
            }

            Task::none()
        }

        Message::ProjetCreer => {
            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                norme: String::new(),
                etagere: String::new(),
                nom: String::new(),
                titre: String::new(),
                auteur: String::new(),
                cible: None,
            });

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }

        Message::ProjetRenommer(dir) => {
            let nom = state
                .projects
                .iter()
                .find(|projet| projet.dir == dir)
                .map(|projet| projet.name.clone())
                .unwrap_or_default();

            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                norme: String::new(),
                etagere: String::new(),
                nom,
                titre: String::new(),
                auteur: String::new(),
                cible: Some(dir),
            });

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }

        Message::ProjetValider => {
            let Some(dialogue) = state.projet_dialogue.clone() else {
                return Task::none();
            };

            let nom = dialogue.nom.trim().to_string();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Un projet porte un nom.".to_string(),
                ));
                return Task::none();
            }

            // Le verrou suit le dossier pendant tout le renommage. Les vues
            // modifiées sont enregistrées avant que leurs chemins changent.
            let mut verrou_renommage = None;
            let mut verrou_deja_detenu = false;
            if let Some(dir) = &dialogue.cible {
                verrou_deja_detenu = state.espace.verrous.contains_key(dir)
                    || (state.project.dir == *dir && state.verrou.is_some());
                if !verrou_deja_detenu {
                    match crate::project::Verrou::prendre(dir) {
                        Ok(verrou) => verrou_renommage = Some(verrou),
                        Err(probleme) => {
                            state.note = Some(crate::app::state::Note::erreur(format!(
                                "Renommage refusé : {probleme}"
                            )));
                            return Task::none();
                        }
                    }
                }
                if let Err(probleme) = state.sauvegarder_avant_deplacer(dir) {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Renommage annulé : {probleme}. Corrigez la sauvegarde et réessayez."
                    )));
                    return Task::none();
                }
                if verrou_renommage.is_none() {
                    verrou_renommage = state.espace.verrous.remove(dir);
                    if verrou_renommage.is_none() && state.project.dir == *dir {
                        verrou_renommage = state.verrou.take();
                    }
                }
            }

            let rendu = match &dialogue.cible {
                None => crate::project::creer_au_nom(&crate::project::projects_dir(), &nom),
                Some(dir) => crate::project::renommer(dir, &nom),
            };

            match rendu {
                Ok(dir) => {
                    // Les métadonnées choisies à la création : le projet
                    // naît avec les siennes (bug-report-7, point 9).
                    if dialogue.cible.is_none() {
                        let mut meta = crate::project::metadonnees::Metadonnees::base();
                        if !dialogue.titre.trim().is_empty() {
                            meta.definir(
                                "",
                                "titre".to_string(),
                                dialogue.titre.trim().to_string(),
                            );
                        }
                        if !dialogue.auteur.trim().is_empty() {
                            meta.definir(
                                "",
                                "auteur".to_string(),
                                dialogue.auteur.trim().to_string(),
                            );
                        }
                        let _ = meta.sauver(&dir);
                    }

                    if let Some(mut verrou) = verrou_renommage.take() {
                        verrou.suivre_deplacement(&dir);
                        state.espace.verrous.insert(dir.clone(), verrou);
                    }
                    if let Some(ancien) = &dialogue.cible
                        && ancien != &dir
                    {
                        state.suivre_projet_renomme(ancien, &dir);
                        state.save_session();
                    }
                    state.projet_dialogue = None;
                    // Le nom a changé : la liste des projets se relit pour
                    // que l'onglet et le panneau portent le nouveau nom.
                    state.projects = crate::project::list_projects();

                    // La norme choisie à la création devient celle du
                    // projet : elle s'y pose par la même voie que le
                    // panneau, et le modèle du document ne change pas.
                    if dialogue.cible.is_none()
                        && !dialogue.norme.is_empty()
                        && let Some(mise) = crate::project::norme(&dialogue.norme)
                    {
                        let _ = crate::mise_en_page::appliquer(&dir, &mise, false);
                    }

                    if dialogue.cible.is_none() && !dialogue.etagere.is_empty() {
                        let nom = dir
                            .file_name()
                            .map(|nom| nom.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let _ = crate::project::definir_etagere(&nom, &dialogue.etagere);
                    }

                    update(state, Message::OpenProject(dir))
                }
                Err(probleme) => {
                    if verrou_deja_detenu
                        && let (Some(dir), Some(verrou)) =
                            (&dialogue.cible, verrou_renommage.take())
                    {
                        state.espace.verrous.insert(dir.clone(), verrou);
                    }
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        Message::ProjetAnnuler => {
            state.projet_dialogue = None;
            Task::none()
        }

        Message::ProjetSupprimer(dir) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: "Supprimer ce projet ? Il part à la corbeille.".to_string(),
                    action: Message::ProjetSupprimer(dir),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            // La carte peut viser un projet autre que la vue courante. Un
            // verrou propre à cette cible reste tenu jusqu'au déplacement.
            let deja_verrouille = state.espace.verrous.contains_key(&dir)
                || (state.project.dir == dir && state.verrou.is_some());
            let mut temporaire = if deja_verrouille {
                None
            } else {
                match crate::project::Verrou::prendre(&dir) {
                    Ok(verrou) => Some(verrou),
                    Err(probleme) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "Suppression refusée : {probleme}"
                        )));
                        return Task::none();
                    }
                }
            };

            if let Err(probleme) = state.sauvegarder_avant_deplacer(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Corrigez la sauvegarde et réessayez."
                )));
                return Task::none();
            }

            if let Err(probleme) = state.preparer_suppression_projet(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Le projet reste en place."
                )));
                return Task::none();
            }

            match crate::project::supprimer(&dir) {
                Ok(corbeille) => {
                    if let Some(mut verrou) = temporaire.take() {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if let Some(mut verrou) = state.espace.verrous.remove(&dir) {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if state.project.dir == dir
                        && let Some(mut verrou) = state.verrou.take()
                    {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Le projet part à la corbeille : {}",
                        corbeille.display()
                    )));

                    state.projects.retain(|projet| projet.dir != dir);
                    state.retirer_projet_supprime(&dir);
                    Task::none()
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        // Ce qui défait demande son mot — la corbeille est derrière, mais
        // un geste qu'on fait parfois à tort mérite la question.
        Message::DeleteChapter(path) => {
            if state.geste_confirme {
                state.geste_confirme = false;
                return crate::app::fichiers::update(state, Message::DeleteChapter(path));
            }

            state.confirmation = Some(crate::app::state::Confirmation {
                question: "Supprimer ce document ? Il part à la corbeille.".to_string(),
                action: Message::DeleteChapter(path),
            });

            Task::none()
        }

        Message::DeleteTheme(name) => {
            if state.geste_confirme {
                state.geste_confirme = false;
                return crate::app::themes::update(state, Message::DeleteTheme(name));
            }

            state.confirmation = Some(crate::app::state::Confirmation {
                question: "Supprimer ce thème ? Il part à la corbeille.".to_string(),
                action: Message::DeleteTheme(name),
            });

            Task::none()
        }

        Message::ZoteroDelier => {
            if state.geste_confirme {
                state.geste_confirme = false;
                return liaison_zotero(state, Message::ZoteroDelier);
            }

            state.confirmation = Some(crate::app::state::Confirmation {
                question: "Délier ce projet de Zotero ? Il retrouvera sa bibliothèque locale."
                    .to_string(),
                action: Message::ZoteroDelier,
            });

            Task::none()
        }

        // Un clic avalé par un voile de fenêtre.
        Message::Rien => Task::none(),

        // La bibliothèque de normes : choisir, appliquer, enregistrer.
        Message::NormesNom(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.norme = nom;
            }

            Task::none()
        }

        Message::NormesAppliquer(nom) => {
            match crate::project::norme(&nom) {
                Some(mise) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = mise;
                        dialogue.norme = nom.clone();
                        dialogue.norme_originale = nom.clone();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est appliquée au document."
                    )));
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "La norme « {nom} » n'existe pas."
                    )));
                }
            }

            Task::none()
        }

        // Écraser la norme chargée : le premier temps de l'enregistrement,
        // quand le document en porte une (bug-report-9).
        Message::NormesEcraser => {
            let Some((nom, reglages)) = state
                .mise_en_page
                .as_ref()
                .filter(|dialogue| !dialogue.norme_originale.trim().is_empty())
                .map(|dialogue| {
                    (
                        dialogue.norme_originale.trim().to_string(),
                        dialogue.reglages.clone(),
                    )
                })
            else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Aucune norme chargée à écraser.".to_string(),
                ));
                return Task::none();
            };

            match crate::project::enregistrer_norme(&nom, &reglages) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est remplacée."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesEnregistrer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Une copie a un nom.".to_string(),
                ));
                return Task::none();
            }

            if crate::project::norme(nom).is_some() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "« {nom} » existe déjà — donnez un autre nom à la copie,                      ou écrasez la norme chargée."
                )));
                return Task::none();
            }

            match crate::project::enregistrer_norme(
                nom,
                &state
                    .mise_en_page
                    .as_ref()
                    .map(|d| d.reglages.clone())
                    .unwrap_or_default(),
            ) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.norme = nom.to_string();
                        dialogue.norme_originale = nom.to_string();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La copie « {nom} » est dans la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesSupprimer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                return Task::none();
            }

            match crate::project::supprimer_norme(nom) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut()
                        && dialogue.norme_originale.trim() == nom
                    {
                        // La norme n'existe plus : le document redevient un
                        // porteur de normes personnalisées.
                        dialogue.norme_originale.clear();
                        dialogue.norme.clear();
                        dialogue.modifie = true;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » a quitté la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // La grille de tableau : la taille choisie, la forme s'écrit.
        Message::TableauOuvrir => {
            state.tableau = Some(crate::app::state::TableauDialogue {
                rangs: "3".to_string(),
                colonnes: "3".to_string(),
            });

            Task::none()
        }

        Message::TableauRangs(valeur) => {
            if let Some(tableau) = state.tableau.as_mut() {
                tableau.rangs = valeur;
            }

            Task::none()
        }

        Message::TableauColonnes(valeur) => {
            if let Some(tableau) = state.tableau.as_mut() {
                tableau.colonnes = valeur;
            }

            Task::none()
        }

        Message::TableauAnnuler => {
            state.tableau = None;
            Task::none()
        }

        Message::TableauInserer => {
            let Some(tableau) = state.tableau.take() else {
                return Task::none();
            };

            let rangs: usize = tableau.rangs.trim().parse().unwrap_or(0).clamp(1, 200);
            let colonnes: usize = tableau.colonnes.trim().parse().unwrap_or(0).clamp(1, 20);

            if colonnes == 0 {
                state.note = Some(crate::app::state::Note::erreur(
                    "Un tableau a au moins une colonne.".to_string(),
                ));
                return Task::none();
            }

            let entete = format!("| {} |", vec![" "; colonnes].join(" | "));
            let trait_ = format!("| {} |", vec!["---"; colonnes].join(" | "));
            let rang = format!("| {} |", vec![" "; colonnes].join(" | "));

            let mut grille = vec![entete, trait_];
            grille.extend(std::iter::repeat_n(rang, rangs.saturating_sub(1)));
            let grille = grille.join("\n");

            state.editor.insert_at_cursor(&grille).map(Message::Editor)
        }

        Message::ReferencesReplier => {
            state.references_repliees = !state.references_repliees;
            Task::none()
        }

        Message::ReferencesFiltrer(filtre) => {
            state.references_filtre = filtre;
            Task::none()
        }

        Message::ModificateursTenus { controle, maj } => {
            state.controle_tenu = controle;
            state.maj_tenu = maj;
            Task::none()
        }

        // Shift + molette : le défilement horizontal (bug-report-9).
        Message::Molette(pas) => {
            // Sans `Ctrl`, la molette est au défilement — et elle l'est
            // déjà, prise par ce qui défile.
            if !state.controle_tenu {
                return Task::none();
            }

            update(
                state,
                Message::PreviewZoomStep(if pas > 0.0 { 1 } else { -1 }),
            )
        }

        // La confirmation : le geste attendu part, l'autre s'efface. Le
        // témoin fait passer le geste au travers de la question.
        Message::ConfirmationConfirmer => match state.confirmation.take() {
            Some(confirmation) => {
                state.geste_confirme = true;
                update(state, confirmation.action)
            }
            None => Task::none(),
        },

        Message::ConfirmationAnnuler => {
            // Refuser d'écraser un changement extérieur est une réponse pour ce
            // changement-ci : un autre en sera une question neuve.
            if let Some(crate::app::state::Confirmation {
                action: Message::SauvegardeForcee(empreinte),
                ..
            }) = state.confirmation.as_ref()
            {
                state.conflit_refuse = Some(*empreinte);
            }
            // Un import refusé ne laisse rien derrière lui.
            state.importation = None;
            state.confirmation = None;
            Task::none()
        }

        // La carte des raccourcis : une bascule, rien de plus.
        Message::AideBasculer => {
            state.aide = !state.aide;
            Task::none()
        }

        // Le chemin des données est visible dans l'aide, et s'ouvre d'un
        // geste : voir où vivent ses documents n'est pas une devinette.
        Message::OuvrirDossierDonnees => {
            crate::project::ouvrir_chemin(&crate::project::work_dir());
            Task::none()
        }

        // La récupération est à portée de geste : la corbeille garde ce qui
        // a été retiré, les sauvegardes gardent ce qui a été remplacé.
        Message::OuvrirCorbeille => {
            crate::project::ouvrir_chemin(&crate::project::corbeille());
            Task::none()
        }

        Message::OuvrirSauvegardes => {
            let dir = crate::project::sauvegardes_dir(&state.project.dir);
            let _ = std::fs::create_dir_all(&dir);
            crate::project::ouvrir_chemin(&dir);
            Task::none()
        }

        // Le domaine des réglages du document a son module.
        Message::MiseEnPageOuvrir
        | Message::MiseEnPageFermer
        | Message::MetaChamp(_, _, _)
        | Message::MetaAjouter
        | Message::MetaProfilNom(_)
        | Message::MetaProfilActiver(_)
        | Message::MetaProfilEnregistrer(_)
        | Message::MetaProfilSupprimer(_)
        | Message::MetaRemplacer
        | Message::MetaSupprimer(_)
        | Message::MiseEnPageChamp(_, _)
        | Message::MiseEnPageMolette(_, _)
        | Message::MiseEnPageBascule(_)
        | Message::MiseEnPagePanneau(_)
        | Message::MiseEnPageTypographieDefaut(_)
        | Message::MiseEnPageAccrocher(_)
        | Message::MiseEnPageAppliquer
        | Message::MiseEnPageImporter
        | Message::MiseEnPageImportee(_)
        | Message::MiseEnPageExporter
        | Message::MiseEnPageDestination(_, _)
        | Message::MiseEnPageNote(_)
        | Message::EntetePiedImage(_)
        | Message::EntetePiedImageChoisie(_, _) => {
            crate::app::reglages::update(state, message).unwrap_or_else(Task::none)
        }

        // Le domaine des fichiers et du projet a son module.
        Message::NewFile
        | Message::OpenFile
        | Message::FileOpened(_)
        | Message::OpenChapter(_)
        | Message::ChapterOpened(_, _, _)
        | Message::NewChapter
        | Message::NewProject
        | Message::ProjetCreerAvecModele(_)
        | Message::OpenProject(_)
        | Message::RefreshProject
        | Message::StartRename(_)
        | Message::RenameChanged(_)
        | Message::RenameCommit
        | Message::RenameCancel
        | Message::ImageInserer
        | Message::ImageChoisie(_)
        | Message::StyleCslImporter
        | Message::StyleCslChoisi(_)
        | Message::StyleZoteroChoisi(_)
        | Message::StyleParentTelecharger
        | Message::SommaireInserer
        | Message::BibliographieSectionInserer
        | Message::GlossaireSectionInserer
        | Message::InsertCitation(_)
        | Message::GlossaireInserer(_)
        | Message::ExtensionOuvrir
        | Message::ExtensionChoisir(_)
        | Message::ExtensionParam(_, _)
        | Message::ExtensionInserer
        | Message::ExtensionAnnuler
        | Message::NoteInserer
        | Message::NoteTexte(_)
        | Message::NoteValider
        | Message::NoteAnnuler
        | Message::FigureInserer
        | Message::EtiquetteInserer
        | Message::RenvoiInserer
        | Message::SaveFile
        | Message::SauvegardeForcee(_)
        | Message::ImportAppliquer
        | Message::SaveAs
        | Message::FileSaved(_)
        | Message::ExportPdf
        | Message::SymbolRenameStart
        | Message::SymbolRenameChanged(_)
        | Message::SymbolRenameAsk
        | Message::SymbolRenameApply
        | Message::SymbolRenameCancel
        | Message::PdfDestination(_)
        | Message::PdfExported(_)
        | Message::ProjetExportTw
        | Message::ProjetTwExporte(_)
        | Message::ProjetImportTw
        | Message::ProjetTwImporte(_)
        | Message::ProjetExportPortable
        | Message::ProjetPortableExporte(_)
        | Message::ProjetImportSauvegarde
        | Message::ProjetImportChoisi(_)
        | Message::ProjetImportConfirmer(_, _)
        | Message::ExportMarkdown
        | Message::ExportMarkdownChoisi(_)
        | Message::ExportDocx
        | Message::ExportDocxChoisi(_) => crate::app::fichiers::update(state, message),

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
        | Message::ZoteroExport(_, _, _, _)
        | Message::ZoteroActiverApi
        | Message::ZoteroSynchroniser
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
        // Le glossaire : la fiche, les gestes de la liste, la sauvegarde
        // du `glossaire.toml`.
        // La fenêtre du glossaire (bug-report-10, 5.3).
        Message::GlossaireFenetreOuvrir => {
            state.glossaire_fenetre = true;

            Task::none()
        }
        Message::GlossaireFenetreFermer => {
            state.glossaire_fenetre = false;
            state.glossaire_filtre.clear();

            Task::none()
        }
        Message::GlossaireCreer => {
            state.edition_glossaire = Some(crate::app::state::EditionGlossaire {
                cle: String::new(),
                court: String::new(),
                long: String::new(),
                description: String::new(),
                pluriel_court: String::new(),
                pluriel_long: String::new(),
                groupe: String::new(),
                creation: true,
            });

            iced::widget::operation::focus(crate::ui::glossaire::id_premier_champ())
        }
        Message::GlossaireModifier(cle) => {
            let entree = state
                .glossaire
                .iter()
                .find(|entree| entree.key == cle)
                .cloned();

            let Some(entree) = entree else {
                return Task::none();
            };

            state.edition_glossaire = Some(crate::app::state::EditionGlossaire {
                cle: entree.key,
                court: entree.short.unwrap_or_default(),
                long: entree.long.unwrap_or_default(),
                description: entree.description.unwrap_or_default(),
                pluriel_court: entree.plural.unwrap_or_default(),
                pluriel_long: entree.longplural.unwrap_or_default(),
                groupe: entree.group.unwrap_or_default(),
                creation: false,
            });

            iced::widget::operation::focus(crate::ui::glossaire::id_premier_champ())
        }
        Message::GlossaireSupprimer(cle) => {
            let mut glossaire = crate::project::glossaire::Glossaire {
                entrees: state.glossaire.clone(),
            };

            if glossaire.supprimer(&cle) {
                match glossaire.sauver(&state.project.dir) {
                    Ok(()) => {
                        state.glossaire = glossaire.entrees;
                        state.note = Some(crate::app::state::Note::info("Entrée supprimée."));
                    }
                    Err(erreur) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "Suppression impossible : {erreur}"
                        )));
                    }
                }
            }

            Task::none()
        }
        Message::SurvolGlossaire(cle) => {
            state.survol_glossaire = cle;
            Task::none()
        }
        Message::GlossaireRechercheBascule => {
            state.glossaire_recherche_visible = !state.glossaire_recherche_visible;
            Task::none()
        }
        Message::GlossaireReplier => {
            state.glossaire_replie = !state.glossaire_replie;
            Task::none()
        }
        Message::GlossaireFiltrer(filtre) => {
            state.glossaire_filtre = filtre;
            Task::none()
        }
        Message::EditionGlossaireChamp(champ, valeur) => {
            if let Some(edition) = state.edition_glossaire.as_mut() {
                match champ.as_str() {
                    "cle" => edition.cle = valeur,
                    "court" => edition.court = valeur,
                    "long" => edition.long = valeur,
                    "description" => edition.description = valeur,
                    "pluriel_court" => edition.pluriel_court = valeur,
                    "pluriel_long" => edition.pluriel_long = valeur,
                    "groupe" => edition.groupe = valeur,
                    _ => {}
                }
            }

            Task::none()
        }
        Message::EditionGlossaireFermer => {
            state.edition_glossaire = None;

            Task::none()
        }
        Message::EditionGlossaireEnregistrer => {
            let Some(edition) = state.edition_glossaire.clone() else {
                return Task::none();
            };

            // La clé est un identifiant : une clé fausse se refuse, la
            // fiche reste ouverte — elle ne jette jamais le travail.
            if !crate::project::glossaire::EntreeGlossaire::cle_valide(&edition.cle) {
                state.note = Some(crate::app::state::Note::erreur(
                    "La clé d'un terme est un identifiant : lettres, chiffres, - _ . —                      sans espace ni deux-points.",
                ));
                return Task::none();
            }

            let entree = crate::project::glossaire::EntreeGlossaire {
                key: edition.cle.clone(),
                short: champ_renseigne(&edition.court),
                long: champ_renseigne(&edition.long),
                description: champ_renseigne(&edition.description),
                plural: champ_renseigne(&edition.pluriel_court),
                longplural: champ_renseigne(&edition.pluriel_long),
                group: champ_renseigne(&edition.groupe),
            };

            let mut glossaire = crate::project::glossaire::Glossaire {
                entrees: state.glossaire.clone(),
            };
            glossaire.definir(entree);

            match glossaire.sauver(&state.project.dir) {
                Ok(()) => {
                    state.glossaire = glossaire.entrees;
                    state.edition_glossaire = None;
                    state.note = Some(crate::app::state::Note::info("Entrée enregistrée."));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement impossible : {erreur}"
                    )));
                }
            }

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
        | Message::ThemeLoaded(_) => crate::app::themes::update(state, message),

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
                            let ouverture =
                                crate::langage::parse::appel_autour(&contenu, note.debut + 1)
                                    .map(|(references, debut, fin)| {
                                        ouvrir_edition(state, references, debut, fin)
                                    })
                                    .unwrap_or_else(Task::none);

                            Task::batch([task, ouverture])
                        }
                        crate::langage::marques::Nature::PiedDePage => {
                            // L'overlay de note : le texte s'édite en bas
                            // de l'écran, sans déplier la note sous le
                            // curseur ni déranger le fil du document.
                            let texte = &contenu[note.debut..note.fin];
                            let nu = texte
                                .strip_prefix("^[")
                                .and_then(|s| s.strip_suffix(']'))
                                .unwrap_or(texte);
                            state.note_overlay = Some(crate::app::state::NoteOverlay {
                                contenu: nu.to_string(),
                                debut: note.debut,
                                fin: note.fin,
                            });
                            Task::none()
                        }
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

        Message::TitreInserer(niveau) => {
            // L'insertion simple au curseur : `#` × n + espace. Pas de
            // reformatage de bloc — on tape, comme à la main.
            let marque = "#".repeat(niveau.clamp(1, 6) as usize);
            state
                .editor
                .insert_at_cursor(&format!("{marque} "))
                .map(Message::Editor)
        }

        Message::Format(action) => {
            if !state.editor.is_intermediate() {
                return Task::none();
            }

            // TOPBAR-04-07 : toute couleur choisie devient la couleur courante.
            if let crate::app::message::FormatAction::Couleur(ref nom) = action {
                state.derniere_couleur = Some(nom.clone());
            }

            // Le bouton « citation » est un `@` tapé par la main de l'outil.
            let declencheur_tape = matches!(action, crate::app::message::FormatAction::Citation);

            state.render.words_pending = true;
            let edit = state.editor.apply_format(action).map(Message::Editor);
            let completion = request_completion(state, true, declencheur_tape);

            Task::batch([edit, completion])
        }

        // Le sélecteur de couleur : la roue, la carte, et la couleur qui
        // s'applique au texte. La fermeture n'écrit rien — l'annulation est
        // le droit de changer d'avis.
        Message::PickerCouleurOuvrir => {
            let (teinte, saturation, valeur) = state
                .derniere_couleur
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: None,
                champ: None,
                reglage: None,
            });
            Task::none()
        }

        Message::PickerCouleurFermer => {
            state.picker_couleur = None;
            Task::none()
        }

        Message::PickerCouleurTeinte(teinte) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.teinte = teinte;
            }
            Task::none()
        }

        Message::PickerCouleurSatVal(saturation, valeur) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.saturation = saturation;
                picker.valeur = valeur;
            }
            Task::none()
        }

        // La vignette d'une couleur des normes ouvre la roue, pour ce
        // champ-là (demande du 2026-11-04).
        Message::PickerCouleurPourReglage(champ) => {
            let courante = state
                .mise_en_page
                .as_ref()
                .map(|dialogue| dialogue.reglages.valeur(champ))
                .unwrap_or_default();
            // La roue s'ouvre **sur la couleur courante** — un nom (« gray »),
            // un hex, ce que la pastille sait teindre (bug-report-8, T6.2).
            let hex = crate::ui::reglages::couleur_depuis(&courante).map(|c| {
                format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0).round() as u8,
                    (c.g * 255.0).round() as u8,
                    (c.b * 255.0).round() as u8
                )
            });
            let (teinte, saturation, valeur) = hex
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: None,
                champ: None,
                reglage: Some(champ),
            });
            Task::none()
        }

        // Le choix d'une couleur personnalisée : elle s'enregistre pour
        // la pastille de la barre — et rien ne va au texte tant qu'on ne
        // l'applique pas (bug-report-10, 5.4).
        Message::PickerCouleurEnregistrer => {
            let Some(picker) = state.picker_couleur.as_ref() else {
                return Task::none();
            };

            let hex = crate::ui::picker_couleur::hsv_vers_hex(
                picker.teinte,
                picker.saturation,
                picker.valeur,
            );
            state.derniere_couleur = Some(hex.clone());
            state.note = Some(crate::app::state::Note::info(format!(
                "« {hex} » est votre dernière couleur — la pastille de la barre en insère la balise."
            )));

            Task::none()
        }

        Message::PickerCouleurAppliquer => {
            let Some(picker) = state.picker_couleur.take() else {
                return Task::none();
            };
            let hex = crate::ui::picker_couleur::hsv_vers_hex(
                picker.teinte,
                picker.saturation,
                picker.valeur,
            );
            // La couleur employée devient la dernière — la pastille la
            // garde, quel que soit le mode (bug-report-10, 5.4).
            state.derniere_couleur = Some(hex.clone());
            if let Some(reglage) = picker.reglage {
                return update(state, Message::MiseEnPageChamp(reglage, hex));
            }
            match picker.champ {
                // Le thème : le champ du thème reçoit le hex.
                Some(champ) => update(state, Message::ThemeEditeurChamp(champ, hex)),
                // Le texte : la sélection se teinte.
                None => update(
                    state,
                    Message::Format(crate::app::message::FormatAction::Couleur(hex)),
                ),
            }
        }

        // Le code hex pilote la roue : ce qui se tape devient la couleur,
        // et si la saisie n'est pas encore un hex, elle reste sous les
        // doigts sans rien casser.
        Message::PickerCouleurHex(texte) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.hex_saisi = Some(texte.clone());
                if let Some((teinte, saturation, valeur)) =
                    crate::ui::picker_couleur::hex_vers_hsv(&texte)
                {
                    picker.teinte = teinte;
                    picker.saturation = saturation;
                    picker.valeur = valeur;
                }
            }
            Task::none()
        }

        // Une couleur rapide suit la **cible de la roue** : le réglage
        // des normes, le champ de thème, ou le texte — jamais le texte
        // quand on règle le document (bug-report-9, T1.1). La roue reste
        // ouverte pour la suite.
        Message::PickerCouleurRapide(nom) => {
            let cible = state
                .picker_couleur
                .as_ref()
                .map(|picker| (picker.reglage, picker.champ));

            match cible {
                Some((Some(reglage), _)) => update(state, Message::MiseEnPageChamp(reglage, nom)),
                Some((None, Some(champ))) => update(state, Message::ThemeEditeurChamp(champ, nom)),
                _ => update(
                    state,
                    Message::Format(crate::app::message::FormatAction::Couleur(nom)),
                ),
            }
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

        Message::NoteOverlayChamp(texte) => {
            if let Some(note) = state.note_overlay.as_mut() {
                note.contenu = texte;
            }
            Task::none()
        }

        Message::NoteOverlayEnregistrer => {
            if let Some(note) = state.note_overlay.take() {
                let nouveau = format!("^[{}]", note.contenu);
                let tache = state
                    .editor
                    .replace_span(note.debut, note.fin, &nouveau)
                    .map(Message::Editor);
                return tache;
            }
            Task::none()
        }

        Message::NoteOverlayFermer => {
            state.note_overlay = None;
            Task::none()
        }

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

        // Une fenêtre de rendu doit montrer son document : la première
        // compilation, puis son lecteur (bug-report-9).
        Message::RenduInitialiser => {
            if state.render.pending || state.render.export_en_cours {
                return Task::none();
            }

            // Le partage d'abord : si ce texte a déjà son rendu, la
            // fenêtre le montre — le même PDF pour tout le monde.
            let empreinte = state.compilation_fingerprint();

            if let Some(partage) = state.rendus_partages.get(&empreinte) {
                if partage.document.is_some() {
                    state.render.document.clone_from(&partage.document);
                }

                if partage.lecteur.is_some() {
                    state.render.lecteur.clone_from(&partage.lecteur);
                    let total = state
                        .render
                        .document
                        .as_deref()
                        .map_or(0, crate::typst::render::page_count);
                    state.render.invalidate_rasterization(total);
                    return rasterize_visible(state);
                }
            }

            if state.render.document.is_none() && !state.render.done {
                state.render.pending = true;
                return compile_task(state);
            }

            if state.render.document.is_some()
                && state.render.lecteur.is_none()
                && !state.render.export_tente
            {
                state.render.export_tente = true;
                return generer_le_pdf_du_rendu(state);
            }

            Task::none()
        }

        Message::RenderTick => {
            // Le geste de zoom est fini : le rendu net part, une seule
            // fois, et non un rendu par cran de molette.
            if state
                .render
                .zoom_depuis
                .is_some_and(|depuis| depuis.elapsed() >= DUREE_GESTE_ZOOM)
            {
                state.render.zoom_depuis = None;
                return rasterize_visible(state);
            }

            if let Some(tache) = crate::app::epreuves::driver_zoom(state) {
                return tache;
            }

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

            // Chaque fenêtre de rendu visible montre son document — celle
            // du démarrage aussi (bug-report-9 : l'initialisation laissait
            // l'aperçu vide tant qu'on ne basculait pas de mode).
            if let Some(tache) = rendre_les_fenetres_de_rendu(state) {
                return tache;
            }

            // Le compte de mots est une donnée de la vue, et la vue n'a pas le
            // droit de modifier l'état : il se calcule donc ici, au rythme des
            // tics, et seulement quand le texte a changé. Le calculer chez elle,
            // c'était cloner le document entier à chaque image — deux fois,
            // puisque la barre d'état et le panneau le demandent chacun.
            // Les glyphes suivent le texte, **quel que soit** le chemin qui
            // l'a changé : frappe, citation insérée, remplacement, bloc
            // déplacé. Leur repose est sensible au changement : un document
            // immobile ne coûte rien, et il n'y a plus de chemin oublié.
            state.editor.marques_a_jour();

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
            // Un retour périmé ne se fait pas passer pour l'état courant : le
            // texte a changé depuis la compilation, ou le projet portant ce
            // chemin a été remplacé sous le même nom. Comparer les deux
            // empreintes — celle que `compile` rapporte et celle que la vue
            // compilerait maintenant — est la seule façon de le savoir ; le
            // tick suivant recompile ce qui est réellement à l'écran.
            if outcome.content != state.compilation_fingerprint() {
                return Task::none();
            }
            state.render.export_tente = false;
            state.render.done = true;
            let chapter_page = outcome.chapter_page;

            state.render.content = outcome.content;
            state.render.errors = outcome.errors;
            state.render.warnings = outcome.warnings;

            // Un échec ne vide pas l'aperçu : la dernière image bonne
            // reste affichée sous les erreurs, et ses textures gardent
            // leur place jusqu'à ce qu'une nouvelle compilation les
            // remplace. C'était la promesse du double tampon, tenue.
            // Le document compilé rejoint le partage : toute fenêtre qui
            // montre ce texte le montre lui, jamais une copie (bug-report-10).
            state
                .rendus_partages
                .entry(state.render.content.clone())
                .or_default()
                .completant(&crate::app::state::RenduPartage {
                    document: outcome.document.clone(),
                    lecteur: None,
                });

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

        // La réponse de la génération du PDF lancée par la bascule vers la
        // vue rendu : le lecteur s'installe, l'aperçu se refait au neuf.
        Message::PdfApercu(resultat) => {
            state.render.export_en_cours = false;

            let octets = match resultat {
                Ok(octets) => octets,
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    return Task::none();
                }
            };

            match crate::typst::render::ouvrir_pdf(octets) {
                Some(pdf) => {
                    let partage = std::sync::Arc::new(pdf);

                    if let Some(empreinte) = state.render.export_empreinte.take() {
                        state
                            .rendus_partages
                            .entry(empreinte)
                            .or_default()
                            .completant(&crate::app::state::RenduPartage {
                                document: None,
                                lecteur: Some(partage.clone()),
                            });
                    }

                    state.render.lecteur = Some(partage);
                    let total = state
                        .render
                        .document
                        .as_deref()
                        .map_or(0, crate::typst::render::page_count);
                    state.render.invalidate_rasterization(total);
                    rasterize_visible(state)
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(
                        "Le PDF généré n'est pas lisible : le rendu reste en attente.".to_string(),
                    ));
                    Task::none()
                }
            }
        }

        Message::PageRendered(revision, index, image, boite) => {
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
                    crate::app::state::Tranche {
                        gauche: boite.0,
                        haut: boite.1,
                        largeur: boite.2,
                        hauteur: boite.3,
                        image: Handle::from_rgba(image.width, image.height, image.pixels),
                    },
                );

                // Une image peut arriver après qu'on a défilé ailleurs :
                // la refuser tout de suite, plutôt que de la garder pour rien.
                state.render.bound_page_cache();
            }

            // Une place s'est libérée : ce qui attendait part.
            rasterize_visible(state)
        }

        Message::ViewportChanged(first, last, vue, scroll) => {
            // Le rectangle de fenêtre ne vient que d'un vrai défilement :
            // les sondes de taille du panneau le rapportent à l'origine,
            // et une ancre de zoom sur cette origine serait fausse.
            if scroll.is_some() {
                state.render.fenetre = Some(vue);
            }
            let scroll = scroll.unwrap_or(state.render.scroll);
            let bounds = vue.size();
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
                && state.render.viewport == Some(bounds)
                && state.render.scroll == scroll;

            state.render.visible = Some((first, last));
            state.render.scroll = scroll;

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
            // Une page sans tranche couvrant la fenêtre visible part au
            // rendu : défiler hors de la marge coûte un rendu, défiler
            // dedans rien du tout.
            if let Some(document) = state.render.document.clone() {
                let sizes =
                    crate::ui::preview::sizes_of_with_zoom(&document, bounds, state.render.zoom);

                for index in first..=last {
                    let Some(taille) = sizes.get(index).copied() else {
                        continue;
                    };
                    let haut = crate::ui::preview::scroll_offset_for_page(&sizes, index);
                    let region = crate::ui::preview::region_pour_page(
                        taille,
                        haut,
                        scroll,
                        (bounds.width, bounds.height),
                        0.0,
                    );

                    if !state
                        .render
                        .page_cache
                        .get(&index)
                        .is_some_and(|tranche| crate::ui::preview::tranche_couvre(tranche, region))
                    {
                        state.render.stale.insert(index);
                    }
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
                let ecrit = state.persist();
                state.refresh_context();

                // Le serveur suit les écritures : c'est ainsi qu'il sait
                // recompiler le projet, et que les diagnostics des *autres*
                // chapitres se mettent à jour. Une sauvegarde refusée n'a rien
                // écrit dont il pourrait témoigner.
                if ecrit && let Some(session) = state.lsp.session.as_mut() {
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

            // La bascule rafraîchit : ce qui reparaît est ce qu'on écrit,
            // pas la mémoire du dernier rendu. Jamais de purge — l'ancien
            // reste affiché jusqu'au neuf, sans clignotement.
            if state.layout.preview_visible {
                return compile_task(state);
            }

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

        Message::RechercheOuvrir
        | Message::RechercheFermer
        | Message::RechercheMotif(_)
        | Message::RechercheBasculerMode
        | Message::RechercheRemplacement(_)
        | Message::RechercheSuivant
        | Message::RecherchePrecedent
        | Message::RechercheRemplacer
        | Message::RechercheRemplacerTout => recherche(state, message),

        // Les gestes d'un élément de liste (bug-report-10, 3.2, 3.3, 3.6).
        Message::ListeIndenter => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = indenter_la_ligne(&contenu, offset);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        Message::ListeDesindenter => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = desindenter_la_ligne(&contenu, offset);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        // `Entrée` crée l'élément suivant (bug-report-10, 3.4).
        Message::ListeEnchainer => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();

            match marque_suivante(&contenu, offset) {
                Some(marque) => state.editor.insert_at_cursor(&marque).map(Message::Editor),
                None => Task::none(),
            }
        }

        Message::ListeMonter | Message::ListeDescendre => {
            let haut = matches!(message, Message::ListeMonter);
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = deplacer_la_ligne(&contenu, offset, haut);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }

        Message::BlocMonter => state.editor.deplacer_le_bloc(true).map(Message::Editor),
        Message::BlocDescendre => state.editor.deplacer_le_bloc(false).map(Message::Editor),

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
/// La recherche : chercher, naviguer, remplacer.
///
/// Tout le travail est dans [`crate::app::recherche`], qui ne voit ni
/// l'éditeur ni l'écran ; ici ne se font que les gestes.
fn recherche(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::RechercheOuvrir => {
            state.recherche = Some(crate::app::recherche::Recherche::default());

            return iced::widget::operation::focus(crate::ui::recherche::id_du_motif());
        }

        Message::RechercheFermer => {
            state.recherche = None;
        }

        Message::RechercheMotif(motif) => {
            if let Some(etat) = state.recherche.as_mut() {
                etat.motif = motif;
            }

            return rafraichir_la_recherche(state);
        }

        Message::RechercheBasculerMode => {
            if let Some(etat) = state.recherche.as_mut() {
                etat.mode_regex = !etat.mode_regex;
            }

            return rafraichir_la_recherche(state);
        }

        Message::RechercheRemplacement(remplacement) => {
            if let Some(etat) = state.recherche.as_mut() {
                etat.remplacement = remplacement;
            }
        }

        Message::RechercheSuivant => return sauter_dans_la_recherche(state, 1),
        Message::RecherchePrecedent => return sauter_dans_la_recherche(state, -1),
        Message::RechercheRemplacer => return remplacer_courant(state),
        Message::RechercheRemplacerTout => return remplacer_tout(state),

        _ => {}
    }

    Task::none()
}

/// Recompte les occurrences du motif dans le texte.
fn rafraichir_la_recherche(state: &mut State) -> Task<Message> {
    let Some(etat) = state.recherche.as_mut() else {
        return Task::none();
    };

    let texte = state.editor.content();
    let trouve = crate::app::recherche::occurrences(etat.mode_regex, &etat.motif, &texte);
    let apercu =
        crate::app::recherche::apercu(etat.mode_regex, &etat.motif, &etat.remplacement, &texte);

    let etat = state.recherche.as_mut().expect("recherche ouverte");
    etat.courante = 0;

    match trouve {
        Ok(occurrences) => {
            etat.occurrences = occurrences;
            etat.erreur = None;
        }
        Err(erreur) => {
            etat.occurrences = Vec::new();
            etat.erreur = Some(erreur);
        }
    }

    // L'aperçu des changements : ce que chaque occurrence devient — voir
    // avant de remplacer (R6). Un motif cassé n'a pas d'aperçu qui mente.
    etat.apercu = apercu.unwrap_or_default();

    // FIND-02 : surligner les occurrences dans l'éditeur — le motif
    // effectif : la regex du motif, ou le littéral échappé.
    if etat.motif.is_empty() {
        let _ = state.editor.editor.close_search_dialog();
    } else {
        let motif = crate::app::recherche::motif_effectif(etat.mode_regex, &etat.motif);
        state.editor.editor.set_search_query(motif);
    }

    sauter_dans_la_recherche(state, 0)
}

/// Va à l'occurrence voulue, et la montre dans l'éditeur.
fn sauter_dans_la_recherche(state: &mut State, direction: i8) -> Task<Message> {
    let Some(etat) = state.recherche.as_mut() else {
        return Task::none();
    };

    if etat.occurrences.is_empty() {
        return Task::none();
    }

    let total = etat.occurrences.len();
    if direction > 0 {
        etat.courante = (etat.courante + 1) % total;
    } else if direction < 0 {
        etat.courante = (etat.courante + total - 1) % total;
    }

    let (debut, fin) = etat.occurrences[etat.courante];

    state
        .editor
        .selectionner_plage(debut, fin)
        .map(Message::Editor)
}

/// Remplace l'occurrence courante — les groupes du motif y sont rappelés.
fn remplacer_courant(state: &mut State) -> Task<Message> {
    let Some(etat) = state.recherche.as_ref() else {
        return Task::none();
    };

    let Some((debut, fin)) = etat.occurrences.get(etat.courante).copied() else {
        return Task::none();
    };

    let (motif, remplacement) = (etat.motif.clone(), etat.remplacement.clone());
    let texte = state.editor.content();

    let neuf = crate::app::recherche::remplacement(
        etat.mode_regex,
        &motif,
        &remplacement,
        &texte,
        debut,
        fin,
    );

    match neuf {
        Ok(neuf) => {
            let tache = state
                .editor
                .replace_span(debut, fin, &neuf)
                .map(Message::Editor);
            Task::batch([tache, rafraichir_la_recherche(state)])
        }
        Err(erreur) => {
            state.note = Some(crate::app::state::Note::erreur(erreur));
            Task::none()
        }
    }
}

/// Remplace tout, d'un geste.
fn remplacer_tout(state: &mut State) -> Task<Message> {
    let Some(etat) = state.recherche.as_ref() else {
        return Task::none();
    };

    let (motif, remplacement) = (etat.motif.clone(), etat.remplacement.clone());
    let texte = state.editor.content();

    let neuf =
        crate::app::recherche::remplacer_tout(etat.mode_regex, &motif, &remplacement, &texte);

    match neuf {
        Ok(neuf) => {
            let compte = etat.occurrences.len();
            let tache = state
                .editor
                .replace_span(0, texte.len(), &neuf)
                .map(Message::Editor);

            state.note = Some(crate::app::state::Note::info(format!(
                "{compte} occurrence(s) remplacée(s)."
            )));

            Task::batch([tache, rafraichir_la_recherche(state)])
        }
        Err(erreur) => {
            state.note = Some(crate::app::state::Note::erreur(erreur));
            Task::none()
        }
    }
}

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
                Message::ZoteroCollections,
            )
        }

        Message::ZoteroCollections(resultat) => match resultat {
            Ok(collections) => {
                state.liaison = Some(crate::app::state::LiaisonZotero::Choix(collections));
                Task::none()
            }
            Err(erreur) => {
                // Le réglage d'API est le seul désordre qu'on puisse
                // réparer soi-même — et le préparer avant même le premier
                // lancement de Zotero est le meilleur accueil : le panneau
                // propose le geste dans les deux cas.
                let reglable = matches!(
                    erreur,
                    crate::zotero::client::Erreur::ApiFermee
                        | crate::zotero::client::Erreur::Injoignable(_)
                );

                state.liaison = Some(crate::app::state::LiaisonZotero::Erreur {
                    motif: erreur.to_string(),
                    reglable,
                });

                Task::none()
            }
        },

        Message::ZoteroActiverApi => {
            match crate::zotero::activer_l_api_locale() {
                Ok(profils) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Réglage posé dans {} profil(s) : redémarrez Zotero — ou cochez la                          case dans Zotero → Réglages → Avancé — puis « Réessayer ».",
                        profils.len()
                    )));

                    // La piste est tracée : on retente aussitôt, au cas où
                    // la case serait déjà prise.
                    return Task::perform(
                        async { crate::zotero::client::collections() },
                        Message::ZoteroCollections,
                    );
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                }
            }

            Task::none()
        }

        Message::ZoteroSynchroniser => {
            // La collection bouge avec Zotero : l'export du projet suit.
            let Some(lien) = state.project.zotero.clone() else {
                return Task::none();
            };

            let (demande, rendu, projet) = (
                lien.cle.clone(),
                (lien.cle, lien.nom),
                state.project.dir.clone(),
            );

            Task::perform(
                async move { crate::zotero::client::bibtex(&demande).map_err(|e| e.to_string()) },
                move |resultat| Message::ZoteroExport(rendu.0, rendu.1, projet.clone(), resultat),
            )
        }

        Message::ZoteroChoisir(cle, nom) => {
            // L'export vient de Zotero : on le demande avant d'écrire quoi
            // que ce soit au projet.
            state.liaison = Some(crate::app::state::LiaisonZotero::Chargement);

            let (demande, rendu, projet) = (cle.clone(), (cle, nom), state.project.dir.clone());

            Task::perform(
                async move { crate::zotero::client::bibtex(&demande).map_err(|e| e.to_string()) },
                move |resultat| Message::ZoteroExport(rendu.0, rendu.1, projet.clone(), resultat),
            )
        }

        Message::ZoteroExport(cle, nom, projet, resultat) => {
            // La requête est rattachée au projet demandeur : si l'on a
            // changé de projet en cours de route, rien ne s'écrit ici —
            // jamais dans un projet qui n'a rien demandé.
            if projet != state.project.dir {
                state.liaison = None;
                state.note = Some(crate::app::state::Note::info(
                    "La synchronisation Zotero visait un autre projet : rien n'a été écrit dans celui-ci.",
                ));

                return Task::none();
            }

            match resultat {
                Ok(bibtex) => {
                    let dir = state.project.dir.clone();

                    // Les clés se stabilisent avant l'écriture : une clé
                    // attribuée ne bouge plus, quoi que la collection devienne.
                    // Les clés de la bibliothèque locale sont prises d'abord —
                    // une notice Zotero ne vient jamais heurter un livre tenu
                    // par l'écrivain lui-même.
                    let mut pris = std::collections::BTreeSet::new();

                    for entree in &state.bibliography {
                        pris.insert(entree.key.clone());
                    }

                    let chemin = dir.join("citations.json");
                    let mut trousseau = crate::zotero::keys::Keyring::load(&chemin);
                    let bibtex = trousseau.stabiliser(&bibtex, &pris);

                    // La transaction : clés, bibliothèque et liaison — les
                    // trois fichiers produits, ensemble ou pas du tout. Jamais
                    // de clés revendiquées sans bibliothèque, ni de liaison
                    // sans clés.
                    let ecrits = {
                        let cles = trousseau.texte();
                        let liaison = crate::zotero::Lien { cle, nom }.texte();

                        match (cles, liaison) {
                            (Ok(cles), Ok(liaison)) => vec![
                                (chemin.clone(), cles),
                                (dir.join("references-zotero.bib"), bibtex),
                                (dir.join("zotero.json"), liaison),
                            ],
                            (Err(erreur), _) | (_, Err(erreur)) => {
                                state.liaison = Some(crate::app::state::LiaisonZotero::Erreur {
                                    motif: format!(
                                        "La bibliothèque n'a pas pu être préparée (rien n'a changé) : {erreur}"
                                    ),
                                    reglable: false,
                                });

                                return Task::none();
                            }
                        }
                    };

                    if let Err(erreur) = crate::project::ecrire_en_transaction(&ecrits) {
                        state.liaison = Some(crate::app::state::LiaisonZotero::Erreur {
                            motif: format!(
                                "La bibliothèque n'a pas pu être écrite (rien n'a changé) : {erreur}"
                            ),
                            reglable: false,
                        });

                        return Task::none();
                    }

                    // Le document cite les bibliothèques du projet : sans
                    // cela, les citations Zotero sont introuvables et un
                    // `references.bib` manquant casse la compilation
                    // (bug-report-9).
                    let main_typ = dir.join("main.typ");

                    if let Ok(contenu) = std::fs::read_to_string(&main_typ)
                        && let Some(branche) =
                            crate::project::model::brancher_les_bibliographies(&dir, &contenu)
                        && let Err(erreur) = crate::project::ecrire_atomique(&main_typ, &branche)
                    {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "La bibliographie n'a pas pu être branchée dans main.typ : {erreur}"
                        )));
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
                    state.liaison = Some(crate::app::state::LiaisonZotero::Erreur {
                        motif: erreur,
                        reglable: false,
                    });
                    Task::none()
                }
            }
        }

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

/// Ce que la table du clavier a besoin de savoir de l'écran.
///
/// Rien de plus : un état complet suppose un projet ouvert, ce que la
/// table n'a pas à savoir — et c'est ce qui la rend vérifiable.
#[derive(Debug, Default, Clone, Copy)]
struct Situation {
    /// L'outil de tableau est ouvert — `Échap` annule.
    tableau: bool,
    /// La vue des projets est ouverte — `Échap` la ferme.
    projets: bool,
    /// L'éditeur de thème est ouvert — `Échap` le ferme.
    theme: bool,
    /// Le dialogue de projet est ouvert — `Échap` annule.
    projet: bool,
    /// Une confirmation attend son mot — `Échap` est « Annuler ».
    confirmation: bool,
    /// Le curseur est sur une ligne de liste (bug-report-10, 3.2).
    sur_liste: bool,
    /// Le composeur est ouvert : sa frappe est à lui.
    composeur: bool,
    /// Une fenêtre est ouverte — c'est le contrat qui s'applique.
    fenetre: bool,
    /// La fiche de référence : `Entrée` y enregistre.
    fiche: bool,
    /// Ce qui est au-dessus de la pile des fenêtres.
    aide: bool,
    recherche: bool,
    liaison: bool,
    /// Le panneau de la note de bas de page — `Échap` annule.
    note: bool,
    /// Le composeur d'extension — `Échap` annule.
    extension: bool,
    /// La liste de complétion, le survol — et le mode focus.
    completion: bool,
    survol: bool,
    focus: bool,
}

impl Situation {
    fn de(state: &State) -> Self {
        Situation {
            tableau: state.tableau.is_some(),
            projets: state.projets_vue,
            theme: state.editeur_theme.is_some(),
            projet: state.projet_dialogue.is_some(),
            confirmation: state.confirmation.is_some(),
            sur_liste: est_sur_une_liste(&state.editor.content(), state.editor.cursor_offset()),
            composeur: state.composeur.is_some(),
            fenetre: fenetre_ouverte(state),
            fiche: state.edition_reference.is_some(),
            aide: state.aide,
            recherche: state.recherche.is_some(),
            liaison: state.liaison.is_some(),
            note: state.note_dialogue.is_some(),
            extension: state.extension.is_some(),
            completion: state.lsp.completion.is_some(),
            survol: state.lsp.hover.is_some(),
            focus: state.layout.focus_mode,
        }
    }
}

/// Ce qu'une touche décide — la table du clavier, sans son exécution.
///
/// Séparer le geste de l'action : c'est ce qui rend la table vérifiable,
/// et c'est ce qui empêche qu'une fenêtre ouverte fasse oublier le reste.
#[derive(Debug)]
enum Action {
    /// Un message à faire passer, ni plus ni moins.
    Message(Message),

    /// Le focus va au champ suivant, ou revient au précédent.
    FocusSuivant,
    FocusPrecedent,

    /// Enregistrer : le seul geste qui n'est pas un message.
    Enregistrer,

    /// La touche ne fait rien : elle était à quelqu'un d'autre.
    Rien,
}

/// La marque de l'élément suivant (bug-report-10, 3.4) : la ligne
/// d'avant porte le marqueur, la suivante reçoit le sien — `- ` reste,
/// `3. ` devient `4. `. Rien quand la ligne d'avant n'est pas une liste.
fn marque_suivante(contenu: &str, offset: usize) -> Option<String> {
    let offset = offset.min(contenu.len());
    let debut = contenu[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    if debut == 0 {
        return None;
    }

    // La ligne d’avant : de son début au saut qui la termine — la
    // première du texte commence à zéro.
    let avant = contenu[..debut - 1]
        .rfind(char::from(10))
        .map(|i| i + 1)
        .unwrap_or(0);
    let ligne = &contenu[avant..debut - 1];
    let en_tete = ligne.len() - ligne.trim_start().len();
    let tete = ligne.trim_start();

    let marque = if tete.starts_with("- ") {
        "- ".to_string()
    } else {
        let chiffres = tete.chars().take_while(|c| c.is_ascii_digit()).count();
        if chiffres == 0 || !tete[chiffres..].starts_with(". ") {
            return None;
        }

        let numero: usize = tete[..chiffres].parse().ok()?;
        format!("{}. ", numero + 1)
    };

    Some(format!("{}{}", " ".repeat(en_tete), marque))
}

/// La ligne du curseur : sa plage dans le contenu, et son texte.
fn ligne_du_curseur(contenu: &str, offset: usize) -> (usize, usize, &str) {
    let offset = offset.min(contenu.len());
    let debut = contenu[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let fin = contenu[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(contenu.len());

    (debut, fin, &contenu[debut..fin])
}

/// Le curseur est-il sur une ligne de liste ?
fn est_sur_une_liste(contenu: &str, offset: usize) -> bool {
    let (_, _, ligne) = ligne_du_curseur(contenu, offset);
    let tete = ligne.trim_start();

    if tete.starts_with("- ") {
        return true;
    }

    let chiffres = tete.chars().take_while(|c| c.is_ascii_digit()).count();
    chiffres > 0 && tete[chiffres..].starts_with(". ")
}

/// Indente la ligne du curseur (bug-report-10, 3.2).
fn indenter_la_ligne(contenu: &str, offset: usize) -> (String, usize) {
    let (debut, _, _) = ligne_du_curseur(contenu, offset);
    let mut neuf = String::with_capacity(contenu.len() + 2);
    neuf.push_str(&contenu[..debut]);
    neuf.push_str("  ");
    neuf.push_str(&contenu[debut..]);

    (neuf, offset + 2)
}

/// Désindente la ligne du curseur (bug-report-10, 3.3) — deux espaces
/// s'il y en a, sinon rien.
fn desindenter_la_ligne(contenu: &str, offset: usize) -> (String, usize) {
    let (debut, _, ligne) = ligne_du_curseur(contenu, offset);
    let en_trop = (ligne.len() - ligne.trim_start().len()).min(2);
    let mut neuf = String::with_capacity(contenu.len());
    neuf.push_str(&contenu[..debut]);
    neuf.push_str(&contenu[debut + en_trop..]);

    (neuf, offset.saturating_sub(en_trop))
}

/// Déplace la ligne du curseur vers le haut ou vers le bas (bug-report-10,
/// 3.6) — le curseur suit sa ligne.
fn deplacer_la_ligne(contenu: &str, offset: usize, haut: bool) -> (String, usize) {
    let (debut, fin, _) = ligne_du_curseur(contenu, offset);
    let dans_la_ligne = offset - debut;

    if haut {
        if debut == 0 {
            return (contenu.to_string(), offset);
        }

        let avant = contenu[..debut - 1].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let mut neuf = String::with_capacity(contenu.len());
        neuf.push_str(&contenu[..avant]);
        neuf.push_str(&contenu[debut..fin]);
        neuf.push('\n');
        neuf.push_str(&contenu[avant..debut - 1]);
        neuf.push_str(&contenu[fin..]);

        (neuf, avant + dans_la_ligne)
    } else {
        if fin >= contenu.len() {
            return (contenu.to_string(), offset);
        }

        let apres = fin + 1;
        let fin_suivante = contenu[apres..]
            .find('\n')
            .map(|i| apres + i)
            .unwrap_or(contenu.len());
        let mut neuf = String::with_capacity(contenu.len());
        neuf.push_str(&contenu[..debut]);
        neuf.push_str(&contenu[apres..fin_suivante]);
        neuf.push('\n');
        neuf.push_str(&contenu[debut..fin]);
        neuf.push_str(&contenu[fin_suivante..]);

        (neuf, debut + (fin_suivante - apres) + 1 + dans_la_ligne)
    }
}

/// La table du clavier, dans l'ordre où les choses la prennent.
fn plan_touche(
    etat: Situation,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Action {
    use iced::keyboard::{Key, key::Named};

    // Le composeur, quand il est ouvert, prend ses touches : sa frappe est
    // à lui, et rien ne doit l'entamer.
    if etat.composeur && est_touche_composeur(key) {
        return Action::Rien; // le relais lui donne, ici on ne touche pas.
    }

    // **Le contrat des fenêtres.** Toute fenêtre se conduit de même :
    // `Tabulation` circule, `Maj+Tabulation` revient, `Échap` ferme celle
    // du dessus. Un seul contrat pour toutes — c'est ce qui fait qu'on
    // n'a jamais à chercher comment sortir.
    if etat.fenetre {
        match key {
            Key::Named(Named::Tab) if modifiers.shift() => return Action::FocusPrecedent,
            Key::Named(Named::Tab) => return Action::FocusSuivant,
            Key::Named(Named::Escape) => return Action::Message(fermeture_du_dessus(etat)),

            // La fiche de référence est un formulaire : `Entrée` y
            // enregistre. Ailleurs, la touche est aux champs, qui ont la
            // leur par `on_submit`.
            Key::Named(Named::Enter) if etat.fiche => {
                return Action::Message(Message::EditionReferenceEnregistrer);
            }

            _ => {}
        }
    }

    // Les touches nommées, d'une seule main quand elles le peuvent.
    match key {
        Key::Named(Named::F1) => return Action::Message(Message::AideBasculer),
        Key::Named(Named::F2) => return Action::Message(Message::SymbolRenameStart),
        Key::Named(Named::F3) => {
            return Action::Message(if !etat.recherche {
                Message::RechercheOuvrir
            } else if modifiers.shift() {
                Message::RecherchePrecedent
            } else {
                Message::RechercheSuivant
            });
        }
        Key::Named(Named::F11) => return Action::Message(Message::ToggleFocusMode),
        Key::Named(Named::F12) => return Action::Message(Message::GoToDefinition),
        Key::Named(Named::Escape) => {
            // Ce qui s'est ouvert en dernier se ferme en premier : la
            // liste, puis le survol, puis le mode focus — il y a toujours
            // une sortie.
            if etat.completion || etat.survol {
                return Action::Message(Message::CompletionDismiss);
            }
            if etat.focus {
                return Action::Message(Message::ToggleFocusMode);
            }
            return Action::Rien;
        }
        _ => {}
    }

    // Sur une liste, `Tabulation` change de niveau — `Maj` ou `Ctrl`
    // reviennent (bug-report-10, 3.2, 3.3) — et `Ctrl+Maj+Haut/Bas`
    // déplace l'élément, pas le bloc tout entier (3.6).
    if etat.sur_liste {
        match key {
            Key::Named(Named::Tab) if modifiers.shift() || modifiers.control() => {
                return Action::Message(Message::ListeDesindenter);
            }
            Key::Named(Named::Tab) => return Action::Message(Message::ListeIndenter),
            // `Entrée` crée l'élément suivant (bug-report-10, 3.4) : la
            // touche a déjà posé son saut de ligne chez le widget, le
            // geste n'ajoute que la marque suivante.
            Key::Named(Named::Enter) => return Action::Message(Message::ListeEnchainer),
            Key::Named(Named::ArrowUp) if modifiers.control() && modifiers.shift() => {
                return Action::Message(Message::ListeMonter);
            }
            Key::Named(Named::ArrowDown) if modifiers.control() && modifiers.shift() => {
                return Action::Message(Message::ListeDescendre);
            }
            _ => {}
        }
    }

    // Le bloc entier se déplace : `Ctrl+Maj+Haut/Bas`.
    if modifiers.control()
        && modifiers.shift()
        && matches!(key, Key::Named(Named::ArrowUp | Named::ArrowDown))
    {
        return Action::Message(if matches!(key, Key::Named(Named::ArrowUp)) {
            Message::BlocMonter
        } else {
            Message::BlocDescendre
        });
    }

    if !modifiers.control() || modifiers.alt() || modifiers.logo() {
        return Action::Rien;
    }

    if modifiers.control() && matches!(key, Key::Named(Named::Tab)) {
        return if modifiers.shift() {
            Action::Message(Message::PanneauSuivant)
        } else {
            Action::Message(Message::OngletSuivant)
        };
    }

    let Key::Character(character) = key else {
        return Action::Rien;
    };

    match character.to_lowercase().as_str() {
        "s" => Action::Enregistrer,
        "i" => Action::Message(Message::ShowHover),
        "n" => Action::Message(Message::NewFile),
        "o" => Action::Message(Message::OpenFile),
        "f" => Action::Message(Message::RechercheOuvrir),
        "t" => Action::Message(if modifiers.shift() {
            Message::TitrePrecedent
        } else {
            Message::TitreSuivant
        }),
        // L'aperçu suit — ou ne suit plus — le curseur.
        "l" if modifiers.shift() => Action::Message(Message::TogglePreviewSync),
        // Les normes du document : `Ctrl+P`, la page. Il n'y a pas
        // d'impression ici — l'export a la sienne.
        "p" => Action::Message(Message::MiseEnPageOuvrir),
        // Le zoom : les touches de tout lecteur de document.
        "e" if modifiers.shift() => Action::Message(Message::ModeTypst),
        "e" => Action::Message(Message::ModeBasculer),
        // Scinder la vue : `\` divise, Maj inverse la direction.
        "\\" if modifiers.shift() => {
            Action::Message(Message::Scinder(iced::widget::pane_grid::Edge::Bottom))
        }
        "\\" => Action::Message(Message::Scinder(iced::widget::pane_grid::Edge::Right)),
        // Titres : Ctrl+1 à Ctrl+6 insèrent le niveau demandé.
        "1" => Action::Message(Message::TitreInserer(1)),
        "2" => Action::Message(Message::TitreInserer(2)),
        "3" => Action::Message(Message::TitreInserer(3)),
        "4" => Action::Message(Message::TitreInserer(4)),
        "5" => Action::Message(Message::TitreInserer(5)),
        "6" => Action::Message(Message::TitreInserer(6)),
        "+" | "=" => Action::Message(Message::PreviewZoomStep(1)),
        "-" => Action::Message(Message::PreviewZoomStep(-1)),
        "0" => Action::Message(Message::PreviewZoom(
            crate::app::state::PreviewZoomMode::FitWidth,
        )),
        _ => Action::Rien,
    }
}

/// Le message de fermeture de la fenêtre du dessus de la pile.
fn fermeture_du_dessus(etat: Situation) -> Message {
    if etat.tableau {
        return Message::TableauAnnuler;
    }

    if etat.note {
        return Message::NoteAnnuler;
    }

    if etat.extension {
        return Message::ExtensionAnnuler;
    }

    if etat.projets {
        return Message::ProjetsVue;
    }

    if etat.theme {
        return Message::ThemeEditeurFermer;
    }

    if etat.projet {
        return Message::ProjetAnnuler;
    }

    if etat.confirmation {
        return Message::ConfirmationAnnuler;
    }

    if etat.aide {
        return Message::AideBasculer;
    }
    if etat.recherche {
        return Message::RechercheFermer;
    }
    if etat.liaison {
        return Message::ZoteroLiaisonFermer;
    }
    if etat.fiche {
        return Message::EditionReferenceFermer;
    }
    Message::MiseEnPageFermer
}

/// Quelque fenêtre est-elle ouverte ?
fn fenetre_ouverte(state: &State) -> bool {
    state.tableau.is_some()
        || state.note_dialogue.is_some()
        || state.extension.is_some()
        || state.projets_vue
        || state.editeur_theme.is_some()
        || state.projet_dialogue.is_some()
        || state.confirmation.is_some()
        || state.aide
        || state.recherche.is_some()
        || state.liaison.is_some()
        || state.edition_reference.is_some()
        || state.mise_en_page.is_some()
}

fn handle_shortcut(
    state: &mut State,
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    // Le composeur garde la sienne, relaïée même capturée.
    if let Some(task) = composeur_touche(state, &key, modifiers) {
        return task;
    }

    // La liste de complétion prend les siennes, quand elle est ouverte.
    if let Some(task) = completion_shortcut(state, &key, modifiers) {
        return task;
    }

    match plan_touche(Situation::de(state), &key, modifiers) {
        Action::Enregistrer => {
            state.persist();
            state.refresh_context();
            Task::none()
        }
        Action::Message(message) => update(state, message),
        Action::FocusSuivant => iced::widget::operation::focus_next(),
        Action::FocusPrecedent => iced::widget::operation::focus_previous(),
        Action::Rien => Task::none(),
    }
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

    // Le PDF est la source du rendu : sans lui, rien à rasteriser — la
    // bascule vers la vue rendu le génère systématiquement.
    let Some(pdf) = state.render.lecteur.clone() else {
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
    let ecran = f64::from(scale_factor).max(crate::typst::render::MIN_SCREEN_SCALE);

    let sizes = crate::ui::preview::sizes_of_with_zoom(&document, box_size, state.render.zoom);
    let scroll = state.render.scroll;
    let fenetre = (box_size.width, box_size.height);

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
            let pdf = pdf.clone();

            // La densité est celle de **l'affichage** : la page est montrée
            // réduite — le panneau est plus étroit qu'une page — et le rendu
            // n'a besoin que des pixels que cette réduction consomme. Elle se
            // calcule page par page, parce qu'une page en paysage n'a pas le
            // même rapport qu'une page en portrait.
            let page = crate::typst::render::page_size(&document, index).unwrap_or((595.3, 841.9));
            let density = crate::typst::render::pixel_per_pt(
                crate::ui::preview::display_ratio_for_zoom(box_size, page, state.render.zoom),
                scale_factor,
            );

            // La **région** à rendre : la fenêtre visible, élargie de la
            // marge — jamais la page entière (bug-report-9, fluidité).
            let taille = sizes.get(index).copied().unwrap_or((595.3, 841.9));
            let haut = crate::ui::preview::scroll_offset_for_page(&sizes, index);
            let boite =
                crate::ui::preview::region_pour_page(taille, haut, scroll, fenetre, MARGE_RENDU);
            // Fractions de la page -> pixels de sortie, à la densité du rendu.
            let (x0, y0) = (
                (boite.0 * taille.0 * ecran as f32) as u32,
                (boite.1 * taille.1 * ecran as f32) as u32,
            );
            let (largeur, hauteur) = (
                (boite.2 * taille.0 * ecran as f32).ceil() as u32,
                (boite.3 * taille.1 * ecran as f32).ceil() as u32,
            );

            Task::perform(
                async move {
                    crate::typst::render::render_region(
                        &pdf, index, density, x0, y0, largeur, hauteur,
                    )
                },
                move |image| Message::PageRendered(revision, index, image, boite),
            )
        })
        .collect();

    Task::batch(tasks)
}

/// La bascule vers la vue rendu génère systématiquement le PDF (bug-report-9).
///
/// C'est lui que l'aperçu affiche, et lui que l'export écrit : le montreur
/// ne travaille jamais que sur le fichier livré. La génération est celle de
/// l'export — même appel, mêmes octets.
/// Chaque fenêtre de rendu visible montre son document.
///
/// Au démarrage comme quand un panneau se montre, l'aperçu n'attend
/// personne : la vue reçoit l'ordre de compiler, puis de fabriquer son
/// lecteur. C'était la bascule de mode qui le faisait — et elle n'arrive
/// jamais pour une fenêtre déjà ouverte (bug-report-9).
fn rendre_les_fenetres_de_rendu(state: &mut State) -> Option<Task<Message>> {
    let mut ids: Vec<u64> = state
        .espace
        .grille
        .iter()
        .filter_map(|(_, p)| p.onglets.get(p.actif).copied())
        .collect();
    ids.push(state.espace.active);
    ids.sort_unstable();
    ids.dedup();

    let mut taches = Vec::new();

    for id in ids {
        let active = id == state.espace.active;
        let (mode, compile, pdf) = if active {
            (
                state.mode,
                state.render.document.is_none() && !state.render.done && !state.render.pending,
                state.render.document.is_some()
                    && state.render.lecteur.is_none()
                    && !state.render.export_en_cours
                    && !state.render.export_tente,
            )
        } else {
            let Some(vue) = state.espace.vues.get(&id) else {
                continue;
            };
            (
                vue.mode,
                vue.render.document.is_none() && !vue.render.done && !vue.render.pending,
                vue.render.document.is_some()
                    && vue.render.lecteur.is_none()
                    && !vue.render.export_en_cours
                    && !vue.render.export_tente,
            )
        };

        if mode != crate::app::state::ModeVue::Rendu || (!compile && !pdf) {
            continue;
        }

        taches.push(Task::done(Message::RetourVue(
            id,
            Box::new(Message::RenduInitialiser),
        )));
    }

    (!taches.is_empty()).then(|| Task::batch(taches))
}

fn generer_le_pdf_du_rendu(state: &mut State) -> Task<Message> {
    let dir = state.document_dir();
    let (entry, overrides) = state.compilation_unit();
    state.render.export_en_cours = true;
    // L'empreinte du texte exporté : le registre s'en souvient, et les
    // autres fenêtres du même fichier prendront ce PDF (bug-report-10).
    state.render.export_empreinte = Some(state.compilation_fingerprint());

    Task::perform(
        async move { crate::typst::render::render_pdf(&dir, &entry, &overrides) },
        |octets| {
            Message::PdfApercu(octets.ok_or_else(|| {
                "La compilation a échoué : corrigez les erreurs signalées dans le panneau."
                    .to_string()
            }))
        },
    )
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
        id_du_scrollable(state),
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

    let box_size = state
        .render
        .viewport
        .unwrap_or_else(crate::ui::preview::default_viewport);
    let document = state.render.document.clone();

    // L'ancre du zoom : le point sous le curseur — le milieu de la fenêtre
    // quand le curseur n'est pas connu. C'est lui qui ne doit pas bouger
    // (bug-report-9 : « centrée sur l'emplacement du curseur »).
    let fenetre = state
        .render
        .fenetre
        .unwrap_or(iced::Rectangle::new(iced::Point::ORIGIN, box_size));
    let curseur = state.layout.cursor;
    let (cx, cy) = (
        (curseur.x - fenetre.x).clamp(0.0, fenetre.width),
        (curseur.y - fenetre.y).clamp(0.0, fenetre.height),
    );

    // Le rapport ancien → nouveau, lu sur la page qui porte l'ancre.
    let facteur = document
        .as_deref()
        .map(|document| {
            let avant =
                crate::ui::preview::sizes_of_with_zoom(document, box_size, state.render.zoom);
            let apres = crate::ui::preview::sizes_of_with_zoom(document, box_size, zoom);
            let y = state.render.scroll.1 + cy;
            let index = (0..avant.len())
                .find(|index| {
                    crate::ui::preview::scroll_offset_for_page(&avant, *index) + avant[*index].1 > y
                })
                .unwrap_or(0);
            apres[index].1 / avant[index].1.max(1.0)
        })
        .unwrap_or(1.0);

    state.render.zoom = zoom;
    let total = state
        .render
        .document
        .as_deref()
        .map_or(0, crate::typst::render::page_count);
    state.render.invalidate_rasterization(total);
    state.render.bound_page_cache();

    // Le geste continue : le rendu net part quand il s'arrête — un cran de
    // molette ne vaut pas un rendu (bug-report-9, geste différé).
    state.render.zoom_depuis = Some(std::time::Instant::now());

    // La cible de défilement : l'ancre reste sous le curseur. Elle est
    // **imposée** : c'est aussi ce qui neutralise le défilement que la
    // molette a laissé fuiter pendant ctrl+molette.
    let cible_x = (state.render.scroll.0 + cx) * facteur - cx;
    let cible_y = (state.render.scroll.1 + cy) * facteur - cy;

    let (largeur_contenu, hauteur_contenu) = document
        .as_deref()
        .map(|document| {
            let apres = crate::ui::preview::sizes_of_with_zoom(document, box_size, zoom);
            let mut largeur: f32 = 0.0;
            for (l, _) in &apres {
                largeur = largeur.max(*l);
            }
            let hauteur: f32 = apres.iter().map(|(_, h)| *h).sum();
            (largeur, hauteur)
        })
        .unwrap_or((0.0, 0.0));

    let cible_x = cible_x.clamp(0.0, (largeur_contenu - fenetre.width).max(0.0));
    let cible_y = cible_y.clamp(0.0, (hauteur_contenu - fenetre.height).max(0.0));

    iced::advanced::widget::operate(iced::advanced::widget::operation::scrollable::scroll_to::<
        Message,
    >(
        id_du_scrollable(state),
        iced::advanced::widget::operation::scrollable::AbsoluteOffset {
            x: Some(cible_x),
            y: Some(cible_y),
        },
    ))
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
    let cle = (
        state.project.dir.clone(),
        state
            .editor
            .path
            .clone()
            .unwrap_or_else(|| state.project.main.clone()),
    );
    let dir = state.document_dir();
    let (entry, overrides, preparation) = state.compilation_unit_avec_diagnostics();

    // Les métadonnées du projet deviennent les entrées `sys.inputs` : ce
    // sont elles que les quick parts appellent ({titre}, {auteur}… —
    // bug-report-7, point 9). Avant, personne ne les remplissait.
    let entrees = crate::project::metadonnees::Metadonnees::charger(&dir).entrees();

    Task::perform(
        async move {
            let mut resultat =
                crate::typst::render::compile_avec_entrees(&dir, &entry, &overrides, &entrees);
            // Les citations neutralisées pendant la préparation sont des
            // avertissements au même titre que ceux du compilateur — les
            // masquer ici les condamnerait au silence.
            resultat.warnings.splice(0..0, preparation);
            resultat
        },
        move |resultat| Message::RetourDocument(cle, Box::new(Message::RenderDone(resultat))),
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
    iced::event::listen_with(|event, status, _window| message_evenement(event, status))
}

/// Traduction des événements natifs, y compris ceux capturés par le rendu.
fn message_evenement(event: Event, status: Status) -> Option<Message> {
    match event {
        Event::Window(iced::window::Event::CloseRequested) => Some(Message::FermetureDemandee),
        // Ctrl seul est annoncé par ModifiersChanged, indépendamment de
        // toute frappe. Le suivre ici évite aussi d'avaler Ctrl+S, Ctrl+E…
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::ModificateursTenus {
                controle: modifiers.control(),
                maj: modifiers.shift(),
            })
        }
        Event::Window(iced::window::Event::Unfocused) => Some(Message::ModificateursTenus {
            controle: false,
            maj: false,
        }),
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match status {
            Status::Ignored => Some(Message::RawKey(key, modifiers)),
            Status::Captured => None,
        },

        Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) if !modifiers.control() => {
            Some(Message::ModificateursTenus {
                controle: false,
                maj: modifiers.shift(),
            })
        }

        // La molette tient le zoom — `Ctrl` de la souris, comme dans tout
        // lecteur de document. Le défilement reste au reste.
        Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
            let pas = match delta {
                iced::mouse::ScrollDelta::Lines { y, .. }
                | iced::mouse::ScrollDelta::Pixels { y, .. } => y,
            };

            (pas != 0.0).then_some(Message::Molette(pas))
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests_de_la_molette {
    use super::*;
    use iced::keyboard::Key;

    #[test]
    fn les_raccourcis_ctrl_restent_transmis_et_la_perte_de_focus_relache_ctrl() {
        let frappe = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character("s".into()),
            modified_key: Key::Character("s".into()),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyS),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::CTRL,
            text: None,
            repeat: false,
        });
        assert!(matches!(
            message_evenement(frappe.clone(), Status::Ignored),
            Some(Message::RawKey(_, _))
        ));
        assert!(message_evenement(frappe, Status::Captured).is_none());
        assert!(matches!(
            message_evenement(
                Event::Window(iced::window::Event::Unfocused),
                Status::Ignored,
            ),
            Some(Message::ModificateursTenus {
                controle: false,
                maj: false
            })
        ));
    }

    #[test]
    fn ctrl_seul_puis_la_molette_zoome_sans_autre_frappe() {
        let ctrl = message_evenement(
            Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::CTRL)),
            Status::Captured,
        );
        assert!(
            matches!(
                ctrl,
                Some(Message::ModificateursTenus { controle: true, .. })
            ),
            "Ctrl seul doit armer le zoom, sans attendre une lettre"
        );
        assert!(matches!(message_evenement(
            Event::Mouse(iced::mouse::Event::WheelScrolled {
                delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            }), Status::Captured,
        ), Some(Message::Molette(pas)) if pas == 1.0));
        assert!(matches!(
            message_evenement(
                Event::Keyboard(keyboard::Event::ModifiersChanged(
                    keyboard::Modifiers::default()
                )),
                Status::Captured,
            ),
            Some(Message::ModificateursTenus {
                controle: false,
                ..
            })
        ));
    }
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

    // ---- La table du clavier -------------------------------------

    fn touche(c: &str) -> iced::keyboard::Key {
        iced::keyboard::Key::Character(c.into())
    }

    fn nomme(n: iced::keyboard::key::Named) -> iced::keyboard::Key {
        iced::keyboard::Key::Named(n)
    }

    /// F1 ouvre la carte des raccourcis — et rien ne l'en empêche.
    #[test]
    fn f1_ouvre_la_carte_partout() {
        let vide = Situation::default();

        assert!(matches!(
            plan_touche(
                vide,
                &nomme(iced::keyboard::key::Named::F1),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::AideBasculer)
        ));

        // Même une fenêtre ouverte ne prend pas F1.
        let fenetre = Situation {
            fenetre: true,
            recherche: true,
            ..Situation::default()
        };

        assert!(matches!(
            plan_touche(
                fenetre,
                &nomme(iced::keyboard::key::Named::F1),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::AideBasculer)
        ));
    }

    /// `Échap` ferme la fenêtre du dessus, pas une autre.
    #[test]
    fn echap_ferme_celle_du_dessus() {
        let deux = Situation {
            fenetre: true,
            recherche: true,
            aide: true,
            ..Situation::default()
        };

        // La carte est au-dessus : elle part la première.
        assert!(matches!(
            plan_touche(
                deux,
                &nomme(iced::keyboard::key::Named::Escape),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::AideBasculer)
        ));

        let une = Situation {
            fenetre: true,
            recherche: true,
            ..Situation::default()
        };

        assert!(matches!(
            plan_touche(
                une,
                &nomme(iced::keyboard::key::Named::Escape),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::RechercheFermer)
        ));
    }

    /// `Tabulation` circule dans les fenêtres — toutes, pas une seule.
    #[test]
    fn tabulation_circule_dans_les_fenetres() {
        let fenetre = Situation {
            fenetre: true,
            ..Situation::default()
        };

        assert!(matches!(
            plan_touche(
                fenetre,
                &nomme(iced::keyboard::key::Named::Tab),
                iced::keyboard::Modifiers::empty()
            ),
            Action::FocusSuivant
        ));
        assert!(matches!(
            plan_touche(
                fenetre,
                &nomme(iced::keyboard::key::Named::Tab),
                iced::keyboard::Modifiers::SHIFT
            ),
            Action::FocusPrecedent
        ));
    }

    /// F3 cherche ; la recherche fermée, il l'ouvre d'abord.
    #[test]
    fn f3_suit_la_recherche() {
        let vide = Situation::default();

        assert!(matches!(
            plan_touche(
                vide,
                &nomme(iced::keyboard::key::Named::F3),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::RechercheOuvrir)
        ));

        let ouverte = Situation {
            fenetre: true,
            recherche: true,
            ..Situation::default()
        };

        assert!(matches!(
            plan_touche(
                ouverte,
                &nomme(iced::keyboard::key::Named::F3),
                iced::keyboard::Modifiers::empty()
            ),
            Action::Message(Message::RechercheSuivant)
        ));
        assert!(matches!(
            plan_touche(
                ouverte,
                &nomme(iced::keyboard::key::Named::F3),
                iced::keyboard::Modifiers::SHIFT
            ),
            Action::Message(Message::RecherchePrecedent)
        ));
    }

    /// Le zoom répond aux touches de tout lecteur de document.
    #[test]
    fn le_zoom_suit_les_lecteurs() {
        let vide = Situation::default();
        let ctrl = iced::keyboard::Modifiers::CTRL;

        assert!(matches!(
            plan_touche(vide, &touche("+"), ctrl),
            Action::Message(Message::PreviewZoomStep(1))
        ));
        assert!(matches!(
            plan_touche(vide, &touche("-"), ctrl),
            Action::Message(Message::PreviewZoomStep(-1))
        ));
        assert!(matches!(
            plan_touche(vide, &touche("0"), ctrl),
            Action::Message(Message::PreviewZoom(_))
        ));
    }

    /// Une touche nue n'est jamais un raccourci : taper reste de la frappe.
    #[test]
    fn la_frappe_n_est_jamais_un_raccourci() {
        let vide = Situation::default();

        for c in ["a", "e", "s", "t", "p", "l"] {
            assert!(
                matches!(
                    plan_touche(vide, &touche(c), iced::keyboard::Modifiers::empty()),
                    Action::Rien
                ),
                "{c} sans modificateur ne devrait rien faire"
            );
        }
    }

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

#[cfg(test)]
mod tests_plan_touche {
    use super::{Action, Message, Situation, plan_touche};
    use iced::keyboard::{Key, Modifiers, key::Named};

    fn touche(caractere: &str) -> Key {
        Key::Character(caractere.into())
    }

    /// Une frappe de texte ne lance **aucun** raccourci : les raccourcis
    /// exigent le Contrôleur, et la saisie reste à la saisie (R6, « sans
    /// conflit avec la saisie »).
    #[test]
    fn une_frappe_sans_ctrl_ne_lance_aucun_raccourci() {
        let etat = Situation::default();

        for caractere in [
            "s", "i", "n", "o", "f", "t", "p", "e", "1", "2", "3", "4", "5", "6", "+", "0", "\\",
        ] {
            assert!(
                matches!(
                    plan_touche(etat, &touche(caractere), Modifiers::empty()),
                    Action::Rien
                ),
                "« {caractere} » nu ne doit rien lancer"
            );

            // Même avec Maj : le texte garde ses capitales.
            assert!(
                matches!(
                    plan_touche(etat, &touche(caractere), Modifiers::SHIFT),
                    Action::Rien
                ),
                "« {caractere} » en majuscule ne doit rien lancer"
            );
        }
    }

    /// Les raccourcis annoncés font leur geste — et l'aide a sa touche.
    #[test]
    fn les_raccourcis_ctrl_font_leur_geste() {
        let etat = Situation::default();

        assert!(matches!(
            plan_touche(etat, &touche("s"), Modifiers::CTRL),
            Action::Enregistrer
        ));
        assert!(matches!(
            plan_touche(etat, &touche("f"), Modifiers::CTRL),
            Action::Message(Message::RechercheOuvrir)
        ));
        assert!(matches!(
            plan_touche(etat, &touche("1"), Modifiers::CTRL),
            Action::Message(Message::TitreInserer(1))
        ));
        assert!(matches!(
            plan_touche(etat, &Key::Named(Named::F1), Modifiers::empty()),
            Action::Message(Message::AideBasculer)
        ));
    }

    /// Contrôleur + Alt : le raccourci ne part pas — les combinaisons du
    /// système gardent leurs touches.
    #[test]
    fn ctrl_alt_ne_lance_pas_le_raccourci() {
        let etat = Situation::default();

        assert!(matches!(
            plan_touche(etat, &touche("s"), Modifiers::CTRL | Modifiers::ALT),
            Action::Rien
        ));
    }
}

#[cfg(test)]
mod tests_des_gestes_de_liste {
    use super::*;

    fn contenu() -> String {
        let saut = &char::from(10).to_string();
        ["- un", "  - sous", "- deux", ""].join(saut)
    }

    /// Le curseur reconnaît une ligne de liste (bug-report-10, 3.2).
    #[test]
    fn le_curseur_reconnait_une_liste() {
        let texte = contenu();
        assert!(
            est_sur_une_liste(&texte, 2),
            "la première ligne est une puce"
        );
        assert!(est_sur_une_liste(&texte, 10), "la ligne indentée aussi");
        assert!(
            est_sur_une_liste(&texte, texte.len() - 1),
            "la dernière l'est encore"
        );

        let paragraphe = "Du texte.".to_string();
        assert!(
            !est_sur_une_liste(&paragraphe, 3),
            "un paragraphe n'est pas une liste"
        );
    }

    /// Tab augmente le niveau, Shift+Tab le diminue (3.2, 3.3).
    #[test]
    fn le_niveau_monte_et_redescend() {
        let texte = contenu();

        let (indente, curseur) = indenter_la_ligne(&texte, 2);
        assert!(
            indente.starts_with("  - un"),
            "le niveau monte : {indente:?}"
        );
        assert_eq!(curseur, 4, "le curseur suit son texte");

        let (revient, curseur) = desindenter_la_ligne(&indente, curseur);
        assert_eq!(revient, texte, "le retour est exact");
        assert_eq!(curseur, 2, "le curseur revient");
    }

    /// `Entrée` crée l'élément suivant (3.4) : la marque suit — la puce
    /// reste, le numéro avance.
    #[test]
    fn la_suite_de_la_liste_se_continue() {
        let saut = &char::from(10).to_string();
        let contenu = ["- un", "1. deux", ""].join(saut);

        // Le curseur vient de poser son saut au milieu : la ligne d'avant
        // porte le marqueur, la suivante reçoit le sien.
        let apres_un = contenu.find("1. deux").unwrap();
        assert_eq!(
            marque_suivante(&contenu, apres_un),
            Some("- ".to_string()),
            "la puce continue la puce"
        );
        assert_eq!(
            marque_suivante(&contenu, contenu.len()),
            Some("2. ".to_string()),
            "le numéro avance"
        );

        let paragraphe = ["Du texte.", ""].join(saut);
        assert_eq!(
            marque_suivante(&paragraphe, paragraphe.len()),
            None,
            "hors liste, rien ne s'enchaîne"
        );
    }

    /// L'élément se déplace, et le curseur le suit (3.6).
    #[test]
    fn l_element_se_deplace() {
        let texte = contenu();
        let offset = 2; // le curseur sur « - un »

        let (bas, curseur) = deplacer_la_ligne(&texte, offset, false);
        assert!(bas.starts_with("  - sous"), "le premier descend : {bas:?}");
        assert_eq!(curseur, 11, "le curseur suit sa ligne");

        let (haut, _) = deplacer_la_ligne(&bas, curseur, true);
        assert_eq!(haut, texte, "le déplacement inverse est exact");
    }
}
