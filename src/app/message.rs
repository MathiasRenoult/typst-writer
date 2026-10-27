use crate::typst::render::{CompileOutcome, PageImage};
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;

/// Mise en forme appliquée à la sélection ou au bloc courant du `document.tw`.
#[derive(Debug, Clone)]
pub enum FormatAction {
    Gras,
    Italique,
    Couleur(String),
    Titre(u8),
    ListePuce,
    ListeNumero,
    CitationBloc,
    Citation,
    Note,
    BlocTypst,
}

#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage),
    Format(FormatAction),

    NewFile,

    OpenFile,
    FileOpened(Option<(PathBuf, String)>),

    /// Ouvre un chapitre du projet (clic dans le panneau de travail).
    OpenChapter(PathBuf),
    ChapterOpened(PathBuf, String),

    /// Crée un chapitre, puis l'ouvre.
    NewChapter,

    /// Crée un projet, puis l'ouvre.
    NewProject,

    /// Ouvre un autre projet.
    OpenProject(PathBuf),

    /// Rescanne le projet (après création ou suppression).
    RefreshProject,

    /// Supprime un chapitre : retiré de l'assemblage, puis du disque.
    DeleteChapter(PathBuf),

    /// Commence le renommage d'un chapitre, en place dans le panneau.
    StartRename(PathBuf),

    /// Le titre est modifié dans le champ de saisie.
    RenameChanged(String),

    /// Valide le renommage.
    RenameCommit,

    /// Abandonne le renommage.
    RenameCancel,

    /// Insère une citation `@clé` à la position du curseur.
    InsertCitation(String),

    /// Ouvre la fenêtre d'édition d'une référence.
    ModifierReference(String),

    /// Un champ de la fenêtre d'édition a changé.
    EditionReferenceChamp(String, String),

    /// Enregistre la référence éditée dans la bibliothèque locale.
    EditionReferenceEnregistrer,

    /// Ferme la fenêtre d'édition sans rien écrire.
    ZoteroLierOuvrir,

    ZoteroCollections(Result<Vec<crate::zotero::client::Collection>, String>),

    ZoteroChoisir(String, String),

    ZoteroExport(String, String, Result<String, String>),

    ZoteroDelier,

    ZoteroLiaisonFermer,

    EditionReferenceNouveau,

    EditionReferenceFermer,

    SaveFile,
    SaveAs,
    FileSaved(Option<PathBuf>),

    /// Exporte le document courant en PDF (dialogue de destination).
    ExportPdf,

    /// Résultat de l'export PDF : chemin du fichier ou message d'erreur.
    PdfExported(Result<String, String>),

    /// Tic de compilation périodique (relance si le document a changé).
    RenderTick,

    /// Résultat de la compilation (document et diagnostics).
    RenderDone(CompileOutcome),

    /// Résultat de la rasterisation d'une page.
    ///
    /// Porte le numéro de révision du document pour lequel le rendu a été
    /// demandé : si une compilation est arrivée entre-temps, l'image est
    /// écartée au lieu de remplacer une image plus récente.
    PageRendered(u64, usize, Option<PageImage>),

    /// Le viewport de l'aperçu a changé : première page visible, dernière, et
    /// la place que le panneau laisse à une page.
    ///
    /// Les bornes viennent de la vue — seule à mesurer la place disponible — et
    /// ne changent qu'au redimensionnement : un défilement les retransmet
    /// identiques, et c'est cette égalité qui permet de ne rien refaire.
    ViewportChanged(usize, usize, iced::Size),

    /// Ajuste l'aperçu à la largeur ou à la hauteur, ou choisit un zoom libre.
    PreviewZoom(crate::app::state::PreviewZoomMode),

    /// Modifie le zoom libre par incréments de 10 %.
    PreviewZoomStep(i8),

    /// Tic de sauvegarde périodique (écriture différée sur le disque).
    SaveTick,

    /// Le battement d'écoute du serveur de langage.
    ///
    /// On **relève** ses événements au lieu de les attendre : son canal est un
    /// canal ordinaire, et l'attendre dans un abonnement immobiliserait un fil
    /// de l'exécuteur pour un serveur qui parle rarement.
    LspPoll,

    /// Un événement du serveur de langage.
    Lsp(crate::lsp::client::Event),

    /// La liste de complétion : on la parcourt.
    CompletionMove(isize),

    /// On accepte l'élément sélectionné.
    CompletionAccept,

    /// On accepte l'élément désigné par la souris.
    CompletionPick(usize),

    /// On ferme la liste sans rien accepter.
    CompletionDismiss,

    /// Le composeur de notes : ce qui est tapé dans le champ de l'étape —
    /// le filtre des clés, ou la mention de page.
    ComposeurSaisie(String),

    /// Tab : prendre la référence choisie, ou valider la page saisie.
    ComposeurAvancer,

    /// Entrée : insérer la note, et fermer.
    ComposeurTerminer,

    /// Échap : fermer sans rien laisser.
    ComposeurFermer,

    /// ↑↓ : déplacer la sélection dans la liste des clés.
    ComposeurDeplacer(isize),

    /// Un clic sur une ligne de la liste : la prendre, comme Tab.
    ComposeurPrendre(usize),

    /// Un clic sur une ligne de la note : la remettre sous les doigts, à son
    /// rang — première, deuxième référence, laquelle on veut.
    ComposeurChoisir(usize),

    /// Aller à la définition de ce qui est sous le curseur.
    GoToDefinition,

    /// Demander ce que le serveur sait de ce qui est sous le curseur.
    ShowHover,

    /// Va à un titre du sommaire (le rang dans la liste du document).
    GotoHeading(usize),

    /// Ouvre la fenêtre de mise en page.
    MiseEnPageOuvrir,

    /// La ferme sans rien appliquer.
    MiseEnPageFermer,

    /// Un champ de la mise en page change.
    MiseEnPageChamp(crate::mise_en_page::modele::Champ, String),

    /// Une case à cocher bascule.
    MiseEnPageBascule(crate::mise_en_page::modele::Champ),

    /// L'accroche à `main.typ` est demandée ou retirée.
    MiseEnPageAccrocher(bool),

    /// Applique la copie de travail au projet.
    MiseEnPagePanneau(usize),

    MiseEnPageTypographieDefaut(String),

    MiseEnPageAppliquer,

    /// Importe une mise en page depuis un fichier.
    MiseEnPageImporter,

    /// Le fichier importé est arrivé.
    MiseEnPageImportee(String),

    /// Exporte la mise en page vers un fichier.
    MiseEnPageExporter,

    /// Une note pour la barre d'état : ce qui vient de se faire, ou pas.
    MiseEnPageNote(String),

    /// Ouvre le renommage du symbole sous le curseur.
    SymbolRenameStart,

    /// Le nouveau nom change dans la case.
    SymbolRenameChanged(String),

    /// Demande au serveur ce que ce renommage changerait — c'est ce qu'on
    /// montrera avant toute écriture.
    SymbolRenameAsk,

    /// Applique ce que le serveur a proposé, que l'utilisateur a vu.
    SymbolRenameApply,

    /// Renonce au renommage.
    SymbolRenameCancel,

    /// La fenêtre a changé de taille : (largeur, hauteur).
    WindowResized(iced::Size),

    /// Le facteur d'échelle de l'écran, demandé à l'ouverture de la fenêtre.
    ///
    /// La densité du rendu en découle : rendue constante, elle rasterise plus
    /// fin que l'écran ne peut montrer sur les écrans denses, et paie la mémoire
    /// au carré du rapport.
    ScaleFactor(f32),

    /// Tic de mesure de la mémoire : actif seulement si l'environnement le
    /// demande (`TYPST_WRITER_MEMOIRE`).
    ///
    /// Un instrument, pas une fonction de l'application : il est là parce qu'un
    /// rapport d'explosion mémoire ne se diagnostique pas à l'œil, et que la
    /// mémoire graphique n'apparaît dans aucun profil ordinaire.
    MemoryTick,

    /// Le mode de thème du système a été lu au démarrage ou a changé.
    SystemTheme(crate::ui::theme::SystemMode),

    /// Affiche ou masque la bibliothèque.
    ToggleSidebar,

    /// Affiche ou masque l'aperçu.
    TogglePreview,

    /// Bascule le suivi de l'aperçu.
    TogglePreviewSync,

    /// Change la position de l'aperçu.
    SetPreviewPosition(crate::app::state::PreviewPosition),

    /// Un séparateur est saisi.
    SplitterPressed(crate::app::state::Splitter),

    /// La souris s'est déplacée.
    ///
    /// Les deux coordonnées sont transmises : l'abscisse gouverne les
    /// dispositions en lignes, l'ordonnée la disposition en colonnes. La
    /// position est mémorisée même hors glissement, pour que la saisie d'un
    /// séparateur démarre à l'endroit exact du clic.
    MouseMoved(iced::Point),

    /// Le séparateur est relâché.
    SplitterReleased,

    /// Bascule le mode focus : plein écran, le texte seul.
    ToggleFocusMode,

    /// Une fenêtre s'est ouverte : on retient son identifiant.
    WindowOpened(iced::window::Id),

    /// Fait cycler le thème : système → clair → sombre → système.
    CycleTheme,

    /// Applique le thème portant ce nom.
    SelectTheme(String),

    /// Relit le dossier des thèmes.
    RefreshThemes,

    /// Ouvre le dossier des thèmes dans le gestionnaire de fichiers.
    OpenThemesFolder,

    /// Ouvre le sélecteur de fichier pour charger un thème.
    LoadTheme,

    /// Le chemin du fichier de thème choisi, s'il y en a un.
    ThemeLoaded(Option<PathBuf>),

    /// Supprime un thème local, désigné par son nom.
    DeleteTheme(String),

    /// Saute à la position d'un diagnostic dans l'éditeur.
    GotoError(usize, usize),

    /// Événement clavier non consommé par les widgets (raccourcis globaux).
    RawKey(iced::keyboard::Key, iced::keyboard::Modifiers),
}
