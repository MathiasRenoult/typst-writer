use iced::Task;
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::path::PathBuf;

pub struct EditorState {
    pub editor: CodeEditor,
    pub path: Option<PathBuf>,
}

impl Default for EditorState {
    fn default() -> Self {
        let content = r#"#set page(
  margin: 2cm,
)

= Introduction

Bienvenue dans Typst Writer.

Ceci est notre premier document Typst.
"#;

        Self {
            editor: CodeEditor::new(content, "typst"),
            path: None,
        }
    }
}

impl EditorState {
    pub fn new_file(path: PathBuf, content: String) -> Self {
        Self {
            editor: CodeEditor::new(&content, "typst"),
            path: Some(path),
        }
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        self.editor.update(&message)
    }

    pub fn filename(&self) -> &str {
        self.path
            .as_deref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Sans titre")
    }

    pub fn content(&self) -> String {
        self.editor.content()
    }

    pub fn is_modified(&self) -> bool {
        self.editor.is_modified()
    }
}
