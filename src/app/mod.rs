pub mod message;
pub mod state;

pub use message::Message;
pub use state::State;

use std::fs;
use std::path::PathBuf;

pub fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::Editor(message) => {
            let task = state.editor.update(message);

            // Le rendu live est recalculé à chaque frappe : le parse est
            // quasi instantané, aucun délai n'est nécessaire.
            state.doc = crate::typst::renderer::parse(&state.editor.content());

            task.map(Message::Editor)
        }

        Message::NewFile => {
            let (path, content) = create_new_document();
            state.editor = crate::editor::EditorState::new_file(path, content);
            state.doc = crate::typst::renderer::parse(&state.editor.content());
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
                state.doc = crate::typst::renderer::parse(&state.editor.content());
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
            state.doc = crate::typst::renderer::parse(&state.editor.content());
            state.export_status = None;

            iced::Task::none()
        }

        Message::ExportPdf => {
            let content = state.editor.content();
            let source = state
                .editor
                .path
                .clone()
                .unwrap_or_else(|| work_dir().join("document-1.typ"));

            iced::Task::perform(
                async move {
                    // Le fichier local est toujours à jour avant compilation.
                    if let Err(error) = fs::write(&source, &content) {
                        return Err(format!("Erreur d'écriture du fichier local : {error}"));
                    }

                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .set_file_name("export.pdf")
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé".to_string());
                    };

                    crate::typst::compiler::compile_to(&source, file.path())
                        .map(|pdf| pdf.display().to_string())
                        .map_err(|error| format!("Erreur de compilation : {error}"))
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

        Message::OpenLink(url) => {
            let _ = std::process::Command::new(if cfg!(target_os = "macos") {
                "open"
            } else if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "xdg-open"
            })
            .arg(&url)
            .spawn();

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
    std::env::var("HOME")
        .map(PathBuf::from)
        .map(|mut path| {
            path.push(".typst-writer");
            path
        })
        .unwrap_or_else(|_| PathBuf::from(".typst-writer"))
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
