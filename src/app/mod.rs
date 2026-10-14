pub mod message;
pub mod state;

pub use message::Message;
pub use state::State;

use iced::event::{Event, Status};
use iced::keyboard;
use iced::{Subscription, Task, time};

use std::path::PathBuf;

/// Intervalle de recompilation automatique après la dernière frappe.
const RENDER_INTERVAL_MS: u64 = 300;

impl State {
    /// État initial : le dernier document de la session est rouvert, ou le
    /// plus récent de la bibliothèque, ou un premier document est créé.
    pub fn new() -> Self {
        let mut state = Self {
            editor: crate::editor::EditorState::default(),
            export_status: None,
            library: Vec::new(),
            bibliography: Vec::new(),
            render: Default::default(),
        };

        // 1. Le document de la session précédente, s'il existe toujours.
        let session = crate::project::load_session();

        let restored = session
            .last_document
            .clone()
            .filter(|path| path.is_file())
            .and_then(|path| {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            });

        // 2. Sinon le document le plus récent de la bibliothèque.
        let restored = restored.or_else(|| {
            crate::project::list_documents()
                .into_iter()
                .find_map(|document| {
                    std::fs::read_to_string(&document.path)
                        .ok()
                        .map(|content| (document.path, content))
                })
        });

        // 3. Sinon un premier document, créé sur place.
        match restored {
            Some((path, content)) => {
                state.editor = crate::editor::EditorState::new_file(path, content);
            }
            None => {
                let content = crate::project::DOCUMENT_TEMPLATE.to_string();

                match crate::project::create_document(&content) {
                    Ok(path) => {
                        state.editor = crate::editor::EditorState::new_file(path, content);
                    }
                    Err(error) => eprintln!("Création du premier document : {error}"),
                }
            }
        }

        state.refresh_context();

        // La session est enregistrée dès le démarrage : le document ouvert
        // (restauré ou créé) sera celui de la prochaine session.
        if let Some(path) = state.editor.path.clone() {
            crate::project::save_session(&crate::project::Session {
                last_document: Some(path),
            });
        }

        state
    }

    /// Rescanne la bibliothèque et recharge la bibliographie du document
    /// actif.
    fn refresh_context(&mut self) {
        self.library = crate::project::list_documents();

        self.bibliography = self
            .editor
            .path
            .as_ref()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .and_then(|dir| crate::project::find_bibliography(&dir))
            .map(|bib| crate::project::parse_bibliography(&bib))
            .unwrap_or_default();
    }

    /// Dossier du document actif (base des imports relatifs).
    fn document_dir(&self) -> PathBuf {
        self.editor
            .path
            .as_ref()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .unwrap_or_else(crate::project::work_dir)
    }

    /// Écrit le contenu courant et enregistre la session.
    fn persist(&mut self) {
        if let Some(path) = self.editor.path.clone() {
            if let Err(error) = self.editor.save() {
                eprintln!("Sauvegarde de {path:?} : {error}");
            }

            crate::project::save_session(&crate::project::Session {
                last_document: Some(path),
            });
        }
    }
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Editor(message) => state.editor.update(message).map(Message::Editor),

        Message::NewFile => {
            let content = crate::project::DOCUMENT_TEMPLATE.to_string();

            match crate::project::create_document(&content) {
                Ok(path) => {
                    state.editor = crate::editor::EditorState::new_file(path, content);
                    state.export_status = None;
                    state.refresh_context();
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du document : {error}"));
                }
            }

            Task::none()
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
            if let Some((path, content)) = result {
                state.editor = crate::editor::EditorState::new_file(path, content);
                state.export_status = None;
                state.refresh_context();
            }

            Task::none()
        }

        Message::OpenLibraryDocument(path) => Task::perform(
            async move {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            },
            |result| match result {
                Some((path, content)) => Message::LibraryDocumentOpened(path, content),
                None => Message::RefreshLibrary,
            },
        ),

        Message::LibraryDocumentOpened(path, content) => {
            state.editor = crate::editor::EditorState::new_file(path, content);
            state.export_status = None;
            state.refresh_context();

            crate::project::save_session(&crate::project::Session {
                last_document: state.editor.path.clone(),
            });

            Task::none()
        }

        Message::RefreshLibrary => {
            state.refresh_context();

            Task::none()
        }

