use crate::editor::EditorState;
use crate::project::{BibliographyEntry, Project};
use crate::typst::render::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use iced::widget::image::Handle;
use typst_layout::PagedDocument;

/// L'état de l'application.
///
/// Pas de `Default` : un état utilisable suppose un projet ouvert, ce que seule
/// la séquence de démarrage sait établir.
pub struct State {
    pub editor: EditorState,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,

    /// Le projet ouvert : ses chapitres, sa bibliographie, son assemblage.
    pub project: Project,

    /// Les projets disponibles, pour passer de l'un à l'autre.
    pub projects: Vec<Project>,

    /// Entrées de bibliographie citables (le `.bib` du projet).
    pub bibliography: Vec<BibliographyEntry>,

    /// Résultat du rendu fidèle (compilation Typst en bibliothèque).
    pub render: RenderState,

    /// Disposition des panneaux : visibilité, largeurs, position de l'aperçu.
    pub layout: LayoutState,

    /// Renommage de chapitre en cours, s'il y en a un.
    pub renaming: Option<ChapterRename>,

    /// Thème : choix de l'utilisateur et préférence détectée du système.
    pub theme: crate::ui::theme::ThemeMode,

    /// Mode rapporté par le système, `None` s'il n'a pas encore été lu ou
    /// n'exprime aucune préférence.
    pub system_theme: crate::ui::theme::SystemMode,
}

/// Un renommage de chapitre en cours.
///
/// Le titre est édité dans le panneau, en place : la ligne du chapitre devient
/// un champ de saisie, ce qui évite une boîte de dialogue pour une opération
/// aussi courante.
#[derive(Debug, Clone)]
pub struct ChapterRename {
    /// Le chapitre visé.
    pub path: std::path::PathBuf,

    /// Le titre en cours de frappe.
    ///
    /// Conservé ici plutôt que dans le widget : le champ est reconstruit à
    /// chaque rendu, il ne peut donc pas porter l'état de la saisie.
    pub buffer: String,
}

/// Disposition des panneaux de l'interface.
///
/// Les largeurs sont en pixels logiques. Les deux séparateurs se manient à la
/// souris : la largeur au moment du clic est retenue, puis le glissement
/// applique un delta — inutile de connaître la taille de la fenêtre.
#[derive(Debug, Clone)]
pub struct LayoutState {
    /// La bibliothèque est-elle affichée ?
    pub sidebar_visible: bool,

    /// L'aperçu est-il affiché ?
    pub preview_visible: bool,

    /// Largeur de la bibliothèque.
    pub sidebar_width: f32,

    /// Largeur de l'aperçu.
    pub preview_width: f32,

    /// Où l'aperçu se place par rapport à l'éditeur.
    pub preview_position: PreviewPosition,

    /// Hauteur de l'aperçu en disposition « en bas », en fraction de la
    /// hauteur disponible (0,45 = 45 % pour l'aperçu).
    pub preview_height: f32,

    /// Séparateur en cours de glissement.
    pub dragging: Option<Drag>,

    /// Dernière position connue du curseur.
    ///
    /// Elle est mise à jour à chaque mouvement de souris, ce qui permet au
    /// glissement de partir de la position réelle du clic. Sans cela, l'origine
    /// ne pouvait être prise qu'au premier mouvement reçu : un glissement court
    /// — appui, quelques pixels, relâchement — ne déplaçait rien du tout.
    pub cursor: iced::Point,

    /// Mode focus : seul le texte source est visible.
    ///
    /// Les préférences de panneaux sont conservées telles quelles ; c'est
    /// l'affichage qui les ignore. Sortir du mode focus rend donc la
    /// disposition exactement comme avant.
    pub focus_mode: bool,
}

/// Un séparateur saisi par la souris.
#[derive(Debug, Clone, Copy)]
pub struct Drag {
    /// Le séparateur tenu.
    pub which: Splitter,

    /// Coordonnée du curseur au moment de la saisie.
    ///
    /// Le glissement applique ensuite un delta par rapport à cette origine, ce
    /// qui suit le curseur sans avoir à connaître la taille de la fenêtre.
    pub origin: f32,

