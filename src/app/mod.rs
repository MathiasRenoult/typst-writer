pub mod message;
pub mod state;

pub use message::{Message, PreviewOutcome};
pub use state::{Preview, State};

use iced::Subscription;
use iced::time;

use std::fs;
use std::path::PathBuf;

/// Délai d'inactivité avant recompilation automatique (debounce).
const PREVIEW_INTERVAL_MS: u64 = 1000;

pub fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::Editor(message) => state.editor.update(message).map(Message::Editor),

        Message::NewFile => {
            let (path, content) = create_new_document();
            state.editor = crate::editor::EditorState::new_file(path, content);
            state.preview = None;
            state.last_compiled_content = None;

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
                state.preview = None;
                state.last_compiled_content = None;
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

            save_and_compile_task(path, content, state.preview_counter)
        }

        Message::SaveAs => save_as_task(state),

        Message::FileSaved(path) => {
            if let Some(path) = path {
                state.editor.path = Some(path.clone());
                state.editor.editor.mark_saved();

                let content = state.editor.content();
                let counter = state.preview_counter;
                state.last_compiled_content = Some(content.clone());

                iced::Task::perform(
                    async move { compile_and_render(&path, counter, content) },
                    Message::PreviewOutcome,
                )
            } else {
                iced::Task::none()
            }
        }

        Message::CloseFile => {
            state.editor = crate::editor::EditorState::default();
            state.preview = None;
            state.last_compiled_content = None;

            iced::Task::none()
        }

        Message::Tick => {
            let content = state.editor.content();
            let compiled = state.last_compiled_content.as_deref().unwrap_or("");

            if content == compiled {
                // Le document n'a pas changé : rien à recompiler.
                return iced::Task::none();
            }

            // Le document actif est toujours un fichier local de
            // l'application : on en crée un si besoin, puis on y écrit le
            // contenu courant avant de compiler.
            if state.editor.path.is_none() {
                let (path, _) = create_new_document();
                state.editor.path = Some(path);
            }

            let path = state.editor.path.clone().unwrap();
            let counter = state.preview_counter;

            save_and_compile_task(path, content, counter)
        }

        Message::Refresh => {
            if state.editor.path.is_none() {
                let (path, _) = create_new_document();
                state.editor.path = Some(path);
            }

            let path = state.editor.path.clone().unwrap();
            let content = state.editor.content();

            save_and_compile_task(path, content, state.preview_counter)
        }

        Message::PreviewPagePrev => {
            if let Some(preview) = state.preview.as_mut() {
                preview.current = preview.current.saturating_sub(1);
            }

            iced::Task::none()
        }

        Message::PreviewPageNext => {
            if let Some(preview) = state.preview.as_mut() {
                if preview.current + 1 < preview.pages.len() {
                    preview.current += 1;
                }
            }

            iced::Task::none()
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
            state.last_compiled_content = Some(outcome.content);

            match outcome.pdf {
                Some(pdf) => {
                    state.preview_counter = state.preview_counter.wrapping_add(1);
                    state.editor.editor.mark_saved();

                    state.preview = Some(Preview {
                        pdf,
                        pages: outcome.pages,
                        status: outcome.status,
                        current: 0,
                    });
                }
                None => {
                    // Échec : on conserve la dernière bonne prévisualisation
                    // et on se contente d'afficher l'erreur.
                    match state.preview.as_mut() {
                        Some(preview) => preview.status = outcome.status,
                        None => {
                            state.preview = Some(Preview {
                                pdf: PathBuf::new(),
                                pages: Vec::new(),
                                status: outcome.status,
                                current: 0,
                            })
                        }
                    }
                }
            }

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

pub fn subscription(_state: &State) -> Subscription<Message> {
    time::every(time::milliseconds(PREVIEW_INTERVAL_MS)).map(|_| Message::Tick)
}

/// Écrit le contenu dans le fichier local, compile, puis rend le PDF en images.
fn save_and_compile_task(path: PathBuf, content: String, counter: usize) -> iced::Task<Message> {
    iced::Task::perform(
        async move {
            if let Err(error) = fs::write(&path, &content) {
                return PreviewOutcome {
                    content,
                    pdf: None,
                    pages: Vec::new(),
                    status: format!("Erreur d'écriture du fichier : {error}"),
                };
            }

            compile_and_render(&path, counter, content)
        },
        Message::PreviewOutcome,
    )
}

/// Compile le document et rend chaque page du PDF en image PNG.
fn compile_and_render(path: &PathBuf, counter: usize, content: String) -> PreviewOutcome {
    match crate::typst::compiler::compile(path) {
        Ok(pdf) => match crate::typst::preview::render_pdf_to_png(&pdf, counter) {
            Ok(pages) => {
                let status = format!(
                    "Compilé — {} page{}",
                    pages.len(),
                    if pages.len() > 1 { "s" } else { "" }
                );

                PreviewOutcome {
                    content,
                    pdf: Some(pdf),
                    pages,
                    status,
                }
            }
            Err(error) => PreviewOutcome {
                content,
                pdf: Some(pdf),
                pages: Vec::new(),
                status: format!("Compilé, mais le rendu a échoué : {error}"),
            },
        },
        Err(error) => PreviewOutcome {
            content,
            pdf: None,
            pages: Vec::new(),
            status: format!("Erreur de compilation :\n{error}"),
        },
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
