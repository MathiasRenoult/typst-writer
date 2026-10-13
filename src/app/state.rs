use crate::editor::EditorState;
use crate::typst::renderer::Doc;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,

    /// Modèle de rendu du document courant, mis à jour à chaque frappe.
    pub doc: Doc,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,
}