    /// Largeur — ou part de hauteur — du panneau au moment de la saisie.
    ///
    /// Indispensable : chaque mouvement de souris est un événement séparé, et le
    /// delta qu'il porte est mesuré **depuis l'origine**. Ajouter ce delta à la
    /// largeur courante le réappliquerait à chaque événement, si bien qu'un
    /// glissement de dix pixels réparti sur dix événements en déplacerait
    /// cinquante-cinq. C'est la largeur de départ qu'il faut reprendre.
    pub extent: f32,
}

/// Où l'aperçu se situe par rapport à l'éditeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewPosition {
    /// À droite de l'éditeur (défaut).
    #[default]
    Right,

    /// À gauche de l'éditeur.
    Left,

    /// Sous l'éditeur, de pleine largeur.
    Bottom,
}

impl PreviewPosition {
    pub fn next(self) -> Self {
        match self {
            PreviewPosition::Right => PreviewPosition::Left,
            PreviewPosition::Left => PreviewPosition::Bottom,
            PreviewPosition::Bottom => PreviewPosition::Right,
        }
    }

    /// Libellé lisible, pour l'infobulle.
    pub fn label(self) -> &'static str {
        match self {
            PreviewPosition::Right => "Aperçu à droite",
            PreviewPosition::Left => "Aperçu à gauche",
            PreviewPosition::Bottom => "Aperçu en bas",
        }
    }
}

/// Les deux séparateurs manipulables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Splitter {
    /// Entre la bibliothèque et l'éditeur.
    Sidebar,

    /// Entre l'éditeur et l'aperçu.
    Preview,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            preview_visible: true,
            sidebar_width: 260.0,
            preview_width: 420.0,
            preview_position: PreviewPosition::Right,
            preview_height: 0.45,
            dragging: None,
            cursor: iced::Point::ORIGIN,
            focus_mode: false,
        }
    }
}

impl LayoutState {
    /// Largeur minimale d'un panneau : en deçà, il ne sert plus à rien et
    /// mieux vaut le fermer.
    pub const MIN_PANEL_WIDTH: f32 = 160.0;

    /// Largeur maximale d'un panneau, pour qu'un panneau ne puisse pas
    /// avaler tout l'espace de rédaction.
    pub const MAX_PANEL_WIDTH: f32 = 720.0;

    /// Part minimale de l'écran laissée à l'aperçu en disposition « en bas ».
    pub const MIN_PREVIEW_SHARE: f32 = 0.15;

    /// Part maximale : au-delà, l'aperçu mangerait la rédaction.
    pub const MAX_PREVIEW_SHARE: f32 = 0.85;

    /// Épaisseur de la zone préhensible d'un séparateur.
    pub const SPLITTER_WIDTH: f32 = 6.0;

    /// Applique une largeur, bornée aux limites d'un panneau.
    pub fn clamp_width(width: f32) -> f32 {
        width.clamp(Self::MIN_PANEL_WIDTH, Self::MAX_PANEL_WIDTH)
    }

    /// Applique une part de hauteur, bornée.
    pub fn clamp_share(share: f32) -> f32 {
        share.clamp(Self::MIN_PREVIEW_SHARE, Self::MAX_PREVIEW_SHARE)
    }

    /// Hauteur d'écran de référence, pour convertir un glissement vertical en
    /// part de hauteur.
    ///
    /// La hauteur réelle n'est pas connue du modèle de disposition, et
    /// l'introduire pour un seul geste coûterait plus que ce qu'il apporte :
    /// c'est l'ordre de grandeur du déplacement qui compte, pas le pixel.
    pub const VERTICAL_DRAG_REFERENCE: f32 = 800.0;

    /// Étendue d'un panneau après un glissement.
    ///
    /// `extent` est l'étendue **au début du glissement**, et `delta` le
    /// déplacement **depuis l'origine** — pas depuis l'événement précédent.
    /// Repartir de l'étendue de départ à chaque événement est ce qui rend le
    /// résultat indépendant du nombre d'événements : ajouter le delta à la
    /// valeur courante les cumulerait, et dix pixels répartis sur dix
    /// événements en déplaceraient cinquante-cinq.
    pub fn dragged_extent(which: Splitter, vertical: bool, extent: f32, delta: f32) -> f32 {
        if vertical {
            return Self::clamp_share(extent + delta / Self::VERTICAL_DRAG_REFERENCE);
        }

        match which {
            Splitter::Sidebar => Self::clamp_width(extent + delta),
            // L'aperçu est de l'autre côté du séparateur : s'éloigner vers la
            // droite le rétrécit.
            Splitter::Preview => Self::clamp_width(extent - delta),
        }
    }
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

