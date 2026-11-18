//! Le catalogue des commandes, et la palette qui les rend accessibles.
//!
//! Deux entrées, un seul moteur :
//!
//! - `..` dans le texte ouvre les **balises et snippets** insérables ;
//! - `/`, `Ctrl+K` ouvre toutes les **commandes**.
//!
//! Un `Message` n'est pas une commande : c'est ce que la commande produit.
//! Le catalogue porte le nom, l'icône, la description et l'alias ; la palette
//! les cherche, et l'exécuteur les transforme en message ou en insertion.

use iced::Task;
use iced_code_editor::Message as EditorMessage;

use crate::app::message::FormatAction;
use crate::app::{Message, State};
use crate::base::texte::{normaliser_recherche as normaliser, offset_sur_caractere};

/// Le mode de la palette : ce qu'elle cherche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModePalette {
    /// Les balises et les snippets insérables.
    Balises,
    /// Toutes les commandes.
    Systeme,
    /// Les éléments linkables du document (`[[`).
    Liens,
}

/// L'icône d'une commande. Le rendu la traduit en icône Lucide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icone {
    Fichier,
    Dossier,
    Bibliotheque,
    Disquette,
    Telecharger,
    Mode,
    Focus,
    Recherche,
    Aide,
    Note,
    Image,
    Citation,
    Tableau,
    Graphique,
    Encadre,
    Glossaire,
    Bible,
    Etiquette,
    Lien,
    Calendrier,
    Palette,
    EtiquetteMeta,
    Fragment,
    Reglages,
    Puzzle,
    Plus,
    Panneau,
    Numeros,
    Texte,
    Gras,
    Italique,
    Souligne,
    Barre,
    Code,
    Titre,
    Teinte,
    Importer,
    Theme,
    Norme,
    Plugin,
}

/// La famille d'une commande — ce qui donne un premier classement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Categorie {
    Fichier,
    Projet,
    Insertion,
    Texte,
    Affichage,
    Recherche,
    Aide,
    Reglage,
}

/// L'identité d'une commande système.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandeId {
    Enregistrer,
    Ouvrir,
    NouveauChapitre,
    NouveauProjet,
    ExporterPdf,
    BasculerRendu,
    NumerosDeLigne,
    Focus,
    Bibliotheque,
    Normes,
    Plugins,
    Recherche,
    Aide,
    Citation,
    GlossaireFenetre,
    BasculerPanneau,
    ThemeSuivant,
    ModeTypst,
    ExporterMarkdown,
    ExporterDocx,
    ExporterProjetTw,
    ExporterProjetPortable,
    OuvrirDossierDonnees,
    SauverSous,
    // Mise en forme du texte.
    Gras,
    Italique,
    Souligne,
    Barre,
    GrasItalique,
    Code,
    Formule,
    Titre1,
    Titre2,
    Titre3,
    Titre4,
    Titre5,
    Titre6,
    ListePuce,
    ListeNumero,
    CitationBloc,
    Note,
    BlocTypst,
    ChoisirCouleur,
    CouleurOrange,
    CouleurVert,
    CouleurDerniere,
    CouleurRouge,
    CouleurBleu,
    CouleurViolet,
    CouleurGris,
    CouleurNoir,
    // Déplacements dans le texte.
    BlocMonter,
    BlocDescendre,
    TitreSuivant,
    TitrePrecedent,
    ListeIndenter,
    ListeDesindenter,
    ListeMonter,
    ListeDescendre,
    // Import et export de fichiers.
    NouveauFichier,
    ImporterTw,
    ImporterSauvegarde,
    ImporterNormeFichier,
    ExporterNormes,
    ImporterTheme,
    EditeurTheme,
    DossierThemes,
    RafraichirThemes,
    ImporterBibliographie,
    ImporterVersionBiblique,
    ImporterStyleCsl,
    ImporterGraphique,
    BasculerApercu,
    SuiviApercu,
    // Plugins.
    PluginBibliographie,
    PluginGlossaire,
    PluginGraphiques,
    PluginCallouts,
    PluginReferencesBibliques,
}

/// Une balise insérable. Les balises qui ouvrent un panneau gardent leur
/// panneau : c'est lui qui porte les champs, pas une copie du formulaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagId {
    Couleur,
    Date,
    Glossaire,
    Bible,
    Note,
    Figure,
    Image,
    Lien,
    Renvoi,
    Etiquette,
    Tableau,
    Graphique,
    Encadre,
    Extension,
    Sommaire,
    Bibliographie,
    GlossaireSection,
    IndexBiblique,
    Garde,
    Meta,
}

/// Un fragment de texte prêt à poser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetId {
    These,
    Chapitre,
    Section,
    Citation,
    Tableau,
}

/// L'effet d'une commande.
#[derive(Clone, Copy)]
pub enum Action {
    Message(fn() -> Message),
}

/// Une commande du catalogue.
pub struct Commande {
    pub id: CommandeId,
    pub titre: &'static str,
    pub description: &'static str,
    pub alias: &'static [&'static str],
    pub categorie: Categorie,
    pub icone: Icone,
    pub raccourci: Option<&'static str>,
    pub action: Action,
}

/// Une balise du catalogue.
pub struct Balise {
    pub id: TagId,
    pub titre: &'static str,
    pub description: &'static str,
    pub alias: &'static [&'static str],
    pub icone: Icone,
}

/// Un snippet du catalogue.
pub struct Snippet {
    pub id: SnippetId,
    pub titre: &'static str,
    pub description: &'static str,
    pub alias: &'static [&'static str],
    pub icone: Icone,
}

