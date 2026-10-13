use iced::Task;
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::path::PathBuf;

pub struct EditorState {
    pub editor: CodeEditor,
    pub path: Option<PathBuf>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            editor: CodeEditor::new(crate::project::DOCUMENT_TEMPLATE, "typst"),
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

    /// Insère du texte à la position du curseur.
    ///
    /// Le mécanisme est le message `Paste` de l'éditeur — c'est son insertion
    /// documentée au curseur (utilisée par la palette de commandes), pas un
    /// accès au presse-papiers système.
    pub fn insert_at_cursor(&mut self, text: &str) -> Task<EditorMessage> {
        self.editor.update(&EditorMessage::Paste(text.to_string()))
    }

    /// Écrit le contenu courant sur le disque.
    ///
    /// Renvoie `Ok(())` si l'écriture a réussi, l'erreur sinon.
    pub fn save(&mut self) -> std::io::Result<()> {
        let Some(path) = self.path.clone() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "aucun chemin de fichier",
            ));
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::write(&path, self.content())?;

        self.editor.mark_saved();

        Ok(())
    }
}
