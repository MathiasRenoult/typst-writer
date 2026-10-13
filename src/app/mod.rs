pub mod message;
pub mod state;

pub use message::Message;
pub use state::State;

use iced::Subscription;
use iced::time;
use iced_code_editor::Message as EditorMessage;

use std::fs;
use std::path::PathBuf;

/// Intervalle de recompilation automatique après la dernière frappe.
const RENDER_INTERVAL_MS: u64 = 300;

pub fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::Editor(message) => state.editor.update(message).map(Message::Editor),

        Message::NewFile => {
            let (path, content) = create_new_document();
            state.editor = crate::editor::EditorState::new_file(path, content);
            state.export_status = None;

            iced::Task::none()
        }

        Message::OpenFile => iced::Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Typst", &["typ"])
                    .pick_file()
                    .await
                    .and_then(|file| {
                        let path = file.path().to_path_buf();

                        fs::read_to_string(&path)
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
            }

            iced::Task::none()
        }

        Message::SaveFile => {
            let content = state.editor.content();
            let path = state
                .editor
                .path
                .clone()
                .unwrap_or_else(|| work_dir().join("document-1.typ"));

            iced::Task::perform(
                async move {
                    match fs::write(&path, content) {
                        Ok(_) => Some(path),
                        Err(error) => {
                            eprintln!("Erreur lors de la sauvegarde : {error}");
                            None
                        }
                    }
                },
                Message::FileSaved,
            )
        }

        Message::SaveAs => save_as_task(state),

        Message::FileSaved(path) => {
            if let Some(path) = path {
                state.editor.path = Some(path);
                state.editor.editor.mark_saved();
            }

            iced::Task::none()
        }

        Message::CloseFile => {
            state.editor = crate::editor::EditorState::default();
            state.export_status = None;

            iced::Task::none()
        }

        Message::ExportPdf => {
            let content = state.editor.content();

            // Le dossier du document actif : les imports relatifs s'y
            // résolvent comme dans l'aperçu.
            let dir = state
                .editor
                .path
                .as_ref()
                .and_then(|path| path.parent())
                .map(|parent| parent.to_path_buf())
                .unwrap_or_else(work_dir);

            iced::Task::perform(
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

                    fs::write(&path, &bytes)
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

            iced::Task::none()
        }

        Message::RenderTick => {
            if state.render.pending {
                // Un seul rendu à la fois ; si le contenu a changé pendant
                // ce temps, le tick suivant relancera.
                return iced::Task::none();
            }

            let content = state.editor.content();

            if state.render.content == content {
                return iced::Task::none();
            }

            state.render.pending = true;

            render_task(state, content)
        }

        Message::RenderDone(outcome) => {
            state.render.pending = false;
            state.render.done = true;
            state.render.content = outcome.content;
            state.render.errors = outcome.errors;
            state.render.warnings = outcome.warnings;
            state.render.pages = outcome.pages;

            iced::Task::none()
        }

        Message::GotoError(line, column) => {
            // Les positions de typst sont 1-based ; l'éditeur attend du
            // 0-based.
            let _ = state.editor.update(EditorMessage::GotoPosition(
                line.saturating_sub(1),
                column.saturating_sub(1),
            ));

            iced::Task::none()
        }

        Message::InsertReference => {
            println!("Insert reference");
            iced::Task::none()
        }

        Message::OpenSettings => {
            println!("Open settings");
            iced::Task::none()
        }
    }
}

/// Lance le rendu fidèle du contenu (compilation Typst en bibliothèque).
fn render_task(state: &State, content: String) -> iced::Task<Message> {
    // Le dossier du document actif : les imports relatifs s'y résolvent
    // comme à l'export.
    let dir = state
        .editor
        .path
        .as_ref()
        .and_then(|path| path.parent())
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(work_dir);

    iced::Task::perform(
        async move { crate::typst::render::render(&dir, &content) },
        Message::RenderDone,
    )
}

/// Recompile le document périodiquement : le moteur n'est relancé que si le
/// contenu a changé depuis le dernier rendu (debounce par inactivité).
pub fn subscription(_state: &State) -> Subscription<Message> {
    time::every(time::milliseconds(RENDER_INTERVAL_MS)).map(|_| Message::RenderTick)
}

fn save_as_task(state: &State) -> iced::Task<Message> {
    let content = state.editor.content().to_owned();

    iced::Task::perform(
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

fn work_dir() -> PathBuf {
    let dir = std::env::var("HOME")
        .map(PathBuf::from)
        .map(|mut path| {
            path.push(".typst-writer");
            path
        })
        .unwrap_or_else(|_| PathBuf::from(".typst-writer"));

    let _ = fs::create_dir_all(&dir);

    dir
}

fn create_new_document() -> (PathBuf, String) {
    let work_dir = work_dir();
    let _ = fs::create_dir_all(&work_dir);

    let path = find_free_document_path(&work_dir);
    let content = DOCUMENT_TEMPLATE.to_string();

    let _ = fs::write(&path, &content);

    (path, content)
}

fn find_free_document_path(work_dir: &PathBuf) -> PathBuf {
    (1..=1000)
        .map(|n| work_dir.join(format!("document-{n}.typ")))
        .find(|path| !path.exists())
        .unwrap_or_else(|| work_dir.join("document.typ"))
}

const DOCUMENT_TEMPLATE: &str = r#"#set page(margin: 2cm)

= Introduction

Bienvenue dans Typst Writer.

Ceci est notre premier document Typst.
"#;
