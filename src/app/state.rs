use crate::editor::EditorState;
use crate::typst::render::Diagnostic;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,

    /// Résultat du rendu fidèle (compilation Typst en bibliothèque).
    pub render: RenderState,
}

/// État du rendu fidèle : compilation par le moteur Typst complet.
#[derive(Default)]
pub struct RenderState {
    /// Contenu du dernier rendu terminé.
    pub content: String,

    /// Un rendu est en cours (un seul à la fois).
    pub pending: bool,

    /// Au moins un rendu a été effectué.
    pub done: bool,

    /// Erreurs remontées par le compilateur.
    pub errors: Vec<Diagnostic>,

    /// Avertissements remontés par le compilateur.
    pub warnings: Vec<Diagnostic>,

    /// Une chaîne SVG par page, vide si la compilation a échoué.
    pub pages: Vec<String>,
}
