use crate::editor::EditorState;
use crate::project::{BibliographyEntry, Project};
use crate::typst::render::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Délai de repos avant que l'aperçu ne se mette à jour.
///
/// C'est le réglage qui gouverne tout le coût du rendu : deux secondes sans
/// écrire déclenchent la compilation, la frappe ne la déclenche jamais.
pub const IDLE_DELAY_MS: u64 = 2000;

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

    /// L'assistance à la frappe : tinymist, s'il est là.
    pub lsp: LspState,

    /// Taille de la fenêtre, telle que la vue la rapporte.
    ///
    /// Elle sert à savoir **où il y a de la place** pour la liste de complétion :
    /// un menu qui sort du panneau ne sert à rien. La valeur de départ est celle
    /// demandée à l'ouverture, remplacée dès la première mesure.
    pub window_size: iced::Size,

    /// Fenêtre de l'application, retenue dès son ouverture.
    ///
    /// Elle est nécessaire pour passer en plein écran en mode focus : le
    /// gestionnaire de messages ne reçoit pas l'identifiant de la fenêtre, et
    /// le demander par une tâche à chaque bascule serait un détour pour une
    /// donnée qui ne change pas.
    pub window: Option<iced::window::Id>,

    /// Facteur d'échelle de l'écran, demandé à l'ouverture de la fenêtre.
    ///
    /// Il règle la **densité du rendu** (`crate::typst::render::pixel_per_pt`) :
    /// rasteriser plus fin que ce que l'écran peut montrer ne se voit pas, et
    /// coûte la mémoire au carré du rapport. Un pixel par point tant que l'écran
    /// ne s'est pas déclaré — la mesure la plus économique, et la bonne sur un
    /// écran non dense.
    pub scale_factor: f32,

    /// Renommage de chapitre en cours, s'il y en a un.
    pub renaming: Option<ChapterRename>,

    /// L'endroit à atteindre une fois le document ouvert : ligne, puis colonne.
    ///
    /// Un aller à la définition peut viser un autre chapitre. Le chapitre est lu
    /// et ouvert sans que le curseur puisse y être posé tout de suite : c'est ce
    /// rendez-vous qui fait le trajet en deux temps.
    pub jump: Option<(usize, usize)>,

    /// Thème : choix de l'utilisateur et préférence détectée du système.
    pub theme: crate::ui::theme::ThemeMode,

    /// Les thèmes disponibles : les natifs, puis les fichiers locaux.
    pub themes: Vec<crate::theming::Theme>,

    /// Les fichiers de thème qui n'ont pas pu être lus.
    ///
    /// Conservés pour être **montrés** : un thème qui n'apparaît pas dans la
    /// liste sans explication est un défaut de plus, pas une discrétion.
    pub theme_problems: Vec<String>,

    /// Les thèmes retenus pour chaque apparence.
    pub chosen: crate::theming::Chosen,

    /// Le thème appliqué, sous sa forme exploitable par l'affichage.
    ///
    /// Résolu une fois par changement, et non à chaque image : construire le
    /// thème iced et la palette de coloration n'est pas gratuit, et rien ne
    /// changerait d'une image à l'autre.
    pub resolved: crate::ui::theme::Resolved,

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

/// L'assistance du serveur de langage, quand il est là.
///
/// Le serveur est **facultatif** : sans lui, tout le reste de l'application
/// fonctionne à l'identique — l'aperçu compile par ses propres moyens. Cet état
/// ne décrit donc que ce qui *s'ajoute*, et l'interface le dit tel quel plutôt
/// que de présenter son absence comme une panne.
#[derive(Default)]
pub struct LspState {
    /// La session avec le serveur, tant qu'il répond.
    pub session: Option<crate::lsp::client::Session>,

    /// Le serveur a été cherché, et n'est pas là.
    ///
    /// Retenu, pour ne pas payer un démarrage raté à chaque document ouvert :
    /// le chercher une fois suffit, et l'utilisateur qui l'installe en cours de
    /// route relancera l'application.
    pub unavailable: bool,

