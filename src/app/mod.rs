pub mod assistance;
pub mod epreuves;
mod journal;
pub mod message;
pub mod state;

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
            .filter(|path| state.project.owns(path));

        let chapter = wanted.or_else(|| {
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
        }

        state.refresh_context();
        // Les thèmes sont lus après le contexte : c'est le premier endroit où
        // l'état est complet, et la fenêtre doit s'ouvrir avec le thème retenu.
        refresh_themes(&mut state);

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
        self.export_status = None;
        self.render.reset();
        // Le compte de mots du document qu'on ouvre, tout de suite : la barre
        // d'état et le panneau le lisent, et attendre le prochain tic
        // afficherait un zéro entre-temps.
        self.render.words = Some(crate::project::count_words(&self.editor.content()));
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

        let wanted = crate::lsp::client::Session::uri_for(&path);

        if self.lsp.document.as_deref() == Some(wanted.as_str()) {
            return;
        }

        let text = self.editor.content();

        if let Some(session) = self.lsp.session.as_mut() {
            session.open_document(&path, &text);
            self.lsp.document = Some(session.uri().to_string());
            self.lsp.diagnostics.clear();

            return;
        }

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

        let overrides = match open {
            // L'assemblage est l'entrée : il n'a pas à figurer aussi en mémoire.
            Some(path) if assembly_is_open => {
                let _ = path;

                Vec::new()
            }
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
        if let Some(path) = self.editor.path.clone() {
            if let Err(error) = self.editor.save() {
                eprintln!("Sauvegarde de {path:?} : {error}");
            }

            self.save_session();
        }
    }
}

