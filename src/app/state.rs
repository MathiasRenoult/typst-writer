use crate::editor::EditorState;
use crate::editor::completion::Completion;
use crate::project::{BibliographyEntry, Project};
use crate::typst::render::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::app::recherche_apercu::RechercheApercu;
use crate::typst::texte::{DocumentTexte, SelectionApercu};

/// Délai de repos avant que l'aperçu ne se mette à jour.
///
/// C'est le réglage qui gouverne tout le coût du rendu : trois dixièmes de
/// seconde sans écrire déclenchent la compilation, la frappe ne la déclenche
/// jamais. Une compilation de moins de dix pages coûte quelques dizaines de
/// millisecondes ; attendre plus longtemps ne protège plus rien et donne
/// l'impression que l'aperçu ne suit pas.
pub const IDLE_DELAY_MS: u64 = 300;

use iced::widget::image::Handle;
use typst_layout::PagedDocument;

/// Mode d'ajustement du zoom de l'aperçu.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PreviewZoomMode {
    /// La page occupe la largeur disponible.
    #[default]
    FitWidth,
    /// La page occupe la hauteur visible.
    FitHeight,
    /// Rapport d'affichage libre, en pixels logiques par point Typst.
    Free(f64),
}

/// L'outil de tableau : la grille s'écrit, il n'y a qu'à choisir sa taille.
#[derive(Clone)]
pub struct TableauDialogue {
    pub rangs: String,
    pub colonnes: String,
    /// Le titre du tableau, vide pour un tableau sans légende.
    pub titre: String,
}

/// Le mode d'une fenêtre d'éditeur : ce qu'elle montre.
///
/// Trois modes, et pas plus : l'**édition** (la source, écrite à la
/// markdown), le **rendu** (le document composé), et le **Typst** (ce que
/// l'application écrit vraiment — utile quand une formule refuse de se
/// composer). `Ctrl+e` bascule rendu/édition, `Ctrl+Maj+e` ouvre le
/// Typst (bug 7 du rapport 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ModeVue {
    /// La source, écrite.
    #[default]
    Edition,
    /// Le document composé.
    Rendu,
    /// Le Typst généré.
    Typst,
}

/// Les panneaux de la page Paramètres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParametresPanneau {
    /// Les boutons de la barre supérieure.
    #[default]
    Barre,
    /// Les préférences de l'éditeur.
    Editeur,
    /// Les thèmes et l'apparence de l'application.
    Apparence,
    /// Les dossiers locaux de l'application.
    Donnees,
}

impl ParametresPanneau {
    /// Le libellé de l'entrée dans le rail.
    pub fn libelle(self) -> &'static str {
        match self {
            Self::Barre => "Barre d'outils",
            Self::Editeur => "Éditeur",
            Self::Apparence => "Apparence",
            Self::Donnees => "Données",
        }
    }
}

/// Le dialogue de projet : créer, ou renommer celui qu'on nomme.
#[derive(Clone)]
pub struct ProjetDialogue {
    /// La norme choisie pour le nouveau projet — vide : celle du modèle.
    pub norme: String,

    /// L'étagère choisie — posée au projet à sa création.
    pub etagere: String,
    /// Le nom en cours de frappe.
    pub nom: String,
    /// Les métadonnées choisies à la création (bug-report-7, point 9).
    pub titre: String,
    pub auteur: String,

    /// Les bases proposées : les projets marqués comme modèles.
    pub bases: Vec<String>,

    /// La base choisie : le nom d'un projet-modèle, ou vide.
    pub base: String,

    /// Les fonctionnalités du projet à naître (demande de l'auteur).
    pub fonctionnalites: crate::mise_en_page::modele::Fonctionnalites,

    /// `None` pour créer ; le dossier pour renommer.
    pub cible: Option<std::path::PathBuf>,
}

/// Le changement de normes pour plusieurs documents sélectionnés dans la
/// bibliothèque : une seule norme, appliquée à tous.
#[derive(Debug, Clone)]
pub struct NormesLot {
    /// Les documents visés, dans l'ordre de la sélection.
    pub cibles: Vec<PathBuf>,

    /// La norme choisie dans le catalogue.
    pub norme: String,

    /// Écrire l'accroche dans les `main.typ` qui ne l'ont pas encore.
    pub accrocher: bool,
}

/// Le sélecteur de couleur : une teinte, une saturation, une valeur.
///
/// C'est la roue entière, en trois nombres — les conversions vivent dans
/// `crate::ui::picker_couleur`, pures et testées.
/// Le composeur d'extension : quelle extension, et quels paramètres.
#[derive(Debug, Clone, Default)]
pub struct ExtensionDialogue {
    /// Le rang de l'extension au registre.
    pub choix: usize,
    /// Les valeurs saisies, par nom de paramètre.
    pub valeurs: std::collections::BTreeMap<String, String>,
}

/// Le panneau d'un graphique primaviz : ses paramètres et son CSV.
///
/// Le CSV vit dans un `text_editor` : l'auteur y colle plusieurs lignes et
/// voit ce qu'il colle, au lieu d'un champ d'une seule ligne qui mangerait
/// les retours à la ligne.
pub struct ChartDialogue {
    /// Le type : `pie`, `bar`, `line`, `scatter`, `box`, `area` ou `lollipop`.
    pub type_: String,
    /// Le titre, vide quand il n'y en a pas.
    pub titre: String,
    /// Le sous-titre, vide quand il n'y en a pas.
    pub sous_titre: String,
    /// Le titre de l'axe horizontal.
    pub x_label: String,
    /// Le titre de l'axe vertical.
    pub y_label: String,
    /// La largeur Typst.
    pub largeur: String,
    /// La hauteur Typst.
    pub hauteur: String,
    /// `centre`, `gauche`, `droite` ou `pleine-largeur`.
    pub placement: String,
    /// Afficher la légende.
    pub legende: bool,
    /// Afficher les valeurs ou les pourcentages.
    pub valeurs: bool,
    /// Afficher la grille.
    pub grille: bool,
    /// Afficher les graduations des axes.
    pub graduations: bool,
    /// Les points de la ligne.
    pub points: bool,
    /// Remplir l'aire sous la ligne.
    pub aire: bool,
    /// `linear`, `smooth` ou `catmull-rom`.
    pub courbe: String,
    /// Le trou du camembert (`donut`).
    pub donut: bool,
    /// La part du rayon occupée par le trou, en pourcentage.
    pub anneau: String,
    /// Des barres horizontales.
    pub horizontal: bool,
    /// Les données CSV, telles que collées ou importées.
    pub csv: iced::widget::text_editor::Content,
}

impl Default for ChartDialogue {
    fn default() -> Self {
        Self {
            type_: "bar".to_string(),
            titre: String::new(),
            sous_titre: String::new(),
            x_label: String::new(),
            y_label: String::new(),
            largeur: "10cm".to_string(),
            hauteur: "6cm".to_string(),
            placement: "centre".to_string(),
            legende: true,
            valeurs: true,
            grille: false,
            graduations: false,
            points: true,
            aire: false,
            courbe: "linear".to_string(),
            donut: false,
            anneau: "40%".to_string(),
            horizontal: false,
            csv: iced::widget::text_editor::Content::new(),
        }
    }
}

impl ChartDialogue {
    /// L'identifiant du champ CSV : la fenêtre s'ouvre dessus.
    pub fn id_csv() -> iced::widget::Id {
        iced::widget::Id::new("chart-csv")
    }

    /// La forme attendue par le générateur.
    pub fn vers_schema(&self) -> crate::langage::schema::Chart {
        crate::langage::schema::Chart {
            type_: self.type_.clone(),
            titre: self.titre.clone(),
            sous_titre: self.sous_titre.clone(),
            x_label: self.x_label.clone(),
            y_label: self.y_label.clone(),
            largeur: self.largeur.clone(),
            hauteur: self.hauteur.clone(),
            placement: self.placement.clone(),
            legende: self.legende,
            valeurs: self.valeurs,
            grille: self.grille,
            graduations: self.graduations,
            points: self.points,
            aire: self.aire,
            courbe: self.courbe.clone(),
            donut: self.donut,
            anneau: self.anneau.clone(),
            horizontal: self.horizontal,
            donnees: self.csv.text(),
        }
    }
}

