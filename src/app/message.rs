use crate::typst::render::{CompileOutcome, PageImage};
use iced_code_editor::Message as EditorMessage;
use std::path::PathBuf;
use std::sync::Arc;

/// Le vocabulaire de la mise en forme vit chez l'éditeur ; le message
/// de l'application ne fait que le porter.
pub use crate::editor::FormatAction;

/// Le format d'un export demandé depuis la bibliothèque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatExport {
    Pdf,
    Tw,
    Markdown,
    Docx,
}

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
    /// Applique la dernière couleur choisie à la sélection.
    FormatDerniereCouleur,
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
    /// Bascule une fonctionnalité depuis une commande, et l'applique.
    BasculerFonctionnalite(String),

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
    /// Ouvre le composeur de citation au curseur, ou sur la citation qui s'y trouve.
    CitationOuvrir,

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

    /// PDF produit depuis le document déjà compilé, pour la vue de rendu.
    ///
    /// L'empreinte est celle du document compilé : un PDF qui arrive après
    /// une compilation plus récente ne remplace jamais le lecteur courant.
    PdfDocumentPret(String, Result<Vec<u8>, String>),

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
    PageRendered(u64, usize, Option<PageImage>, (f32, f32, f32, f32), bool),

    /// La couche texte d'une compilation est prête.
    TexteApercuPret(String, Arc<crate::typst::texte::DocumentTexte>),

    /// Un geste de sélection dans l'aperçu : page, point de la page et
    /// nombre de clics (1, 2 ou 3). `etendue` vient de `Maj`.
    SelectionApercuAppuyer {
        page: usize,
        point: (f32, f32),
        etendue: bool,
        clics: u8,
    },
    SelectionApercuGlisser {
        page: usize,
        point: (f32, f32),
    },
    SelectionApercuRelacher,
    SelectionApercuCopier,

    /// Un lien de l'aperçu a été activé : page et rang du lien.
    LienApercuActiver(usize, usize),

    RechercheApercuOuvrir,
    RechercheApercuFermer,
    RechercheApercuMotif(String),
    RechercheApercuSuivant,
    RechercheApercuPrecedent,

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
    /// La recherche dans le panneau d'insertion.
    CompletionChercher(String),

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

    RechercheOuvrir,

    NormesNom(String),
    NormesAppliquer(String),
    NormesEcraser,
    NormesEnregistrer(String),
    NormesSupprimer(String),
    /// Demande le mot de la suppression d'une norme (la corbeille n'efface
    /// pas un fichier de normes : seule l'entrée de la bibliothèque part).
    NormesSupprimerDemande(String),
    /// Applique directement au document la norme nommée, sans passer par
    /// la copie de travail de la fenêtre.
    NormesAppliquerAuDocument(String),
    /// Exporte une norme de la bibliothèque dans un fichier TOML.
    NormesExporter(String),
    /// Importe un fichier de normes dans la bibliothèque interne.
    NormesFichierImporter,
    /// Le fichier choisi pour l'import, ou l'annulation.
    NormesFichierImporte(Option<std::path::PathBuf>),

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
    ReferenceBibliqueInserer,
    /// Le panneau de construction d'une citation biblique.
    BiblePanneauRecherche(String),
    BiblePanneauLivre(usize),
    BiblePanneauChapitre(String),
    BiblePanneauVersets(String),
    BiblePanneauTexte(String),
    BiblePanneauAjouter,
    BiblePanneauRetirer(usize),
    BiblePanneauForme(Option<crate::plugins::bible::FormeLivre>),
    BiblePanneauInserer,
    BiblePanneauFermer,
    /// La recherche et le tri des statistiques bibliques.
    BibleRechercheBascule,
    BibleFiltrer(String),
    BibleTri(crate::plugins::bible::TriReferences),
    /// Aller à la première occurrence de cette référence dans l'éditeur.
    BibleAllerReference(String),
    /// Aller à la première occurrence d'un lien interne cassé.
    LienInterneAller(String),
    /// Choisir la traduction de la consultation biblique.
    BibleTraduction(crate::plugins::bible::Traduction),
    /// Choisir un fichier de version biblique à installer.
    BibleVersionImporter,
    /// Le fichier choisi, s'il y en a un.
    BibleVersionImported(Option<PathBuf>),
    /// L'installation d'une version, hors du fil de l'interface.
    BibleVersionInstallee(Result<crate::langage::versions::VersionBiblique, String>),
    /// Ouvrir le dossier des versions bibliques ajoutées.
    BibleVersionDossier,
    /// Supprimer une version après confirmation (elle part à la corbeille).
    BibleVersionSupprimer(String),
    /// Copier le fichier JSON d'une version vers une destination choisie.
    BibleVersionExporter(String),
    /// Résultat de la copie d'une version ; `None` si le choix a été annulé.
    BibleVersionExportee(Result<Option<String>, String>),
    /// Enregistrer un modèle JSON de version.
    BibleVersionModele,
    /// Résultat de l'enregistrement du modèle ; `None` si annulé.
    BibleVersionModeleEcrit(Result<Option<String>, String>),
    /// Ouvrir l'édition d'une référence de la consultation.
    BibleReferenceModifier(usize),
    BibleEditionLivre(usize),
    BibleEditionChapitre(String),
    BibleEditionVersets(String),
    BibleEditionEnregistrer,
    BibleEditionAnnuler,
    /// Insérer l'index des citations bibliques du document au curseur.
    BibleIndexInserer,
    /// Le repli de la section statistiques.
    StatsReplier,
    /// L'insertion d'une référence biblique : la forme s'ouvre.
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

    /// Supprime une notice de la bibliographie locale.
    ReferenceSupprimer(String),

    /// Retire toutes les notices de la bibliographie locale, après
    /// confirmation et copie dans la corbeille.
    ReferencesSupprimerTout,

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
    /// Le pointeur quitte l'éditeur : la bulle de verset ne doit pas rester
    /// collée à l'écran derrière lui.
    BibleSurvolQuitter,
    SurvolOnglet(Option<u64>),
    SurvolProjet(Option<std::path::PathBuf>),
    BibliographieCreer,
    ZoteroRecherche(String),

    ProjetsVue,
    /// Un clic sur une carte de la bibliothèque : ouvrir, ou sélectionner
    /// selon les modificateurs tenus.
    ProjetCliquer(std::path::PathBuf),
    /// La sélection de la bibliothèque se vide.
    SelectionVider,
    /// Une étagère nouvelle se crée, même sans projet dedans.
    EtagereCreer,
    /// Une étagère monte ou descend dans l'ordre de la bibliothèque.
    EtagereMonter(String),
    EtagereDescendre(String),
    /// La fenêtre de normes pour les documents sélectionnés.
    SelectionNormes,
    NormesLotChoisir(String),
    NormesLotAccrocher(bool),
    NormesLotAppliquer,
    NormesLotFermer,
    /// Exporter les documents sélectionnés dans un dossier choisi.
    SelectionExporter(FormatExport),
    /// Le compte rendu de l'export de la sélection.
    SelectionExportee(Result<String, String>),
    /// Une entrée de sommaire est survolée (ou quittée).
    SommaireSurvol(Option<usize>),
    /// Le sommaire du panneau se replie ou se déplie.
    SommaireReplier,
    /// La section « Chapitres » se replie ou se déplie.
    ChapitresReplier,

    /// Choisit la tranche affichée : `None` pour le document entier, sinon
    /// l'index d'un chapitre dans `render.chapitres`.
    ChapitreChoisir(Option<usize>),
    /// La section « Projets » du panneau se replie ou se déplie.
    ProjetsSectionReplier,
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

    /// Le panneau du graphique primaviz s'ouvre.
    ChartOuvrir,
    /// Le type du graphique choisi.
    ChartType(String),
    /// Le titre du graphique change.
    ChartTitre(String),
    /// Le sous-titre du graphique change.
    ChartSousTitre(String),
    /// Le titre de l'axe horizontal change.
    ChartXLabel(String),
    /// Le titre de l'axe vertical change.
    ChartYLabel(String),
    /// Une action dans l'éditeur CSV.
    ChartCsv(iced::widget::text_editor::Action),
    /// Importer un fichier CSV dans le panneau.
    ChartImporter,
    /// Le contenu importé, ou l'annulation.
    ChartImporte(Option<String>),
    /// La largeur change.
    ChartLargeur(String),
    /// La hauteur change.
    ChartHauteur(String),
    /// Le placement change.
    ChartPlacement(String),
    /// La légende s'affiche ou non.
    ChartLegende(bool),
    /// Les valeurs s'affichent ou non.
    ChartValeurs(bool),
    /// La grille s'affiche ou non.
    ChartGrille(bool),
    /// Les graduations s'affichent ou non.
    ChartGraduations(bool),
    /// Les points d'une ligne s'affichent ou non.
    ChartPoints(bool),
    /// L'aire sous une ligne est remplie ou non.
    ChartAire(bool),
    /// L'interpolation de la ligne change.
    ChartCourbe(String),
    /// Le camembert devient un anneau, ou cesse de l'être.
    ChartDonut(bool),
    /// La part du rayon du trou de l'anneau change.
    ChartAnneau(String),
    /// Les barres deviennent horizontales, ou redeviennent verticales.
    ChartHorizontal(bool),
    /// Le graphique se pose dans le document.
    ChartInserer,
    /// Le panneau se ferme sans rien poser.
    ChartFermer,
    /// Le panneau d'un callout s'ouvre.
    CalloutOuvrir,
    /// Le préréglage du callout choisi.
    CalloutType(String),
    /// Le titre du callout change.
    CalloutTitre(String),
    /// Un champ du style du callout change.
    CalloutChamp(crate::langage::callout::CalloutChamp, String),
    /// Une case du style du callout bascule (`gras`, `rupture`).
    CalloutBascule(crate::langage::callout::CalloutChamp, bool),
    /// Le style revient à celui du préréglage.
    CalloutReinitialiser,
    /// Un préréglage de callout est choisi dans les paramètres.
    CalloutPresetChoisir(usize),
    /// Un champ d'un préréglage change dans les paramètres.
    CalloutPresetChamp(usize, crate::langage::callout::CalloutChamp, String),
    /// Une case d'un préréglage bascule dans les paramètres.
    CalloutPresetBascule(usize, crate::langage::callout::CalloutChamp, bool),
    /// Un préréglage est ajouté à la norme.
    CalloutPresetAjouter,
    /// Un préréglage est retiré de la norme.
    CalloutPresetSupprimer(usize),
    /// Le callout s'applique à la citation, ou pose un bloc vide.
    CalloutInserer,
    /// Le panneau se ferme sans rien poser.
    CalloutFermer,
    /// Le texte de la note en cours d'écriture dans son panneau.
    NoteTexte(String),
    /// La note s'écrit dans la ligne, la marque seule.
    NoteValider,
    /// La note est abandonnée — rien n'est écrit.
    NoteAnnuler,
    ModeBasculer,
    ModeTypst,
    /// Affiche ou masque les numéros de ligne de l'éditeur.
    NumerosDeLigneBascule,
    /// Les couvertures dont la première page a changé, prêtes en arrière-plan.
    VignettesRafraichies(Vec<(String, Vec<u8>, u32, u32)>),
    /// Choix d'une image pour remplacer la miniature d'un projet.
    MiniatureChoisir(PathBuf),
    /// L'image choisie, déjà recadrée au format A4 — ou l'erreur qui l'a
    /// empêchée. `Ok(None)` : le choix a été annulé.
    MiniaturePrete(
        PathBuf,
        Result<Option<crate::project::CouverturePreparee>, String>,
    ),

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
    /// L'archive contient des versions bibliques : l'auteur confirme ses droits.

    /// La fin de l'import de sauvegarde, hors du thread de l'interface.
    ProjetImporte(crate::project::portable::SortieImport),

    /// Exporte le document courant en Markdown.
    ExportMarkdown,
    /// La destination choisie pour le Markdown, `None` si annulé.
    ExportMarkdownChoisi(Option<std::path::PathBuf>),
    /// Ouvre le dossier des données de l'application.
    OuvrirDossierDonnees,
    /// Ouvre la corbeille — ce qui a été retiré s'y retrouve.
    /// Ouvre les sauvegardes du projet — les versions antérieures.
    OuvrirSauvegardes,
    /// Ouvre l'écran de restauration d'une version antérieure.
    RestaurationOuvrir,
    /// Ferme l'écran de restauration.
    RestaurationFermer,
    /// Choisit une copie dans l'écran de restauration.
    RestaurationChoisir(usize),
    /// Restaure la copie choisie après confirmation.
    RestaurationRestaurer,
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

    /// La palette de commandes : les balises, ou toutes les commandes.
    PaletteOuvrir(crate::app::palette::ModePalette),
    PaletteFermer,
    PaletteRecherche(String),
    PaletteDeplacer(i32),
    PaletteChoisir(usize),
    PaletteExecuter,
    PaletteConfigurer,

    /// Le formulaire générique d'une balise simple : un champ, puis insérer.
    BaliseChamp(String, String),
    BaliseInserer,
    BaliseAnnuler,

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
    /// Ouvre la roue pour la couleur du formulaire de balise ouvert.
    PickerCouleurPourBalise,
    /// Ouvre la roue pour une couleur de callout : le style en cours ou un
    /// préréglage des paramètres.
    PickerCouleurPourCallout(crate::app::state::CibleCouleur),
    /// Le choix : la couleur devient la dernière enregistrée — rien
    /// n'est inséré tant qu'on ne l'applique pas (bug-report-10, 5.4).
    PickerCouleurEnregistrer,
    PickerCouleurAppliquer,
    /// Le code hex est saisi au clavier — il pilote la roue.
    PickerCouleurHex(String),
    /// Une couleur rapide s'applique tout de suite, la roue reste ouverte.
    PickerCouleurRapide(String),

    /// Ouvrir la corbeille dans l'écran de récupération.
    CorbeilleOuvrir,
    /// Fermer l'écran de la corbeille.
    CorbeilleFermer,
    /// Choisir une entrée de la corbeille.
    CorbeilleChoisir(usize),
    /// Restaurer l'entrée choisie à son emplacement d'origine.
    CorbeilleRestaurer,
    /// Ouvrir le dossier de la corbeille dans le gestionnaire de fichiers.
    CorbeilleDossier,

    RechercheFermer,

    RechercheMotif(String),
    RechercheBasculerMode,
    RechercheBasculerPortee,
    /// Ouvrir un résultat de recherche projet à sa position.
    RechercheProjetOuvrir(String, usize, usize),
    /// Les résultats d'une recherche projet, avec la génération demandée :
    /// une réponse tardive ne remplace jamais la frappe courante, ni les
    /// résultats d'un autre projet.
    RechercheProjetResultats(
        std::path::PathBuf,
        u64,
        Result<Vec<crate::app::recherche::ResultatProjet>, String>,
    ),

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
    /// Ouvrir les paramètres des plugins (bibliographie, biblique,
    /// glossaire, graphiques) séparément des normes.
    PluginsOuvrir,

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
    SystemTheme(crate::design::theme::SystemMode),

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

