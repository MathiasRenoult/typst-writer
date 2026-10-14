use crate::editor::EditorState;
use crate::project::{BibliographyEntry, DocumentInfo};
use crate::typst::render::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use iced::widget::image::Handle;
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

    /// Images déjà rasterisées : index → `Handle`.
    ///
    /// **C'est le `Handle` qui est mis en cache, pas les octets.** Un
    /// `Handle` porte un identifiant stable dont le cache de textures du
    /// backend GPU se sert pour retrouver une texture déjà téléversée.
    ///
    /// Le recréer à chaque image produisait un identifiant neuf à chaque
    /// frame, donc un miss de cache, donc un décodage et une nouvelle
    /// texture par page : plusieurs dizaines de mégaoctets de VRAM créés et
    /// détruits chaque seconde — d'où le clignotement, le lag sur les gros
    /// fichiers et les crashes du compositeur.
    ///
    /// `view()` se contente donc de cloner le `Handle` d'ici : même
    /// identifiant, texture retrouvée, aucun travail graphique.
    pub page_cache: HashMap<usize, Handle>,

    /// Pages dont la rasterisation est demandée mais pas terminée.
    pub rendering: HashSet<usize>,

    /// Pages dont l'image doit être (re)produite.
    ///
    /// Une page entre ici quand le document vient d'être recompilé, ou
    /// quand elle apparaît dans le viewport sans image valide.
    /// `rasterize_visible` ne travaille que sur l'intersection de cet
    /// ensemble avec le viewport : un défilement sur des images à jour ne
    /// coûte rien, et une page périmée reste affichée avec son ancienne
    /// image jusqu'à l'arrivée de la nouvelle (aucun cadre gris).
    pub stale: HashSet<usize>,

    /// Première et dernière page visibles à l'écran (inclusives), d'après le
    /// dernier scroll. `None` avant le premier rendu de la vue.
    pub visible: Option<(usize, usize)>,

    /// Numéro du document compilé, incrémenté à chaque compilation.
    ///
    /// Un rendu de page lancé sur un document depuis dépassé se reconnaît à
    /// ce numéro et n'écrase pas le cache avec une image obsolète.
    pub revision: u64,

    /// Le contenu a changé depuis la dernière écriture sur le disque.
    pub save_pending: bool,
}

impl RenderState {
    /// Vide l'état de rendu (changement de document ouvert).
    ///
    /// Les `Handle` mis en cache sont relâchés : le backend libère alors les
    /// textures du document précédent à son prochain nettoyage.
    pub fn reset(&mut self) {
        self.content.clear();
        self.pending = false;
        self.done = false;
        self.errors.clear();
        self.warnings.clear();
        self.document = None;
        self.page_cache.clear();
        self.rendering.clear();
        self.stale.clear();
        self.visible = None;
        self.save_pending = false;
        self.revision += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une image quelconque : seule son identité compte pour ces tests.
    fn handle() -> Handle {
        Handle::from_rgba(2, 2, vec![0u8; 16])
    }

    /// Le `Handle` mis en cache est bien réutilisé tel quel par `view()`.
    ///
    /// C'est le contrat qui supprime le clignotement : deux clones du `Handle`
    /// d'une même page doivent porter le même identifiant, sinon le backend
    /// graphique reconstruit une texture à chaque image.
    #[test]
    fn cached_handles_keep_a_stable_identity() {
        let mut render = RenderState::default();
        render.page_cache.insert(0, handle());

        let first = render.page_cache.get(&0).expect("image en cache").clone();
        let second = render.page_cache.get(&0).expect("image en cache").clone();

        assert_eq!(first.id(), second.id(), "l'identifiant doit être stable");

        // À l'inverse, une image recréée de toutes pièces porte un
        // identifiant neuf : c'est ce que l'ancien code faisait à chaque
        // frame, forçant le backend à retéléverser la texture.
        assert_ne!(
            first.id(),
            handle().id(),
            "une image neuve a une identité neuve"
        );
    }

    /// Le changement de document repart d'un rendu vierge, sans conserver
    /// d'image ni de travail en cours.
    #[test]
    fn reset_clears_every_trace_of_the_previous_document() {
        let mut render = RenderState::default();

        render.content = "ancien document".to_string();
        render.done = true;
        render.page_cache.insert(0, handle());
        render.rendering.insert(1);
        render.stale.insert(2);
        render.visible = Some((0, 4));
        render.save_pending = true;
        let previous = render.revision;

        render.reset();

        assert!(render.page_cache.is_empty(), "aucune image conservée");
        assert!(render.rendering.is_empty(), "aucun rendu en cours");
        assert!(render.stale.is_empty(), "aucune page périmée");
        assert!(render.content.is_empty());
        assert!(render.document.is_none());
        assert!(render.visible.is_none());
        assert!(!render.save_pending);

        // La révision doit avancer : un rendu de page encore en vol pour
        // l'ancien document sera alors reconnu comme périmé et écarté.
        assert!(
            render.revision > previous,
            "la révision doit avancer pour invalider les rendus en vol"
        );
    }
}