/// Le panneau d'un callout : le préréglage, le titre, et le bloc à habiller.
///
/// Le panneau ne recopie pas la citation : il l'habille sur place quand le
/// curseur y est déjà, sinon il pose un bloc de citation vide et laisse le
/// curseur dedans. Une seule vérité, le texte du document.
pub struct CalloutDialogue {
    /// La clé du préréglage : `citation`, `note`, `attention`, ou un nom
    /// défini dans les paramètres du plugin.
    pub type_: String,
    /// Le titre de la bande, vide pour un cadre sans titre.
    pub titre: String,
    /// Le curseur était-il déjà dans une citation à l'ouverture ?
    pub sur_citation: bool,
    /// Le style en cours d'édition.
    pub style: crate::langage::callout::CalloutPreset,
    /// Le préréglage d'origine, pour n'écrire que les écarts utiles.
    pub base: crate::langage::callout::CalloutPreset,
}

impl Default for CalloutDialogue {
    fn default() -> Self {
        let base = crate::langage::callout::preset_defaut("note");
        Self {
            type_: base.cle.clone(),
            titre: base.titre.clone(),
            sur_citation: false,
            style: base.clone(),
            base,
        }
    }
}

impl CalloutDialogue {
    /// L'identifiant du champ de titre : la fenêtre s'ouvre dessus.
    pub fn id_titre() -> iced::widget::Id {
        iced::widget::Id::new("callout-titre")
    }

    /// Ouvre le panneau sur un préréglage du document, ou sur `note`.
    pub fn depuis(presets: &[crate::langage::callout::CalloutPreset], sur_citation: bool) -> Self {
        let base = presets
            .iter()
            .find(|preset| preset.cle == "note")
            .cloned()
            .or_else(|| presets.first().cloned())
            .unwrap_or_else(|| crate::langage::callout::preset_defaut("note"));
        Self {
            type_: base.cle.clone(),
            titre: base.titre.clone(),
            sur_citation,
            style: base.clone(),
            base,
        }
    }

    /// Applique un préréglage choisi dans le panneau.
    pub fn choisir(&mut self, preset: crate::langage::callout::CalloutPreset) {
        self.type_ = preset.cle.clone();
        self.titre = preset.titre.clone();
        self.style = preset.clone();
        self.base = preset;
    }

    /// La balise source, avec seulement les écarts de style.
    pub fn vers_source(&self) -> String {
        crate::langage::callout::source_avec_style(
            &self.type_,
            &self.titre,
            &self.style,
            &self.base,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickerCouleur {
    /// La teinte, en degrés (0..=360).
    pub teinte: f32,
    /// La saturation (0..=1).
    pub saturation: f32,
    /// La valeur (0..=1).
    pub valeur: f32,
    /// Le code hex en cours de saisie : tant qu'il est là, c'est lui que
    /// le champ montre — la roue ne l'écrase pas sous les doigts.
    pub hex_saisi: Option<String>,
    /// La cible : `None` = la sélection du texte ; `Some(champ)` = un champ
    /// de couleur de l'éditeur de thème. La même roue sert aux deux.
    pub champ: Option<&'static str>,
    /// Une cible de normes : le champ des normes qui reçoit le hex — la
    /// vignette d'une couleur ouvre la roue pour cette couleur-là.
    pub reglage: Option<crate::mise_en_page::modele::Champ>,
    /// Une cible de callout : une couleur du panneau d'insertion ou d'un
    /// préréglage des paramètres.
    pub callout: Option<CibleCouleur>,
    /// La roue écrit dans le formulaire de la balise `couleur` ouverte
    /// dans la palette.
    pub balise: bool,
}

/// Où la roue écrit une couleur de callout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CibleCouleur {
    /// Le style du callout en cours d'insertion.
    Insertion(crate::langage::callout::CalloutChamp),
    /// Un préréglage de la norme, dans les paramètres du plugin.
    Preset(usize, crate::langage::callout::CalloutChamp),
}

/// Un geste qui défait quelque chose, en attente de son mot.
///
/// La corbeille garde tout — mais un geste qui part au clic reste un
/// geste qu'on fait parfois à tort. La question est courte, la réponse
/// nette, et `Échap` est toujours une réponse.
pub struct Confirmation {
    /// Ce qu'on demande, en une phrase.
    pub question: String,

    /// Ce que « Confirmer » fera.
    pub action: crate::app::Message,
}

/// Un import en attente de décision.
///
/// La transformation est calculée d'avance et montrée — ce qui s'est
/// converti, ce qui reste brut — avant d'entrer dans le texte. Le fichier
/// d'origine n'est ni lu autrement, ni modifié, ni supprimé.
pub struct Importation {
    /// Le fichier importé, à nommer dans la question et à l'application.
    pub chemin: std::path::PathBuf,

    /// Le résultat de la conversion, prêt à rejoindre la fin du tampon.
    pub texte: String,

    /// L'empreinte du fichier au moment de l'aperçu : si le disque change
    /// avant la décision, la conversion montrée n'est plus celle qu'on
    /// appliquerait, et l'import se refuse.
    pub empreinte: u64,
}

/// L'identifiant du champ de la note : le panneau s'ouvre dessus.
pub const ID_NOTE_CHAMP: &str = "note-champ";

/// Le petit menu du clic droit, quand le clic ne vise ni une balise ni une
/// en-tête de tableau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuContextuel {
    /// Le point du clic, dans la vue de rédaction.
    pub position: iced::Point,
}

/// L'état de l'application.
///
/// Pas de `Default` : un état utilisable suppose un projet ouvert, ce que seule
/// la séquence de démarrage sait établir.
pub struct State {
    /// La liste de complétion ouverte — locale (feuille de route, phase 1).
    pub completion: Option<Completion>,
    /// La palette de commandes, quand elle est ouverte.
    pub palette: Option<crate::app::palette::Palette>,
    /// Le formulaire d'une balise simple, quand il est ouvert.
    pub balise_dialogue: Option<crate::app::palette::BaliseDialogue>,
    /// Le menu du clic droit ordinaire, quand il est ouvert.
    pub menu_contextuel: Option<MenuContextuel>,
    /// Les normes appliquées du projet : l'interface y lit les
    /// fonctionnalités branchées pour montrer ou cacher leurs gestes
    /// (demande de l'auteur).
    pub reglages: crate::mise_en_page::modele::MiseEnPage,
    /// Le nom de la norme de la bibliothèque qui correspond exactement aux
    /// réglages appliqués — vide pour une norme personnalisée. Il ne se
    /// relit pas à chaque image : il est recalculé aux moments où le
    /// document change de normes.
    pub norme_document: String,
    /// Les rendus partagés par empreinte de texte : un seul PDF par
    /// fichier, quelle que soit la fenêtre qui le montre (bug-report-10).
    pub rendus_partages: std::collections::HashMap<String, RenduPartage>,

    pub espace: crate::app::espace::Espace,
    /// L'état du sélecteur de polices : sa liste complète, pour le
    /// vrai menu déroulant rechercheable (bug-report-9, T1.3).
    pub police_combo: iced::widget::combo_box::State<String>,
    pub typst_genere: String,
    pub editor: EditorState,

    /// Les numéros de ligne de l'éditeur sont-ils affichés ?
    ///
    /// Réglage d'application, conservé dans la session : c'est une
    /// préférence de lecture, pas une propriété du document.
    pub numeros_de_ligne: bool,

    /// Statut du dernier export PDF (`None` tant qu'aucun export n'a eu lieu).
    pub export_status: Option<String>,

    /// Le projet ouvert : ses chapitres, sa bibliographie, son assemblage.
    pub project: Project,

    /// Les projets disponibles, pour passer de l'un à l'autre.
    pub projects: Vec<Project>,
    /// Les étagères des projets, chargées avec la liste — jamais par la
    /// vue (feuille de route, phase 4, point 4.1).
    pub etageres: std::collections::BTreeMap<String, String>,
    /// L'ordre des étagères, y compris les étagères vides.
    pub etageres_ordre: Vec<String>,
    /// Les projets sélectionnés dans la bibliothèque (Ctrl/Maj+clic).
    pub selection_projets: std::collections::BTreeSet<PathBuf>,
    /// L'ancre d'une sélection par Maj+clic.
    pub dernier_projet_selectionne: Option<PathBuf>,
    /// Le changement de normes en lot, quand il est ouvert.
    pub normes_lot: Option<NormesLot>,
    /// L'état des plugins de la vue active : le biblique y range son
    /// panneau, sa référence cliquée et son analyse ; `Vue::echanger` le fait
    /// voyager avec l'onglet.
    pub plugins: crate::plugins::Plugins,

    /// Le projet et les paramètres de plugins qui ont déjà configuré
    /// `plugins`. Cela évite de réinitialiser un choix de vue à chaque
    /// sauvegarde, tout en suivant un changement de projet ou de norme.
    pub plugins_pour: Option<(
        std::path::PathBuf,
        crate::mise_en_page::modele::ParametresPlugins,
        crate::mise_en_page::modele::Fonctionnalites,
    )>,

