use crate::typst::render::{CompileOutcome, PageImage};
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;

/// Le vocabulaire de la mise en forme vit chez l'éditeur ; le message
/// de l'application ne fait que le porter.
pub use crate::editor::FormatAction;

#[derive(Debug, Clone)]
pub enum Message {
    VersVue(u64, Box<Message>),
    RetourVue(u64, Box<Message>),
    RetourDocument((PathBuf, PathBuf), Box<Message>),
    VueChoisir(u64),
    VueFermer(u64),
    VueDupliquer,
    Scinder(iced::widget::pane_grid::Edge),
    OngletSuivant,
    PanneauSuivant,
    PanneauChoisir(iced::widget::pane_grid::Pane),
    PanneauRedimensionner(iced::widget::pane_grid::ResizeEvent),
    OngletPrendre(u64),
    OngletCible(
        iced::widget::pane_grid::Pane,
        Option<iced::widget::pane_grid::Edge>,
    ),
    OngletPoser,
    Editor(EditorMessage),
    Format(FormatAction),
    /// Insère `#` × n + espace au curseur — Ctrl+1 à Ctrl+6.
    TitreInserer(u8),

    NewFile,

    OpenFile,
    FileOpened(Option<(PathBuf, String)>),

    /// Ouvre un chapitre du projet (clic dans le panneau de travail).
    OpenChapter(PathBuf),
    ChapterOpened(PathBuf, String, bool),

    /// Crée un chapitre, puis l'ouvre.
    NewChapter,

    /// Crée un projet, puis l'ouvre.
    NewProject,

    /// La base choisie à la création : un projet-modèle, ou rien.
    ProjetBase(String),

    /// Une fonctionnalité du projet se branche ou se débranche.
    ProjetFonctionnalite(String, bool),

    /// Ce projet est un modèle — ou cesse de l'être.
    ProjetModeleBascule,

    /// Une fonctionnalité du projet se branche dans les normes.
    ReglagesFonctionnalite(String, bool),

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

    ZoteroCollections(
        Result<Vec<crate::zotero::client::Collection>, crate::zotero::client::Erreur>,
    ),

    ZoteroChoisir(String, String),

    /// Résultat d'un export Zotero (clé, nom de collection, projet
    /// demandeur, export) : écrit **dans le projet demandeur** seulement.
    ZoteroExport(String, String, std::path::PathBuf, Result<String, String>),

    ZoteroActiverApi,

    ZoteroSynchroniser,

    ZoteroDelier,

    ZoteroLiaisonFermer,

    EditionReferenceNouveau,

    EditionReferenceFermer,

    SaveFile,
    SaveAs,
    FileSaved(Option<PathBuf>),

    /// Écrase un changement extérieur après décision de l'auteur, avec
    /// l'empreinte de ce changement : elle évite de reposer la question
    /// après un refus.
    SauvegardeForcee(u64),

    /// Pose dans le tampon l'import prévisualisé — le fragment converti
    /// rejoint la fin du texte d'un seul geste, annulable d'un seul.
    ImportAppliquer,

    /// Exporte le document courant en PDF (dialogue de destination).
    ExportPdf,

    /// Destination choisie et PDF compilé, avant écriture.
    PdfDestination(Result<(PathBuf, Vec<u8>), String>),
    PdfApercu(Result<Vec<u8>, String>),

    /// Résultat de l'export PDF : chemin du fichier ou message d'erreur.
    PdfExported(Result<String, String>),

    /// Tic de compilation périodique (relance si le document a changé).
    /// Une fenêtre de rendu doit montrer son document : compiler si elle
    /// ne l'a pas encore, fabriquer son lecteur ensuite.
    RenduInitialiser,
    RenderTick,

    /// Résultat de la compilation (document et diagnostics).
    RenderDone(CompileOutcome),

    /// Résultat de la rasterisation d'une page.
    ///
    /// Porte le numéro de révision du document pour lequel le rendu a été
    /// demandé : si une compilation est arrivée entre-temps, l'image est
    /// écartée au lieu de remplacer une image plus récente.
    PageRendered(u64, usize, Option<PageImage>, (f32, f32, f32, f32)),