impl Message {
    /// Le plugin propriétaire du message, s'il y en a un.
    ///
    /// La porte d'activation s'appuie dessus : un add-on désactivé ne reçoit
    /// plus aucun de ses messages, même si une vue ancienne en émet encore.
    pub fn plugin(&self) -> Option<&'static str> {
        match self {
            Message::GlossaireReplier
            | Message::GlossaireRechercheBascule
            | Message::GlossaireFiltrer(_)
            | Message::GlossaireSectionInserer
            | Message::GlossaireInserer(_)
            | Message::GlossaireFenetreOuvrir
            | Message::GlossaireFenetreFermer
            | Message::GlossaireCreer
            | Message::GlossaireModifier(_)
            | Message::GlossaireSupprimer(_)
            | Message::SurvolGlossaire(_)
            | Message::EditionGlossaireChamp(_, _)
            | Message::EditionGlossaireEnregistrer
            | Message::EditionGlossaireFermer => Some(crate::base::valeurs::CLE_GLOSSAIRE),

            Message::ReferenceBibliqueInserer
            | Message::BiblePanneauRecherche(_)
            | Message::BiblePanneauLivre(_)
            | Message::BiblePanneauChapitre(_)
            | Message::BiblePanneauVersets(_)
            | Message::BiblePanneauTexte(_)
            | Message::BiblePanneauAjouter
            | Message::BiblePanneauRetirer(_)
            | Message::BiblePanneauForme(_)
            | Message::BiblePanneauInserer
            | Message::BiblePanneauFermer
            | Message::BibleRechercheBascule
            | Message::BibleFiltrer(_)
            | Message::BibleTri(_)
            | Message::BibleAllerReference(_)
            | Message::BibleTraduction(_)
            | Message::BibleVersionImporter
            | Message::BibleVersionImported(_)
            | Message::BibleVersionInstallee(_)
            | Message::BibleVersionDossier
            | Message::BibleVersionSupprimer(_)
            | Message::BibleVersionExporter(_)
            | Message::BibleVersionExportee(_)
            | Message::BibleVersionModele
            | Message::BibleVersionModeleEcrit(_)
            | Message::BibleReferenceModifier(_)
            | Message::BibleEditionLivre(_)
            | Message::BibleEditionChapitre(_)
            | Message::BibleEditionVersets(_)
            | Message::BibleEditionEnregistrer
            | Message::BibleEditionAnnuler
            | Message::BibleIndexInserer
            | Message::StatsReplier => Some(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES),

            Message::ReferencesReplier
            | Message::ReferencesFiltrer(_)
            | Message::BibliographieSectionInserer
            | Message::BibliographieImporter
            | Message::BibliographieCreer
            | Message::BibImporte(_)
            | Message::ReferenceSupprimer(_)
            | Message::ReferencesSupprimerTout
            | Message::ReferenceRestaurer(_)
            | Message::EditionReferenceChamp(_, _)
            | Message::EditionReferenceEnregistrer
            | Message::EditionReferenceNouveau
            | Message::EditionReferenceFermer
            | Message::ReferencesRechercheBascule
            | Message::SurvolReference(_)
            | Message::ModifierReference(_)
            | Message::CitationOuvrir => Some(crate::base::valeurs::CLE_BIBLIOGRAPHIE),

            Message::ChartOuvrir
            | Message::ChartType(_)
            | Message::ChartTitre(_)
            | Message::ChartSousTitre(_)
            | Message::ChartXLabel(_)
            | Message::ChartYLabel(_)
            | Message::ChartCsv(_)
            | Message::ChartImporter
            | Message::ChartImporte(_)
            | Message::ChartLargeur(_)
            | Message::ChartHauteur(_)
            | Message::ChartPlacement(_)
            | Message::ChartLegende(_)
            | Message::ChartValeurs(_)
            | Message::ChartGrille(_)
            | Message::ChartGraduations(_)
            | Message::ChartPoints(_)
            | Message::ChartAire(_)
            | Message::ChartCourbe(_)
            | Message::ChartDonut(_)
            | Message::ChartAnneau(_)
            | Message::ChartHorizontal(_)
            | Message::ChartInserer => Some(crate::base::valeurs::CLE_GRAPHIQUES),

            Message::CalloutOuvrir
            | Message::CalloutType(_)
            | Message::CalloutTitre(_)
            | Message::CalloutChamp(_, _)
            | Message::CalloutBascule(_, _)
            | Message::CalloutReinitialiser
            | Message::CalloutInserer
            | Message::CalloutFermer => Some(crate::base::valeurs::CLE_CALLOUTS),

            _ => None,
        }
    }
}

#[cfg(test)]
mod epreuves_de_plugin {
    use super::*;

    #[test]
    fn chaque_famille_de_messages_a_son_plugin() {
        assert_eq!(
            Message::GlossaireReplier.plugin(),
            Some(crate::base::valeurs::CLE_GLOSSAIRE)
        );
        assert_eq!(
            Message::BiblePanneauInserer.plugin(),
            Some(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
        );
        assert_eq!(
            Message::ReferencesReplier.plugin(),
            Some(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
        );
        assert_eq!(
            Message::CalloutOuvrir.plugin(),
            Some(crate::base::valeurs::CLE_CALLOUTS)
        );
        assert_eq!(Message::ProjetNom(String::new()).plugin(), None);
    }
}