    /// Entrées de bibliographie citables (le `.bib` du projet).
    pub bibliography: Vec<BibliographyEntry>,

    /// La collaboration avec Zotero : liée ou non, et d'où viennent les
    /// références montrées.
    pub zotero: crate::zotero::Etat,

    /// Le dernier mot de l'application : succès, information ou erreur.
    ///
    /// Il vit dans la barre d'état, **toujours visible** : un retour ne
    /// doit jamais disparaître avec le panneau qui l'a produit, ni tomber
    /// dans un journal que personne ne lit.
    pub note: Option<Note>,

    /// Le verrou du projet avant l'initialisation de l'espace.
    ///
    /// `initialiser_espace` le déplace dans `espace.verrous`, indexé par
    /// dossier : après ce point, ce champ est `None` et la table des verrous
    /// fait foi. `projet_verrouille` consulte les deux.
    pub verrou: Option<crate::project::Verrou>,

    /// Le dialogue de projet ouvert — créer, ou renommer `cible`.
    pub projet_dialogue: Option<ProjetDialogue>,

    /// `Ctrl` est-il tenu ? La molette du zoom n'en dit rien elle-même.
    pub controle_tenu: bool,

    /// `Maj` est-il tenu ? Le shift+clic ouvre un nouvel onglet au lieu de
    /// remplacer le contenu de l'onglet actif.
    pub maj_tenu: bool,

    /// Les projets ouverts, en onglets. Le premier est le courant tant
    /// qu'un autre n'est pas choisi — chaque projet garde son monde : sa
    /// bibliographie, ses normes, sa session.
    pub onglets: Vec<std::path::PathBuf>,

    /// La recherche dans les collections Zotero.
    pub zotero_recherche: String,

    /// L'onglet survolé, pour n'en montrer le × qu'à ce moment-là.
    pub survol_onglet: Option<u64>,

    /// Le projet survolé dans la liste — ses gestes (renommer, supprimer)
    /// ne paraissent qu'à ce moment (LEFT-11).
    pub survol_projet: Option<std::path::PathBuf>,

    /// L'éditeur de thème : sa copie de travail, quand il est ouvert.
    pub editeur_theme: Option<crate::theming::ThemeFile>,

    /// La saisie brute de chaque champ de couleur de l'éditeur de thème.
    /// Une valeur partiellement tapée (`#1a`) doit rester visible au
    /// clavier sans être éjectée par le parseur de couleur.
    pub editeur_theme_textes: std::collections::HashMap<String, String>,

    /// Le picker RVB intégré : le champ édité et les composantes RGB.
    /// Le nom de l'étagère à poser sur un projet — la vue des projets.
    pub etagere_brouillon: String,
    /// La liste des projets montre tous les projets (sinon : l'étagère
    /// du projet courant — bug-report-7, point 18).
    pub projets_tous: bool,
    /// La recherche dans la liste des projets.
    pub projets_recherche: String,
    /// La recherche des projets idem.
    pub projets_recherche_visible: bool,
    /// L'entrée de sommaire survolée : le texte se teinte sous la main.
    pub survol_sommaire: Option<usize>,
    /// Le sommaire du panneau, replié par l'auteur.
    pub sommaire_replie: bool,
    /// La section « Projets » du panneau, repliée par l'auteur.
    pub projets_section_replie: bool,
    /// L'étagère choisie à la liste des projets — « Mon étagère »,
    /// « Tous », ou le nom d'une étagère : un menu, pas une bascule.
    pub projets_etage: String,
    /// Le panneau de la note de bas de page : on y écrit la note, et la
    /// seule marque vient dans la ligne (bug-report-7, point 15).
    pub note_dialogue: Option<String>,

    /// Le composeur d'extension (bug-report-7, point 4) : l'extension
    /// choisie et ses paramètres — ses champs naissent de `parametres()`.
    pub extension: Option<ExtensionDialogue>,

    /// Le panneau d'un graphique primaviz — ouvert par le bouton de la
    /// barre d'outils ou par le menu Projet.
    pub chart: Option<ChartDialogue>,

    /// Le panneau d'un callout — ouvert par la barre d'outils.
    pub callout: Option<CalloutDialogue>,

    /// Les vignettes des projets : nom → pixels RGBA.
    pub vignettes: Vec<(String, Vec<u8>, u32, u32)>,

    /// Jeton d'annulation du rafraîchissement des vignettes : changer de
    /// valeur interrompt la boucle qui compilerait les projets en arrière-plan.
    pub vignettes_generation: Arc<AtomicU64>,

    /// Le mode de la fenêtre d'édition.
    pub mode: ModeVue,

    /// Le projet qu'on déplace d'une étagère à l'autre — le geste du
    /// glisser-déposer (bug 3 du rapport 2).
    pub drag_projet: Option<String>,

    /// L'étagère survolée pendant le déplacement.
    pub drag_etagere: Option<String>,

    /// La vue des projets est-elle ouverte ?
    pub projets_vue: bool,

    /// La page des paramètres de l'application est-elle ouverte ?
    pub parametres_vue: bool,

    /// Le panneau choisi dans cette page.
    pub parametres_panneau: ParametresPanneau,

    /// Les boutons de mise en forme sont-ils dans la barre ?
    ///
    /// Préférence d'application, retenue dans la session : un lecteur de
    /// markdown peut écrire sans la rangée de boutons.
    pub outils_mise_en_page: bool,

    /// Le menu d'insertion est-il dans la barre ?
    pub outils_insertion: bool,

    /// L'outil de tableau : ses rangs et ses colonnes, en attente.
    pub tableau: Option<TableauDialogue>,

    /// La confirmation en cours, quand un geste défait quelque chose.
    pub confirmation: Option<Confirmation>,

    /// Le geste attendu vient d'être confirmé : il passe au travers de la
    /// question, qui ne se reposera pas.
    ///
    /// Drapeau à usage unique : `ConfirmationConfirmer` le pose, rejoue
    /// l'action, puis le remet à `false`. Les gestionnaires destructeurs le
    /// consomment eux-mêmes avant d'exécuter ; le rejeu le retombe dans tous
    /// les cas, pour qu'une action confirmée non destructrice n'arme pas la
    /// suppression suivante.
    pub geste_confirme: bool,

    /// Le changement extérieur dont l'écrasement a été refusé, identifié par
    /// l'empreinte de ce qu'il a mis sur disque. Un autre changement extérieur
    /// est une question neuve ; le même ne se représente pas à chaque sauvegarde.
    pub conflit_refuse: Option<u64>,

    /// L'import en attente de décision : la transformation est montrée avant
    /// d'entrer dans le texte, et l'original ne se touche jamais.
    pub importation: Option<Importation>,

    /// La copie de la dernière notice supprimée, à la corbeille : c'est le
    /// chemin du regret possible.
    pub derniere_notice_supprimee: Option<std::path::PathBuf>,

    /// Un style dépendant attend son parent : son titre et l'adresse du
    /// parent, tant que la question de téléchargement est ouverte.
    pub style_dependant_en_attente: Option<(String, String)>,

    /// La carte des raccourcis, quand elle est demandée (`F1`).
    pub aide: bool,

    /// La recherche en cours, quand elle est ouverte.
    pub recherche: Option<crate::app::recherche::Recherche>,

    /// L'écran de restauration des versions antérieures.
    pub restauration: Option<crate::app::restauration::Restauration>,

    /// La corbeille ouverte, quand l'écran de récupération la montre.
    pub corbeille: Option<crate::app::corbeille::Corbeille>,

    /// Le panneau de liaison à Zotero, quand il est ouvert.
    pub liaison: Option<LiaisonZotero>,

    /// Résultat du rendu fidèle (compilation Typst en bibliothèque).
    pub render: RenderState,

    /// Disposition des panneaux : visibilité, largeurs, position de l'aperçu.
    pub layout: LayoutState,

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

    /// Le dialogue de mise en page, s'il est ouvert.
    pub mise_en_page: Option<DialogueMiseEnPage>,

    /// Le composeur de notes de références, s'il est ouvert.
    ///
    /// Il tient le clavier tant qu'il est là : le document ne reçoit la note
    /// qu'à la fin, et Échap ne laisse rien.
    pub composeur: Option<crate::app::composeur::Composeur>,

    /// Renommage de chapitre en cours, s'il y en a un.
    pub renaming: Option<ChapterRename>,

