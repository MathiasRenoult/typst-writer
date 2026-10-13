use crate::editor::EditorState;
use crate::typst::check::CheckError;
use crate::typst::renderer::Doc;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,

    /// Modèle de rendu du document courant, mis à jour à chaque frappe.
    pub doc: Doc,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,

    /// Résultat de la vérification par le compilateur Typst.
    pub check: CheckState,
}

/// État de la vérification par le vrai compilateur.
#[derive(Default)]
pub struct CheckState {
    /// Contenu du dernier check terminé.
    pub checked_content: String,

    /// Un check est en cours (un seul à la fois).
    pub pending: bool,

    /// Au moins un check a été effectué.
    pub done: bool,

    /// Erreurs remontées par le compilateur.
    pub errors: Vec<CheckError>,

    /// Avertissements remontés par le compilateur.
    pub warnings: Vec<CheckError>,
}
