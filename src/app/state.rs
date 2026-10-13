use crate::editor::EditorState;

#[derive(Default)]
pub struct State {
    pub title: String,
    pub editor: EditorState,
}
