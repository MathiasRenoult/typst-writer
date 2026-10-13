use crate::typst::render::CompileOutcome;
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage),

    NewFile,

    OpenFile,
    FileOpened(Option<(PathBuf, String)>),

    /// Ouvre un document de la bibliothèque (clic dans la barre latérale).
    OpenLibraryDocument(PathBuf),
    LibraryDocumentOpened(PathBuf, String),

    /// Rescanne le dossier de travail (après création ou suppression).
    RefreshLibrary,

    /// Supprime un document de la bibliothèque.
    DeleteDocument(PathBuf),

    /// Insère une citation `@clé` à la position du curseur.
    InsertCitation(String),

    SaveFile,
    SaveAs,
    FileSaved(Option<PathBuf>),

    /// Exporte le document courant en PDF (dialogue de destination).
    ExportPdf,

    /// Résultat de l'export PDF : chemin du fichier ou message d'erreur.
    PdfExported(Result<String, String>),

    /// Tic de compilation périodique (relance si le document a changé).
    RenderTick,

    /// Résultat de la compilation (document et diagnostics).
    RenderDone(CompileOutcome),

    /// Résultat de la rasterisation d'une page (index → octets PNG).
    PageRendered(usize, Option<Vec<u8>>),

    /// Le viewport de l'aperçu a changé : (première page visible, dernière).
    ViewportChanged(usize, usize),

    /// Saute à la position d'un diagnostic dans l'éditeur.
    GotoError(usize, usize),

    /// Événement clavier non consommé par les widgets (raccourcis globaux).
    RawKey(iced::keyboard::Key, iced::keyboard::Modifiers),

    OpenSettings,
}