    /// Le viewport de l'aperçu a changé : première page visible, dernière, et
    /// la place que le panneau laisse à une page.
    ///
    /// Les bornes viennent de la vue — seule à mesurer la place disponible — et
    /// ne changent qu'au redimensionnement : un défilement les retransmet
    /// identiques, et c'est cette égalité qui permet de ne rien refaire.
    ViewportChanged(usize, usize, iced::Rectangle, Option<(f32, f32)>),

    /// Ajuste l'aperçu à la largeur ou à la hauteur, ou choisit un zoom libre.
    PreviewZoom(crate::app::state::PreviewZoomMode),

    /// Modifie le zoom libre par incréments de 10 %.
    PreviewZoomStep(i8),

    /// Le système demande la fermeture ; les documents doivent être sauvés.
    FermetureDemandee,

    /// Tic de sauvegarde périodique (écriture différée sur le disque).
    SaveTick,

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
    NoteOverlayChamp(String),
    NoteOverlayEnregistrer,
    NoteOverlayFermer,

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

    RechercheOuvrir,

    NormesNom(String),
    NormesAppliquer(String),
    NormesEcraser,
    NormesEnregistrer(String),
    NormesSupprimer(String),

    TableauOuvrir,
    TableauRangs(String),
    TableauColonnes(String),
    TableauInserer,
    TableauAnnuler,

    ReferencesReplier,
    /// Replier une entrée de bibliographie (bug-report-10, 2.2).
    ReferencesFiltrer(String),

    /// Le glossaire du projet (panneau gauche).
    GlossaireReplier,
    GlossaireRechercheBascule,
    GlossaireFiltrer(String),
    /// Insère la référence au terme au curseur — `{glo:clé}` ou
    /// `#gls("clé")`, selon le dialecte du tampon.
    /// La section bibliographie, insérée au curseur (bug-report-9).
    BibliographieSectionInserer,
    /// La section glossaire, insérée au curseur (bug-report-9).
    /// La table des matières, insérée au curseur (bug-report-10, 1.3).
    SommaireInserer,
    GlossaireSectionInserer,
    GlossaireInserer(String),
    /// La fenêtre du glossaire : la liste, ses gestes (bug-report-10, 5.3).
    GlossaireFenetreOuvrir,
    GlossaireFenetreFermer,
    GlossaireCreer,
    GlossaireModifier(String),
    GlossaireSupprimer(String),
    SurvolGlossaire(Option<String>),
    EditionGlossaireChamp(String, String),
    EditionGlossaireEnregistrer,
    EditionGlossaireFermer,

    ImageInserer,
    ImageChoisie(Option<std::path::PathBuf>),

    ReferenceSupprimer(String),

    /// Remet dans la bibliographie la notice que la corbeille a recueillie.
    ReferenceRestaurer(std::path::PathBuf),

    /// Importer un style CSL local dans le projet.
    StyleCslImporter,

    /// Le fichier CSL choisi, prêt à être contrôlé et copié.
    StyleCslChoisi(Option<std::path::PathBuf>),

    /// Télécharger le parent d'un style dépendant — jamais sans le
    /// consentement donné par la question.
    StyleParentTelecharger,

    /// Choisir un style installé dans Zotero, par son identifiant CSL.
    StyleZoteroChoisi(String),

    /// Insérer une note de bas de page au curseur.
    NoteInserer,

    /// Insérer une figure avec sa légende au curseur.
    FigureInserer,

    /// Insérer une étiquette de renvoi au curseur.
    EtiquetteInserer,

    /// Insérer un renvoi (`@`), la liste des étiquettes venant avec la
    /// complétion.
    RenvoiInserer,

    /// Le fichier BibLaTeX choisi, prêt à rejoindre la bibliothèque.
    BibImporte(Option<std::path::PathBuf>),

    /// Ouvrir le sélecteur d'un fichier `.bib` à importer.
    BibliographieImporter,

    ProjetNom(String),
    SurvolReference(Option<String>),
    SurvolOnglet(Option<u64>),
    SurvolProjet(Option<std::path::PathBuf>),
    BibliographieCreer,
    ZoteroRecherche(String),

