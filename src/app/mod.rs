pub mod message;
pub mod state;

pub use message::{Message, PreviewOutcome};
pub use state::{Preview, State};

use std::fs;
use std::path::PathBuf;

pub fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::Editor(message) => state.editor.update(message).map(Message::Editor),

        Message::NewFile => {
            state.editor = crate::editor::EditorState::default();

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
            }

            iced::Task::none()
        }

        Message::SaveFile => save_as_task(state),

        Message::SaveAs => save_as_task(state),

        Message::FileSaved(path) => {
            if let Some(path) = path {
                state.editor.path = Some(path.clone());
                state.editor.editor.mark_saved();

                compile_task(path.clone())
            } else {
                iced::Task::none()
            }
        }

        Message::Refresh => {
            if let Some(path) = state.editor.path.clone() {
                compile_task(path)
            } else {
                save_as_task(state)
            }
        }

        Message::OpenPreview => {
            if let Some(preview) = state.preview.as_ref() {
                if let Some(path) = preview.pdf.to_str() {
                    if cfg!(target_os = "windows") {
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "", path])
                            .spawn();
                    } else {
                        let _ = std::process::Command::new(if cfg!(target_os = "macos") {
                            "open"
                        } else {
                            "xdg-open"
                        })
                        .arg(path)
                        .spawn();
                    }
                }
            }

            iced::Task::none()
        }

        Message::PreviewOutcome(outcome) => {
            state.preview = Some(Preview {
                pdf: outcome.pdf.clone().unwrap_or_default(),
                status: outcome.status.clone(),
            });

            iced::Task::none()
        }

        Message::CloseFile => {
            state.editor = crate::editor::EditorState::default();

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

fn compile_task(path: PathBuf) -> iced::Task<Message> {
    iced::Task::perform(
        async move {
            match crate::typst::compiler::compile(&path) {
                Ok(pdf) => (Some(pdf), "Compilation réussie".to_string()),
                Err(error) => (
                    None,
                    format!("Erreur de compilation :\n{error}"),
                ),
            }
        },
        |outcome| Message::PreviewOutcome(PreviewOutcome {
            pdf: outcome.0,
            status: outcome.1,
        }),
    )
}

fn save_as_task(state: &State) -> iced::Task<Message> {
    let content = state.editor.content().to_owned();

    iced::Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Typst", &["typ"])
                .set_file_name("main.typ")
                .save_file()
                .await?;

            let path = file.path().to_path_buf();

            match std::fs::write(&path, content) {
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
