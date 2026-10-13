use crate::editor::EditorState;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,
    pub preview: Option<Preview>,
}

pub struct Preview {
    pub pdf: std::path::PathBuf,
    pub status: String,
}