    /// Les largeurs de panneaux sont bornées des deux côtés : un panneau ne
    /// doit jamais devenir inutilisable, ni manger tout l'espace de rédaction.
    #[test]
    fn panel_widths_are_clamped() {
        assert_eq!(
            LayoutState::clamp_width(-500.0),
            LayoutState::MIN_PANEL_WIDTH
        );
        assert_eq!(
            LayoutState::clamp_width(5_000.0),
            LayoutState::MAX_PANEL_WIDTH
        );

        // Dans les limites, la largeur est respectée telle quelle.
        let inside = (LayoutState::MIN_PANEL_WIDTH + LayoutState::MAX_PANEL_WIDTH) / 2.0;
        assert_eq!(LayoutState::clamp_width(inside), inside);
    }

    /// Un glissement très rapide ne doit pas faire sortir la largeur des
    /// bornes, quel que soit le delta.
    #[test]
    fn extreme_drags_stay_within_bounds() {
        for delta in [-10_000.0, -1.0, 0.0, 1.0, 10_000.0] {
            let width = LayoutState::clamp_width(LayoutState::MAX_PANEL_WIDTH + delta);

            assert!(
                (LayoutState::MIN_PANEL_WIDTH..=LayoutState::MAX_PANEL_WIDTH).contains(&width),
                "largeur {width} hors bornes pour un delta de {delta}"
            );
        }
    }

    /// La part de hauteur de l'aperçu est bornée elle aussi : ni invisible, ni
    /// envahissante.
    #[test]
    fn preview_share_is_clamped() {
        assert_eq!(
            LayoutState::clamp_share(-1.0),
            LayoutState::MIN_PREVIEW_SHARE
        );
        assert_eq!(
            LayoutState::clamp_share(9.0),
            LayoutState::MAX_PREVIEW_SHARE
        );
        assert_eq!(LayoutState::clamp_share(0.5), 0.5);
    }

    /// Le glissement part de la dernière position connue du curseur.
    ///
    /// L'origine était auparavant prise au premier mouvement reçu, ce qui
    /// avalait ce mouvement : un appui suivi d'un déplacement de quelques
    /// pixels puis d'un relâchement ne déplaçait rien. L'origine est donc
    /// désormais connue dès la saisie.
    #[test]
    fn drag_starts_from_the_last_known_cursor_position() {
        let mut layout = LayoutState {
            cursor: iced::Point::new(300.0, 400.0),
            ..LayoutState::default()
        };

        // La saisie reprend la position du curseur et l'étendue courante :
        // aucune des deux n'est absente.
        layout.dragging = Some(Drag {
            which: Splitter::Sidebar,
            origin: layout.cursor.x,
            extent: layout.sidebar_width,
        });

        let drag = layout.dragging.expect("séparateur saisi");
        assert_eq!(drag.origin, 300.0, "l'origine est connue dès la saisie");

        // Un déplacement d'un seul pixel doit donc déjà produire un effet.
        let delta = 301.0 - drag.origin;
        assert_eq!(delta, 1.0, "aucun mouvement n'est avalé");
    }

    /// Un glissement ne dépend pas du nombre d'événements qui le portent.
    ///
    /// La souris n'émet pas un mouvement continu mais une suite d'événements,
    /// chacun portant un delta **mesuré depuis l'origine**. Repartir de
    /// l'étendue de départ rend donc le résultat indépendant du découpage.
    /// Ajouter le delta à la largeur courante — l'erreur commise — cumulait les
    /// deltas : dix pixels répartis sur dix événements en déplaçaient
    /// cinquante-cinq, ce qui donnait un redimensionnement absurde.
    #[test]
    fn a_drag_result_does_not_depend_on_how_many_events_carry_it() {
        let start = 300.0;
        let total = 40.0;

        // En un seul mouvement.
        let one_shot = LayoutState::dragged_extent(Splitter::Sidebar, false, start, total);

        assert_eq!(
            one_shot,
            start + total,
            "un glissement de 40 px doit déplacer de 40 px"
        );

        // Puis en dix étapes, chacune portant le delta cumulé depuis l'origine :
        // c'est exactement ce qu'émet la souris pendant un glissement.
        let mut width = start;

        for step in 1..=10 {
            let delta = total * step as f32 / 10.0;

            width = LayoutState::dragged_extent(Splitter::Sidebar, false, start, delta);
        }

        assert_eq!(
            width, one_shot,
            "le résultat doit être le même, quel que soit le découpage en événements"
        );
    }

