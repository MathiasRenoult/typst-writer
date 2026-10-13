use crate::typst::check::CheckOutcome;
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

    /// Tic de vérification périodique (relance la vérification si le
    /// document a changé).
    CheckTick,

    /// Résultat de la vérification par le compilateur Typst.
    CheckDone(CheckOutcome),

    /// Saute à la position d'un diagnostic dans l'éditeur.
    GotoError(usize, usize),

    /// Ouvre un lien dans le navigateur.
    OpenLink(String),

    InsertReference,
    OpenSettings,
}