/// Le catalogue des commandes système.
pub const COMMANDES: &[Commande] = &[
    Commande {
        id: CommandeId::Enregistrer,
        titre: "Enregistrer",
        description: "Écrire le document sur le disque",
        alias: &["save", "sauver"],
        categorie: Categorie::Fichier,
        icone: Icone::Disquette,
        raccourci: Some("Ctrl+S"),
        action: Action::Message(|| Message::SaveFile),
    },
    Commande {
        id: CommandeId::Ouvrir,
        titre: "Importer un document .typ",
        description: "Ouvrir un fichier Typst dans le projet",
        alias: &["open", "ouvrir", "importer typ", "typ"],
        categorie: Categorie::Fichier,
        icone: Icone::Dossier,
        raccourci: Some("Ctrl+O"),
        action: Action::Message(|| Message::OpenFile),
    },
    Commande {
        id: CommandeId::NouveauChapitre,
        titre: "Nouveau chapitre",
        description: "Créer un chapitre dans le projet",
        alias: &["chapitre", "chapter"],
        categorie: Categorie::Projet,
        icone: Icone::Plus,
        raccourci: None,
        action: Action::Message(|| Message::NewChapter),
    },
    Commande {
        id: CommandeId::NouveauProjet,
        titre: "Nouveau projet",
        description: "Créer un projet d'écriture",
        alias: &["projet", "project"],
        categorie: Categorie::Projet,
        icone: Icone::Bibliotheque,
        raccourci: None,
        action: Action::Message(|| Message::NewProject),
    },
    Commande {
        id: CommandeId::ExporterPdf,
        titre: "Exporter en PDF",
        description: "Composer et enregistrer le document",
        alias: &["export", "pdf"],
        categorie: Categorie::Fichier,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::ExportPdf),
    },
    Commande {
        id: CommandeId::BasculerRendu,
        titre: "Basculer édition / rendu",
        description: "Voir le document composé ou revenir au texte",
        alias: &["rendu", "apercu", "preview"],
        categorie: Categorie::Affichage,
        icone: Icone::Mode,
        raccourci: Some("Ctrl+E"),
        action: Action::Message(|| Message::ModeBasculer),
    },
    Commande {
        id: CommandeId::NumerosDeLigne,
        titre: "Afficher les numéros de ligne",
        description: "Montrer ou masquer les numéros dans l'éditeur",
        alias: &["numeros", "ligne", "line numbers"],
        categorie: Categorie::Affichage,
        icone: Icone::Numeros,
        raccourci: None,
        action: Action::Message(|| Message::NumerosDeLigneBascule),
    },
    Commande {
        id: CommandeId::Focus,
        titre: "Mode focus",
        description: "Masquer les panneaux et écrire seul",
        alias: &["focus", "plein écran"],
        categorie: Categorie::Affichage,
        icone: Icone::Focus,
        raccourci: Some("F11"),
        action: Action::Message(|| Message::ToggleFocusMode),
    },
    Commande {
        id: CommandeId::Bibliotheque,
        titre: "Bibliothèque des projets",
        description: "Ouvrir la liste des projets et des étagères",
        alias: &["projets", "library"],
        categorie: Categorie::Projet,
        icone: Icone::Bibliotheque,
        raccourci: None,
        action: Action::Message(|| Message::ProjetsVue),
    },
    Commande {
        id: CommandeId::Normes,
        titre: "Normes du document",
        description: "Papier, marges, titres, métadonnées",
        alias: &["normes", "mise en page", "metadata"],
        categorie: Categorie::Reglage,
        icone: Icone::Reglages,
        raccourci: Some("Ctrl+P"),
        action: Action::Message(|| Message::MiseEnPageOuvrir),
    },
    Commande {
        id: CommandeId::Plugins,
        titre: "Paramètres des plugins",
        description: "Bibliographie, biblique, glossaire, graphiques",
        alias: &["plugins", "extensions"],
        categorie: Categorie::Reglage,
        icone: Icone::Puzzle,
        raccourci: None,
        action: Action::Message(|| Message::PluginsOuvrir),
    },
    Commande {
        id: CommandeId::Recherche,
        titre: "Rechercher dans le document",
        description: "Trouver un mot ou une expression",
        alias: &["chercher", "find", "search"],
        categorie: Categorie::Recherche,
        icone: Icone::Recherche,
        raccourci: Some("Ctrl+F"),
        action: Action::Message(|| Message::RechercheOuvrir),
    },
    Commande {
        id: CommandeId::Aide,
        titre: "Aide et raccourcis",
        description: "Voir la carte des gestes",
        alias: &["aide", "help", "raccourcis"],
        categorie: Categorie::Aide,
        icone: Icone::Aide,
        raccourci: Some("F1"),
        action: Action::Message(|| Message::AideBasculer),
    },
    Commande {
        id: CommandeId::Citation,
        titre: "Insérer une citation",
        description: "Composer une référence bibliographique",
        alias: &["citation", "référence", "zotero"],
        categorie: Categorie::Insertion,
        icone: Icone::Citation,
        raccourci: None,
        action: Action::Message(|| Message::CitationOuvrir),
    },
    Commande {
        id: CommandeId::GlossaireFenetre,
        titre: "Ouvrir le glossaire",
        description: "Voir et gérer les termes du projet",
        alias: &["glossaire", "glossary"],
        categorie: Categorie::Projet,
        icone: Icone::Glossaire,
        raccourci: None,
        action: Action::Message(|| Message::GlossaireFenetreOuvrir),
    },
    Commande {
        id: CommandeId::BasculerPanneau,
        titre: "Afficher ou masquer le panneau",
        description: "Le panneau de travail à gauche",
        alias: &["panneau", "sidebar", "latéral"],
        categorie: Categorie::Affichage,
        icone: Icone::Panneau,
        raccourci: None,
        action: Action::Message(|| Message::ToggleSidebar),
    },
    Commande {
        id: CommandeId::ThemeSuivant,
        titre: "Changer de thème",
        description: "Système, clair, sombre, puis système",
        alias: &["thème", "apparence", "theme"],
        categorie: Categorie::Affichage,
        icone: Icone::Mode,
        raccourci: None,
        action: Action::Message(|| Message::CycleTheme),
    },
    Commande {
        id: CommandeId::ModeTypst,
        titre: "Voir le Typst généré",
        description: "Le code que l'application écrit vraiment",
        alias: &["typst", "source", "code"],
        categorie: Categorie::Affichage,
        icone: Icone::Fichier,
        raccourci: Some("Ctrl+Maj+E"),
        action: Action::Message(|| Message::ModeTypst),
    },
    Commande {
        id: CommandeId::ExporterMarkdown,
        titre: "Exporter en Markdown",
        description: "Un document Markdown à partager",
        alias: &["markdown", "md", "export"],
        categorie: Categorie::Fichier,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::ExportMarkdown),
    },
    Commande {
        id: CommandeId::ExporterDocx,
        titre: "Exporter en DOCX",
        description: "Un document Word, notes comprises",
        alias: &["docx", "word", "export"],
        categorie: Categorie::Fichier,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::ExportDocx),
    },
    Commande {
        id: CommandeId::ExporterProjetTw,
        titre: "Exporter le projet en .tw unique",
        description: "Un seul fichier transportable",
        alias: &["tw", "projet", "unique"],
        categorie: Categorie::Projet,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::ProjetExportTw),
    },
    Commande {
        id: CommandeId::ExporterProjetPortable,
        titre: "Exporter le projet en archive",
        description: "Tout le projet dans une archive portable",
        alias: &["archive", "portable", "projet"],
        categorie: Categorie::Projet,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::ProjetExportPortable),
    },
    Commande {
        id: CommandeId::OuvrirDossierDonnees,
        titre: "Ouvrir le dossier de données",
        description: "Les projets et leurs fichiers",
        alias: &["dossier", "données", "data"],
        categorie: Categorie::Reglage,
        icone: Icone::Dossier,
        raccourci: None,
        action: Action::Message(|| Message::OuvrirDossierDonnees),
    },
    Commande {
        id: CommandeId::SauverSous,
        titre: "Enregistrer sous…",
        description: "Écrire le document à un autre chemin",
        alias: &["sauver sous", "save as", "copie"],
        categorie: Categorie::Fichier,
        icone: Icone::Disquette,
        raccourci: None,
        action: Action::Message(|| Message::SaveAs),
    },
    Commande {
        id: CommandeId::Gras,
        titre: "Gras",
        description: "Mettre la sélection en gras",
        alias: &["gras", "bold", "b"],
        categorie: Categorie::Texte,
        icone: Icone::Gras,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Gras)),
    },
    Commande {
        id: CommandeId::Italique,
        titre: "Italique",
        description: "Mettre la sélection en italique",
        alias: &["italique", "italic", "i"],
        categorie: Categorie::Texte,
        icone: Icone::Italique,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Italique)),
    },
    Commande {
        id: CommandeId::Souligne,
        titre: "Souligné",
        description: "Souligner la sélection",
        alias: &["souligné", "souligne", "underline"],
        categorie: Categorie::Texte,
        icone: Icone::Souligne,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Souligne)),
    },
    Commande {
        id: CommandeId::Barre,
        titre: "Barré",
        description: "Barrer la sélection",
        alias: &["barré", "barre", "strike", "strikethrough"],
        categorie: Categorie::Texte,
        icone: Icone::Barre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Barre)),
    },
    Commande {
        id: CommandeId::GrasItalique,
        titre: "Gras et italique",
        description: "Mettre la sélection en gras italique",
        alias: &["gras italique", "bold italic", "bi"],
        categorie: Categorie::Texte,
        icone: Icone::Gras,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::GrasItalique)),
    },
    Commande {
        id: CommandeId::Code,
        titre: "Code en ligne",
        description: "Entourer la sélection de code",
        alias: &["code", "inline code", "monospace"],
        categorie: Categorie::Texte,
        icone: Icone::Code,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Code)),
    },
    Commande {
        id: CommandeId::Formule,
        titre: "Formule en ligne",
        description: "Entourer la sélection d'une formule",
        alias: &["formule", "math", "équation", "equation"],
        categorie: Categorie::Texte,
        icone: Icone::Code,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Formule)),
    },
    Commande {
        id: CommandeId::Titre1,
        titre: "Titre de niveau 1",
        description: "Transformer le bloc en titre de chapitre",
        alias: &["titre 1", "h1", "chapitre"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(1))),
    },
    Commande {
        id: CommandeId::Titre2,
        titre: "Titre de niveau 2",
        description: "Transformer le bloc en sous-titre",
        alias: &["titre 2", "h2", "section"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(2))),
    },
    Commande {
        id: CommandeId::Titre3,
        titre: "Titre de niveau 3",
        description: "Transformer le bloc en titre de niveau 3",
        alias: &["titre 3", "h3"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(3))),
    },
    Commande {
        id: CommandeId::Titre4,
        titre: "Titre de niveau 4",
        description: "Transformer le bloc en titre de niveau 4",
        alias: &["titre 4", "h4"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(4))),
    },
    Commande {
        id: CommandeId::Titre5,
        titre: "Titre de niveau 5",
        description: "Transformer le bloc en titre de niveau 5",
        alias: &["titre 5", "h5"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(5))),
    },
    Commande {
        id: CommandeId::Titre6,
        titre: "Titre de niveau 6",
        description: "Transformer le bloc en titre de niveau 6",
        alias: &["titre 6", "h6"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Titre(6))),
    },
    Commande {
        id: CommandeId::ListePuce,
        titre: "Liste à puces",
        description: "Basculer le bloc en liste à puces",
        alias: &["puce", "liste", "bullet"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::ListePuce)),
    },
    Commande {
        id: CommandeId::ListeNumero,
        titre: "Liste numérotée",
        description: "Basculer le bloc en liste numérotée",
        alias: &["numérotée", "liste", "numbered", "ol"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::ListeNumero)),
    },
    Commande {
        id: CommandeId::CitationBloc,
        titre: "Citation de bloc",
        description: "Basculer le bloc en citation",
        alias: &["citation", "citation de bloc", "quote", "blockquote"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::CitationBloc)),
    },
    Commande {
        id: CommandeId::Note,
        titre: "Note de bas de page",
        description: "Mettre la sélection en note",
        alias: &["note", "footnote"],
        categorie: Categorie::Texte,
        icone: Icone::Note,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Note)),
    },
    Commande {
        id: CommandeId::BlocTypst,
        titre: "Bloc Typst brut",
        description: "Entourer la sélection de Typst",
        alias: &["typst", "brut", "raw"],
        categorie: Categorie::Texte,
        icone: Icone::Fragment,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::BlocTypst)),
    },
    Commande {
        id: CommandeId::ChoisirCouleur,
        titre: "Choisir une couleur…",
        description: "Ouvrir la roue des couleurs pour la sélection",
        alias: &["couleur", "roue", "picker", "teinte personnalisée"],
        categorie: Categorie::Texte,
        icone: Icone::Palette,
        raccourci: None,
        action: Action::Message(|| Message::PickerCouleurOuvrir),
    },
    Commande {
        id: CommandeId::CouleurOrange,
        titre: "Teinter en orange",
        description: "Appliquer la couleur orange à la sélection",
        alias: &["orange", "teinte orange", "couleur orange"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("orange".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurVert,
        titre: "Teinter en vert",
        description: "Appliquer la couleur verte à la sélection",
        alias: &["vert", "green", "teinte verte", "couleur verte"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("green".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurDerniere,
        titre: "Dernière couleur choisie",
        description: "Teinter avec la dernière couleur utilisée",
        alias: &["dernière couleur", "last color", "couleur courante"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::FormatDerniereCouleur),
    },
    Commande {
        id: CommandeId::CouleurRouge,
        titre: "Teinter en rouge",
        description: "Appliquer la couleur rouge à la sélection",
        alias: &["rouge", "red", "teinte rouge"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("red".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurBleu,
        titre: "Teinter en bleu",
        description: "Appliquer la couleur bleue à la sélection",
        alias: &["bleu", "blue", "teinte bleue"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("blue".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurViolet,
        titre: "Teinter en violet",
        description: "Appliquer la couleur violette à la sélection",
        alias: &["violet", "purple", "teinte violette"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("purple".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurGris,
        titre: "Teinter en gris",
        description: "Appliquer la couleur grise à la sélection",
        alias: &["gris", "gray", "grey", "teinte grise"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("gray".to_string()))),
    },
    Commande {
        id: CommandeId::CouleurNoir,
        titre: "Teinter en noir",
        description: "Appliquer la couleur noire à la sélection",
        alias: &["noir", "black", "teinte noire"],
        categorie: Categorie::Texte,
        icone: Icone::Teinte,
        raccourci: None,
        action: Action::Message(|| Message::Format(FormatAction::Couleur("black".to_string()))),
    },
    Commande {
        id: CommandeId::BlocMonter,
        titre: "Monter le bloc",
        description: "Déplacer le bloc courant vers le haut",
        alias: &["monter", "bloc", "haut", "move up"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::BlocMonter),
    },
    Commande {
        id: CommandeId::BlocDescendre,
        titre: "Descendre le bloc",
        description: "Déplacer le bloc courant vers le bas",
        alias: &["descendre", "bloc", "bas", "move down"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::BlocDescendre),
    },
    Commande {
        id: CommandeId::TitreSuivant,
        titre: "Titre suivant",
        description: "Aller au titre suivant",
        alias: &["titre suivant", "heading next"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::TitreSuivant),
    },
    Commande {
        id: CommandeId::TitrePrecedent,
        titre: "Titre précédent",
        description: "Aller au titre précédent",
        alias: &["titre précédent", "heading previous"],
        categorie: Categorie::Texte,
        icone: Icone::Titre,
        raccourci: None,
        action: Action::Message(|| Message::TitrePrecedent),
    },
    Commande {
        id: CommandeId::ListeIndenter,
        titre: "Indenter la ligne",
        description: "Enfoncer la ligne de liste courante",
        alias: &["indenter", "liste", "tab"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::ListeIndenter),
    },
    Commande {
        id: CommandeId::ListeDesindenter,
        titre: "Désindenter la ligne",
        description: "Remonter la ligne de liste courante",
        alias: &["désindenter", "desindenter", "liste", "shift tab"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::ListeDesindenter),
    },
    Commande {
        id: CommandeId::ListeMonter,
        titre: "Monter la ligne de liste",
        description: "Échanger avec la ligne de liste précédente",
        alias: &["liste monter", "ligne haut"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::ListeMonter),
    },
    Commande {
        id: CommandeId::ListeDescendre,
        titre: "Descendre la ligne de liste",
        description: "Échanger avec la ligne de liste suivante",
        alias: &["liste descendre", "ligne bas"],
        categorie: Categorie::Texte,
        icone: Icone::Texte,
        raccourci: None,
        action: Action::Message(|| Message::ListeDescendre),
    },
    Commande {
        id: CommandeId::NouveauFichier,
        titre: "Nouveau document",
        description: "Ouvrir un nouveau document d'écriture",
        alias: &["nouveau", "document", "new file"],
        categorie: Categorie::Fichier,
        icone: Icone::Fichier,
        raccourci: Some("Ctrl+N"),
        action: Action::Message(|| Message::NewFile),
    },
    Commande {
        id: CommandeId::ImporterTw,
        titre: "Importer un .tw unique",
        description: "Reconstruire un projet depuis un fichier .tw",
        alias: &["importer tw", "import tw"],
        categorie: Categorie::Projet,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::ProjetImportTw),
    },
    Commande {
        id: CommandeId::ImporterSauvegarde,
        titre: "Importer une sauvegarde",
        description: "Restaurer un projet depuis une archive",
        alias: &["importer sauvegarde", "archive", "import backup"],
        categorie: Categorie::Projet,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::ProjetImportSauvegarde),
    },
    Commande {
        id: CommandeId::ImporterNormeFichier,
        titre: "Importer un fichier de normes",
        description: "Ajouter une norme TOML à la bibliothèque",
        alias: &["importer norme", "norme toml", "import norme"],
        categorie: Categorie::Reglage,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::NormesFichierImporter),
    },
    Commande {
        id: CommandeId::ExporterNormes,
        titre: "Exporter les normes du document",
        description: "Écrire les normes courantes dans un fichier TOML",
        alias: &["exporter normes", "normes toml", "export normes"],
        categorie: Categorie::Reglage,
        icone: Icone::Telecharger,
        raccourci: None,
        action: Action::Message(|| Message::MiseEnPageExporter),
    },
    Commande {
        id: CommandeId::ImporterTheme,
        titre: "Charger un thème",
        description: "Importer un thème depuis un fichier TOML",
        alias: &[
            "charger thème",
            "importer thème",
            "load theme",
            "theme toml",
        ],
        categorie: Categorie::Reglage,
        icone: Icone::Theme,
        raccourci: None,
        action: Action::Message(|| Message::LoadTheme),
    },
    Commande {
        id: CommandeId::EditeurTheme,
        titre: "Customiser un thème",
        description: "Ouvrir l'éditeur du thème actif",
        alias: &["éditeur thème", "customiser thème", "theme editor"],
        categorie: Categorie::Reglage,
        icone: Icone::Theme,
        raccourci: None,
        action: Action::Message(|| Message::ThemeEditeurOuvrir),
    },
    Commande {
        id: CommandeId::DossierThemes,
        titre: "Ouvrir le dossier des thèmes",
        description: "Voir les fichiers de thèmes sur le disque",
        alias: &["dossier thèmes", "themes folder"],
        categorie: Categorie::Reglage,
        icone: Icone::Dossier,
        raccourci: None,
        action: Action::Message(|| Message::OpenThemesFolder),
    },
    Commande {
        id: CommandeId::RafraichirThemes,
        titre: "Recharger les thèmes",
        description: "Relire les thèmes du dossier",
        alias: &["recharger thèmes", "refresh themes"],
        categorie: Categorie::Reglage,
        icone: Icone::Theme,
        raccourci: None,
        action: Action::Message(|| Message::RefreshThemes),
    },
    Commande {
        id: CommandeId::ImporterBibliographie,
        titre: "Importer une bibliographie",
        description: "Ajouter un fichier BibLaTeX au projet",
        alias: &["importer bibliographie", "bib", "biblatex"],
        categorie: Categorie::Fichier,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::BibliographieImporter),
    },
    Commande {
        id: CommandeId::ImporterVersionBiblique,
        titre: "Importer une version biblique",
        description: "Ajouter une version JSON au registre",
        alias: &["importer bible", "version biblique", "bible json"],
        categorie: Categorie::Fichier,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::BibleVersionImporter),
    },
    Commande {
        id: CommandeId::ImporterStyleCsl,
        titre: "Importer un style CSL",
        description: "Ajouter un style de citation au projet",
        alias: &["importer csl", "style citation", "csl"],
        categorie: Categorie::Fichier,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::StyleCslImporter),
    },
    Commande {
        id: CommandeId::ImporterGraphique,
        titre: "Importer des données de graphique",
        description: "Charger un CSV dans le panneau graphique",
        alias: &["importer csv", "graphique csv", "chart csv"],
        categorie: Categorie::Fichier,
        icone: Icone::Importer,
        raccourci: None,
        action: Action::Message(|| Message::ChartImporter),
    },
    Commande {
        id: CommandeId::BasculerApercu,
        titre: "Afficher ou masquer l'aperçu",
        description: "Le panneau de rendu à côté du texte",
        alias: &["aperçu", "preview", "rendu"],
        categorie: Categorie::Affichage,
        icone: Icone::Mode,
        raccourci: None,
        action: Action::Message(|| Message::TogglePreview),
    },
    Commande {
        id: CommandeId::SuiviApercu,
        titre: "Suivre le curseur dans l'aperçu",
        description: "Faire défiler le rendu avec l'écriture",
        alias: &["suivi aperçu", "sync preview"],
        categorie: Categorie::Affichage,
        icone: Icone::Mode,
        raccourci: None,
        action: Action::Message(|| Message::TogglePreviewSync),
    },
    Commande {
        id: CommandeId::PluginBibliographie,
        titre: "Activer ou désactiver la bibliographie",
        description: "Brancher ou débrancher le plugin bibliographie",
        alias: &["plugin bibliographie", "bibliographie"],
        categorie: Categorie::Reglage,
        icone: Icone::Plugin,
        raccourci: None,
        action: Action::Message(|| Message::BasculerFonctionnalite("bibliographie".to_string())),
    },
    Commande {
        id: CommandeId::PluginGlossaire,
        titre: "Activer ou désactiver le glossaire",
        description: "Brancher ou débrancher le plugin glossaire",
        alias: &["plugin glossaire", "glossaire"],
        categorie: Categorie::Reglage,
        icone: Icone::Plugin,
        raccourci: None,
        action: Action::Message(|| Message::BasculerFonctionnalite("glossaire".to_string())),
    },
    Commande {
        id: CommandeId::PluginGraphiques,
        titre: "Activer ou désactiver les graphiques",
        description: "Brancher ou débrancher le plugin graphiques",
        alias: &["plugin graphiques", "graphiques", "charts"],
        categorie: Categorie::Reglage,
        icone: Icone::Plugin,
        raccourci: None,
        action: Action::Message(|| Message::BasculerFonctionnalite("graphiques".to_string())),
    },
    Commande {
        id: CommandeId::PluginCallouts,
        titre: "Activer ou désactiver les callouts",
        description: "Brancher ou débrancher le plugin des encadrés",
        alias: &["plugin callouts", "encadrés", "callouts"],
        categorie: Categorie::Reglage,
        icone: Icone::Plugin,
        raccourci: None,
        action: Action::Message(|| Message::BasculerFonctionnalite("callouts".to_string())),
    },
    Commande {
        id: CommandeId::PluginReferencesBibliques,
        titre: "Activer ou désactiver les références bibliques",
        description: "Brancher ou débrancher le plugin biblique",
        alias: &["plugin biblique", "références bibliques", "bible"],
        categorie: Categorie::Reglage,
        icone: Icone::Plugin,
        raccourci: None,
        action: Action::Message(|| {
            Message::BasculerFonctionnalite("references_bibliques".to_string())
        }),
    },
];

/// Le catalogue des balises.
pub const BALISES: &[Balise] = &[
    Balise {
        id: TagId::Couleur,
        titre: "Couleur",
        description: "Teinter un passage",
        alias: &["c", "couleur", "color", "teinte"],
        icone: Icone::Palette,
    },
    Balise {
        id: TagId::Date,
        titre: "Date",
        description: "Écrire une date selon les normes",
        alias: &["d", "date", "calendrier"],
        icone: Icone::Calendrier,
    },
    Balise {
        id: TagId::Glossaire,
        titre: "Terme du glossaire",
        description: "Appeler un terme défini dans le projet",
        alias: &["g", "glossaire", "glo", "terme"],
        icone: Icone::Glossaire,
    },
    Balise {
        id: TagId::Bible,
        titre: "Référence biblique",
        description: "Composer une référence ou un groupe",
        alias: &["b", "bible", "verset", "référence"],
        icone: Icone::Bible,
    },
    Balise {
        id: TagId::Garde,
        titre: "Page de garde",
        description: "Composer des pages avant le sommaire, sans numéro",
        alias: &["garde", "page de garde", "front", "front matter"],
        icone: Icone::Fichier,
    },
    Balise {
        id: TagId::Meta,
        titre: "Métadonnée",
        description: "Appeler une valeur de metadonnees.toml",
        alias: &["métadonnée", "metadata", "meta", "m"],
        icone: Icone::EtiquetteMeta,
    },
    Balise {
        id: TagId::Note,
        titre: "Note de bas de page",
        description: "Écrire une note au curseur",
        alias: &["note", "footnote"],
        icone: Icone::Note,
    },
    Balise {
        id: TagId::Figure,
        titre: "Figure",
        description: "Insérer une image légendée",
        alias: &["figure", "image", "légende"],
        icone: Icone::Image,
    },
    Balise {
        id: TagId::Image,
        titre: "Image",
        description: "Choisir une image du projet",
        alias: &["image", "photo"],
        icone: Icone::Image,
    },
    Balise {
        id: TagId::Lien,
        titre: "Lien",
        description: "Écrire un lien avec son texte",
        alias: &["lien", "link", "url"],
        icone: Icone::Lien,
    },
    Balise {
        id: TagId::Renvoi,
        titre: "Renvoi",
        description: "Appeler une étiquette du document",
        alias: &["renvoi", "référence croisée", "ref"],
        icone: Icone::Etiquette,
    },
    Balise {
        id: TagId::Etiquette,
        titre: "Étiquette",
        description: "Poser une étiquette sur une figure",
        alias: &["étiquette", "label", "ancre"],
        icone: Icone::Etiquette,
    },
    Balise {
        id: TagId::Tableau,
        titre: "Tableau",
        description: "Choisir la taille et insérer la grille",
        alias: &["t", "tableau", "table"],
        icone: Icone::Tableau,
    },
    Balise {
        id: TagId::Graphique,
        titre: "Graphique",
        description: "Composer un graphique avec ses données",
        alias: &["gr", "ch", "graphique", "chart", "diagramme"],
        icone: Icone::Graphique,
    },
    Balise {
        id: TagId::Encadre,
        titre: "Encadré",
        description: "Habiller une citation d'un encadré",
        alias: &["e", "encadré", "callout", "encadre"],
        icone: Icone::Encadre,
    },
    Balise {
        id: TagId::Extension,
        titre: "Extension",
        description: "Insérer un fragment Typst paramétré",
        alias: &["extension", "typst"],
        icone: Icone::Fragment,
    },
    Balise {
        id: TagId::Sommaire,
        titre: "Sommaire",
        description: "Poser la table des matières",
        alias: &["s", "sommaire", "toc", "matières"],
        icone: Icone::Tableau,
    },
    Balise {
        id: TagId::Bibliographie,
        titre: "Bibliographie",
        description: "Poser la section bibliographie",
        alias: &["bib", "bibliographie", "biblio"],
        icone: Icone::Citation,
    },
    Balise {
        id: TagId::GlossaireSection,
        titre: "Section glossaire",
        description: "Poser la liste finale du glossaire",
        alias: &["gs", "glossaire", "section"],
        icone: Icone::Glossaire,
    },
    Balise {
        id: TagId::IndexBiblique,
        titre: "Index biblique",
        description: "Poser l'index des citations bibliques",
        alias: &["idx", "index", "bible", "citations"],
        icone: Icone::Bible,
    },
];

/// Le catalogue des snippets.
pub const SNIPPETS: &[Snippet] = &[
    Snippet {
        id: SnippetId::These,
        titre: "Structure de thèse",
        description: "Titre, introduction et cadre de travail",
        alias: &["thèse", "these", "structure"],
        icone: Icone::Fragment,
    },
    Snippet {
        id: SnippetId::Chapitre,
        titre: "Nouveau chapitre",
        description: "Titre de niveau 1 et espace de rédaction",
        alias: &["chapitre", "chapter"],
        icone: Icone::Fichier,
    },
    Snippet {
        id: SnippetId::Section,
        titre: "Nouvelle section",
        description: "Titre de niveau 2 et espace de rédaction",
        alias: &["section", "sous-titre"],
        icone: Icone::Fichier,
    },
    Snippet {
        id: SnippetId::Citation,
        titre: "Bloc de citation",
        description: "Une citation à compléter",
        alias: &["citation", "quote"],
        icone: Icone::Citation,
    },
    Snippet {
        id: SnippetId::Tableau,
        titre: "Tableau simple",
        description: "Grille de deux colonnes à compléter",
        alias: &["tableau", "grille"],
        icone: Icone::Tableau,
    },
];

/// Un résultat de la palette : une commande, une balise, un snippet ou une
/// métadonnée du projet.
#[derive(Debug, Clone)]
pub struct Item {
    pub titre: String,
    pub description: String,
    pub alias: String,
    pub categorie: Categorie,
    pub icone: Icone,
    pub raccourci: Option<&'static str>,
    pub action: ItemAction,
}

#[derive(Debug, Clone)]
pub enum ItemAction {
    Commande(CommandeId),
    Tag(TagId),
    Snippet(SnippetId),
    Meta(String),
    LienInterne {
        reference: String,
    },
    /// Appliquer une norme de la bibliothèque au document.
    Norme(String),
    /// Exporter une norme de la bibliothèque dans un fichier TOML.
    ExporterNorme(String),
    /// Choisir un thème par son nom.
    Theme(String),
}

/// L'état de la palette ouverte.
#[derive(Debug, Clone)]
pub struct Palette {
    pub mode: ModePalette,
    pub requete: String,
    pub selection: usize,
    /// La plage du déclencheur (`..` ou `/`) à retirer à l'insertion.
    pub remplacement: Option<(usize, usize)>,
    /// Les métadonnées visibles au moment de l'ouverture : la palette ne
    /// relit pas `metadonnees.toml` à chaque image.
    pub metadonnees: Vec<(String, String)>,
    /// Les normes de la bibliothèque, lues à l'ouverture : les commandes de
    /// chargement ne fouillent pas le disque à chaque frappe.
    pub normes: Vec<String>,
}

/// Un champ du formulaire d'une balise simple.
#[derive(Debug, Clone)]
pub struct ChampBalise {
    pub cle: &'static str,
    pub libelle: &'static str,
    pub valeur: String,
}

/// Le formulaire d'une balise simple : les champs suffisent, le panneau
/// spécialisé n'est pas nécessaire.
#[derive(Debug, Clone)]
pub struct BaliseDialogue {
    pub tag: TagId,
    pub champs: Vec<ChampBalise>,
}

impl BaliseDialogue {
    /// Le formulaire d'une balise, ou `None` si elle a déjà son panneau.
    pub fn depuis(tag: TagId) -> Option<Self> {
        let champs = match tag {
            TagId::Couleur => vec![
                champ("couleur", "Couleur (red, #e74c3c…)", "red"),
                champ("contenu", "Texte à teinter", ""),
            ],
            TagId::Date => vec![champ("spec", "Date (1789.07.14, XIXs…)", "")],
            TagId::Glossaire => vec![champ("cle", "Clé du terme", "")],
            TagId::Meta => vec![champ("cle", "Clé de la métadonnée", "auteur")],
            TagId::Lien => vec![
                champ("texte", "Texte du lien", ""),
                champ("cible", "Adresse (https://…)", ""),
            ],
            _ => return None,
        };

        Some(Self { tag, champs })
    }

    pub fn id_champ(index: usize) -> iced::widget::Id {
        iced::widget::Id::from(format!("balise-champ-{index}"))
    }

    pub fn valeur(&self, cle: &str) -> &str {
        self.champs
            .iter()
            .find(|champ| champ.cle == cle)
            .map(|champ| champ.valeur.as_str())
            .unwrap_or("")
    }

    /// Le texte canonique, ou le motif qui empêche l'insertion.
    pub fn texte(&self) -> Result<String, String> {
        match self.tag {
            TagId::Couleur => {
                let couleur = self.valeur("couleur").trim();
                if couleur.is_empty() {
                    return Err("La couleur ne peut pas être vide.".to_string());
                }
                Ok(format!("{{{{c {couleur}|{}}}}}", self.valeur("contenu")))
            }
            TagId::Date => {
                let spec = self.valeur("spec").trim();
                if spec.is_empty() {
                    return Err("La date ne peut pas être vide.".to_string());
                }
                Ok(format!("{{{{d {spec}}}}}"))
            }
            TagId::Glossaire => {
                let cle = self.valeur("cle").trim();
                if cle.is_empty() {
                    return Err("La clé du terme ne peut pas être vide.".to_string());
                }
                Ok(format!("{{{{g {cle}}}}}"))
            }
            TagId::Meta => {
                let cle = self.valeur("cle").trim();
                if cle.is_empty() {
                    return Err("La clé de la métadonnée ne peut pas être vide.".to_string());
                }
                Ok(format!("{{{{m {cle}}}}}"))
            }
            TagId::Lien => {
                let texte = self.valeur("texte");
                let cible = self.valeur("cible").trim();
                if cible.is_empty() {
                    return Err("L'adresse du lien ne peut pas être vide.".to_string());
                }
                Ok(format!("[{texte}]({cible})"))
            }
            _ => Err("Cette balise n'a pas de formulaire simple.".to_string()),
        }
    }
}

fn champ(cle: &'static str, libelle: &'static str, valeur: &str) -> ChampBalise {
    ChampBalise {
        cle,
        libelle,
        valeur: valeur.to_string(),
    }
}

impl Palette {
    /// L'identifiant du champ de recherche.
    pub const ID_CHAMP: &'static str = "palette-commande";

    pub fn id_champ() -> iced::widget::Id {
        iced::widget::Id::new(Self::ID_CHAMP)
    }

    /// Ouvre la palette, en retenant le déclencheur qui vient d'être tapé.
    fn depuis(mode: ModePalette, contenu: &str, curseur: usize) -> Self {
        Self {
            mode,
            requete: String::new(),
            selection: 0,
            remplacement: declencheur(contenu, curseur, mode),
            metadonnees: Vec::new(),
            normes: Vec::new(),
        }
    }

    /// Les résultats visibles, filtrés par la requête et par les plugins
    /// réellement branchés dans le document.
    pub fn resultats(&self, state: &State) -> Vec<Item> {
        let mut items = Vec::new();

        if self.mode == ModePalette::Liens {
            let document = crate::langage::parse(&state.editor.content());
            for element in crate::langage::liens::collecter(&document) {
                items.push(Item {
                    titre: element.libelle,
                    description: element.description,
                    alias: element.reference.clone(),
                    categorie: Categorie::Insertion,
                    icone: Icone::Lien,
                    raccourci: None,
                    action: ItemAction::LienInterne {
                        reference: element.reference,
                    },
                });
            }
        } else {
            if self.mode == ModePalette::Systeme {
                for commande in COMMANDES.iter().filter(|commande| {
                    commande_active(&state.reglages.fonctionnalites, commande.id)
                }) {
                    items.push(item_de_commande(commande));
                }

                // Les normes de la bibliothèque sont des commandes de
                // chargement direct : choisir un nom l'applique au document.
                for nom in &self.normes {
                    items.push(Item {
                        titre: format!("Norme · {nom}"),
                        description: "Charger et appliquer cette norme au document".to_string(),
                        alias: format!("norme {nom} charger appliquer"),
                        categorie: Categorie::Reglage,
                        icone: Icone::Norme,
                        raccourci: None,
                        action: ItemAction::Norme(nom.clone()),
                    });
                    items.push(Item {
                        titre: format!("Exporter la norme · {nom}"),
                        description: "Écrire cette norme dans un fichier TOML".to_string(),
                        alias: format!("exporter norme {nom} toml"),
                        categorie: Categorie::Reglage,
                        icone: Icone::Telecharger,
                        raccourci: None,
                        action: ItemAction::ExporterNorme(nom.clone()),
                    });
                }

                // Chaque thème disponible est une commande de choix.
                for theme in &state.themes {
                    items.push(Item {
                        titre: format!("Thème · {}", theme.name),
                        description: "Choisir ce thème".to_string(),
                        alias: format!("thème {} theme {}", theme.name, theme.name),
                        categorie: Categorie::Affichage,
                        icone: Icone::Theme,
                        raccourci: None,
                        action: ItemAction::Theme(theme.name.clone()),
                    });
                }
            }

            for balise in BALISES
                .iter()
                .filter(|balise| balise_active(&state.reglages.fonctionnalites, balise.id))
            {
                items.push(item_de_balise(balise));
            }
            for snippet in SNIPPETS {
                items.push(item_de_snippet(snippet));
            }

            // Les métadonnées du projet sont des balises comme les autres :
            // `{{m auteur}}` est aussi naturel que `{{date:...}}`.
            for (cle, valeur) in &self.metadonnees {
                items.push(Item {
                    titre: format!("Métadonnée · {cle}"),
                    description: valeur.clone(),
                    alias: cle.clone(),
                    categorie: Categorie::Insertion,
                    icone: Icone::EtiquetteMeta,
                    raccourci: None,
                    action: ItemAction::Meta(cle.clone()),
                });
            }
        }

        let requete = normaliser(&self.requete);
        if !requete.is_empty() {
            items.retain(|item| {
                normaliser(&item.titre).contains(&requete)
                    || normaliser(&item.alias).contains(&requete)
                    || normaliser(&item.description).contains(&requete)
            });
        }

        // Un préfixe passe avant une simple présence : taper « fig » doit
        // d'abord proposer « Figure », pas une description qui le contient.
        items.sort_by_key(|item| {
            let titre = normaliser(&item.titre);
            let alias = normaliser(&item.alias);
            if titre.starts_with(&requete) || alias.starts_with(&requete) {
                0
            } else {
                1
            }
        });

        items.truncate(10);
        items
    }

    /// L'élément sélectionné, s'il existe encore après un filtre.
    pub fn selectionne(&self, state: &State) -> Option<Item> {
        let items = self.resultats(state);
        items
            .get(self.selection.min(items.len().saturating_sub(1)))
            .cloned()
    }
}

/// Ouvre la palette depuis un message (`Ctrl+K`, `/`).
pub fn ouvrir(state: &mut State, mode: ModePalette) -> Task<Message> {
    poser(state, mode)
}

/// Installe une palette ouverte et lui donne le focus.
fn poser(state: &mut State, mode: ModePalette) -> Task<Message> {
    let contenu = state.editor.content();
    let curseur = state.editor.cursor_offset();
    let mut palette = Palette::depuis(mode, &contenu, curseur);
    palette.metadonnees = metadonnees_visibles(state);
    if mode == ModePalette::Systeme {
        palette.normes = crate::project::noms_de_normes();
    }
    state.palette = Some(palette);
    state.editor.editor.lose_focus();
    iced::widget::operation::focus(Palette::id_champ())
}

/// Ouvre la palette si la frappe vient de poser un déclencheur.
///
/// Appelé après chaque frappe qui peut changer le contenu. Le déclencheur
/// reste dans le texte tant qu'aucun choix n'est fait : Échap ne perd rien.
pub fn ouvrir_si_declencheur(state: &mut State) -> Task<Message> {
    if state.palette.is_some() {
        return Task::none();
    }

    let contenu = state.editor.content();
    let curseur = state.editor.cursor_offset();

    let mode = if declencheur(&contenu, curseur, ModePalette::Liens).is_some() {
        ModePalette::Liens
    } else if declencheur(&contenu, curseur, ModePalette::Systeme).is_some() {
        ModePalette::Systeme
    } else if declencheur(&contenu, curseur, ModePalette::Balises).is_some() {
        ModePalette::Balises
    } else {
        return Task::none();
    };

    poser(state, mode)
}

/// Le déclencheur reconnu à la fin du texte, avec sa plage.
pub fn declencheur(contenu: &str, curseur: usize, mode: ModePalette) -> Option<(usize, usize)> {
    let curseur = offset_sur_caractere(contenu, curseur.min(contenu.len()));

    match mode {
        ModePalette::Liens => {
            let debut = offset_sur_caractere(contenu, curseur.saturating_sub(2));
            if &contenu[debut..curseur] != "[[" {
                return None;
            }
            if !frontiere(contenu, debut) {
                return None;
            }
            Some((debut, curseur))
        }
        ModePalette::Balises => {
            let debut = offset_sur_caractere(contenu, curseur.saturating_sub(2));
            if &contenu[debut..curseur] != ".." {
                return None;
            }
            if !frontiere(contenu, debut) {
                return None;
            }
            Some((debut, curseur))
        }
        ModePalette::Systeme => {
            let debut = offset_sur_caractere(contenu, curseur.saturating_sub(1));
            if &contenu[debut..curseur] != "/" {
                return None;
            }
            if !frontiere(contenu, debut) {
                return None;
            }
            Some((debut, curseur))
        }
    }
}

/// Le début d'un déclencheur est-il une frontière de mot ?
fn frontiere(contenu: &str, debut: usize) -> bool {
    if debut == 0 {
        return true;
    }

    contenu[..debut]
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
}

/// Les métadonnées visibles du projet, sans les vides.
fn metadonnees_visibles(state: &State) -> Vec<(String, String)> {
    if let Some(dialogue) = state.mise_en_page.as_ref() {
        return dialogue
            .metadonnees
            .paires
            .iter()
            .filter(|(_, valeur)| !valeur.trim().is_empty())
            .cloned()
            .collect();
    }

    crate::project::metadonnees::Metadonnees::charger(&state.project.dir)
        .paires
        .into_iter()
        .filter(|(_, valeur)| !valeur.trim().is_empty())
        .collect()
}

/// Une commande n'apparaît que si le plugin qu'elle pilote est branché.
fn commande_active(
    fonctionnalites: &crate::mise_en_page::modele::Fonctionnalites,
    id: CommandeId,
) -> bool {
    use crate::base::valeurs::{
        CLE_BIBLIOGRAPHIE, CLE_GLOSSAIRE, CLE_GRAPHIQUES, CLE_REFERENCES_BIBLIQUES,
    };

    match id {
        CommandeId::Citation | CommandeId::ImporterBibliographie | CommandeId::ImporterStyleCsl => {
            fonctionnalites.actif(CLE_BIBLIOGRAPHIE)
        }
        CommandeId::GlossaireFenetre => fonctionnalites.actif(CLE_GLOSSAIRE),
        CommandeId::ImporterVersionBiblique => fonctionnalites.actif(CLE_REFERENCES_BIBLIQUES),
        CommandeId::ImporterGraphique => fonctionnalites.actif(CLE_GRAPHIQUES),
        _ => true,
    }
}

/// Une balise n'apparaît que si son plugin est branché. Les balises de base
/// (couleur, date, tableau, lien…) restent toujours disponibles.
fn balise_active(
    fonctionnalites: &crate::mise_en_page::modele::Fonctionnalites,
    id: TagId,
) -> bool {
    use crate::base::valeurs::{
        CLE_BIBLIOGRAPHIE, CLE_CALLOUTS, CLE_GLOSSAIRE, CLE_GRAPHIQUES, CLE_REFERENCES_BIBLIQUES,
    };

    let plugin = match id {
        TagId::Bible | TagId::IndexBiblique => Some(CLE_REFERENCES_BIBLIQUES),
        TagId::Glossaire | TagId::GlossaireSection => Some(CLE_GLOSSAIRE),
        TagId::Bibliographie => Some(CLE_BIBLIOGRAPHIE),
        TagId::Graphique => Some(CLE_GRAPHIQUES),
        TagId::Encadre => Some(CLE_CALLOUTS),
        _ => None,
    };

    plugin.is_none_or(|cle| fonctionnalites.actif(cle))
}

fn item_de_commande(commande: &Commande) -> Item {
    Item {
        titre: commande.titre.to_string(),
        description: commande.description.to_string(),
        alias: commande.alias.join(" "),
        categorie: commande.categorie,
        icone: commande.icone,
        raccourci: commande.raccourci,
        action: ItemAction::Commande(commande.id),
    }
}

fn item_de_balise(balise: &Balise) -> Item {
    Item {
        titre: balise.titre.to_string(),
        description: balise.description.to_string(),
        alias: balise.alias.join(" "),
        categorie: Categorie::Insertion,
        icone: balise.icone,
        raccourci: None,
        action: ItemAction::Tag(balise.id),
    }
}

fn item_de_snippet(snippet: &Snippet) -> Item {
    Item {
        titre: snippet.titre.to_string(),
        description: snippet.description.to_string(),
        alias: snippet.alias.join(" "),
        categorie: Categorie::Insertion,
        icone: snippet.icone,
        raccourci: None,
        action: ItemAction::Snippet(snippet.id),
    }
}

/// Traite les messages de la palette. `None` : ce message ne la concerne pas.
pub fn traiter(state: &mut State, message: &Message) -> Option<Task<Message>> {
    match message {
        Message::PaletteOuvrir(mode) => Some(ouvrir(state, *mode)),
        Message::PaletteFermer => {
            state.palette = None;
            Some(redonner_le_clavier(state))
        }
        Message::PaletteRecherche(requete) => {
            if let Some(palette) = state.palette.as_mut() {
                palette.requete = requete.clone();
                palette.selection = 0;
            }
            Some(Task::none())
        }
        Message::PaletteDeplacer(delta) => {
            let delta = *delta;
            if state.palette.is_some() {
                let total = state
                    .palette
                    .as_ref()
                    .map(|palette| palette.resultats(state).len())
                    .unwrap_or(0);

                if total > 0 {
                    let total = total as i32;
                    if let Some(palette) = state.palette.as_mut() {
                        palette.selection =
                            ((palette.selection as i32 + delta).rem_euclid(total)) as usize;
                    }
                }
            }
            Some(Task::none())
        }
        Message::PaletteChoisir(rang) => Some(choisir_rang(state, Some(*rang), false)),
        Message::PaletteExecuter => Some(choisir(state, false)),
        Message::PaletteConfigurer => Some(choisir(state, true)),
        Message::BaliseChamp(cle, valeur) => {
            if let Some(dialogue) = state.balise_dialogue.as_mut()
                && let Some(champ) = dialogue.champs.iter_mut().find(|champ| champ.cle == cle)
            {
                champ.valeur = valeur.clone();
            }
            Some(Task::none())
        }
        Message::BaliseInserer => {
            let Some(dialogue) = state.balise_dialogue.take() else {
                return Some(Task::none());
            };
            match dialogue.texte() {
                Ok(texte) => {
                    let insertion = state.editor.insert_at_cursor(&texte).map(Message::Editor);
                    Some(Task::batch([insertion, redonner_le_clavier(state)]))
                }
                Err(motif) => {
                    state.balise_dialogue = Some(dialogue);
                    state.note = Some(crate::app::state::Note::erreur(motif));
                    Some(Task::none())
                }
            }
        }
        Message::BaliseAnnuler => {
            state.balise_dialogue = None;
            Some(redonner_le_clavier(state))
        }
        _ => None,
    }
}

/// Choisit l'élément sélectionné, puis l'exécute.
fn choisir(state: &mut State, configurer: bool) -> Task<Message> {
    choisir_rang(state, None, configurer)
}

/// Choisit un rang précis — le clic sur une ligne — ou la sélection
/// courante quand aucun rang n'est donné.
fn choisir_rang(state: &mut State, rang: Option<usize>, configurer: bool) -> Task<Message> {
    let Some(palette) = state.palette.clone() else {
        return Task::none();
    };

    let item = match rang {
        Some(rang) => palette.resultats(state).get(rang).cloned(),
        None => palette.selectionne(state),
    };
    let Some(item) = item else {
        return Task::none();
    };

    state.palette = None;
    let refocus = redonner_le_clavier(state);

    let retrait = palette
        .remplacement
        .map(|(debut, fin)| {
            state
                .editor
                .replace_span(debut, fin, "")
                .map(Message::Editor)
        })
        .unwrap_or_else(Task::none);

    let action = executer_item(state, &item, configurer);
    Task::batch([retrait, refocus, action])
}

/// Exécute une entrée du catalogue.
fn executer_item(state: &mut State, item: &Item, configurer: bool) -> Task<Message> {
    match &item.action {
        ItemAction::Commande(id) => COMMANDES
            .iter()
            .find(|commande| commande.id == *id)
            .map(|commande| match commande.action {
                Action::Message(message) => Task::done(message()),
            })
            .unwrap_or_else(Task::none),
        ItemAction::Tag(tag) => {
            if configurer {
                configurer_tag(state, *tag)
            } else {
                inserer_tag(state, *tag)
            }
        }
        ItemAction::Snippet(snippet) => inserer_snippet(state, *snippet),
        ItemAction::Meta(cle) => inserer_texte(state, &format!("{{{{m {cle}}}}}"), 0),
        ItemAction::LienInterne { reference, .. } => {
            inserer_texte(state, &format!("[[{reference}|]]"), 2)
        }
        ItemAction::Norme(nom) => Task::done(Message::NormesAppliquerAuDocument(nom.clone())),
        ItemAction::ExporterNorme(nom) => Task::done(Message::NormesExporter(nom.clone())),
        ItemAction::Theme(nom) => Task::done(Message::SelectTheme(nom.clone())),
    }
}

/// L'insertion par défaut d'une balise : le gabarit minimal, curseur dedans.
fn inserer_tag(state: &mut State, tag: TagId) -> Task<Message> {
    match tag {
        TagId::Couleur => inserer_texte(state, "{{c red|}}", 2),
        TagId::Date => inserer_texte(state, "{{d }}", 2),
        TagId::Glossaire => inserer_texte(state, "{{g }}", 2),
        TagId::Bible => inserer_texte(state, "{{b }}", 2),
        TagId::Lien => inserer_texte(state, "[]()", 3),
        TagId::Meta => inserer_texte(state, "{{m }}", 2),
        TagId::Note => Task::done(Message::NoteInserer),
        TagId::Figure => Task::done(Message::FigureInserer),
        TagId::Image => Task::done(Message::ImageInserer),
        TagId::Renvoi => Task::done(Message::RenvoiInserer),
        TagId::Etiquette => Task::done(Message::EtiquetteInserer),
        TagId::Tableau => Task::done(Message::TableauOuvrir),
        TagId::Graphique => Task::done(Message::ChartOuvrir),
        TagId::Encadre => inserer_texte(state, "{{e type=note titre=\"Note\"|}}", 2),
        TagId::Extension => Task::done(Message::ExtensionOuvrir),
        TagId::Sommaire => Task::done(Message::SommaireInserer),
        TagId::Bibliographie => Task::done(Message::BibliographieSectionInserer),
        TagId::GlossaireSection => Task::done(Message::GlossaireSectionInserer),
        TagId::IndexBiblique => Task::done(Message::BibleIndexInserer),
        TagId::Garde => inserer_texte(state, "{{garde}}\n\n{{/garde}}", 11),
    }
}

/// La configuration d'une balise : son formulaire simple, ou son panneau
/// quand elle a des champs plus riches.
fn configurer_tag(state: &mut State, tag: TagId) -> Task<Message> {
    if let Some(dialogue) = BaliseDialogue::depuis(tag) {
        state.balise_dialogue = Some(dialogue);
        state.editor.editor.lose_focus();
        return iced::widget::operation::focus(BaliseDialogue::id_champ(0));
    }

    match tag {
        TagId::Note => Task::done(Message::NoteInserer),
        TagId::Figure => Task::done(Message::FigureInserer),
        TagId::Image => Task::done(Message::ImageInserer),
        TagId::Tableau => Task::done(Message::TableauOuvrir),
        TagId::Graphique => Task::done(Message::ChartOuvrir),
        TagId::Encadre => Task::done(Message::CalloutOuvrir),
        TagId::Extension => Task::done(Message::ExtensionOuvrir),
        TagId::Bible => Task::done(Message::ReferenceBibliqueInserer),
        _ => inserer_tag(state, tag),
    }
}

fn inserer_snippet(state: &mut State, snippet: SnippetId) -> Task<Message> {
    let (texte, recul) = match snippet {
        SnippetId::These => (
            "# Titre de la thèse\n\n## Introduction\n\nLe propos s'ouvre ici.\n\n",
            0,
        ),
        SnippetId::Chapitre => ("\n\n# Titre du chapitre\n\n", 0),
        SnippetId::Section => ("\n\n## Titre de la section\n\n", 0),
        SnippetId::Citation => ("\n\n> ", 0),
        SnippetId::Tableau => ("\n\n{{t}}\n| A | B |\n| --- | --- |\n|  |  |\n\n", 0),
    };

    inserer_texte(state, texte, recul)
}

fn inserer_texte(state: &mut State, texte: &str, recul: usize) -> Task<Message> {
    state
        .editor
        .insert_avec_curseur(texte, recul)
        .map(Message::Editor)
}

/// Rend le clavier et le curseur à l'éditeur après une palette ou un
/// formulaire. `request_focus` seul ne suffit pas : le widget garde son
/// drapeau de focus interne, et le curseur restait invisible.
fn redonner_le_clavier(state: &mut State) -> Task<Message> {
    state.editor.editor.request_focus();
    state
        .editor
        .update(EditorMessage::CanvasFocusGained)
        .map(Message::Editor)
}

#[cfg(test)]
mod tests_de_la_palette {
    use super::*;

    #[test]
    fn les_declencheurs_ne_prennent_que_leurs_frontieres() {
        assert_eq!(
            declencheur("Un texte ..", 11, ModePalette::Balises),
            Some((9, 11)),
            "deux points après un blanc ouvrent les balises"
        );
        assert_eq!(
            declencheur("/", 1, ModePalette::Systeme),
            Some((0, 1)),
            "la barre en début de ligne ouvre les commandes"
        );
        assert_eq!(
            declencheur("et /", 4, ModePalette::Systeme),
            Some((3, 4)),
            "la barre après un blanc ouvre les commandes"
        );
        assert_eq!(
            declencheur("et/ou", 5, ModePalette::Systeme),
            None,
            "une barre collée à un mot ne déclenche rien"
        );
        assert_eq!(
            declencheur("http://", 7, ModePalette::Systeme),
            None,
            "une URL ne déclenche pas la palette"
        );
        assert_eq!(
            declencheur("phrase ..", 9, ModePalette::Balises),
            Some((7, 9)),
            "les points après un blanc ouvrent les balises"
        );
        assert_eq!(
            declencheur("phrase...", 9, ModePalette::Balises),
            None,
            "une ellipse collée à un mot ne déclenche rien"
        );
    }

    #[test]
    fn un_decalage_dans_un_accent_n_ouvre_pas_la_palette() {
        // Le curseur peut désigner le milieu d'un caractère accentué ;
        // le déclencheur doit se replier sur la frontière, pas découper
        // la chaîne au milieu de l'accent.
        assert_eq!(
            declencheur("TEXTE NON SAUVÉ ", 15, ModePalette::Balises),
            None,
            "un décalage dans É ne panique pas et n'ouvre rien"
        );
        assert_eq!(
            declencheur("É ..", 5, ModePalette::Balises),
            Some((3, 5)),
            "un déclencheur après un accent reste reconnu"
        );
    }

    #[test]
    fn le_catalogue_a_des_noms_et_des_icones() {
        assert!(COMMANDES.iter().all(|commande| !commande.titre.is_empty()));
        assert!(BALISES.iter().all(|balise| !balise.titre.is_empty()));
        assert!(SNIPPETS.iter().all(|snippet| !snippet.titre.is_empty()));
        assert!(
            BALISES.iter().any(|balise| balise.id == TagId::Meta),
            "la métadonnée a sa balise"
        );
    }

    #[test]
    fn le_catalogue_couvre_les_formats_les_imports_et_les_plugins() {
        for id in [
            CommandeId::Gras,
            CommandeId::Italique,
            CommandeId::Souligne,
            CommandeId::Barre,
            CommandeId::CouleurOrange,
            CommandeId::CouleurVert,
            CommandeId::CouleurDerniere,
            CommandeId::ImporterTw,
            CommandeId::ImporterTheme,
            CommandeId::ImporterNormeFichier,
            CommandeId::NouveauFichier,
            CommandeId::EditeurTheme,
            CommandeId::PluginBibliographie,
            CommandeId::PluginGlossaire,
        ] {
            assert!(
                COMMANDES.iter().any(|commande| commande.id == id),
                "commande absente du catalogue : {id:?}"
            );
        }
    }

    #[test]
    fn le_formulaire_d_une_metadonnee_produit_sa_balise() {
        let dialogue = BaliseDialogue::depuis(TagId::Meta).expect("formulaire");
        assert_eq!(dialogue.texte().expect("texte"), "{{m auteur}}");
    }

    #[test]
    fn un_formulaire_incomplet_ne_produit_rien() {
        let mut dialogue = BaliseDialogue::depuis(TagId::Date).expect("formulaire");
        dialogue.champs[0].valeur.clear();
        assert!(dialogue.texte().is_err(), "une date vide est refusée");
    }

    #[test]
    fn le_formulaire_de_couleur_produit_une_balise_courte() {
        let mut dialogue = BaliseDialogue::depuis(TagId::Couleur).expect("formulaire");
        dialogue.champs[0].valeur = "#e74c3c".to_string();
        dialogue.champs[1].valeur = "important".to_string();
        assert_eq!(dialogue.texte().expect("texte"), "{{c #e74c3c|important}}");
    }

    #[test]
    fn seuls_les_plugins_branches_proposent_leurs_commandes() {
        let mut fonctionnalites = crate::mise_en_page::modele::Fonctionnalites::default();
        assert!(commande_active(&fonctionnalites, CommandeId::Citation));
        assert!(balise_active(&fonctionnalites, TagId::Bible));

        fonctionnalites.bibliographie = false;
        fonctionnalites.references_bibliques = false;
        fonctionnalites.glossaire = false;
        assert!(
            !commande_active(&fonctionnalites, CommandeId::Citation),
            "la citation bibliographique disparaît"
        );
        assert!(
            !commande_active(&fonctionnalites, CommandeId::GlossaireFenetre),
            "la fenêtre du glossaire disparaît"
        );
        assert!(
            !balise_active(&fonctionnalites, TagId::Bible),
            "la balise biblique disparaît"
        );
        assert!(
            !balise_active(&fonctionnalites, TagId::Glossaire),
            "la balise de glossaire disparaît"
        );
        assert!(
            balise_active(&fonctionnalites, TagId::Couleur),
            "les balises de base restent"
        );
    }
}