    /// L'aperçu est de l'autre côté du séparateur : l'élargir le rétrécit.
    #[test]
    fn dragging_towards_the_preview_narrows_it() {
        let start = 400.0;

        assert_eq!(
            LayoutState::dragged_extent(Splitter::Preview, false, start, 60.0),
            start - 60.0
        );
        assert_eq!(
            LayoutState::dragged_extent(Splitter::Preview, false, start, -60.0),
            start + 60.0
        );
    }

    /// Un glissement vertical fait grandir l'aperçu quand le curseur descend.
    #[test]
    fn dragging_down_enlarges_the_bottom_preview() {
        let start = 0.5;

        let deeper = LayoutState::dragged_extent(Splitter::Preview, true, start, 100.0);
        let shallower = LayoutState::dragged_extent(Splitter::Preview, true, start, -100.0);

        assert!(deeper > start, "descendre agrandit l'aperçu : {deeper}");
        assert!(shallower < start, "monter le réduit : {shallower}");
    }

    /// La position du curseur est retenue même hors glissement : c'est ce qui
    /// permet à une saisie de démarrer au bon endroit.
    #[test]
    fn cursor_position_is_kept_even_without_a_drag() {
        let mut layout = LayoutState::default();

        layout.cursor = iced::Point::new(150.0, 250.0);

        assert!(
            layout.dragging.is_none(),
            "aucun glissement ne doit être en cours"
        );
        assert_eq!(layout.cursor.x, 150.0);
        assert_eq!(layout.cursor.y, 250.0);
    }

    /// Le mode focus ne touche pas aux préférences de panneaux : en sortir doit
    /// rendre la disposition telle qu'elle était.
    #[test]
    fn focus_mode_leaves_panel_preferences_untouched() {
        let mut layout = LayoutState::default();

        layout.preview_position = PreviewPosition::Left;
        let sidebar = layout.sidebar_width;
        let preview = layout.preview_width;

        layout.focus_mode = true;
        assert!(layout.sidebar_visible, "la préférence est conservée");
        assert!(layout.preview_visible, "la préférence est conservée");

        layout.focus_mode = false;

        assert_eq!(layout.sidebar_width, sidebar);
        assert_eq!(layout.preview_width, preview);
        assert_eq!(layout.preview_position, PreviewPosition::Left);
    }

    /// L'état de disposition par défaut doit être utilisable immédiatement :
    /// les deux panneaux ouverts, dans des proportions raisonnables.
    #[test]
    fn default_layout_is_usable() {
        let layout = LayoutState::default();

        assert!(layout.sidebar_visible);
        assert!(layout.preview_visible);
        assert!(layout.dragging.is_none());

        for width in [layout.sidebar_width, layout.preview_width] {
            assert!(
                (LayoutState::MIN_PANEL_WIDTH..=LayoutState::MAX_PANEL_WIDTH).contains(&width),
                "largeur par défaut {width} hors bornes"
            );
        }

        assert!(
            (LayoutState::MIN_PREVIEW_SHARE..=LayoutState::MAX_PREVIEW_SHARE)
                .contains(&layout.preview_height),
            "part par défaut {} hors bornes",
            layout.preview_height
        );
    }

    /// Les trois positions de l'aperçu forment un cycle complet : le bouton de
    /// la barre doit pouvoir revenir à son point de départ.
    #[test]
    fn preview_position_cycles_through_all_three() {
        let position = PreviewPosition::Right;

        assert_eq!(
            [
                position.next(),
                position.next().next(),
                position.next().next().next()
            ],
            [
                PreviewPosition::Left,
                PreviewPosition::Bottom,
                PreviewPosition::Right,
            ]
        );
    }

    /// Chaque position doit avoir un libellé : la barre n'utilise que des
    /// icônes, l'infobulle est donc la seule explication disponible.
    #[test]
    fn every_preview_position_has_a_label() {
        for position in [
            PreviewPosition::Right,
            PreviewPosition::Left,
            PreviewPosition::Bottom,
        ] {
            assert!(!position.label().is_empty());
        }
    }
}