    ProjetsVue,
    /// Une entrée de sommaire est survolée (ou quittée).
    SommaireSurvol(Option<usize>),
    /// La recherche des références se montre ou se cache.
    ReferencesRechercheBascule,
    /// La recherche des projets se montre ou se cache.
    ProjetsRechercheBascule,
    /// L'étagère choisie à la liste des projets.
    ProjetsEtageChoisie(String),
    /// La liste des projets bascule : étagère courante, ou tous.
    ProjetsFiltreBascule,
    /// La recherche dans la liste des projets.
    ProjetsRecherche(String),
    /// Une métadonnée choisie à la création : champ, valeur.
    ProjetMeta(String, String),
    /// Une métadonnée du document change : ancienne clé, nouvelle clé,
    /// valeur — la clé s'édite comme le reste.
    MetaChamp(String, String, String),
    /// Une ligne de métadonnée s'ajoute.
    MetaAjouter,
    /// Une ligne de métadonnée se retire, par sa clé.
    MetaSupprimer(String),
    /// Les profils de métadonnées (bug-report-10, 4.1–4.3).
    MetaProfilNom(String),
    MetaProfilActiver(String),
    MetaProfilEnregistrer(String),
    MetaProfilSupprimer(String),
    /// Les balises `{clé}` du texte remplacées par leurs valeurs (4.4).
    MetaRemplacer,
    /// Le composeur d'extension s'ouvre (bug-report-7, point 4).
    ExtensionOuvrir,
    /// L'extension choisie, par son rang au registre.
    ExtensionChoisir(usize),
    /// Un paramètre d'extension change.
    ExtensionParam(String, String),
    /// Le code de l'extension se pose dans le document.
    ExtensionInserer,
    /// Le composeur se ferme, rien n'est posé.
    ExtensionAnnuler,
    /// Le texte de la note en cours d'écriture dans son panneau.
    NoteTexte(String),
    /// La note s'écrit dans la ligne, la marque seule.
    NoteValider,
    /// La note est abandonnée — rien n'est écrit.
    NoteAnnuler,
    ModeBasculer,
    ModeTypst,
    VignettesFaites(Vec<(String, Vec<u8>, u32, u32)>),

    DragCommencer(String),
    DragSurvol(String),
    DragPoser(String),

    OngletChoisir(std::path::PathBuf),
    OngletFermer(std::path::PathBuf),

