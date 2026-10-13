pub mod message;
pub mod state;

pub use message::Message;
pub use state::State;

use std::fs;

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

        Message::SaveFile => {
            let Some(path) = state.editor.path.clone() else {
                return save_as_task(state);
            };

            let content = state.editor.content().to_owned();

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

            iced::Task::none()
        }

        Message::OpenProject => {
            println!("Open project");
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