    /// Le serveur a répondu à l'initialisation : il est prêt à être questionné.
    pub ready: bool,

    /// L'adresse du document ouvert auprès du serveur.
    pub document: Option<String>,

    /// Ce que le serveur signale dans le document ouvert.
    ///
    /// Résumé dans la barre d'état tant que l'éditeur ne sait pas les dessiner.
    pub diagnostics: Vec<crate::lsp::messages::Diagnostic>,

    /// Une réponse est attendue : le battement d'écoute se resserre.
    pub awaiting: bool,

    /// La liste de complétion ouverte, s'il y en a une.
    pub completion: Option<Completion>,

    /// Ce que le serveur dit du symbole sous le curseur, tel qu'il s'affiche.
    pub hover: Option<Hover>,

    /// Numéro de la dernière demande de survol ou de définition.
    ///
    /// Une réponse plus ancienne que la dernière question est celle d'un endroit
    /// que le curseur a déjà quitté : elle est ignorée.
    pub attention: u64,

    /// Numéro de la dernière demande de complétion.
    ///
    /// Une réponse qui porte un numéro périmé est celle d'une frappe
    /// précédente : la garder ferait apparaître une liste qui ne correspond plus
    /// à ce qui est écrit.
    pub tokens: u64,
}

/// Ce que le serveur dit d'un symbole, prêt à être montré.
pub struct Hover {
    /// Le texte renvoyé par le serveur, en markdown.
    pub text: String,

    /// Où il s'accroche : la position du curseur au moment de la question.
    pub anchor: iced::Point,
}

/// La liste de complétion, telle qu'elle s'affiche.
pub struct Completion {
    /// Ce que le serveur a proposé.
    pub items: Vec<crate::lsp::messages::CompletionItem>,

    /// Les indices des éléments qui correspondent à ce qui est tapé, dans
    /// l'ordre voulu par le serveur.
    pub shown: Vec<usize>,

    /// La sélection, dans [`Self::shown`].
    pub selected: usize,

    /// Ce qui a été tapé après le déclencheur.
    pub typed: String,

    /// Le caractère qui a ouvert la liste.
    pub trigger: char,

    /// Le décalage, en octets, du début de ce qui serait remplacé.
    pub start: usize,

    /// Où la liste s'accroche, dans le repère de l'éditeur.
    pub anchor: iced::Point,
}

impl Completion {
    /// L'élément sélectionné, s'il y en a un.
    pub fn selected_item(&self) -> Option<&crate::lsp::messages::CompletionItem> {
        self.shown
            .get(self.selected)
            .and_then(|index| self.items.get(*index))
    }