/// Nom du projet créé au premier démarrage.
const FIRST_PROJECT: &str = "mon-travail";

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Editor(message) => {
            // Le texte a peut-être changé : le compte de mots est à refaire, au
            // prochain tic, et non à chaque image.
            state.render.words_pending = true;

            let task = state.editor.update(message).map(Message::Editor);

            // C'est la frappe qui décide de la liste, et elle seule : après
            // chaque message de l'éditeur, on regarde si le texte demande une
            // complétion à l'endroit du curseur.
            let completion = request_completion(state);

            if std::env::var_os("TYPST_WRITER_LSP_VERBOSE").is_some() {
                report_lsp(state);
            }

            Task::batch(vec![task, completion])
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

        Message::NewFile => {
            // Un « nouveau document » est désormais un nouveau chapitre : c'est
            // l'unité de travail du projet.
            update(state, Message::NewChapter)
        }

        Message::OpenFile => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Typst", &["typ"])
                    .pick_file()
                    .await
                    .and_then(|file| {
                        let path = file.path().to_path_buf();

                        std::fs::read_to_string(&path)
                            .ok()
                            .map(|content| (path, content))
                    })
            },
            Message::FileOpened,
        ),

        Message::FileOpened(result) => {
            let Some((path, content)) = result else {
                return Task::none();
            };

            // L'assemblage est le fichier du projet qui n'est pas un chapitre :
            // l'ouvrir doit le laisser où il est, et n'ajouter aucun chapitre.
            if crate::project::same_file(&path, &state.project.main) {
                state.open_document(path, content);

                return Task::none();
            }

            // Charger un fichier, c'est l'**ajouter au projet** : un fichier
            // simplement affiché ne serait ni compté, ni inclus dans
            // l'assemblage, donc jamais rendu — il n'existerait que dans
            // l'éditeur.
            let chapter = match crate::project::import_chapter(&state.project, &path) {
                Ok(chapter) => chapter,
                Err(error) => {
                    state.export_status = Some(format!("Ajout au projet : {error}"));

                    return Task::none();
                }
            };

            // Un chapitre déjà installé dans le projet se charge avec ce qu'on
            // vient de lire ; une copie doit se lire à sa nouvelle adresse.
            let imported = !crate::project::same_file(&chapter, &path);

            let content = if imported {
                match std::fs::read_to_string(&chapter) {
                    Ok(content) => content,
                    Err(error) => {
                        state.export_status = Some(format!("Ouverture du chapitre : {error}"));

                        return Task::none();
                    }
                }
            } else {
                content
            };

            state.open_document(chapter.clone(), content);

            // Le message se pose après `open_document`, qui efface le statut et
            // relit le projet : c'est le seul moment où le titre du chapitre
            // fraîchement importé est connu. Copier un fichier est un effet de
            // bord qu'il faut dire, sans quoi le chapitre apparaîtrait dans le
            // panneau sans qu'on sache pourquoi.
            if imported {
                let title = state
                    .project
                    .chapters
                    .iter()
                    .find(|entry| crate::project::same_file(&entry.path, &chapter))
                    .map(|entry| entry.title.clone())
                    .unwrap_or_else(|| {
                        chapter
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("chapitre")
                            .to_string()
                    });

                state.export_status = Some(format!("« {title} » ajouté au projet."));
            }

            Task::none()
        }

        Message::OpenChapter(path) => Task::perform(
            async move {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            },
            |result| match result {
                Some((path, content)) => Message::ChapterOpened(path, content),
                None => Message::RefreshProject,
            },
        ),

        Message::ChapterOpened(path, content) => {
            state.renaming = None;
            state.open_document(path, content);

            // Le rendez-vous d'un aller à la définition : le chapitre est ouvert,
            // il reste à y poser le curseur.
            match state.jump.take() {
                Some((line, column)) => state.editor.set_cursor(line, column).map(Message::Editor),
                None => Task::none(),
            }
        }

        Message::NewChapter => {
            let title = default_chapter_title(&state.project);

            match crate::project::create_chapter(&state.project, &title) {
                Ok(path) => {
                    state.refresh_context();

                    match std::fs::read_to_string(&path) {
                        Ok(content) => state.open_document(path, content),
                        Err(error) => {
                            state.export_status = Some(format!("Ouverture du chapitre : {error}"));
                        }
                    }
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du chapitre : {error}"));
                }
            }

            Task::none()
        }

        Message::NewProject => {
            let name = next_project_name(&state.projects);

            match crate::project::create(&name) {
                Ok(dir) => {
                    state.project = crate::project::load(&dir);
                    state.projects = crate::project::list_projects();
                    state.editor = crate::editor::EditorState::default();
                    state.render.reset();
                    state.refresh_context();

                    update(state, Message::NewChapter)
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du projet : {error}"));

                    Task::none()
                }
            }
        }

        Message::OpenProject(dir) => {
            state.project = crate::project::load(&dir);
            state.projects = crate::project::list_projects();
            state.render.reset();
            state.refresh_context();

            // On repart sur le premier chapitre du nouveau projet : rouvrir le
            // chapitre d'un autre projet n'aurait pas de sens.
            match state.project.chapters.first().map(|c| c.path.clone()) {
                Some(path) => update(state, Message::OpenChapter(path)),
                None => {
                    state.editor = crate::editor::EditorState::default();
                    state.save_session();

                    Task::none()
                }
            }
        }

        Message::RefreshProject => {
            state.refresh_context();

            Task::none()
        }

        Message::DeleteChapter(path) => {
            let _ = crate::project::remove_chapter(&state.project, &path);

            state.refresh_context();

            // Le chapitre ouvert vient d'être supprimé : on ouvre le suivant, ou
            // l'éditeur reste sur un fichier qui n'existe plus.
            if state.editor.path.as_deref() == Some(path.as_path()) {
                match state.project.chapters.first().map(|c| c.path.clone()) {
                    Some(next) => return update(state, Message::OpenChapter(next)),
                    None => {
                        state.editor = crate::editor::EditorState::default();
                        state.save_session();
                    }
                }
            }

            Task::none()
        }

        Message::InsertCitation(key) => state
            .editor
            .insert_at_cursor(&format!("@{key}"))
            .map(Message::Editor),

        Message::SaveFile => {
            state.persist();
            state.refresh_context();

            Task::none()
        }

        Message::SaveAs => save_as_task(state),

        Message::FileSaved(path) => {
            if let Some(path) = path {
                state.editor.path = Some(path);
                state.editor.editor.mark_saved();
            }

            state.refresh_context();

            Task::none()
        }

        Message::ExportPdf => {
            let dir = state.document_dir();
            let (entry, overrides) = state.compilation_unit();

            Task::perform(
                async move {
                    let Some(bytes) = crate::typst::render::render_pdf(&dir, &entry, &overrides)
                    else {
                        return Err(
                            "La compilation a échoué : corrigez les erreurs signalées dans le panneau."
                                .to_string(),
                        );
                    };

                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .set_file_name("export.pdf")
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé".to_string());
                    };

                    let path = file.path().to_path_buf();

                    std::fs::write(&path, &bytes)
                        .map(|_| path.display().to_string())
                        .map_err(|error| format!("Écriture du PDF : {error}"))
                },
                Message::PdfExported,
            )
        }

        Message::PdfExported(result) => {
            state.export_status = Some(match result {
                Ok(path) => format!("PDF exporté : {path}"),
                Err(error) => error,
            });

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
            state.render.image_hashes.clear();

            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            state.render.stale.extend(0..total);

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

                state.render.words = Some(crate::project::count_words(&content));
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
            let compiled_ok = outcome.errors.is_empty();
            let chapter_page = outcome.chapter_page;

            state.render.content = outcome.content;
            state.render.errors = outcome.errors;
            state.render.warnings = outcome.warnings;
            state.render.document = outcome.document;

            // Nouveau document : toute image en cache est dorénavant suspecte.
            // On ne les supprime pas pour autant — elles restent affichées
            // (double tampon) — mais elles sont marquées périmées, et seules
            // les pages visibles seront rasterisées à nouveau.
            state.render.revision += 1;

            // Les entrées de pages qui n'existent plus sont retirées, pour ne
            // pas retenir des images inutilisées.
            if let Some(document) = state.render.document.as_deref() {
                let total = crate::typst::render::page_count(document);

                state.render.page_cache.retain(|index, _| *index < total);
                state.render.rendering.retain(|index| *index < total);
                state.render.stale.retain(|index| *index < total);
            }

            // Sauvegarde différée plutôt qu'immédiate : écrire le fichier à
            // chaque compilation, c'était une écriture disque toutes les
            // 300 ms pendant la frappe.
            if compiled_ok && state.editor.is_modified() {
                state.render.save_pending = true;
            }

            state.render.chapter_page = chapter_page;

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
            state.render.rendering.remove(&index);

            // Une compilation est arrivée pendant le rendu : cette image
            // appartient à un document dépassé, on la jette plutôt que de
            // faire régresser l'affichage.
            if revision != state.render.revision {
                state.render.stale.insert(index);

                return Task::none();
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

            Task::none()
        }

        Message::ViewportChanged(first, last, bounds) => {
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
                state.render.image_hashes.clear();

                let total = state
                    .render
                    .document
                    .as_deref()
                    .map_or(0, crate::typst::render::page_count);

                state.render.stale.extend(0..total);
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

        Message::SaveTick => {
            // Écriture différée : une seule par tick, et seulement si le
            // document compile et a réellement changé.
            if state.render.save_pending && state.editor.is_modified() {
                state.persist();
                state.refresh_context();
                state.render.save_pending = false;

                // Le serveur suit les écritures : c'est ainsi qu'il sait
                // recompiler le projet, et que les diagnostics des *autres*
                // chapitres se mettent à jour.
                if let Some(session) = state.lsp.session.as_mut() {
                    session.did_save();
                }
            }

            Task::none()
        }

        Message::SystemTheme(mode) => {
            state.system_theme = mode;
            retheme(state);

            Task::none()
        }

        Message::CycleTheme => {
            state.theme = state.theme.next();
            retheme(state);
            state.save_session();

            Task::none()
        }

        Message::RefreshThemes => {
            refresh_themes(state);

            Task::none()
        }

        Message::OpenThemesFolder => {
            let dir = crate::theming::themes_dir();

            open_folder(&dir);

            // Le chemin est dit dans tous les cas : sur un système où
            // l'ouverture n'est pas prévue, il reste au moins de quoi y aller.
            state.export_status = Some(format!("Dossier des thèmes : {}", dir.display()));

            Task::none()
        }

        Message::SelectTheme(name) => {
            let Some(theme) = state.themes.iter().find(|theme| theme.name == name) else {
                return Task::none();
            };

            let appearance = theme.appearance;
            let name = theme.name.clone();

            state.chosen.set(appearance, &name);

            // Choisir un thème fixe le mode sur son apparence. Sans cela, en
            // mode système, le thème clair choisi disparaîtrait dès que le
            // système annonce une apparence sombre — on croirait que le clic
            // n'a rien fait.
            state.theme = match appearance {
                crate::theming::Appearance::Light => crate::ui::theme::ThemeMode::Light,
                crate::theming::Appearance::Dark => crate::ui::theme::ThemeMode::Dark,
            };

            retheme(state);
            state.save_session();

            Task::none()
        }

        Message::LoadTheme => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Thème", &["toml"])
                    .pick_file()
                    .await
                    .map(|file| file.path().to_path_buf())
            },
            Message::ThemeLoaded,
        ),

        Message::ThemeLoaded(result) => {
            let Some(path) = result else {
                return Task::none();
            };

            match crate::theming::import(&path) {
                Ok(loaded) => {
                    let name = loaded.meta.name.clone();

                    // Un thème du même nom existait déjà : le fichier a été
                    // remplacé, et le dire évite de croire à un doublon.
                    let updated = state.themes.iter().any(|theme| theme.name == name);

                    refresh_themes(state);

                    // On applique le thème qu'on vient de charger : c'est ce
                    // qu'on veut voir, et sans cela il faudrait le chercher
                    // dans la liste après l'avoir chargé.
                    if let Some(theme) = state.themes.iter().find(|theme| theme.name == name) {
                        let appearance = theme.appearance;

                        state.chosen.set(appearance, &name);

                        state.theme = match appearance {
                            crate::theming::Appearance::Light => crate::ui::theme::ThemeMode::Light,
                            crate::theming::Appearance::Dark => crate::ui::theme::ThemeMode::Dark,
                        };

                        retheme(state);
                        state.save_session();
                    }

                    // L'auteur, quand il s'est nommé, est une politesse qu'on
                    // lui rend : c'est la seule raison d'écrire ce champ.
                    let verb = if updated { "mis à jour" } else { "chargé" };

                    state.export_status = Some(match loaded.meta.author.as_deref() {
                        Some(author) => format!("Thème « {name} », par {author}, {verb}."),
                        None => format!("Thème « {name} » {verb}."),
                    });
                }
                Err(error) => {
                    state.export_status = Some(format!("Chargement du thème : {error}"));
                }
            }

            Task::none()
        }

        Message::DeleteTheme(name) => {
            match crate::theming::remove(&name) {
                Ok(()) => {
                    // Le thème supprimé ne doit pas rester « retenu » : on
                    // l'oublie, sinon le repli désignerait éternellement un
                    // fichier absent.
                    for appearance in [
                        crate::theming::Appearance::Light,
                        crate::theming::Appearance::Dark,
                    ] {
                        if state.chosen.get(appearance) == name {
                            let native = crate::theming::natives()
                                .into_iter()
                                .find(|theme| theme.appearance == appearance);

                            if let Some(native) = native {
                                state.chosen.set(appearance, &native.name);
                            }
                        }
                    }

                    refresh_themes(state);
                    state.save_session();

                    state.export_status = Some(format!("Thème « {name} » supprimé."));
                }
                Err(error) => {
                    state.export_status = Some(format!("Suppression du thème : {error}"));
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

            if std::env::var_os("TYPST_WRITER_LSP_VERBOSE").is_some() {
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

        Message::StartRename(path) => {
            // Le titre actuel amorce la saisie : on corrige plus souvent qu'on
            // ne réécrit entièrement.
            let buffer = state
                .project
                .chapters
                .iter()
                .find(|chapter| chapter.path == path)
                .map(|chapter| chapter.title.clone())
                .unwrap_or_default();

            state.renaming = Some(ChapterRename { path, buffer });

            Task::none()
        }

        Message::RenameChanged(text) => {
            if let Some(renaming) = state.renaming.as_mut() {
                renaming.buffer = text;
            }

            Task::none()
        }

        Message::RenameCancel => {
            state.renaming = None;

            Task::none()
        }

        Message::RenameCommit => {
            let Some(renaming) = state.renaming.clone() else {
                return Task::none();
            };

            let title = renaming.buffer.trim().to_string();

            // Un titre vide laisserait un nom de fichier sans signification.
            if title.is_empty() {
                state.export_status = Some("Le titre ne peut pas être vide.".to_string());

                return Task::none();
            }

            match crate::project::rename_chapter(&state.project, &renaming.path, &title) {
                Ok(new_path) => {
                    state.renaming = None;

                    // Le chapitre renommé peut être celui qui est ouvert : son
                    // chemin a changé, l'éditeur doit suivre.
                    if state.editor.path.as_deref() == Some(renaming.path.as_path()) {
                        state.editor.path = Some(new_path.clone());
                    }

                    state.refresh_context();

                    Task::none()
                }
                Err(error) => {
                    // Le message reste affiché : sans lui, un renommage refusé
                    // (titre déjà pris) serait indiscernable d'un clic perdu.
                    state.export_status = Some(format!("Renommage : {error}"));

                    Task::none()
                }
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
fn handle_shortcut(
    state: &mut State,
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
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
        _ => Task::none(),
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

    let wanted: Vec<usize> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .filter(|index| !state.render.rendering.contains(index))
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
                crate::ui::preview::display_ratio(
                    box_size,
                    crate::typst::render::page_size(&document, index).unwrap_or((595.3, 841.9)),
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

    let (Some(document), Some(page)) = (state.render.document.clone(), state.render.chapter_page)
    else {
        return Task::none();
    };

    // Le chapitre est marqué comme suivi **avant** de calculer le défilement :
    // si le calcul échouait, on ne redéfilerait pas indéfiniment.
    state.render.followed_chapter = state.editor.path.clone();

    let sizes = crate::ui::preview::sizes_of(
        &document,
        state
            .render
            .viewport
            .unwrap_or_else(crate::ui::preview::default_viewport),
    );

    let offset = crate::ui::preview::scroll_offset_for_page(&sizes, page);

    // L'opération est paramétrée par le type de message qu'elle produirait :
    // elle n'en produit aucun, mais c'est ainsi qu'elle se raccorde à la
    // boucle de messages.
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

pub fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch(vec![
        subscription_timer(),
        subscription_save(),
        subscription_system_theme(),
        subscription_window(),
        subscription_drag(),
        subscription_keys(),
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

/// Réapplique le thème aux éléments qui en dépendent mais ne le recoivent pas
/// à chaque rendu — en l'occurrence l'éditeur, dont les couleurs sont une
/// copie prise au moment du réglage.
fn retheme(state: &mut State) {
    let appearance = state.theme.appearance(state.system_theme);

    let index = crate::theming::select(&state.themes, appearance, &state.chosen);

    let Some(theme) = state.themes.get(index) else {
        return;
    };

    state.resolved = crate::ui::theme::Resolved::of(theme);

    state.editor.retheme(&state.resolved);
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

/// Ouvre un dossier dans le gestionnaire de fichiers du système.
///
/// Sous Linux seulement : c'est la seule plateforme où l'application est
/// utilisée, et deviner la commande des autres serait du code jamais vérifié.
/// L'appelant dit le chemin de toute façon, ce qui reste une issue.
#[cfg(target_os = "linux")]
fn open_folder(dir: &std::path::Path) {
    let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
}

#[cfg(not(target_os = "linux"))]
fn open_folder(_dir: &std::path::Path) {}

/// Relit les thèmes du disque, puis réapplique celui qui est retenu.
///
/// Appelé au démarrage et après tout changement dans le dossier : un thème
/// écrit à la main pendant que l'application tourne doit pouvoir être rechargé
/// sans la relancer.
fn refresh_themes(state: &mut State) {
    // Le thème d'exemple sert de documentation : il est écrit dans le dossier
    // au premier besoin, pour que le format soit sous les yeux de qui veut en
    // écrire un, plutôt que dans un fichier de référence à retrouver.
    let _ = crate::theming::write_example();

    let (themes, problems) = crate::theming::list();

    state.themes = themes;
    state.theme_problems = problems;

    retheme(state);
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

fn save_as_task(state: &State) -> Task<Message> {
    let content = state.editor.content().to_owned();

    Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Typst", &["typ"])
                .set_file_name("export.typ")
                .save_file()
                .await?;

            let path = file.path().to_path_buf();

            match std::fs::write(&path, content) {
                Ok(_) => Some(path),
                Err(error) => {
                    eprintln!("Erreur lors de l'export : {error}");
                    None
                }
            }
        },
        Message::FileSaved,
    )
}
