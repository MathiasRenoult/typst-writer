use crate::typst::render::RenderOutcome;
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage),

    NewFile,

    OpenFile,
    FileOpened(Option<(PathBuf, String)>),

    SaveFile,
    SaveAs,
    FileSaved(Option<PathBuf>),

    CloseFile,

    /// Exporte le document courant en PDF (dialogue de destination).
    ExportPdf,

    /// Résultat de l'export PDF : chemin du fichier ou message d'erreur.
    PdfExported(Result<String, String>),

    /// Tic de rendu périodique (relance la compilation si le document a
    /// changé).
    RenderTick,

    /// Résultat du rendu fidèle (pages SVG et diagnostics).
    RenderDone(RenderOutcome),

    /// Saute à la position d'un diagnostic dans l'éditeur.
    GotoError(usize, usize),

    InsertReference,
    OpenSettings,
}