    /// L'endroit à atteindre une fois le document ouvert : ligne, puis colonne.
    ///
    /// Un aller à la définition peut viser un autre chapitre. Le chapitre est lu
    /// et ouvert sans que le curseur puisse y être posé tout de suite : c'est ce
    /// rendez-vous qui fait le trajet en deux temps.
    pub jump: Option<(usize, usize)>,

    /// Thème : choix de l'utilisateur et préférence détectée du système.
    pub theme: crate::base::theme::ThemeMode,

    /// Les thèmes disponibles : les natifs, puis les fichiers locaux.
    pub themes: Vec<crate::theming::Theme>,

    /// La dernière couleur choisie dans la barre — TOPBAR-06.
    pub derniere_couleur: Option<String>,

    /// Le sélecteur de couleur : la roue, quand il est ouvert.
    pub picker_couleur: Option<PickerCouleur>,

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
    pub resolved: crate::design::theme::Resolved,

    /// Mode rapporté par le système, `None` s'il n'a pas encore été lu ou
    /// n'exprime aucune préférence.
    pub system_theme: crate::design::theme::SystemMode,
}

/// Un renommage de chapitre en cours.
///
/// Le titre est édité dans le panneau, en place : la ligne du chapitre devient
/// un champ de saisie, ce qui évite une boîte de dialogue pour une opération
/// Le dialogue de mise en page.
///
/// Ce qu'il montre est une **copie de travail** : la mise en page du projet
/// n'est touchée qu'à l'application. Fermer la fenêtre sans appliquer laisse
/// le document exactement ce qu'il était — la règle de tout dialogue.
pub struct DialogueMiseEnPage {
    /// Les métadonnées du document (bug-report-7, point 9) : elles se
    /// règlent ici, au premier onglet, et alimentent les quick parts.
    pub metadonnees: crate::project::metadonnees::Metadonnees,

    /// Les réglages en cours de réglage.
    pub reglages: crate::mise_en_page::modele::MiseEnPage,

    /// Le nom de la norme de la bibliothèque — appliquer, ou enregistrer.
    pub norme: String,

    /// Le nom du profil de métadonnées à enregistrer (bug-report-10, 4.1).
    pub profil_nom: String,

    /// Le panneau des plugins est-il ouvert, plutôt que les normes ?
    pub plugins: bool,

    /// Le panneau ouvert : 0 la mise en page, 1 la typographie.
    pub panneau: usize,

    /// L'onglet ouvert dans les paramètres des plugins — séparé du panneau
    /// des normes pour que chaque vue retrouve le sien après une bascule.
    pub panneau_plugins: usize,

    /// Le préréglage de callout sélectionné dans l'onglet Callouts.
    pub callout_preset: usize,

    /// Accrocher la mise en page à `main.typ` lors de l'application.
    pub accrocher: bool,

    /// Les deux lignes d'accroche manquent-elles à `main.typ` ?
    ///
    /// C'est ce qui décide si la case se montre : un document déjà accroché
    /// n'a rien à quoi la cocher.
    pub accroche_manquante: bool,

    /// Combien de règles `#set` `main.typ` porte déjà, écrites à la main.
    ///
    /// Elles priment sur les réglages de la fenêtre — c'est ainsi que Typst
    /// ordonne les règles — et il vaut mieux le savoir avant de croire son
    /// réglage appliqué.
    pub regles_main: usize,

    /// L'aperçu du style choisi : le petit jeu représentatif, rendu par le
    /// moteur même — jamais une imitation (R4).
    pub apercu_style: Option<String>,

    /// L'aperçu des titres : les six niveaux rendus par le moteur avec les
    /// normes du dialogue — jamais une imitation (R5).
    /// Les styles installés dans Zotero, tels qu'ils étaient à l'ouverture
    /// de la fenêtre : la troisième porte du choix (R4).
    pub styles_zotero: Vec<crate::zotero::styles::CslStyle>,

    /// Le nom de la norme quand la fenêtre s'est ouverte — pour savoir
    /// si l'utilisateur travaille sur une norme existante ou sur un brouillon.
    pub norme_originale: String,

    /// Les réglages ont-ils changé depuis le chargement ?
    pub modifie: bool,
}

/// Ce que l'application vient de dire.
#[derive(Debug, Clone)]
pub struct Note {
    /// Ce qui s'est passé, en une phrase.
    pub texte: String,

    /// Une erreur, ou pas : la couleur suit.
    pub erreur: bool,
}

impl Note {
    pub fn info(texte: impl Into<String>) -> Self {
        Self {
            texte: texte.into(),
            erreur: false,
        }
    }

    pub fn erreur(texte: impl Into<String>) -> Self {
        Self {
            texte: texte.into(),
            erreur: true,
        }
    }
}

/// Le panneau de liaison à Zotero : où en est sa lecture.
#[derive(Debug, Clone)]
pub enum LiaisonZotero {
    /// Les collections se lisent.
    Chargement,

    /// Elles sont là : il ne reste qu'à choisir.
    Choix(Vec<crate::zotero::client::Collection>),

    /// Quelque chose n'est pas allé — et c'est dit, jamais avale.
    Erreur {
        /// Ce qui s'est passé, en une phrase.
        motif: String,

        /// Est-ce le réglage d'API, que l'application peut poser elle-même ?
        reglable: bool,
    },
}

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

