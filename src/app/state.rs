use crate::editor::EditorState;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,
    pub preview: Option<Preview>,

    /// Contenu correspondant à la dernière compilation lancée : permet de ne
    /// recompiler automatiquement que lorsque le document a réellement changé.
    pub last_compiled_content: Option<String>,

    /// Compteur de compilations. Un nom d'image unique par compilation
    /// (`preview-{n}-{page}.png`) garantit que le widget image recharge
    /// bien la nouvelle image à chaque mise à jour.
    pub preview_counter: usize,
}

pub struct Preview {
    pub pdf: std::path::PathBuf,

    /// Images PNG des pages, dans l'ordre.
    pub pages: Vec<std::path::PathBuf>,

    /// Index de la page actuellement affichée.
    pub current: usize,

    pub status: String,
}
