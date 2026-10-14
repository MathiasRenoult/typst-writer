pub mod message;
pub mod state;

use crate::app::state::{Drag, LayoutState, PreviewPosition, Splitter};
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
            layout: Default::default(),
            theme: Default::default(),
            system_theme: None,
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

    /// Charge un document dans l'éditeur, rendu vierge.
    ///
    /// Les images du document précédent ne doivent pas survivre au
    /// changement : les `Handle` sont relâchés pour que le backend libère les
    /// textures correspondantes.
    fn open_document(&mut self, path: PathBuf, content: String) {
        self.editor = crate::editor::EditorState::new_file(path, content);
        self.export_status = None;
        self.render.reset();
        self.refresh_context();
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
                    state.open_document(path, content);
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
                state.open_document(path, content);
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
            state.open_document(path, content);

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

            // Rien n'a changé : aucune compilation, donc aucun rendu, donc
            // aucune image retravaillée. C'est la condition qui rend l'aperçu
            // inerte — et gratuit — tant que l'on ne tape pas.
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

            // Marquer les pages visibles comme périmées. Tant qu'aucun scroll
            // n'a eu lieu, `visible` est `None` : c'est la première page qui
            // doit être rendue, sinon l'écran resterait rempli de cadres gris.
            let (first, last) = state.render.visible.unwrap_or((0, 0));

            for index in first..=last {
                state.render.stale.insert(index);
            }

            // Rendre immédiatement les pages actuellement visibles.
            rasterize_visible(state)
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
                // Un seul `Handle` par page, créé ici et réutilisé tel quel
                // par `view()` : le backend retrouve sa texture et ne la
                // retéléverse pas. L'ancienne image reste en place jusqu'à
                // ce que celle-ci la remplace — l'écran ne passe jamais par
                // le vide.
                state.render.page_cache.insert(
                    index,
                    Handle::from_rgba(image.width, image.height, image.pixels),
                );
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

        Message::SaveTick => {
            // Écriture différée : une seule par tick, et seulement si le
            // document compile et a réellement changé.
            if state.render.save_pending && state.editor.is_modified() {
                state.persist();
                state.refresh_context();
                state.render.save_pending = false;
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

        Message::SetPreviewPosition(position) => {
            state.layout.preview_position = position;

            Task::none()
        }

        Message::SplitterPressed(which) => {
            // L'origine du glissement n'est pas connue au clic : elle sera
            // prise au premier mouvement du curseur.
            state.layout.dragging = Some(Drag {
                which,
                origin: None,
            });

            Task::none()
        }

        Message::SplitterDragged(at) => {
            let Some(drag) = state.layout.dragging.as_mut() else {
                // Aucun séparateur tenu : l'abonnement émet à chaque mouvement
                // de souris, c'est ici qu'on écarte le bruit.
                return Task::none();
            };

            // L'axe dépend de la disposition : en colonnes, l'aperçu se règle
            // en hauteur.
            let vertical = state.layout.preview_position == PreviewPosition::Bottom
                && drag.which == Splitter::Preview;
            let at = if vertical { at.y } else { at.x };

            let origin = match drag.origin {
                Some(origin) => origin,
                None => {
                    // Premier mouvement : lui seul donne l'origine.
                    drag.origin = Some(at);

                    return Task::none();
                }
            };

            let delta = at - origin;

            if vertical {
                // Le curseur descend vers l'aperçu : sa part grandit.
                state.layout.preview_height =
                    LayoutState::clamp_share(state.layout.preview_height + delta / 600.0);
            } else {
                match drag.which {
                    // Le curseur s'éloigne vers la droite : la bibliothèque
                    // grandit, l'aperçu rétrécit.
                    Splitter::Sidebar => {
                        state.layout.sidebar_width =
                            LayoutState::clamp_width(state.layout.sidebar_width + delta);
                    }
                    Splitter::Preview => {
                        state.layout.preview_width =
                            LayoutState::clamp_width(state.layout.preview_width - delta);
                    }
                }
            }

            Task::none()
        }

        Message::SplitterReleased => {
            state.layout.dragging = None;

            Task::none()
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

/// Lance les rasterisations nécessaires, et rien de plus.
///
/// Une page n'est rasterisée que si elle est visible, marquée périmée, et
/// qu'aucun rendu n'est déjà en cours pour elle. Un document de 5500 mots ne
/// coûte donc qu'une compilation (~15 ms) + 1-2 pages rasterisées (~30 ms
/// chacune) ; un défilement sur des images déjà à jour ne déclenche aucun
/// travail, et un document intact n'est jamais recompilé.
fn rasterize_visible(state: &State) -> Task<Message> {
    let Some(document) = state.render.document.clone() else {
        return Task::none();
    };

    let (first, last) = state.render.visible.unwrap_or((0, 0));
    let total = crate::typst::render::page_count(&document);
    let revision = state.render.revision;

    let tasks: Vec<Task<Message>> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .filter(|index| !state.render.rendering.contains(index))
        .map(|index| {
            let document = document.clone();

            Task::perform(
                async move { crate::typst::render::render_page(&document, index) },
                move |image| Message::PageRendered(revision, index, image),
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
                Some(Message::SplitterDragged(position))
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
        subscription_drag(),
        subscription_keys(),
    ])
}

/// Réapplique le thème aux éléments qui en dépendent mais ne le recoivent pas
/// à chaque rendu — en l'occurrence l'éditeur, dont les couleurs sont une
/// copie prise au moment du réglage.
fn retheme(state: &mut State) {
    let theme = crate::ui::theme::resolve(state.theme, state.system_theme);

    state.editor.retheme(&theme);
}

/// Tâche de démarrage : lire le thème du système avant le premier rendu, pour
/// que la fenêtre n'apparaisse pas une frame en sombre avant de passer au
/// clair (ou l'inverse).
pub fn startup() -> Task<Message> {
    iced::system::theme().map(|mode| Message::SystemTheme(Some(mode)))
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