    /// L'aperçu suit-il le curseur d'écriture ?
    ///
    /// Le suivi évite de croire qu'une ligne n'est pas rendue ; il est aussi
    /// ce qui arrache l'aperçu sous les yeux quand on écrit ailleurs. Les deux
    /// usages sont légitimes : c'est donc un réglage, et non un choix.
    pub follow_preview: bool,

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
    pub const fn next(self) -> Self {
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
            follow_preview: true,
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
    pub const fn clamp_width(width: f32) -> f32 {
        width.clamp(Self::MIN_PANEL_WIDTH, Self::MAX_PANEL_WIDTH)
    }

    /// Applique une part de hauteur, bornée.
    pub const fn clamp_share(share: f32) -> f32 {
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
/// Ce qu'un rendu de page montre : le morceau rasterisé — la fenêtre
/// visible, élargie d'une marge — et où il se place dans la page (bug-report-9,
/// fluidité : la page hors écran n'est jamais rasterisée).
///
/// La position et la taille sont des **fractions de la page** (0..1) : la
/// tranche s'étire donc avec le zoom — retour visuel immédiat pendant le
/// geste — au lieu de rester à la taille qu'elle avait au rendu.
pub struct Tranche {
    /// Position du morceau, en fraction de la page.
    pub gauche: f32,
    pub haut: f32,
    /// Taille du morceau, en fraction de la page.
    pub largeur: f32,
    pub hauteur: f32,
    /// Les pixels, tels que le backend les montre.
    pub image: Handle,
    /// L'empreinte du PDF dont cette tranche est issue.
    ///
    /// Sert à ne pas dessiner la couche texte d'un PDF neuf sur l'image de
    /// l'ancien, le temps que la nouvelle page soit rasterisée.
    pub empreinte: Option<String>,
    /// L'image précédente, gardée **sous** la nouvelle pendant le téléversement
    /// de sa texture. Sans elle, une page changerait en blanc le temps que le
    /// backend graphique reçoive les nouveaux pixels.
    pub precedente: Option<Box<Tranche>>,
}

impl Default for Tranche {
    fn default() -> Self {
        Self {
            gauche: 0.0,
            haut: 0.0,
            largeur: 0.0,
            hauteur: 0.0,
            image: Handle::from_rgba(1, 1, vec![0, 0, 0, 0]),
            empreinte: None,
            precedente: None,
        }
    }
}

/// Le rendu d'un texte, partagé par **toutes** les fenêtres qui le
/// montrent : un seul PDF pour un seul fichier, jamais un par fenêtre
/// (bug-report-10). Les deux pièces sont des `Arc` : chacune avance à
/// son rythme — la compilation pose le document, l'export pose le
/// lecteur — et tous les états de fenêtre qui les tiennent montrent la
/// même chose.
#[derive(Default, Clone)]
pub struct RenduPartage {
    pub document: Option<std::sync::Arc<PagedDocument>>,
    pub lecteur: Option<std::sync::Arc<crate::typst::render::DocumentPdf>>,
}

impl RenduPartage {
    /// Une pièce de plus, sans rien perdre de ce qui était déjà là.
    pub fn completant(&mut self, autre: &Self) {
        if autre.document.is_some() {
            self.document.clone_from(&autre.document);
        }

        if autre.lecteur.is_some() {
            self.lecteur.clone_from(&autre.lecteur);
        }
    }
}

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

    /// Le PDF exporté, source de l'affichage (bug-report-9, T4) : le
    /// lecteur le rasterise, et c'est exactement le fichier livré.
    pub lecteur: Option<Arc<crate::typst::render::DocumentPdf>>,

    /// L'empreinte du texte du PDF affiché.
    ///
    /// Distincte de `revision`, qui change aussi au zoom : c'est elle qui
    /// apparie la couche texte et l'image, jamais deux documents différents.
    pub lecteur_empreinte: Option<String>,

    /// La couche texte du PDF affiché, extraite du `PagedDocument`.
    pub texte: Option<Arc<DocumentTexte>>,

    /// L'empreinte de la couche texte installée.
    pub texte_empreinte: Option<String>,

    /// Une couche texte extraite pour un PDF qui n'est pas encore arrivé.
    pub texte_en_attente: Option<(String, Arc<DocumentTexte>)>,
    /// L'empreinte d'une extraction de texte actuellement en cours.
    ///
    /// Sans ce drapeau, un tick de rendu pouvait relancer la même extraction
    /// avant que la première n'ait répondu : sur un gros document, plusieurs
    /// extractions simultanées saturent un cœur et allouent plusieurs couches
    /// texte pour rien.
    pub texte_en_cours: Option<String>,

    /// La sélection courante dans l'aperçu.
    pub selection: Option<SelectionApercu>,

    /// La recherche dans le document rendu.
    pub recherche_apercu: RechercheApercu,

    /// Un export PDF est en cours — la bascule vers la vue rendu le
    /// déclenche systématiquement (bug-report-9).
    /// L'empreinte du texte exporté, pour nourrir le registre à sa
    /// place (bug-report-10).
    pub export_empreinte: Option<String>,

    pub export_en_cours: bool,
    /// Le PDF a été demandé pour cette compilation : une tentative par
    /// compilation, pour qu'un export qui échoue ne boucle pas.
    pub export_tente: bool,

    /// Le sommaire du document : titres, sources, pages.
    ///
    /// Il sert deux fois : le panneau de travail le montre, et le suivi de
    /// l'aperçu s'y lit pour savoir quelle page porte la ligne qu'on lit.
    pub outline: Vec<crate::typst::render::OutlineEntry>,

    /// Dernière page suivie par le suivi de l'aperçu.
    ///
    /// Sans elle, chaque défilement renverrait la même opération de défilement
    /// à l'aperçu, soixante fois par seconde.
    pub followed_page: Option<usize>,

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
    pub page_cache: HashMap<usize, Tranche>,

    /// Pages dont la rasterisation est demandée mais pas terminée.
    ///
    /// La révision fait partie de la clé : l'ancienne et la nouvelle version
    /// d'une page peuvent être en cours ensemble, sans que l'une libère par
    /// mégarde la réservation de l'autre.
    pub rendering: HashSet<(u64, usize)>,

    /// Pages dont l'image doit être (re)produite.
    ///
    /// Une page entre ici quand le document vient d'être recompilé, ou
    /// quand elle apparaît dans la fenêtre sans image valide.
    /// `rasterize_visible` ne travaille que sur l'intersection de cet
    /// ensemble avec la plage matérialisée : un défilement sur des images à
    /// jour ne coûte rien, et une page périmée reste affichée avec son
    /// ancienne image jusqu'à l'arrivée de la nouvelle (aucun cadre gris).
    pub stale: HashSet<usize>,

    /// Pages entrées dans la fenêtre pendant un défilement. Leur rendu
    /// utilise la passe rapide, qui est aussi la passe finale de l'aperçu.
    pub qualite_rapide: HashSet<usize>,

    /// Première et dernière page visibles à l'écran (inclusives), d'après le
    /// dernier scroll. `None` avant le premier rendu de la vue.
    pub visible: Option<(usize, usize)>,

    /// Les pages qui touchent réellement la fenêtre, sans la marge de
    /// préchargement. C'est cette plage qui décide de la région complète à
    /// rasteriser ; la plage [`Self::visible`] ne sert qu'aux widgets et à
    /// une bande de préchargement par voisin.
    pub visible_exact: Option<(usize, usize)>,

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

    /// Mode de zoom actuellement appliqué à toutes les pages.
    pub zoom: PreviewZoomMode,

    /// Le défilement courant, en pixels logiques du contenu (marge du
    /// panneau déjà retirée). C'est lui qui désigne la région visible —
    /// la seule qu'il faut rasteriser.
    pub scroll: (f32, f32),

    /// Page à rejoindre quand la vue de rendu s'ouvrira après une compilation.
    ///
    /// Une bascule depuis l'édition peut désigner une page avant que le
    /// document ne soit compilé. La consigne survit alors à la compilation et
    /// s'applique dès que le document est là, au lieu de laisser la vue au
    /// début du fichier.
    pub reprise_page: Option<usize>,

    /// La page doit être recalculée depuis la ligne de lecture dès que le
    /// sommaire composé sera connu.
    ///
    /// C'est le cas d'une première ouverture du rendu : avant la compilation,
    /// aucune correspondance ligne → page n'existe encore.
    pub reprise_apres_compilation: bool,

    /// Le prochain affichage de l'aperçu doit restaurer son ancre exacte.
    ///
    /// Le scrollable est reconstruit avec la vue ; la tâche de défilement
    /// partie avant la reconstruction peut ne plus avoir de cible. La vue
    /// elle-même signale son premier affichage, qui est le moment sûr.
    pub reprise_apercu: bool,

    /// Le sens du dernier défilement : `1` vers le bas, `-1` vers le haut,
    /// `0` immobile. Il sert à précharger d'abord la page vers laquelle on
    /// va, comme le fait un lecteur de PDF.
    pub scroll_direction: i8,

    /// La fenêtre du scrollable, dans les coordonnées de la fenêtre :
    /// c'est elle qui convertit la position du curseur en position de
    /// défilement — l'ancre du zoom (bug-report-9).
    pub fenetre: Option<iced::Rectangle>,

    /// Le geste de zoom est en cours : le rendu net attend la fin du geste
    /// (bug-report-9, « geste différé ») plutôt que de relancer un rendu
    /// par cran de molette.
    pub zoom_depuis: Option<std::time::Instant>,

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

    /// Le volume du document : corps hors notes, notes et citations de bloc.
    ///
    /// Mesuré avec `words`, pour que la barre d'état et le panneau lisent la
    /// même photographie du texte.
    pub volume: Option<crate::langage::volume::Volume>,

    /// Un recomptage est dû.
    ///
    /// C'est un drapeau, et non un `None` dans `words` : effacer la valeur
    /// faisait afficher **zéro** jusqu'au tic suivant, à chaque clic et à chaque
    /// coup de molette — car tout message de l'éditeur invalidait le compte,
    /// même ceux qui ne touchent pas au texte.
    pub words_pending: bool,

    /// Instant de la dernière frappe qui a invalidé le compte.
    ///
    /// Un recomptage de `.tw` rejoue le parseur ; le faire à chaque tic pendant
    /// une frappe continue revenait à parser le document dix fois par seconde.
    /// Le compte attend donc le repos, comme la compilation.
    pub words_depuis: Option<std::time::Instant>,

    /// Révision des images attendues par l'aperçu.
    ///
    /// Elle change à chaque compilation et chaque invalidation de densité
    /// (zoom, dimensions du panneau ou écran). Une rasterisation lancée pour
    /// l'ancienne révision ne peut ainsi valider une image devenue trop petite.
    pub revision: u64,

    /// Génération partagée avec les tâches de rasterisation.
    ///
    /// Une tâche qui démarre après une invalidation n'a plus de travail à
    /// faire : elle le constate avant de payer le rendu.
    pub generation: Arc<AtomicU64>,

    /// Génération de la **vue** : elle change à chaque défilement ou zoom.
    ///
    /// La révision précédente invalide les pixels ; celle-ci invalide le
    /// travail d'une fenêtre qu'on a déjà quittée. Les tâches en attente
    /// s'arrêtent avant de lire une page devenue inutile.
    pub vue_generation: Arc<AtomicU64>,

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

/// Ce que l'application sait montrer de son état, sans le modifier.
impl State {
    /// Marque le compte de mots comme périmé, sans le recalculer.
    ///
    /// Toute frappe repousse le repos : le parseur de `.tw` ne sera rejoué
    /// qu'une fois la main levée.
    pub fn mots_a_recalculer(&mut self) {
        self.render.mots_a_recalculer();
    }

    /// Le volume du contenu ouvert, selon son format.
    ///
    /// Le `.tw` passe par l'AST pour que les notes et les citations de bloc
    /// soient distinguées ; un fichier brut se contente de ses mots.
    pub fn content_volume(&self, content: &str) -> crate::langage::volume::Volume {
        let is_intermediate = self
            .editor
            .path
            .as_deref()
            .and_then(|path| path.extension())
            .is_some_and(|extension| extension == "tw");

        if is_intermediate {
            crate::langage::volume::mesurer(&crate::langage::parse(content))
        } else {
            crate::langage::volume::Volume {
                mots: crate::project::count_words(content),
                ..Default::default()
            }
        }
    }

    /// Met à jour d'un seul geste le compte et le volume affichés.
    pub fn actualiser_le_volume(&mut self, content: &str) {
        let volume = self.content_volume(content);
        self.render.words = Some(volume.mots);
        self.render.volume = Some(volume);
    }

    /// Le volume du document affiché, ou des zéros avant la première mesure.
    pub fn volume(&self) -> crate::langage::volume::Volume {
        self.render.volume.unwrap_or_default()
    }

    /// Le nombre de mots d'une sélection, notes exclues.
    pub fn selection_word_count(&self) -> Option<usize> {
        let texte = self.editor.editor.selected_text()?;
        if texte.trim().is_empty() {
            return None;
        }
        Some(crate::langage::volume::mesurer_texte(&texte).mots)
    }

    /// Le nombre de mots à afficher pour la séance en cours.
    ///
    /// Le total du projet est relevé sur le disque, et daterait du dernier
    /// scan ; le fichier ouvert, lui, est compté **en direct**. Une seule
    /// définition, pour que la barre d'état et le panneau ne puissent pas
    /// annoncer deux chiffres différents — ni, comme ils le faisaient, effacer
    /// le leur le temps d'un tic.
    pub fn word_count(&self) -> usize {
        let total = self.project.words();

        if let Some(document) = self.project.document.as_deref() {
            if self.editor.path.as_deref() == Some(document) {
                return total.saturating_sub(self.project.document_words)
                    + self.render.words.unwrap_or(self.project.document_words);
            }

            return total;
        }

        let open = self
            .project
            .chapters
            .iter()
            .find(|chapter| Some(chapter.path.as_path()) == self.editor.path.as_deref());

        match open {
            Some(chapter) => total - chapter.words + self.render.words.unwrap_or(chapter.words),
            None => total,
        }
    }
}

impl RenderState {
    /// Invalide le compte de mots et repousse son recalcul au repos.
    pub fn mots_a_recalculer(&mut self) {
        self.words_pending = true;
        self.words_depuis = Some(Instant::now());
    }

    /// Le dernier aperçu valide est affiché alors que la compilation
    /// actuelle a échoué : il est **ancien**, et doit se dire tel.
    ///
    /// Un échec ne vide pas l'aperçu — la dernière image bonne reste sous
    /// les erreurs — mais sans cette déclaration, un aperçu périmé se ferait
    /// passer pour l'état du texte courant (R3).
    pub fn apercu_perime(&self) -> bool {
        self.document.is_some() && !self.errors.is_empty()
    }

    /// Le voisinage conservé autour des pages visibles.
    ///
    /// Une image de page pèse environ 2 Mo ; un document long parcouru
    /// jusqu'au bout gardait ses centaines de pages en mémoire, sans aucune
    /// borne — exactement la croissance qui a déjà fait tomber le compositeur.
    /// Garder un voisinage évite de re-rasteriser à chaque aller-retour d'un
    /// écran, sans pour autant accumuler le document entier.
    pub const CACHE_MARGIN: usize = 4;

    /// Les pages dont l'image reste en mémoire : celles qu'on voit, plus leur
    /// voisinage.
    pub fn pages_to_keep(&self) -> std::ops::RangeInclusive<usize> {
        let (first, last) = self.visible.unwrap_or_else(|| {
            let total = self
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);
            (0, total.saturating_sub(1))
        });

        first.saturating_sub(Self::CACHE_MARGIN)..=last.saturating_add(Self::CACHE_MARGIN)
    }

    /// Borner le cache d'images — et les empreintes avec lui.
    ///
    /// Les deux vont de pair : une page évincée dont on garderait l'empreinte
    /// des pixels ne serait **plus jamais** redessinée, puisque l'image qui
    /// revient aurait la même empreinte et serait tenue pour déjà affichée.
    pub fn bound_page_cache(&mut self) {
        let keep = self.pages_to_keep();

        self.page_cache.retain(|index, _| keep.contains(index));
        self.image_hashes.retain(|index, _| keep.contains(index));
    }

    /// Combien de rasterisations mènent leur vie en même temps.
    ///
    /// Une page se rasterise en quelques dizaines de millisecondes ; en
    /// laisser passer plus d'un petit nombre à la fois ne rend rien plus
    /// vite, et charge la mémoire de tampons RGBA qui n'ont qu'à attendre.
    /// Ce qui n'a pas eu de place reste périmé, et repart quand une place se
    /// libère : la file est la même, seuls les départs sont espacés.
    pub const SIMULTANEES: usize = 4;

    /// Réserve une rasterisation, sauf si elle est déjà en cours pour cette
    /// révision et cette page — ou s'il n'y a plus de place : la page reste
    /// alors périmée, et repartira quand une place se libérera.
    pub fn begin_page_render(&mut self, revision: u64, index: usize) -> bool {
        if self.rendering.len() >= Self::SIMULTANEES {
            return false;
        }

        self.rendering.insert((revision, index))
    }

    /// Une image d'une révision passée n'est plus bonne à montrer.
    ///
    /// Elle se jette, et la page repart dans les périmées : c'est la
    /// nouvelle révision qui la redessinera. `true` quand l'image est à
    /// jeter.
    pub fn image_perimee(&mut self, revision: u64, index: usize) -> bool {
        if revision != self.revision {
            self.stale.insert(index);
            return true;
        }

        false
    }

    /// Libère la réservation exacte d'une rasterisation terminée.
    ///
    /// Une tâche obsolète ne doit pas libérer celle de la nouvelle révision,
    /// qui peut rasteriser la même page en parallèle.
    pub fn finish_page_render(&mut self, revision: u64, index: usize) {
        self.rendering.remove(&(revision, index));
    }

    /// Invalide les images après un changement de zoom, de viewport ou d'écran.
    ///
    /// Les anciens `Handle` restent affichables le temps du nouveau rendu, mais
    /// toute tâche en vol appartient à une génération périmée. La nouvelle
    /// densité sera rasterisée pour les seules pages visibles.
    pub fn invalidate_rasterization(&mut self, total_pages: usize) {
        self.revision += 1;
        self.generation.store(self.revision, Ordering::Relaxed);

        // Les tâches en vol **gardent leur place** : les effacer rouvrait
        // le plafond à chaque invalidation, et un redimensionnement
        // continu lançait des rafales de rendus dont les précédents
        // tournaient encore. Elles libéreront la leur en finissant.
        self.image_hashes.clear();
        self.qualite_rapide.clear();
        self.stale.retain(|index| *index < total_pages);
        self.stale.extend(0..total_pages);
    }

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

    /// La couche texte correspond-elle déjà à cette compilation ?
    pub fn texte_est_pret(&self, empreinte: &str) -> bool {
        self.texte_empreinte.as_deref() == Some(empreinte)
            || self
                .texte_en_attente
                .as_ref()
                .is_some_and(|(attendue, _)| attendue == empreinte)
            || self.texte_en_cours.as_deref() == Some(empreinte)
    }

    /// Installe la couche texte quand le PDF qu'elle décrit est affiché.
    pub fn preparer_texte(&mut self, empreinte: String, texte: Arc<DocumentTexte>) {
        if self.texte_en_cours.as_deref() == Some(empreinte.as_str()) {
            self.texte_en_cours = None;
        }
        self.texte_en_attente = Some((empreinte, texte));
        self.installer_texte_si_pret();
    }

    /// Installe la couche en attente si son empreinte est celle du PDF.
    pub fn installer_texte_si_pret(&mut self) {
        let Some((empreinte, texte)) = self.texte_en_attente.clone() else {
            return;
        };

        if self.lecteur_empreinte.as_deref() != Some(empreinte.as_str()) {
            return;
        }

        self.texte = Some(texte.clone());
        self.texte_empreinte = Some(empreinte);
        self.texte_en_attente = None;
        self.selection = None;

        if !self.recherche_apercu.motif.is_empty() {
            self.recherche_apercu.resultats =
                crate::app::recherche_apercu::chercher(&texte, &self.recherche_apercu.motif);
            self.recherche_apercu.courante = 0;
        }
    }

    /// Après l'arrivée d'un PDF, garde le texte qui lui correspond.
    pub fn accorder_texte_au_pdf(&mut self) {
        if self.texte_empreinte != self.lecteur_empreinte {
            self.texte = None;
            self.texte_empreinte = None;
            self.selection = None;
            self.recherche_apercu.resultats.clear();
            self.recherche_apercu.courante = 0;
        }

        self.installer_texte_si_pret();
    }

    /// La couche texte réellement affichable.
    pub fn texte_affiche(&self) -> Option<&DocumentTexte> {
        self.texte.as_deref()
    }

    /// Le changement de document repart d'un rendu vierge, sans conserver
    /// d'image ni de travail en cours.
    pub fn reset(&mut self) {
        // Le lecteur montre le PDF du projet quitté : il part avec lui.
        self.lecteur = None;
        self.export_en_cours = false;
        self.export_tente = false;
        self.scroll = (0.0, 0.0);
        self.scroll_direction = 0;
        self.zoom_depuis = None;
        self.content.clear();
        self.pending = false;
        self.done = false;
        self.errors.clear();
        self.warnings.clear();
        self.document = None;
        self.lecteur_empreinte = None;
        self.texte = None;
        self.texte_empreinte = None;
        self.texte_en_attente = None;
        self.texte_en_cours = None;
        self.selection = None;
        self.recherche_apercu = RechercheApercu::default();
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
        self.qualite_rapide.clear();
        self.visible = None;
        self.visible_exact = None;
        self.fenetre = None;
        // La place du panneau appartient à la disposition, pas au document.
        self.requested = 0;
        self.reprise_page = None;
        self.reprise_apres_compilation = false;
        self.reprise_apercu = false;
        // Le compte de mots est un fait du document qu'on quitte : la valeur
        // s'efface, et un recomptage est dû.
        self.words = None;
        self.volume = None;
        self.words_pending = true;
        self.words_depuis = None;
        self.save_pending = false;
        self.revision += 1;
        self.generation.store(self.revision, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {

    /// La couche texte ne s'affiche jamais sur une image d'un autre document.
    #[test]
    fn la_couche_texte_attend_le_pdf_quelle_decrit() {
        let mut render = RenderState::default();
        let texte = Arc::new(crate::typst::texte::DocumentTexte { pages: Vec::new() });

        render.lecteur_empreinte = Some("v1".to_string());
        render.preparer_texte("v2".to_string(), texte.clone());
        assert!(
            render.texte.is_none(),
            "un texte neuf ne se pose pas sur l'ancien PDF"
        );

        render.lecteur_empreinte = Some("v2".to_string());
        render.accorder_texte_au_pdf();
        assert!(render.texte.is_some(), "le PDF v2 installe le texte v2");
    }

    /// Une extraction en cours n'est pas relancée par le tick suivant.
    #[test]
    fn une_extraction_en_cours_ne_se_relance_pas() {
        let render = RenderState {
            texte_en_cours: Some("v1".to_string()),
            ..RenderState::default()
        };

        assert!(render.texte_est_pret("v1"));
        assert!(!render.texte_est_pret("v2"));
    }

    /// Une couche installée est oubliée si le PDF affiché change de version.
    #[test]
    fn un_pdf_d_une_autre_version_retire_l_ancienne_couche() {
        let mut render = RenderState::default();
        let texte = Arc::new(crate::typst::texte::DocumentTexte { pages: Vec::new() });

        render.lecteur_empreinte = Some("v1".to_string());
        render.preparer_texte("v1".to_string(), texte);
        assert!(render.texte.is_some());

        render.lecteur_empreinte = Some("v2".to_string());
        render.accorder_texte_au_pdf();
        assert!(render.texte.is_none(), "le texte de v1 ne reste pas sur v2");
    }

    /// Une invalidation efface les marques de rendu rapide.
    #[test]
    fn une_invalidation_efface_les_marques_de_qualite() {
        let mut render = RenderState::default();
        render.qualite_rapide.insert(3);

        render.invalidate_rasterization(10);

        assert!(render.qualite_rapide.is_empty());
    }

    /// Une invalidation ne rouvre pas le plafond : les rendus en vol
    /// gardent leur place, et c'est leur fin qui libère la suivante.
    /// Les tâches concurrentes (feuille de route, phase 3, point 3.2) : la
    /// même page ne se rasterise pas deux fois pour la même révision.
    /// Les tâches concurrentes (feuille de route, phase 3, point 3.2) : une
    /// tâche obsolète ne libère pas la place de la nouvelle révision.
    #[test]
    fn une_tache_obsolete_ne_libere_pas_la_place_neuve() {
        let mut render = RenderState::default();
        render.begin_page_render(render.revision, 0);
        let ancienne = render.revision;
        render.invalidate_rasterization(1);
        render.begin_page_render(render.revision, 0);
        // L'ancienne tâche finit : sa place à elle est libérée, pas celle de
        // la nouvelle révision.
        render.finish_page_render(ancienne, 0);
        assert!(
            !render.begin_page_render(render.revision, 0),
            "la nouvelle tâche tient encore sa place"
        );
    }

    #[test]
    fn une_page_ne_se_rasterise_pas_deux_fois() {
        let mut render = RenderState::default();
        let revision = render.revision;
        assert!(render.begin_page_render(revision, 3), "la première part");
        assert!(
            !render.begin_page_render(revision, 3),
            "la seconde est refusée : la page est déjà en cours"
        );
        // Une autre page trouve sa place.
        assert!(render.begin_page_render(revision, 4), "une autre page part");
    }

    #[test]
    fn une_invalidation_ne_rouvre_pas_le_plafond() {
        let mut render = RenderState {
            visible: Some((0, 49)),
            ..Default::default()
        };
        render.stale.extend(0..50);

        let mut partis = 0;
        for index in 0..50 {
            if render.begin_page_render(0, index) {
                partis += 1;
            }
        }
        assert_eq!(partis, RenderState::SIMULTANEES);

        // Le zoom, ou le redimensionnement : la révision change.
        render.invalidate_rasterization(50);

        // Aucune place de plus : les quatre tâches tiennent toujours la
        // leur, même périmées.
        let mut de_suite = 0;
        for index in 0..50 {
            if render.begin_page_render(render.revision, index) {
                de_suite += 1;
            }
        }
        assert_eq!(de_suite, 0, "le plafond tient à travers l'invalidation");

        // Une finie, une place s'ouvre — pour la nouvelle révision.
        render.finish_page_render(0, 0);
        assert!(render.begin_page_render(render.revision, 0));
    }

    /// Le zoom ne redemande **pas** une compilation.
    ///
    /// Changer la densité du rendu est un affaire de rasterisation : le
    /// document compilé est bon, seules les images se refont. C'est la
    /// promesse du jalon, et la voici éprouvée.
    /// Les comportements asynchrones (feuille de route, phase 6, point 6.4) :
    /// une compilation invalidée par une nouvelle modification relance
    /// l attente, au lieu de partir sur l état intermédiaire.
    #[test]
    fn une_modification_relance_l_attente() {
        use std::time::{Duration, Instant};
        let mut render = crate::app::state::RenderState::default();
        let t0 = Instant::now();
        assert!(
            !render.compile_due("ab", 0, t0),
            "la première frappe attend"
        );
        assert!(
            !render.compile_due("cd", 0, t0 + Duration::from_millis(10)),
            "la seconde modification relance l attente"
        );
        assert!(
            !render.compile_due("cd", 0, t0 + Duration::from_millis(100)),
            "le temps de la première ne compte plus"
        );
        assert!(
            render.compile_due(
                "cd",
                0,
                t0 + Duration::from_millis(crate::app::state::IDLE_DELAY_MS + 100)
            ),
            "la compilation part après le silence"
        );
    }

    #[test]
    fn le_zoom_ne_redemande_pas_de_compilation() {
        let mut render = RenderState {
            content: "le texte déjà compilé".to_string(),
            done: true,
            ..Default::default()
        };

        let start = Instant::now();

        for zoom in 0..10u64 {
            let now = start + Duration::from_millis(zoom * 300);

            // Le geste de zoom : les images sont périmées, la révision
            // change — et le document compilé reste le bon.
            render.invalidate_rasterization(300);

            assert!(
                !render.compile_due("le texte déjà compilé", 0, now),
                "zoom {zoom} : le zoom ne doit pas recompiler"
            );
        }
    }

    /// Sur un document long, cinquante pages visibles ne partent pas
    /// toutes ensemble : quatre départs, le reste attend sa place.
    #[test]
    fn un_document_long_ne_lance_pas_cinquante_rendus_d_un_coup() {
        let mut render = RenderState {
            visible: Some((0, 49)),
            ..Default::default()
        };
        render.stale.extend(0..500);

        let mut partis = 0;
        for index in 0..50 {
            if render.begin_page_render(0, index) {
                partis += 1;
            }
        }

        assert_eq!(partis, RenderState::SIMULTANEES);
        assert_eq!(render.rendering.len(), RenderState::SIMULTANEES);

        // Et ce qui attend reste périmé : la file n'est pas perdue.
        assert!(render.stale.contains(&12), "la page 12 attend toujours");
    }

    /// Une image d'une révision passée se jette, et sa page repart dans les
    /// périmées : c'est la révision courante qui la redessinera.
    #[test]
    fn une_image_perimee_ne_revient_pas() {
        let mut render = RenderState {
            revision: 3,
            ..Default::default()
        };

        assert!(render.image_perimee(2, 7), "la révision 2 est passée");
        assert!(render.stale.contains(&7), "la page repart en périmée");

        assert!(
            !render.image_perimee(3, 7),
            "la révision courante est bonne"
        );
    }

    /// Les rasterisations ne partent pas toutes d'un coup : il y a un
    /// nombre de places, et ce qui n'en a pas reste périmé.
    #[test]
    fn les_rasterisations_partent_qu_il_y_a_de_la_place() {
        let mut render = RenderState::default();

        // Le premier tour remplit les places.
        let prises: Vec<bool> = (0..RenderState::SIMULTANEES + 2)
            .map(|index| render.begin_page_render(0, index))
            .collect();

        assert_eq!(
            prises.iter().filter(|prise| **prise).count(),
            RenderState::SIMULTANEES,
            "seulement les places disponibles partent"
        );
        assert_eq!(render.rendering.len(), RenderState::SIMULTANEES);

        // Une finie, une place se libère — et la suivante part.
        render.finish_page_render(0, 0);
        assert!(render.begin_page_render(0, RenderState::SIMULTANEES + 2));
    }

    use super::*;

    /// Une image quelconque : seule son identité compte pour ces tests.
    fn tranche() -> Tranche {
        Tranche {
            image: handle(),
            ..Default::default()
        }
    }

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
        render.page_cache.insert(0, tranche());

        let first = render.page_cache.get(&0).expect("image en cache");
        let second = render.page_cache.get(&0).expect("image en cache");

        assert_eq!(
            first.image.id(),
            second.image.id(),
            "l'identifiant doit être stable"
        );

        // À l'inverse, une image recréée de toutes pièces porte un
        // identifiant neuf : c'est ce que l'ancien code faisait à chaque
        // frame, forçant le backend à retéléverser la texture.
        assert_ne!(
            first.image.id(),
            handle().id(),
            "une image neuve a une identité neuve"
        );
    }

    /// Le cache d'images est borné à ce qu'on voit, plus un voisinage.
    ///
    /// C'est la règle qui empêche l'accumulation : un document long parcouru
    /// jusqu'au bout gardait ses centaines de pages — deux mégaoctets chacune —
    /// et rien ne les relâchait jamais. Ce qui est hors du voisinage part, et
    /// sera redessiné au besoin, car son absence le fait marquer périmé.
    #[test]
    fn the_page_cache_is_bounded_to_what_is_visible() {
        let mut render = RenderState {
            visible: Some((20, 22)),
            ..Default::default()
        };

        for index in 0..50 {
            render.page_cache.insert(index, tranche());
            render.image_hashes.insert(index, index as u64);
        }

        render.bound_page_cache();

        let kept: std::collections::BTreeSet<usize> = render.page_cache.keys().copied().collect();
        let expected: std::collections::BTreeSet<usize> = (16..=26).collect();

        assert_eq!(kept, expected, "visibles 20..22 plus quatre de voisinage");

        // L'empreinte des pixels part **avec** l'image : une page dont on
        // garderait l'empreinte après avoir jeté l'image ne serait plus jamais
        // redessinée, puisque la nouvelle porterait la même empreinte et serait
        // tenue pour déjà affichée.
        let hashes: std::collections::BTreeSet<usize> =
            render.image_hashes.keys().copied().collect();

        assert_eq!(hashes, expected, "les empreintes suivent les images");
    }

    /// En haut du document, le voisinage ne descend pas sous zéro.
    #[test]
    fn the_page_cache_never_keeps_a_negative_range() {
        let render = RenderState {
            visible: Some((0, 1)),
            ..Default::default()
        };

        let keep = render.pages_to_keep();

        assert_eq!(*keep.start(), 0, "pas d'index négatif");
        assert_eq!(*keep.end(), 1 + RenderState::CACHE_MARGIN);
    }

    /// Une nouvelle échelle invalide les tâches et images de l'ancienne.
    #[test]
    fn raster_density_invalidation_starts_a_new_generation() {
        let mut render = RenderState {
            visible: Some((1, 2)),
            ..RenderState::default()
        };
        assert!(render.begin_page_render(0, 1));
        render.image_hashes.insert(1, 42);

        render.invalidate_rasterization(4);

        assert_eq!(render.revision, 1);

        // La tâche en vol garde sa place : le plafond tient à travers les
        // générations, et c'est sa fin qui le rouvrira.
        assert!(!render.rendering.is_empty());
        assert!(render.image_hashes.is_empty());
        assert_eq!(render.stale, (0..4).collect());
    }

    /// Une page n'est rasterisée qu'une fois par révision.
    ///
    /// Deux changements de viewport peuvent arriver pendant un rendu. Le
    /// second doit voir la première demande en vol et ne pas lancer une autre
    /// rasterisation de la même page — chacune alloue son tampon de pixels.
    #[test]
    fn a_page_render_is_reserved_once_per_revision() {
        let mut render = RenderState::default();

        assert!(render.begin_page_render(4, 7), "première demande acceptée");
        assert!(
            !render.begin_page_render(4, 7),
            "la même page ne doit pas être rasterisée deux fois"
        );
        assert!(
            render.begin_page_render(5, 7),
            "une nouvelle révision doit pouvoir lancer son propre rendu"
        );

        render.finish_page_render(4, 7);

        assert!(
            !render.begin_page_render(5, 7),
            "terminer un ancien rendu ne libère pas celui de la révision courante"
        );
        assert!(render.rendering.contains(&(5, 7)));
    }

    /// Le changement de document repart d'un rendu vierge, sans conserver
    /// d'image ni de travail en cours.
    #[test]
    fn reset_clears_every_trace_of_the_previous_document() {
        let mut render = RenderState {
            content: "ancien document".to_string(),
            done: true,
            page_cache: [(0, tranche())].into_iter().collect(),
            rendering: [(0, 1)].into_iter().collect(),
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
    /// Dix frappes par seconde pendant trois secondes : la frappe ne
    /// déclenche jamais le rendu — seuls l'arrêt de frappe de trois dixièmes de seconde,
    /// `Entrée` et le basculement de mode le font (bug-report-10, 6.2).
    #[test]
    fn typing_never_triggers_a_compile() {
        let mut render = RenderState::default();

        let start = Instant::now();

        for keystroke in 0..30u64 {
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
    /// tick suivant, sans attendre le repos.
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
            "un retour à la ligne doit montrer le résultat sans attendre le repos"
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

#[cfg(test)]
#[cfg(test)]
mod tests_du_rendu_partage {
    use super::*;

    /// Les pièces se complètent sans s'écraser : le partage garde
    /// toujours le même document (bug-report-10).
    #[test]
    fn les_pieces_du_rendu_se_partagent() {
        let root = std::env::temp_dir().join(format!("partage-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let outcome = crate::typst::render::compile(&root, "Bonjour.", &[]);
        let document = outcome.document.expect("le document compile");
        std::fs::remove_dir_all(&root).ok();

        let mut partage = RenduPartage::default();
        partage.completant(&RenduPartage {
            document: Some(document.clone()),
            lecteur: None,
        });
        assert!(partage.document.is_some(), "le document arrive");
        assert!(partage.lecteur.is_none(), "le lecteur attend encore");

        partage.completant(&RenduPartage {
            document: None,
            lecteur: None,
        });
        assert!(
            partage.document.is_some(),
            "rien ne doit faire perdre le document"
        );
        assert!(
            std::sync::Arc::ptr_eq(partage.document.as_ref().unwrap(), &document),
            "c'est bien le même document partagé"
        );
    }
}