    ThemeEditeurOuvrir,
    ThemeEditeurChoisir(String),
    ThemeEditeurChamp(&'static str, String),
    ThemeEditeurColorPicker(&'static str),
    ThemeEditeurEnregistrer,
    ThemeEditeurReset,
    ThemeEditeurFermer,
    EtagereBrouillon(String),
    EtagereDefinir(String, String),

    ProjetNorme(String),
    ProjetDepuisEtagere(String),
    ProjetCreer,

    /// Exporte le projet en un seul `.tw` transportable.
    ProjetExportTw,
    /// Le `.tw` unique est écrit — ou pas.
    ProjetTwExporte(Result<String, String>),
    /// Importe un `.tw` unique en projet reconstruit.
    ProjetImportTw,
    /// Le projet est reconstruit — ou pas.
    ProjetTwImporte(Result<String, String>),
    /// Exporte le projet en archive portable.
    ProjetExportPortable,
    /// Résultat de l'export portable : ce qui voyage, ou pourquoi pas.
    ProjetPortableExporte(Result<String, String>),
    /// Ouvre le sélecteur d'une sauvegarde à importer.
    ProjetImportSauvegarde,
    /// L'archive choisie pour l'import, `None` si le geste est annulé.
    ProjetImportChoisi(Option<std::path::PathBuf>),
    /// L'import a reçu son mot : les collisions partent à la corbeille.
    ProjetImportConfirmer(std::path::PathBuf, std::path::PathBuf),

    /// La fin de l'import de sauvegarde, hors du thread de l'interface.
    ProjetImporte(crate::project::portable::SortieImport),

    /// Exporte le document courant en Markdown.
    ExportMarkdown,
    /// La destination choisie pour le Markdown, `None` si annulé.
    ExportMarkdownChoisi(Option<std::path::PathBuf>),
    /// Ouvre le dossier des données de l'application.
    OuvrirDossierDonnees,
    /// Ouvre la corbeille — ce qui a été retiré s'y retrouve.
    OuvrirCorbeille,
    /// Ouvre les sauvegardes du projet — les versions antérieures.
    OuvrirSauvegardes,
    /// Exporte le document courant en DOCX (LibreOffice, notes en texte).
    ExportDocx,
    /// La destination choisie pour le DOCX, `None` si annulé.
    ExportDocxChoisi(Option<std::path::PathBuf>),
    ProjetRenommer(std::path::PathBuf),
    ProjetValider,
    ProjetAnnuler,
    ProjetSupprimer(std::path::PathBuf),

    /// Un clic avalé : le voile d'une fenêtre, qui ne doit rien laisser
    /// passer à ce qui est derrière.
    Rien,

    /// `Ctrl` ou `Maj` tenu ou relâché — le zoom et le shift+clic écoutent.
    ModificateursTenus {
        controle: bool,
        maj: bool,
    },

    /// La molette de la souris : le zoom avec `Ctrl`, le reste au défilement.
    Molette(f32),

    ConfirmationConfirmer,
    ConfirmationAnnuler,

    AideBasculer,

    /// Le sélecteur de couleur s'ouvre sur la couleur courante.
    PickerCouleurOuvrir,
    /// Le sélecteur se ferme sans rien appliquer.
    PickerCouleurFermer,
    /// La roue a donné une teinte, en degrés.
    PickerCouleurTeinte(f32),
    /// Le carré a donné saturation et valeur.
    PickerCouleurSatVal(f32, f32),
    /// La couleur choisie s'applique à sa cible (texte ou thème).
    /// Demande une image pour un lieu d'en-tête ou de pied de page.
    EntetePiedImage(crate::mise_en_page::modele::Champ),
    /// Une image choisie : elle se pose dans le champ visé.
    EntetePiedImageChoisie(crate::mise_en_page::modele::Champ, String),
    /// Ouvre la roue pour une couleur des normes : la vignette cliquée.
    PickerCouleurPourReglage(crate::mise_en_page::modele::Champ),
    /// Le choix : la couleur devient la dernière enregistrée — rien
    /// n'est inséré tant qu'on ne l'applique pas (bug-report-10, 5.4).
    PickerCouleurEnregistrer,
    PickerCouleurAppliquer,
    /// Le code hex est saisi au clavier — il pilote la roue.
    PickerCouleurHex(String),
    /// Une couleur rapide s'applique tout de suite, la roue reste ouverte.
    PickerCouleurRapide(String),

    RechercheFermer,

    RechercheMotif(String),
    RechercheBasculerMode,

    RechercheRemplacement(String),

    RechercheSuivant,

    RecherchePrecedent,

    RechercheRemplacer,

    RechercheRemplacerTout,

    BlocMonter,
    /// Les gestes d'un élément de liste (bug-report-10, 3.2, 3.3, 3.6).
    ListeIndenter,
    ListeDesindenter,
    ListeMonter,
    ListeDescendre,
    /// `Entrée` crée l'élément suivant (bug-report-10, 3.4).
    ListeEnchainer,

    BlocDescendre,

    TitreSuivant,

    TitrePrecedent,

    /// Ouvre la fenêtre de mise en page.
    MiseEnPageOuvrir,

    /// La ferme sans rien appliquer.
    MiseEnPageFermer,

    /// Un champ de la mise en page change.
    MiseEnPageChamp(crate::mise_en_page::modele::Champ, String),
    /// Molette ou flèche sur un champ numérique : +1 ou -1.
    MiseEnPageMolette(crate::mise_en_page::modele::Champ, i8),

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

    /// Destination choisie et réglages sérialisés, avant écriture.
    MiseEnPageDestination(std::path::PathBuf, String),

    /// Une note pour la barre d'état : ce qui vient de se faire, ou pas.
    MiseEnPageNote(String),

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