    /// Déplace la sélection, en boucle : on ne bute pas sur les extrémités.
    pub fn move_selection(&mut self, delta: isize) {
        if self.shown.is_empty() {
            return;
        }

        let count = self.shown.len() as isize;
        let wanted = (self.selected as isize + delta).rem_euclid(count);

        self.selected = wanted as usize;
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

    /// Place que le panneau laisse à une page, telle que la vue la mesure.
    ///
    /// C'est elle qui décide de la **taille d'affichage** d'une page, donc du
    /// rapport auquel le rendu doit travailler. La vue est seule à la connaître,
    /// et elle ne change qu'au redimensionnement — le défilement, lui, ne
    /// touche pas aux bornes du viewport et ne la retransmet donc pas.
    ///
    /// `None` avant la première mesure : la géométrie part alors de la place
    /// qu'un panneau prend par défaut, et se corrige à la première image.
    pub viewport: Option<iced::Size>,

    /// Nombre de rasterisations demandées depuis l'ouverture du document.
    ///
    /// Un compteur, et non une donnée de l'application : il rend visible, dans
    /// le journal de mesure, la **cadence** à laquelle l'aperçu redemande des
    /// images — c'est-à-dire ce qu'un cycle de rasterisation consomme.
    pub requested: u64,

    /// Nombre de mots du document ouvert, tel que le texte le donne.
    ///
    /// Tenu à jour par les tics de rendu, et lu par la vue. La vue n'a pas le
    /// droit de modifier l'état, et le recalculer chez elle revenait à cloner le
    /// document entier **soixante fois par seconde** — deux fois, puisque la
    /// barre d'état et le panneau le demandent chacun. Ce n'est pas une fuite,
    /// mais c'est de l'agitation d'allocateur à chaque image, et elle se paie.
    ///
    /// `None` : le texte a changé depuis le dernier tic.
    pub words: Option<usize>,

    /// Numéro du document compilé, incrémenté à chaque compilation.
    ///
    /// Un rendu de page lancé sur un document depuis dépassé se reconnaît à
    /// ce numéro et n'écrase pas le cache avec une image obsolète.
    pub revision: u64,

    /// Page où commence le chapitre en cours, telle que la dernière
    /// compilation l'a rapportée.
    ///
    /// Rappelée à chaque compilation par l'introspection du document, à travers
    /// le repère que la compilation insère devant l'`#include` du chapitre.
    pub chapter_page: Option<usize>,

    /// Chapitre vers lequel l'aperçu a déjà été amené.
    ///
    /// On retient le **chapitre**, pas la page : la page d'un chapitre se
    /// décale à chaque frappe, et s'y accrocher ferait sauter l'aperçu sous les
    /// yeux du rédacteur. L'aperçu n'est amené qu'une fois — quand on change de
    /// chapitre.
    pub followed_chapter: Option<PathBuf>,

    /// Empreinte observée au tick précédent.
    ///
    /// Distincte de `content`, qui est l'empreinte **compilée** : celle-ci est
    /// celle qu'on vient de lire, et qui n'a pas encore été compilée. Ensemble
    /// avec `changed_at`, elle dit depuis quand le texte ne bouge plus.
    pub seen: String,

    /// Instant du dernier changement observé. `None` : rien en attente.
    pub changed_at: Option<Instant>,

    /// Nombre de retours à la ligne du texte **au moment de la dernière
    /// compilation**.
    ///
    /// C'est ce qui fait d'un retour à la ligne une frontière : un retour de
    /// plus qu'à la dernière compilation montre le résultat, sans attendre le
    /// délai de repos. Compter les retours, plutôt que regarder si le texte
    /// finit par un, rend le déclenchement robuste à ce qu'on écrit ensuite —
    /// on continue presque toujours sa phrase après être passé à la ligne.
    pub compiled_newlines: usize,

    /// Couleur du papier du document, lue sur le premier pixel de la première
    /// page rendue.
    ///
    /// Elle sert à remplir le cadre d'une page **avant** que sa texture
    /// n'existe, ou pendant que le backend téléverse la nouvelle : sans elle,
    /// on verrait le fond de l'application à la place de la page.
    pub paper: Option<iced::Color>,

    /// Empreinte des pixels de chaque page en cache.
    ///
    /// Sert à ne pas remplacer une page par une image **identique** : chaque
    /// nouveau `Handle` est une nouvelle texture, et remplacer une page qui n'a
    /// pas changé fait travailler le compositeur pour rien. C'est ce qui se
    /// voyait comme un clignotement.
    pub image_hashes: HashMap<usize, u64>,

    /// Le contenu a changé depuis la dernière écriture sur le disque.
    pub save_pending: bool,
}

impl RenderState {
    /// Vide l'état de rendu (changement de document ouvert).
    ///
    /// Les `Handle` mis en cache sont relâchés : le backend libère alors les
    /// textures du document précédent à son prochain nettoyage.
    /// L'heure est-elle venue de compiler ?
    ///
    /// Appelée à chaque tick, cette règle décide de tout le coût de l'aperçu —
    /// c'est pourquoi elle est ici, isolée et vérifiable, plutôt que dissoute
    /// dans un gestionnaire de messages.
    ///
    /// Deux déclencheurs, et deux seulement :
    ///
    /// - **le texte n'a plus bougé depuis [`IDLE_DELAY_MS`]** : c'est la fin
    ///   d'un moment d'écriture, et le moment naturel de montrer le résultat ;
    /// - **un retour à la ligne a été tapé** depuis la dernière compilation :
    ///   c'est une frontière de la mise en page, et l'on s'y arrête.
    ///
    /// Jamais de compilation sans changement : la première condition est un
    /// retour immédiat.
    ///
    /// Compiler à chaque frappe était le défaut de fond : chaque caractère
    /// coûtait une compilation *et* la reprise de toutes les pages visibles,
    /// avec une texture neuve par page. À dix frappes par seconde, cela faisait
    /// plusieurs cycles par seconde, et l'œil n'y voyait qu'un clignotement.
    pub fn compile_due(&mut self, fingerprint: &str, newlines: usize, now: Instant) -> bool {
        // Déjà compilé : rien à faire, et surtout pas recompiler.
        if fingerprint == self.content {
            self.seen.clear();
            self.changed_at = None;
            self.compiled_newlines = newlines;

            return false;
        }

        // Nouveau changement : l'horloge du repos repart.
        if fingerprint != self.seen {
            self.seen = fingerprint.to_string();
            self.changed_at = Some(now);
        }

        let Some(changed_at) = self.changed_at else {
            return false;
        };

        let idle = now.duration_since(changed_at) >= Duration::from_millis(IDLE_DELAY_MS);

        // Un retour à la ligne de plus qu'à la dernière compilation.
        let line_break = newlines > self.compiled_newlines;

        if idle || line_break {
            // Retenu maintenant : c'est le texte qu'on s'apprête à compiler, et
            // la frontière ne doit se déclencher qu'une fois.
            self.compiled_newlines = newlines;

            return true;
        }

        false
    }

