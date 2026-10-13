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

    Refresh,

    PreviewOutcome(PreviewOutcome),

    OpenPreview,

    InsertReference,
    OpenSettings,
}

#[derive(Debug, Clone)]
pub struct PreviewOutcome {
    pub pdf: Option<PathBuf>,
    pub status: String,
}
