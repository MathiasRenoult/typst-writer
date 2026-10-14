use crate::typst::render::{CompileOutcome, PageImage};
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Editor(EditorMessage),

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
