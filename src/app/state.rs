use crate::editor::EditorState;
use crate::project::{BibliographyEntry, DocumentInfo};
use crate::typst::render::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use typst_layout::PagedDocument;

#[derive(Default)]
pub struct State {
    pub editor: EditorState,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,

    /// Bibliothèque personnelle : les documents du dossier de travail.
    pub library: Vec<DocumentInfo>,

    /// Entrées de bibliographie citables (fichier `.bib` à côté du document).
    pub bibliography: Vec<BibliographyEntry>,

    /// Résultat du rendu fidèle (compilation Typst en bibliothèque).
    pub render: RenderState,
}

/// État du rendu fidèle : compilation Typst + rasterisation à la demande.
#[derive(Default)]
pub struct RenderState {
    /// Contenu du dernier rendu terminé.
    pub content: String,

    /// Une compilation est en cours (une seule à la fois).
    pub pending: bool,

    /// Au moins une compilation a été effectuée.
    pub done: bool,

    /// Erreurs remontées par le compilateur.
    pub errors: Vec<Diagnostic>,

    /// Avertissements remontées par le compilateur.
    pub warnings: Vec<Diagnostic>,

    /// Document compilé : rasteriser une page est alors instantané à
    /// déclencher.
    pub document: Option<Arc<PagedDocument>>,

    /// Pages déjà rasterisées (index → octets PNG).
    ///
    /// Double tampon : quand le document est recompilé, les anciennes images
    /// RESTENT affichées le temps que les nouvelles arrivent. L'aperçu ne
    /// montre donc jamais de cadre gris ni de flash — une page modifiée se
    /// met à jour en place, sans disparaître.
    pub page_cache: HashMap<usize, Vec<u8>>,

    /// Pages dont la rasterisation a été demandée mais n'est pas finie.
    pub rendering: HashSet<usize>,

    /// Pages dont l'image est périmée (le document a changé depuis son
    /// dernier rendu) mais qui restent affichées en attendant la nouvelle.
    ///
    /// C'est le cœur du double tampon : la rasterisation n'est relancée que
    /// pour les pages périmées *visibles*, jamais pour un défilement sur des
    /// images déjà valides, et l'ancienne image n'est retirée du cache que
    /// lorsque la nouvelle est arrivée — l'écran ne passe jamais par le vide.
    pub stale: HashSet<usize>,

    /// Première et dernière page visibles à l'écran (inclusives), d'après le
    /// dernier scroll. `None` avant le premier rendu de la vue.
    pub visible: Option<(usize, usize)>,
}