    /// Faut-il amener l'aperçu sur le chapitre ouvert ?
    ///
    /// La réponse est oui **une fois par chapitre**, et non à chaque
    /// compilation : la page d'un chapitre se décale à chaque frappe, et s'y
    /// accrocher arracherait l'aperçu sous les yeux du rédacteur au milieu
    /// d'une phrase.
    ///
    /// Sans page connue, il n'y a rien à viser : mieux vaut ne pas bouger que
    /// défiler au hasard.
    pub fn should_follow_chapter(&self, open: Option<&std::path::Path>) -> bool {
        let Some(open) = open else {
            return false;
        };

        self.chapter_page.is_some() && self.followed_chapter.as_deref() != Some(open)
    }

    pub fn reset(&mut self) {
        self.content.clear();
        self.pending = false;
        self.done = false;
        self.errors.clear();
        self.warnings.clear();
        self.document = None;
        self.seen.clear();
        self.changed_at = None;
        self.compiled_newlines = 0;
        self.image_hashes.clear();
        // Le papier appartient au document qu'on quitte : le suivant le fera
        // connaître au premier rendu de page.
        self.paper = None;
        self.page_cache.clear();
        self.rendering.clear();
        self.stale.clear();
        self.visible = None;
        // La place du panneau appartient à la disposition, pas au document :
        // elle survit à un changement de chapitre.
        self.requested = 0;
        self.chapter_page = None;
        // Le compte de mots est un fait du document qu'on quitte.
        self.words = None;
        // Un document qu'on ouvre doit être amené à l'écran : oublier le
        // chapitre déjà suivi évite d'hériter du précédent.
        self.followed_chapter = None;
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
        let mut render = RenderState {
            content: "ancien document".to_string(),
            done: true,
            page_cache: [(0, handle())].into_iter().collect(),
            rendering: [1].into_iter().collect(),
            stale: [2].into_iter().collect(),
            visible: Some((0, 4)),
            save_pending: true,
            ..Default::default()
        };
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

    /// L'aperçu ne doit être amené qu'une fois par chapitre.
    ///
    /// C'est la règle qui protège la lecture : suivre le numéro de page ferait
    /// sauter l'aperçu à chaque phrase écrite dans un chapitre qui déborde sur
    /// la page suivante.
    #[test]
    fn the_preview_follows_a_chapter_only_once() {
        let first = std::path::Path::new("/projet/chapitres/01-un.typ");
        let second = std::path::Path::new("/projet/chapitres/02-deux.typ");

        let mut render = RenderState {
            chapter_page: Some(3),
            ..Default::default()
        };

        // Chapitre jamais suivi : on y amène l'aperçu.
        assert!(render.should_follow_chapter(Some(first)));

        render.followed_chapter = Some(first.to_path_buf());

        // Même chapitre : on n'y touche plus, malgré les compilations
        // qui continuent à chaque frappe.
        assert!(!render.should_follow_chapter(Some(first)));

        // Chapitre suivant : on y va.
        assert!(render.should_follow_chapter(Some(second)));

        // Aucun chapitre ouvert, ou aucune page connue : rien à viser.
        render.followed_chapter = None;
        assert!(render.should_follow_chapter(Some(first)));
        assert!(!render.should_follow_chapter(None));

        render.chapter_page = None;
        assert!(!render.should_follow_chapter(Some(first)));
    }

    /// La position du curseur est retenue même hors glissement : c'est ce qui
    /// permet à une saisie de démarrer au bon endroit.
    #[test]
    fn cursor_position_is_kept_even_without_a_drag() {
        let layout = LayoutState {
            cursor: iced::Point::new(150.0, 250.0),
            ..Default::default()
        };

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
        let mut layout = LayoutState {
            preview_position: PreviewPosition::Left,
            ..Default::default()
        };

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

    /// Jamais de compilation sans changement, même au bout de cent ticks.
    ///
    /// C'est la condition qui rend l'aperçu gratuit au repos : le tick tourne
    /// trois fois par seconde, et ne doit rien coûter.
    #[test]
    fn nothing_is_compiled_when_nothing_changed() {
        let mut render = RenderState {
            content: "le texte déjà compilé".to_string(),
            ..Default::default()
        };

        let start = Instant::now();

        for tick in 0..100u64 {
            let now = start + Duration::from_millis(tick * 300);

            assert!(
                !render.compile_due("le texte déjà compilé", 0, now),
                "tick {tick} : rien n'a changé, rien ne doit être compilé"
            );
        }
    }

    /// Tant qu'on tape, rien ne compile.
    ///
    /// Dix frappes par seconde pendant trois secondes : c'est exactement la
    /// situation qui produisait « plusieurs clignotements par seconde ».
    #[test]
    fn typing_never_triggers_a_compile() {
        let mut render = RenderState::default();

        let start = Instant::now();

        for keystroke in 0..30u64 {
            // Le tick de 300 ms voit plusieurs frappes : l'empreinte change
            // donc à chaque tick.
            let now = start + Duration::from_millis(keystroke * 100);
            let fingerprint = format!("texte au caractère {keystroke}");

            assert!(
                !render.compile_due(&fingerprint, 0, now),
                "frappe {keystroke} : la frappe ne doit jamais déclencher le rendu"
            );
        }
    }

    /// Le repos déclenche la compilation, une fois, et une seule.
    #[test]
    fn a_pause_triggers_exactly_one_compile() {
        let mut render = RenderState::default();

        let start = Instant::now();

        assert!(!render.compile_due("un paragraphe écrit", 0, start));

        // Juste avant le délai : rien encore.
        assert!(!render.compile_due(
            "un paragraphe écrit",
            0,
            start + Duration::from_millis(IDLE_DELAY_MS - 1)
        ));

        // Le délai atteint : on compile.
        assert!(render.compile_due(
            "un paragraphe écrit",
            0,
            start + Duration::from_millis(IDLE_DELAY_MS)
        ));

        // Une fois compilé, plus rien, même longtemps après.
        render.content = "un paragraphe écrit".to_string();

        for tick in 0..20u64 {
            let now = start + Duration::from_millis(IDLE_DELAY_MS + tick * 300);

            assert!(
                !render.compile_due("un paragraphe écrit", 0, now),
                "tick {tick} après la compilation"
            );
        }
    }

    /// Un retour à la ligne montre le résultat sans attendre le repos.
    ///
    /// C'est la frontière demandée : on revient à la ligne, et l'aperçu suit au
    /// tick suivant, sans attendre les deux secondes.
    ///
    /// Le déclenchement se compte en **retours de plus** qu'à la dernière
    /// compilation, ce qui le rend robuste à ce qu'on écrit ensuite : on
    /// continue presque toujours sa phrase après être passé à la ligne.
    #[test]
    fn a_line_break_shows_the_result_early() {
        let mut render = RenderState::default();

        let start = Instant::now();
        let two_lines = "première ligne\ndeuxième ligne";

        // Un retour de plus qu'à la dernière compilation : on montre tout de
        // suite.
        assert!(
            render.compile_due(two_lines, 1, start),
            "un retour à la ligne doit montrer le résultat sans attendre les deux secondes"
        );

        // La compilation faite, la frontière est acquise.
        render.content = two_lines.to_string();

        for tick in 0..20u64 {
            let now = start + Duration::from_millis(300 + tick * 300);

            assert!(
                !render.compile_due(two_lines, 1, now),
                "tick {tick} : le retour à la ligne a déjà été honoré"
            );
        }

        // Un second retour à la ligne, lui, montre de nouveau.
        let three_lines = "première ligne\ndeuxième ligne\ntroisième ligne";

        assert!(
            render.compile_due(three_lines, 2, start + Duration::from_millis(500)),
            "un nouveau retour à la ligne doit montrer de nouveau"
        );
    }

    /// Écrire dans une même ligne ne montre rien : c'est la frappe, et elle
    /// n'a jamais à déclencher le rendu.
    #[test]
    fn writing_within_a_line_waits_for_the_pause() {
        let mut render = RenderState::default();

        let start = Instant::now();

        for keystroke in 0..25u64 {
            let now = start + Duration::from_millis(keystroke * 100);
            let content = format!(
                "une phrase sans retour {}",
                "suite ".repeat(keystroke as usize)
            );

            assert!(
                !render.compile_due(&content, 0, now),
                "frappe {keystroke} : rien à montrer avant la pause"
            );
        }
    }

    /// Un nouveau changement relance l'horloge du repos.
    ///
    /// Sans cela, une seconde de frappe continue finirait par déclencher une
    /// compilation en plein milieu d'une phrase.
    #[test]
    fn a_new_change_restarts_the_idle_clock() {
        let mut render = RenderState::default();

        let start = Instant::now();

        assert!(!render.compile_due("a", 0, start));

        // Presque le délai s'écoule, puis on tape encore.
        let almost = start + Duration::from_millis(IDLE_DELAY_MS - 10);

        assert!(!render.compile_due("ab", 0, almost));

        // Dix millisecondes plus tard, le délai total est écoulé depuis le
        // début, mais seulement dix depuis la dernière frappe : rien.
        assert!(
            !render.compile_due("ab", 0, almost + Duration::from_millis(10)),
            "le repos se compte depuis la dernière frappe, pas depuis la première"
        );

        // Le repos complet depuis la dernière frappe : on compile.
        assert!(render.compile_due("ab", 0, almost + Duration::from_millis(IDLE_DELAY_MS)));
    }
}