        Message::DeleteDocument(path) => {
            let _ = crate::project::delete_document(&path);

            state.refresh_context();

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
            let content = state.editor.content();
            let dir = state.document_dir();

            Task::perform(
                async move {
                    let Some(bytes) = crate::typst::render::render_pdf(&dir, &content) else {
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

        Message::RenderTick => {
            if state.render.pending {
                // Une seule compilation à la fois ; si le contenu a changé
                // pendant ce temps, le tick suivant relancera.
                return Task::none();
            }

            let content = state.editor.content();

            if state.render.content == content {
                return Task::none();
            }

            state.render.pending = true;

            compile_task(state, content)
        }

        Message::RenderDone(outcome) => {
            state.render.pending = false;
            state.render.done = true;
            let compiled_ok = outcome.errors.is_empty();

            state.render.content = outcome.content;
            state.render.errors = outcome.errors;
            state.render.warnings = outcome.warnings;
            state.render.document = outcome.document;

            // Le cache de pages est conservé : c'est le double tampon. Les
            // images déjà affichées restent en place jusqu'à ce que leur
            // nouveau rendu arrive — l'aperçu ne montre jamais de cadre gris
            // ni de flash. Seules les entrées de pages disparues sont
            // retirées, pour ne pas garder des images inutilisées.
            if let Some(document) = state.render.document.as_deref() {
                let total = crate::typst::render::page_count(document);

                state.render.page_cache.retain(|index, _| *index < total);
            }

            // Sauvegarde automatique : le document actif est un fichier
            // local de l'app, toujours à jour sur le disque dès que la
            // compilation réussit. Un document invalide n'est pas écrit
            // (le statut « modifié » le signale).
            if compiled_ok && state.editor.is_modified() {
                state.persist();
            }

            // Marquer les pages à rafraîchir comme périmées : leur nouveau
            // rendu est lancé en arrière-plan, mais l'ancienne image reste
            // affichée jusqu'à ce que la nouvelle arrive — aucun flash, et
            // seules les pages réellement visibles sont rasterisées.
            //
            // Tant qu'aucun scroll n'a eu lieu, `visible` est `None` : c'est
            // la première page qui doit être rendue, sinon l'écran resterait
            // rempli de cadres gris.
            let (first, last) = state.render.visible.unwrap_or((0, 0));

            for index in first..=last {
                state.render.stale.insert(index);
            }

            // Rendre immédiatement les pages actuellement visibles.
            rasterize_visible(state)
        }

        Message::PageRendered(index, png) => {
            state.render.rendering.remove(&index);
            state.render.stale.remove(&index);

            if let Some(png) = png {
                // Remplacement en place : l'affichage bascule de l'ancienne
                // image à la nouvelle sans jamais passer par le vide.
                state.render.page_cache.insert(index, png);
            }

            Task::none()
        }

        Message::ViewportChanged(first, last) => {
            state.render.visible = Some((first, last));

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

        Message::OpenSettings => {
            println!("Open settings");
            Task::none()
        }
    }
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
        "n" => update(state, Message::NewFile),
        "o" => update(state, Message::OpenFile),
        "e" => update(state, Message::ExportPdf),
        _ => Task::none(),
    }
}

/// Lance les rasterisations nécessaires et rien de plus.
///
/// Une page n'est rasterisée que si elle est visible, qu'aucun rendu n'est
/// déjà en cours pour elle, et qu'elle est marquée périmée (document modifié
/// ou page jamais rendue). Un document de 5500 mots ne coûte donc qu'une
/// compilation (~15 ms) + 1-2 pages rasterisées (~35 ms chacune), et un
/// défilement sur des images déjà à jour ne coûte rien du tout.
fn rasterize_visible(state: &State) -> Task<Message> {
    let Some(document) = state.render.document.clone() else {
        return Task::none();
    };

    let (first, last) = state.render.visible.unwrap_or((0, 0));
    let total = crate::typst::render::page_count(&document);

    let tasks: Vec<Task<Message>> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .filter(|index| !state.render.rendering.contains(index))
        .map(|index| {
            let document = document.clone();

            Task::perform(
                async move { crate::typst::render::render_page(&document, index) },
                move |png| Message::PageRendered(index, png),
            )
        })
        .collect();

    Task::batch(tasks)
}

/// Lance la compilation du contenu (moteur Typst en bibliothèque).
fn compile_task(state: &State, content: String) -> Task<Message> {
    let dir = state.document_dir();

    Task::perform(
        async move { crate::typst::render::compile(&dir, &content) },
        Message::RenderDone,
    )
}

/// Recompile le document périodiquement : le moteur n'est relancé que si le
/// contenu a changé depuis la dernière compilation (debounce par inactivité).
fn subscription_timer() -> Subscription<Message> {
    time::every(time::milliseconds(RENDER_INTERVAL_MS)).map(|_| Message::RenderTick)
}

/// Écoute des événements clavier au niveau application.
///
/// Les raccourcis globaux (Ctrl+S, Ctrl+N, Ctrl+O, Ctrl+E) n'agissent que
/// lorsque l'événement n'a pas été consommé par un widget — l'éditeur garde
/// donc la priorité sur ses propres raccourcis.
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
    Subscription::batch(vec![subscription_timer(), subscription_keys()])
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
