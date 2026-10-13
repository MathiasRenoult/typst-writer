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

    /// Tic périodique déclenchant la recompilation automatique.
    Tick,

    /// Résultat d'une compilation (réussie ou échouée).
    PreviewOutcome(PreviewOutcome),

    /// Page précédente du PDF prévisualisé.
    PreviewPagePrev,

    /// Page suivante du PDF prévisualisé.
    PreviewPageNext,

    OpenPreview,
    Refresh,

    InsertReference,
    OpenSettings,
}

#[derive(Debug, Clone)]
pub struct PreviewOutcome {
    /// Contenu compilé : sert à ne recompiler que lorsque le document change.
    pub content: String,

    /// Chemin du PDF produit, `None` si la compilation a échoué.
    pub pdf: Option<PathBuf>,

    /// Images PNG des pages du PDF, dans l'ordre.
    pub pages: Vec<PathBuf>,

    /// Message d'état affiché dans le panneau de prévisualisation.
    pub status: String,
}
