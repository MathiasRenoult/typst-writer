//! Le projet : un dossier de travail avec un document d'écriture unique, son
//! en-tête Typst et sa bibliographie. Les chapitres ne subsistent que comme
//! archives de compatibilité.
//!
//! ```text
//! ~/.typst-writer/projets/<nom>/
//!   main.typ              l'assemblage et les normes
//!   document.tw           source d'écriture unique
//!   references.bib        références du projet
//!   chapitres/*.typ       anciens fichiers conservés après migration
//! ```
//!
//! Le rendu compile `main.typ` et y inclut en mémoire le corps Typst généré
//! depuis `document.tw`. Les sources de chapitres héritées ne sont donc ni
//! nécessaires à l'assemblage courant, ni supprimées lors de la migration.

use crate::services::stockage::slug;
use std::path::{Path, PathBuf};

/// Un chapitre : un fichier du dossier `chapitres/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    /// Chemin absolu du fichier.
    pub path: PathBuf,

    /// Numéro d'ordre, tiré du préfixe du nom de fichier.
    pub order: u32,

    /// Titre lisible, tiré du nom de fichier sans le préfixe ni l'extension.
    pub title: String,

    /// Nombre de mots du fichier, relevé au moment de l'analyse.
    ///
    /// C'est le repère de progression : il montre d'un coup d'œil les
    /// chapitres écrits et ceux qui restent à faire.
    pub words: usize,

    /// Horodatage de dernière modification (secondes epoch).
    pub modified: u64,

    /// Le chapitre est-il réellement inclus dans `main.typ` ?
    ///
    /// Un fichier présent dans `chapitres/` mais absent de l'assemblage ne
    /// finira pas dans le PDF : c'est le genre d'oubli qu'un panneau doit
    /// signaler plutôt que de laisser découvrir à l'export.
    pub included: bool,
}

/// Un projet de rédaction.
#[derive(Debug, Clone)]
pub struct Project {
    /// Dossier du projet.
    pub dir: PathBuf,

    /// Ce qui n'a pas pu se lire en chargeant le projet — dit à l'écran,
    /// jamais avale. Un fichier corrompu n'est pas « pas de liaison ».
    pub problemes: Vec<String>,

    /// Nom du projet (nom du dossier).
    pub name: String,

    /// Le fichier d'assemblage.
    pub main: PathBuf,

    /// Le fichier d'écriture intermédiaire, s'il a déjà été créé.
    pub document: Option<PathBuf>,

    /// Nombre de mots visibles dans `document.tw`.
    pub document_words: usize,

    /// Les sources de chapitre héritées, conservées pour compatibilité.
    pub chapters: Vec<Chapter>,

    /// La bibliographie, si un fichier `.bib` est présent.
    pub bibliography: Option<PathBuf>,

    /// Nombre d'entrées de la bibliographie.
    pub bibliography_entries: usize,

    /// La collection Zotero à laquelle le projet est lié, s'il l'est.
    pub zotero: Option<crate::zotero::Lien>,
}

impl Project {
    /// Nombre total de mots du projet.
    ///
    /// Le document unique est compté depuis son AST ; les chapitres hérités ne
    /// servent de repli que tant qu'aucun `document.tw` n'est présent.
    pub fn words(&self) -> usize {
        if self.document.is_some() {
            self.document_words
        } else {
            self.chapters.iter().map(|chapter| chapter.words).sum()
        }
    }

    /// Les chapitres qui ne sont pas inclus dans l'assemblage.
    pub fn orphan_chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.chapters.iter().filter(|chapter| !chapter.included)
    }

    /// Ce chemin appartient-il au projet ?
    ///
    /// Un fichier extérieur au projet n'est pas un chapitre : l'afficher dans
    /// l'éditeur le laisserait dehors — ni compté, ni inclus, donc jamais
    /// rendu. C'est la même distinction que celle de [`import_chapter`], vue
    /// depuis l'autre bout : lui décide ce qui entre, celle-ci vérifie ce qui
    /// est dedans.
    pub fn owns(&self, path: &Path) -> bool {
        same_file(path, &self.main)
            || self
                .document
                .as_deref()
                .is_some_and(|document| same_file(document, path))
            || self
                .chapters
                .iter()
                .any(|chapter| same_file(&chapter.path, path))
    }
}

/// Le dossier racine des projets.
pub fn projects_dir() -> PathBuf {
    let dir = super::work_dir().join("projets");

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Liste les projets, du plus récemment modifié au plus ancien.
pub fn list_projects() -> Vec<Project> {
    let Ok(entries) = std::fs::read_dir(projects_dir()) else {
        return Vec::new();
    };

    let mut projects: Vec<Project> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        // Un dossier caché n'est jamais un projet : c'est un travail en cours
        // de création, qui ne paraîtra qu'une fois complet.
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .map(|entry| load(&entry.path()))
        .collect();

    projects.sort_by_key(|project| std::cmp::Reverse(last_modified(project)));

    projects
}

/// Horodatage de la modification la plus récente d'un projet.
fn last_modified(project: &Project) -> u64 {
    let chapter_modified = project
        .chapters
        .iter()
        .map(|chapter| chapter.modified)
        .max()
        .unwrap_or(0);
    let document_modified = project
        .document
        .as_deref()
        .and_then(|path| std::fs::metadata(path).ok())
        .and_then(|metadata| metadata.modified().ok())
        .map(|modified| {
            modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    chapter_modified.max(document_modified)
}

/// Charge la description d'un projet depuis son dossier.
///
/// Ne renvoie jamais d'erreur : un dossier illisible donne un projet vide, ce
/// qui suffit à l'afficher sans faire échouer le démarrage.
pub fn load(dir: &Path) -> Project {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("projet")
        .to_string();

    let main = dir.join("main.typ");
    let document_path = dir.join("document.tw");
    let document = document_path.is_file().then_some(document_path);
    let document_words = document
        .as_deref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|source| count_words(&crate::langage::parse(&source).texte_nu()))
        .unwrap_or(0);

    let included = read_includes(&main);

    let chapters = list_chapters(dir, &included);

    let mut problemes = Vec::new();
    let zotero = crate::zotero::lien_du_projet(dir).unwrap_or_else(|probleme| {
        problemes.push(probleme);
        None
    });

    // Un projet lié vit de sa collection : l'export Zotero est sa
    // bibliothèque. Sans lui, ou sans liaison, la locale fait foi.
    let bibliography = zotero
        .as_ref()
        .map(|_| dir.join("references-zotero.bib"))
        .filter(|path| path.is_file())
        .or_else(|| find_bibliography(dir));

    let bibliography_entries = bibliography
        .as_ref()
        .map(|path| super::parse_bibliography(path).len())
        .unwrap_or(0);

    Project {
        problemes,
        dir: dir.to_path_buf(),
        name,
        main,
        document,
        document_words,
        zotero,
        chapters,
        bibliography,
        bibliography_entries,
    }
}

/// Reprend les chapitres d'un ancien projet dans son fichier `document.tw`.
///
/// Les includes sont lus dans l'ordre de `main.typ`, non dans l'ordre du
/// dossier. Les fichiers d'origine et l'assemblage restent intacts : la
/// migration crée une copie éditable, sans rendre l'ancien projet illisible.
/// Les constructions Typst non reconnues sont conservées en blocs bruts.
pub fn migrate_to_document(project: &Project) -> std::io::Result<PathBuf> {
    migrate_to_document_avec_rapport(project).map(|(chemin, _)| chemin)
}

/// La même migration, avec le rapport de ce qu'elle n'a pas traduit.
///
/// « La migration ne se fait pas en silence » : le rapport existe, et doit
/// remonter jusqu'à celui qui regarde l'écran.
pub fn migrate_to_document_avec_rapport(
    project: &Project,
) -> std::io::Result<(PathBuf, crate::langage::migration::Rapport)> {
    // La réparation de `main.typ` vient **après** la lecture des includes :
    // un ancien modèle peut encore décrire des chapitres, et les retirer
    // avant la migration perdrait leur contenu. Le remplacement n'a lieu
    // qu'une fois le document écrit.
    let path = project.dir.join("document.tw");
    let mut rapport = crate::langage::migration::Rapport {
        elements_convertis: 0,
        blocs_bruts: 0,
        avertissements: Vec::new(),
    };

    // Un document déjà rempli est la source de vérité. Un fichier vide,
    // en revanche, peut être le résidu d'une création antérieure et ne doit pas
    // empêcher la migration des chapitres qui contiennent le travail.
    if path.exists() && !std::fs::read_to_string(&path)?.trim().is_empty() {
        reparer_assemblage_herite(project)?;
        return Ok((path, rapport));
    }

    let mut document = crate::langage::Document::vide();

    for include in read_includes(&project.main) {
        let Some(chapter) = project
            .chapters
            .iter()
            .find(|chapter| relative_to_chapters(&chapter.path) == include)
        else {
            continue;
        };

        let source = std::fs::read_to_string(&chapter.path)?;
        let (converti, rapport_chapitre) =
            crate::langage::migration::importer_typst_avec_rapport(&source);

        rapport.elements_convertis += rapport_chapitre.elements_convertis;
        rapport.blocs_bruts += rapport_chapitre.blocs_bruts;
        rapport
            .avertissements
            .extend(rapport_chapitre.avertissements);
        document.blocs.extend(converti.blocs);
    }

    let serialized = crate::langage::serialiser(&document);
    let temporary = project.dir.join(".document.tw.tmp");

    std::fs::write(&temporary, serialized)?;

    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);

        return Err(error);
    }

    // Les includes ont été lus : le vieux gabarit peut maintenant céder la
    // place à la seule accroche des normes.
    reparer_assemblage_herite(project)?;

    Ok((path, rapport))
}

/// Reconnaît et retire un assemblage de modèle supprimé.
///
/// Les anciens modèles « devoir court » et « travail long » avaient leur
/// contenu (page de titre, sommaire, introduction, bibliographie) dans
/// `main.typ`, puis l'application y ajoutait son accroche. Aujourd'hui, ces
/// éléments viennent exclusivement des normes et du document Markdown.
///
/// Seul un fichier **identique au gabarit d'origine** — accroche comprise —
/// est remplacé : une ligne ajoutée par l'auteur suffit à le préserver. La
/// version remplacée est d'abord copiée dans `sauvegardes/`, jamais perdue.
fn reparer_assemblage_herite(project: &Project) -> std::io::Result<bool> {
    let Some(ancien) = std::fs::read_to_string(&project.main).ok() else {
        return Ok(false);
    };

    if !est_un_ancien_assemblage(&ancien) {
        return Ok(false);
    }

    crate::project::sauvegarder_version(&project.dir, &project.main, &ancien, true)?;

    // Le gabarit aussi : il est généré, et l'ancien peut encore porter la
    // page de titre ou le sommaire que les normes ont depuis désactivés.
    let normes = crate::mise_en_page::charger(&project.dir);
    let entete = crate::mise_en_page::gabarit::generer(&normes);
    crate::services::stockage::ecrire_atomique(
        &project.dir.join(crate::mise_en_page::gabarit::FICHIER),
        &entete,
    )?;

    let nouveau = crate::mise_en_page::gabarit::accroche();
    crate::services::stockage::ecrire_atomique(&project.main, &nouveau)?;

    Ok(true)
}

/// Le texte est-il exactement l'un des assemblages de modèles disparus,
/// éventuellement précédé de l'accroche des normes ?
fn est_un_ancien_assemblage(texte: &str) -> bool {
    let texte = texte.replace("\r\n", "\n");
    let texte = texte.trim_start();
    let accroche = crate::mise_en_page::gabarit::accroche();
    let sans_accroche = texte
        .strip_prefix(accroche.trim_end())
        .unwrap_or(texte)
        .trim();

    [ANCIEN_TRAVAIL_LONG, ANCIEN_DEVOIR_COURT]
        .iter()
        .any(|ancien| sans_accroche == ancien.trim())
}

/// Importe un fichier `.tw` dans le projet sans écraser un texte existant.
pub fn import_writing_document(project: &Project, source: &Path) -> std::io::Result<PathBuf> {
    let destination = project.dir.join("document.tw");

    if same_file(source, &destination) {
        return Ok(destination);
    }

    if destination.exists()
        && std::fs::read_to_string(&destination)
            .map(|content| !content.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "le projet contient déjà un document non vide",
        ));
    }

    std::fs::copy(source, &destination)?;

    Ok(destination)
}

/// Assemblage de compilation du document unique, sans les includes des anciens
/// chapitres. `main.typ` lui-même n'est jamais réécrit.
pub fn assembly_without_chapters(project: &Project, main: &str) -> String {
    let mut assembly = String::with_capacity(main.len());

    for ligne in main.split_inclusive('\n') {
        let is_chapter = parse_include_line(ligne).is_some_and(|include| {
            project
                .chapters
                .iter()
                .any(|chapter| relative_to_chapters(&chapter.path) == include)
        });

        if !is_chapter {
            assembly.push_str(ligne);
        }
    }

    assembly
}

/// Crée un projet vide sans réutiliser un dossier existant.
///
/// Le projet se prépare complet dans un temporaire caché, sous le verrou et
/// sans remplacement, puis paraît d'un seul renommage ; en cas d'échec, le
/// temporaire est conservé pour qu'aucun fichier ne soit supprimé en silence.
pub fn create(name: &str) -> std::io::Result<PathBuf> {
    let (dir, verrou) = create_avec_verrou(name)?;
    drop(verrou);
    Ok(dir)
}

/// Variante utilisée par la migration, qui garde le verrou pendant la reprise.
pub(super) fn create_avec_verrou(name: &str) -> std::io::Result<(PathBuf, super::Verrou)> {
    creer_classique_verrou(name)
}

/// Crée un projet selon l'un des modèles de R5, en un geste.
/// Le cœur : création verrouillée, sur la base classique — un projet
/// vide, ouvert aux modèles que l'auteur s'est faits (demande de
/// l'auteur : les modèles intégrés sont retirés).
fn creer_classique_verrou(name: &str) -> std::io::Result<(PathBuf, super::Verrou)> {
    let parent = projects_dir();
    let (dir, verrou) = super::creer_et_publier(&parent, &slug(name), true, MAIN_TEMPLATE)?;
    let Some(verrou) = verrou else {
        return Err(std::io::Error::other(
            "création sans verrou : le projet n'a pas été publié",
        ));
    };
    std::fs::write(dir.join("document.tw"), DOCUMENT_CLASSIQUE)?;
    Ok((dir, verrou))
}

/// Crée un chapitre et l'ajoute à l'assemblage.
///
/// Le numéro est le premier libre après le plus grand existant : créer un
/// chapitre ne réordonne jamais les précédents, dont les numéros peuvent avoir
/// été choisis à la main.
///
/// Le numéro est calculé à partir du **contenu réel du dossier**, et non de la
/// liste passée par l'appelant : un instantané périmé ferait attribuer deux fois
/// le même numéro à deux créations successives.
pub fn create_chapter(project: &Project, title: &str) -> std::io::Result<PathBuf> {
    let chapters_dir = project.dir.join("chapitres");

    std::fs::create_dir_all(&chapters_dir)?;

    let existing = list_chapters(&project.dir, &read_includes(&project.main));

    let order = next_order(&existing);

    let path = chapters_dir.join(format!("{order:02}-{}.typ", slug(title)));

    if !path.exists() {
        std::fs::write(&path, format!("= {title}\n\n"))?;
    }

    add_include(&project.main, &path)?;

    Ok(path)
}

/// Vérifie la cible avant toute modification de l'assemblage ou du disque.
fn verifier_chapitre_du_projet(project: &Project, path: &Path) -> std::io::Result<()> {
    let racine = project.dir.canonicalize()?;
    let dossier = project.dir.join("chapitres").canonicalize()?;
    let chapitre = path.canonicalize()?;
    let assemblage = project.main.canonicalize()?;
    if dossier.parent() != Some(racine.as_path())
        || chapitre.parent() != Some(dossier.as_path())
        || assemblage.parent() != Some(racine.as_path())
        || path.extension().is_none_or(|extension| extension != "typ")
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Le chapitre ou l'assemblage ne se trouve pas dans le projet courant.",
        ));
    }
    Ok(())
}

/// Retire un chapitre de l'assemblage, puis du disque.
pub fn remove_chapter(project: &Project, path: &Path) -> std::io::Result<std::path::PathBuf> {
    verifier_chapitre_du_projet(project, path)?;
    remove_include(&project.main, path)?;

    // La corbeille, jamais la destruction : un chapitre supprimé par
    // mégarde se retrouve.
    super::vers_la_corbeille(path)
}

/// Le premier numéro de chapitre libre.
fn next_order(chapters: &[Chapter]) -> u32 {
    chapters
        .iter()
        .map(|chapter| chapter.order)
        .max()
        .map(|max| max + 1)
        .unwrap_or(1)
}

/// Le numéro qu'aura le prochain chapitre créé.
///
/// Lu sur le disque, comme [`create_chapter`] : le titre proposé et le nom du
/// fichier doivent annoncer le même numéro.
pub fn next_chapter_order(project: &Project) -> u32 {
    next_order(&list_chapters(&project.dir, &read_includes(&project.main)))
}

/// Liste les chapitres d'un projet, dans l'ordre de numérotation.
fn list_chapters(dir: &Path, included: &[PathBuf]) -> Vec<Chapter> {
    let Ok(entries) = std::fs::read_dir(dir.join("chapitres")) else {
        return Vec::new();
    };

    let mut chapters: Vec<Chapter> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "typ").unwrap_or(false))
        .filter_map(|path| chapter_of(&path, included))
        .collect();

    chapters.sort_by_key(|chapter| (chapter.order, chapter.title.clone()));

    chapters
}

/// Décrit un chapitre à partir de son fichier.
fn chapter_of(path: &Path, included: &[PathBuf]) -> Option<Chapter> {
    let name = path.file_name()?.to_str()?;

    let stem = name.strip_suffix(".typ")?;

    // Le préfixe numérique est optionnel : un fichier nommé sans numéro reste
    // un chapitre valide, il passe simplement en premier.
    let (order, title) = match stem.split_once('-') {
        Some((prefix, rest)) if !rest.is_empty() => match prefix.parse::<u32>() {
            Ok(order) => (order, rest.to_string()),
            Err(_) => (0, stem.to_string()),
        },
        _ => (0, stem.to_string()),
    };

    let metadata = std::fs::metadata(path).ok()?;

    let words = std::fs::read_to_string(path)
        .map(|content| count_words(&content))
        // Un fichier illisible compte zéro mot plutôt que d'empêcher
        // l'affichage du panneau.
        .unwrap_or(0);

    Some(Chapter {
        path: path.to_path_buf(),
        order,
        title: title.replace('-', " "),
        words,
        modified: metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
        included: included.iter().any(|candidate| same_file(candidate, path)),
    })
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Compare deux chemins en tolérant l'un relatif à l'autre.
///
/// Deux chemins qui **existent** se comparent une fois résolus : c'est exact, et
/// cela distingue deux projets différents dont les fichiers portent le même nom.
///
/// Sinon — un chemin relatif écrit dans l'assemblage, ou un fichier pas encore
/// créé — on se rabat sur les deux derniers composants, ce qui suffit et évite
/// d'échouer sur un chemin inexistant. La comparaison porte alors sur le nom du
/// dossier parent **et** sur celui du fichier : deux projets distincts gardent
/// ainsi leurs `main.typ` séparés.
/// Où une étiquette est posée, dans le projet.
///
/// Une étiquette Typst s'écrit `<nom>` : la chercher dans les chapitres suffit.
/// La première ligne qui la porte est la bonne — un nom d'étiquette est unique
/// dans un document, et une seconde occurrence serait une erreur du document, pas
/// de la recherche.
///
/// Ce n'est pas un détour : tinymist **ne la résout pas** hors du fichier ouvert
/// — c'est mesuré —, alors qu'une étiquette vaut pour tout le document qui la
/// compile. Chercher ici rend le geste utile.
pub fn find_label(project: &Project, label: &str) -> Option<(PathBuf, usize)> {
    if label.is_empty() {
        return None;
    }

    let marker = format!("<{label}>");

    for chapter in &project.chapters {
        let Ok(content) = std::fs::read_to_string(&chapter.path) else {
            continue;
        };

        if let Some(line) = content.lines().position(|line| line.contains(&marker)) {
            return Some((chapter.path.clone(), line));
        }
    }

    None
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Retrouve une clé de citation dans `references.bib`, à sa ligne.
///
/// Comme une étiquette, une citation vaut pour tout le document qui la compile,
/// mais elle vit dans un fichier à part. Sauter d'un `@clé` du texte à son
/// entrée dans le `.bib` est ce qui rend la bibliographie atteignable — sans
/// quoi la référence reste un nom qu'on ne peut jamais voir.
pub fn find_bibliography_entry(project: &Project, key: &str) -> Option<(PathBuf, usize)> {
    let path = project.bibliography.as_ref()?;
    let content = std::fs::read_to_string(path).ok()?;

    let line = content.lines().position(|ligne| {
        let ligne = ligne.trim();
        let Some(reste) = ligne.strip_prefix('@') else {
            return false;
        };
        let Some((_, reste)) = reste.split_once('{') else {
            return false;
        };

        reste.split(',').next().map(str::trim) == Some(key)
    })?;

    Some((path.clone(), line))
}

pub fn same_file(a: &Path, b: &Path) -> bool {
    if let (Ok(a), Ok(b)) = (a.canonicalize(), b.canonicalize()) {
        return a == b;
    }

    fn tail(path: &Path) -> (Option<std::ffi::OsString>, std::ffi::OsString) {
        let mut components = path.components().rev();

        let last = components
            .next()
            .map(|c| c.as_os_str().to_os_string())
            .unwrap_or_default();

        let parent = components.next().map(|c| c.as_os_str().to_os_string());

        (parent, last)
    }

    tail(a) == tail(b)
}

/// Lit les chemins inclus par un fichier d'assemblage.
pub(crate) fn read_includes(main: &Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(main) else {
        return Vec::new();
    };

    content.lines().filter_map(parse_include_line).collect()
}

/// Analyse une ligne `#include "chemin"`.
fn parse_include_line(line: &str) -> Option<PathBuf> {
    let trimmed = line.trim();

    // Une ligne commentée n'est pas un include, même si elle en a la forme.
    if trimmed.starts_with("//") {
        return None;
    }

    let rest = trimmed.strip_prefix("#include")?.trim();

    let rest = rest.strip_prefix('"')?;

    // Un chemin d'assemblage est relatif : c'est `typst` qui le résout.
    let (path, _) = rest.split_once('"')?;

    Some(PathBuf::from(path))
}

/// Renomme un chapitre.
///
/// Le préfixe numérique est conservé : l'ordre des chapitres est une décision
/// de structure, pas une conséquence du titre. L'assemblage est mis à jour
/// **sur place**, à la même ligne — retirer puis ré-ajouter l'include le
/// déplacerait en fin d'assemblage et changerait l'ordre du document.
///
/// Renvoie le nouveau chemin.
pub fn rename_chapter(project: &Project, path: &Path, title: &str) -> std::io::Result<PathBuf> {
    verifier_chapitre_du_projet(project, path)?;
    let order = project
        .chapters
        .iter()
        .find(|chapter| chapter.path == path)
        .map(|chapter| chapter.order)
        .unwrap_or(0);

    let target = path.with_file_name(format!("{order:02}-{}.typ", slug(title)));

    if target == path {
        return Ok(target);
    }

    // Deux chapitres du même nom seraient une confusion garantie dans le
    // document comme dans le panneau : on refuse le titre plutôt que de créer
    // un quasi-doublon.
    let wanted = slug(title);

    let already_used = project
        .chapters
        .iter()
        .filter(|chapter| chapter.path != path)
        .any(|chapter| slug(&chapter.title) == wanted);

    if already_used {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("un chapitre s'appelle déjà « {title} »"),
        ));
    }

    // Garde de dernier recours : un fichier homonyme présent sur le disque.
    if target.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{} existe déjà", target.display()),
        ));
    }

    std::fs::rename(path, &target)?;

    replace_include(&project.main, path, &target)?;

    Ok(target)
}

/// Remplace le chemin d'un `#include` par un autre, à la même position.
fn replace_include(main: &Path, from: &Path, to: &Path) -> std::io::Result<()> {
    let Ok(content) = std::fs::read_to_string(main) else {
        return Ok(());
    };

    let old = relative_to_chapters(from);
    let new = relative_to_chapters(to);

    let mut changed = false;

    let rebuilt: Vec<String> = content
        .lines()
        .map(|line| {
            if parse_include_line(line).as_ref() == Some(&old) {
                changed = true;

                // L'indentation et le reste de la ligne sont conservés : seule
                // la chaîne du chemin change.
                line.replace(&old.display().to_string(), &new.display().to_string())
            } else {
                line.to_string()
            }
        })
        .collect();

    if !changed {
        return Ok(());
    }

    let mut text = rebuilt.join("\n");

    if content.ends_with('\n') {
        text.push('\n');
    }

    std::fs::write(main, text)
}

/// Ajoute un chapitre à un assemblage, s'il n'y est pas déjà.
///
/// Exposé pour la reprise des documents existants, qui doivent être inscrits
/// dans un assemblage sans passer par la création d'un chapitre.
pub fn add_chapter_to_assembly(main: &Path, chapter: &Path) -> std::io::Result<()> {
    add_include(main, chapter)
}

/// Ajoute un fichier existant au projet, comme chapitre à part entière.
///
/// Renvoie le chemin du chapitre **dans** le projet — qui n'est pas forcément
/// celui qu'on a donné.
///
/// Un fichier venu d'ailleurs est **copié** dans `chapitres/`, jamais
/// référencé tel quel : un `#include` ne sort pas du dossier du projet, et le
/// fichier d'origine de l'utilisateur doit rester intact. Un fichier déjà
/// installé dans `chapitres/` n'est pas recopié — on le prend où il est.
///
/// Dans les deux cas l'assemblage est mis à jour : sans cela, le chapitre
/// s'afficherait dans l'éditeur sans jamais être rendu.
///
/// Importer deux fois le même fichier crée bien deux chapitres. Rapprocher les
/// deux demanderait de comparer les contenus, et deux chapitres peuvent
/// légitimement être identiques — un fichier vide, par exemple. Mieux vaut un
/// doublon visible, qu'on supprime, qu'un ajout silencieusement ignoré, qui
/// laisserait croire à une panne.
#[cfg(test)]
pub fn import_chapter(project: &Project, source: &Path) -> std::io::Result<PathBuf> {
    let chapters_dir = project.dir.join("chapitres");

    std::fs::create_dir_all(&chapters_dir)?;

    // Déjà un chapitre de ce projet : rien à copier. Le rendre atteignable,
    // en revanche, est le but même de l'opération.
    if source
        .parent()
        .map(|parent| same_file(parent, &chapters_dir))
        .unwrap_or(false)
    {
        add_include(&project.main, source)?;

        return Ok(source.to_path_buf());
    }

    let content = std::fs::read_to_string(source)?;
    let title = imported_title(source, &content);

    let existing = list_chapters(&project.dir, &read_includes(&project.main));
    let order = next_order(&existing);

    let destination = free_chapter_path(&chapters_dir, order, &slug(&title));

    std::fs::copy(source, &destination)?;

    add_include(&project.main, &destination)?;

    Ok(destination)
}

/// Le titre à donner à un fichier importé.
///
/// Le titre du document — sa première ligne de niveau 1 — passe avant le nom
/// du fichier : un texte venu d'ailleurs porte son propre titre, et le panneau
/// doit annoncer ce que le texte annonce. Le préfixe numérique du nom de
/// fichier n'est pas un titre, et le reprendre tel quel donnerait « 03-mon
/// article » dans le panneau.
#[cfg(test)]
fn imported_title(source: &Path, content: &str) -> String {
    let heading = content.lines().find_map(|line| {
        // Un titre de niveau 1 seulement : `==` est un sous-titre, et
        // l'entête du document n'est pas toujours sur la première ligne.
        let title = line.trim().strip_prefix("= ")?.trim();

        (!title.is_empty()).then(|| title.to_string())
    });

    heading.unwrap_or_else(|| {
        source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("chapitre")
            .to_string()
    })
}

/// Un chemin de chapitre libre, pour ce numéro et ce titre.
///
/// Le numéro est déjà le premier libre, donc une collision suppose un fichier
/// que l'analyse n'a pas su lire comme un chapitre. On ne veut pas pour autant
/// écraser un fichier existant parce qu'il porte le même nom.
#[cfg(test)]
fn free_chapter_path(dir: &Path, order: u32, slug: &str) -> PathBuf {
    let mut path = dir.join(format!("{order:02}-{slug}.typ"));

    let mut suffix = 2;

    while path.exists() {
        path = dir.join(format!("{order:02}-{slug}-{suffix}.typ"));

        suffix += 1;
    }

    path
}

/// Ajoute un `#include` à l'assemblage s'il n'y est pas déjà.
fn add_include(main: &Path, chapter: &Path) -> std::io::Result<()> {
    let relative = relative_to_chapters(chapter);

    if read_includes(main).iter().any(|path| path == &relative) {
        return Ok(());
    }

    let mut content = std::fs::read_to_string(main).unwrap_or_else(|_| MAIN_TEMPLATE.to_string());
    let include = format!("#include \"{}\"", relative.display());

    if let Some(offset) = bibliography_offset(&content) {
        let mut updated = String::with_capacity(content.len() + include.len() + 3);
        let header = &content[..offset];
        updated.push_str(header);

        if !header.is_empty() && !header.ends_with('\n') {
            updated.push('\n');
        }

        updated.push_str(&include);
        updated.push_str("\n\n");
        updated.push_str(&content[offset..]);
        content = updated;
    } else {
        if !content.ends_with('\n') {
            content.push('\n');
        }

        content.push_str(&format!("\n{include}\n"));
    }

    std::fs::write(main, content)
}

/// Renvoie le début de la directive bibliographique non commentée, s'il y en a
/// une, afin que les nouveaux chapitres restent avant elle dans l'assemblage.
fn bibliography_offset(main: &str) -> Option<usize> {
    let mut offset = 0;

    for source_line in main.split_inclusive('\n') {
        if is_bibliography_line(source_line) {
            return Some(offset);
        }

        offset += source_line.len();
    }

    None
}

/// Une ligne porte-t-elle une directive bibliographique Typst, hors commentaire ?
fn is_bibliography_line(source_line: &str) -> bool {
    let line = source_line.trim_start();

    !line.starts_with("//")
        && line
            .strip_prefix("#bibliography")
            .is_some_and(|remainder| remainder.trim_start().starts_with('('))
}

/// Retire le `#include` correspondant à un chapitre.
///
/// Retire aussi le séparateur posé par `add_include` — avant l'include dans les
/// anciens assemblages, après lui lorsqu'il est placé avant la bibliographie —
/// pour qu'un ajout suivi d'un retrait restaure le fichier à l'identique.
fn remove_include(main: &Path, chapter: &Path) -> std::io::Result<()> {
    let Ok(content) = std::fs::read_to_string(main) else {
        return Ok(());
    };

    let wanted = relative_to_chapters(chapter);

    let lines: Vec<&str> = content.lines().collect();

    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());
    let mut skip_bibliography_separator = false;

    for (index, line) in lines.iter().enumerate() {
        if skip_bibliography_separator {
            skip_bibliography_separator = false;
            continue;
        }

        if parse_include_line(line).as_ref() == Some(&wanted) {
            let bibliography_follows = lines
                .get(index + 1)
                .is_some_and(|line| line.trim().is_empty())
                && lines
                    .get(index + 2)
                    .is_some_and(|line| is_bibliography_line(line));

            if bibliography_follows {
                // `add_include` a placé ce séparateur après l'include.
                skip_bibliography_separator = true;
            } else if index > 0 && lines[index - 1].trim().is_empty() {
                // Dans l'assemblage historique, le séparateur précédait l'include.
                let _ = kept.pop();
            }

            continue;
        }

        kept.push(line);
    }

    let mut rebuilt = kept.join("\n");

    if content.ends_with('\n') {
        rebuilt.push('\n');
    }

    std::fs::write(main, rebuilt)
}

/// Chemin d'un chapitre tel qu'il doit apparaître dans l'assemblage.
fn relative_to_chapters(chapter: &Path) -> PathBuf {
    let name = chapter
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("chapitre.typ"));

    PathBuf::from("chapitres").join(name)
}

/// Branche les bibliothèques du projet dans l'appel `#bibliography` **déjà
/// présent** dans le document, et renvoie le document corrigé.
///
/// `None` quand il n'y a pas d'appel — l'insertion reste un geste de
/// l'auteur — ou quand l'appel porte déjà ces bibliothèques.
///
/// Le lien Zotero écrit `references-zotero.bib` : tant que le document ne
/// le cite pas, ses citations sont introuvables — et un fichier cité qui
/// manque casse la compilation (bug-report-9).
pub fn brancher_les_bibliographies(dir: &Path, main: &str) -> Option<String> {
    // Les bibliothèques du projet : la locale, puis l'export Zotero.
    let mut liste: Vec<String> = ["references.bib", "references-zotero.bib"]
        .into_iter()
        .filter(|nom| dir.join(nom).is_file())
        .map(str::to_string)
        .collect();

    // Ce que le document cite déjà, vivant seulement : un fichier qui
    // manque casse la compilation, et n'est plus rien à citer.
    for chemin in noms_bibliographies_cites(main) {
        if dir.join(&chemin).is_file() && !liste.contains(&chemin) {
            liste.push(chemin);
        }
    }

    if liste.is_empty() {
        return None;
    }

    let argument = if liste.len() == 1 {
        format!("\"{}\"", liste[0])
    } else {
        format!(
            "({})",
            liste
                .iter()
                .map(|nom| format!("\"{nom}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    // Aucun appel : le document n'a pas demandé de bibliographie. La
    // fonctionnalité peut être active, mais son insertion reste un geste
    // explicite de l'auteur — rien ne s'ajoute dans son dos.
    let (debut, fin) = premier_argument_bibliographie(main)?;

    // L'appel existe : s'il portait déjà cette liste, rien ne bouge — la
    // comparaison se fait sur les noms, jamais sur la forme.
    if noms_dans_le_plage(&main[debut..fin]) == liste {
        return None;
    }

    let mut entry = String::with_capacity(main.len() + argument.len());
    entry.push_str(&main[..debut]);
    entry.push_str(&argument);
    entry.push_str(&main[fin..]);
    Some(entry)
}

/// Les fichiers `.bib` que le texte cite déjà.
fn noms_bibliographies_cites(main: &str) -> Vec<String> {
    premier_argument_bibliographie(main)
        .map(|(debut, fin)| noms_dans_le_plage(&main[debut..fin]))
        .unwrap_or_default()
}

/// Les noms de fichiers écrits dans une plage d'argument.
fn noms_dans_le_plage(plage: &str) -> Vec<String> {
    let mut noms = Vec::new();
    let mut reste = plage;

    while let Some(debut) = reste.find('"') {
        let Some(fin) = reste[debut + 1..].find('"') else {
            break;
        };

        noms.push(reste[debut + 1..debut + 1 + fin].to_string());
        reste = &reste[debut + 1 + fin + 1..];
    }

    noms
}

/// L'appel `#bibliography(...)` du texte : la plage de son premier
/// argument — celui qui nomme les fichiers.
fn premier_argument_bibliographie(main: &str) -> Option<(usize, usize)> {
    let tete = main.find("#bibliography")?;
    let ouverture = tete + main[tete..].find('(')?;
    let mut profondeur = 0usize;
    let mut guillemets = false;
    let mut echappe = false;

    for (relatif, caractere) in main[ouverture + 1..].char_indices() {
        if guillemets {
            if echappe {
                echappe = false;
            } else if caractere == '\\' {
                echappe = true;
            } else if caractere == '"' {
                guillemets = false;
            }

            continue;
        }

        match caractere {
            '"' => guillemets = true,
            '(' | '[' | '{' => profondeur += 1,
            ')' | ']' | '}' if profondeur == 0 => {
                return Some((ouverture + 1, ouverture + 1 + relatif));
            }
            ')' | ']' | '}' => profondeur -= 1,
            ',' if profondeur == 0 => {
                return Some((ouverture + 1, ouverture + 1 + relatif));
            }
            _ => {}
        }
    }

    None
}

pub fn find_bibliography(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;

    let mut bibliotheques: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map(|ext| ext == "bib").unwrap_or(false))
        .collect();

    // L'ordre de `read_dir` est celui du disque : une bibliothèque ne s'y
    // prend jamais. La locale d'abord, l'export Zotero ensuite — c'est le
    // projet qui dit laquelle vit.
    bibliotheques.sort_by_key(|path| match path.file_name().and_then(|nom| nom.to_str()) {
        Some("references.bib") => 0,
        Some("references-zotero.bib") => 1,
        _ => 2,
    });

    bibliotheques.into_iter().next()
}

/// Nombre de mots d'un contenu.
///
/// Compte les mots du source, balisage compris — c'est la même règle que la
/// barre d'état, donc les deux chiffres s'accordent. Tenir compte réellement de
/// la syntaxe Typst demanderait de l'analyser.
pub fn count_words(content: &str) -> usize {
    content
        .split_whitespace()
        // Un **mot** doit contenir au moins une lettre ou un chiffre. Sans
        // cette règle, les marqueurs de balisage étaient comptés comme des
        // mots : le `=` d'un titre en ajoutait un à chaque chapitre, si bien
        // qu'un projet de vingt chapitres affichait vingt mots de trop.
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .count()
}

/// Assemblage d'un nouveau projet.
/// Le document d'écriture du projet classique : le vide accueillant.
pub const DOCUMENT_CLASSIQUE: &str = "";

/// Ancien assemblage du modèle « travail long », supprimé depuis.
///
/// Il est conservé ici uniquement pour reconnaître, **mot pour mot**, un
/// `main.typ` resté sur le disque après la disparition des modèles
/// intégrés. Le remplacer est sûr : ce n'est pas une modification de
/// l'auteur, c'est le gabarit d'origine.
const ANCIEN_TRAVAIL_LONG: &str = r#"// Métadonnées : les vôtres. Aucune institution n'est imposée — les normes
// du projet (mise-en-page.toml) s'appliquent par-dessus.
#let titre = "Titre du travail"
#let auteur = "Prénom Nom"
#let date = "Date de remise"

#set text(font: "New Computer Modern", size: 11pt)

// Page de titre
#page(margin: 2.5cm, header: none, footer: none)[
  #v(1fr)
  #text(size: 1.6em, weight: "bold")[#titre]
  #v(2cm)
  #auteur
  #linebreak()
  #date
  #v(1fr)
]

// Préliminaires : sommaire à numérotation romaine.
#set page(margin: 2.5cm, numbering: "i", header: none)
#outline(title: "Sommaire", depth: 3)

// Le corps : numérotation arabe, en-tête courant — le moteur résout titre
// et auteur à chaque page ; jamais un champ que le moteur ne saurait lire.
// Les chapitres sont numérotés : un renvoi de titre montre son numéro
// (Typst refuse de référencer un titre sans numérotation).
#set heading(numbering: "1.1")
#counter(page).update(1)
#set page(numbering: "1", header: context [
  #titre #h(1fr) #auteur
])

// L'assemblage : les chapitres sont inclus ici, dans l'ordre.
// Ajouter un chapitre depuis le panneau de travail écrit la ligne pour vous.

= Introduction

// Bibliographie
#bibliography("references.bib", style: "chicago-notes", title: "Bibliographie")
"#;

/// Ancien assemblage du modèle « devoir court », supprimé depuis.
///
/// Il est conservé ici uniquement pour reconnaître, **mot pour mot**, un
/// `main.typ` resté sur le disque après la disparition des modèles
/// intégrés. Le remplacer est sûr : ce n'est pas une modification de
/// l'auteur, c'est le gabarit d'origine.
const ANCIEN_DEVOIR_COURT: &str = r#"// Métadonnées : les vôtres. Aucune institution n'est imposée — les normes
// du projet (mise-en-page.toml) s'appliquent par-dessus.
#let titre = "Titre du devoir"
#let auteur = "Prénom Nom"
#let matiere = "Matière"
#let date = "Date de remise"

#set page(margin: 2.5cm, numbering: "1")
#set text(font: "New Computer Modern", size: 11pt)

// Page de titre
#page(header: none, footer: none)[
  #v(1fr)
  #text(size: 1.5em, weight: "bold")[#titre]
  #v(0.5cm)
  #text(size: 1.1em)[#matiere]
  #v(2cm)
  #auteur
  #linebreak()
  #date
  #v(1fr)
]

// Sommaire
#outline(title: "Sommaire", depth: 2)

// L'assemblage : les chapitres sont inclus ici, dans l'ordre.
// Ajouter un chapitre depuis le panneau de travail écrit la ligne pour vous.

= Introduction

= Développement

= Conclusion

// Bibliographie
#bibliography("references.bib", style: "chicago-notes", title: "Bibliographie")
"#;

pub const MAIN_TEMPLATE: &str = r#"#set page(margin: 2cm)
#set text(font: "New Computer Modern", size: 11pt)

// L'assemblage : les chapitres sont inclus ici, dans l'ordre.
// Ajouter un chapitre depuis le panneau de travail écrit la ligne pour vous.
"#;

pub const BIBLIOGRAPHY_TEMPLATE: &str = r#"% Une entrée par référence, au format BibTeX.
% La clé (ici « exemple2026 ») est ce qui se cite dans le texte : @exemple2026.
%
% Les commentaires d'un fichier .bib commencent par % — BibLaTeX refuse
% tout autre, et le document entier avec.
%
% @article{exemple2026,
%   title = {Un titre},
%   author = {Nom, Prénom},
%   year = {2026},
% }
"#;

#[cfg(test)]
mod tests {

    /// Un projet lié lit sa collection : l'export Zotero est sa bibliographie,
    /// et la locale ne prime plus.
    #[test]
    fn un_projet_lie_lit_sa_collection() {
        let dir = std::env::temp_dir().join(format!("tw-lie-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        std::fs::write(dir.join("main.typ"), "").expect("main");
        std::fs::write(
            dir.join("references.bib"),
            "@book{local,\n  title = {Local},\n}\n",
        )
        .expect("locale");
        std::fs::write(
            dir.join("references-zotero.bib"),
            "@book{zotero,\n  title = {De Zotero},\n}\n",
        )
        .expect("zotero");

        crate::zotero::lier(
            &dir,
            &crate::zotero::Lien {
                cle: "AAAA1111".to_string(),
                nom: "Ma thèse".to_string(),
            },
        )
        .expect("liaison");

        let projet = load(&dir);
        assert_eq!(
            projet.bibliography.as_deref(),
            Some(dir.join("references-zotero.bib").as_path()),
            "le projet lié lit sa collection"
        );
        assert_eq!(
            projet.zotero.as_ref().map(|l| l.nom.as_str()),
            Some("Ma thèse")
        );

        // Délié, il retrouve sa bibliothèque locale.
        crate::zotero::delier(&dir);
        let projet = load(&dir);
        assert!(projet.zotero.is_none());
        assert_eq!(
            projet.bibliography.as_deref(),
            Some(dir.join("references.bib").as_path())
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::*;

    /// Un dossier de projet temporaire, isolé par test.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("typst-writer-projet-{name}"));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Une étiquette se retrouve dans les chapitres, et c'est sa ligne qui compte.
    #[test]
    fn a_label_is_found_where_it_is_written() {
        let dir = scratch("etiquettes");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(
            dir.join("main.typ"),
            "#include \"chapitres/01-intro.typ\"\n#include \"chapitres/02-suite.typ\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("chapitres/01-intro.typ"),
            "= Introduction <intro>\n\nLe début.\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("chapitres/02-suite.typ"),
            "Une phrase.\n\n== Une section <suite>\n\nEncore.\n",
        )
        .unwrap();

        let project = load(&dir);

        // Le titre qui porte l'étiquette est en tête de fichier…
        assert_eq!(
            find_label(&project, "intro"),
            Some((dir.join("chapitres/01-intro.typ"), 0))
        );

        // …et une étiquette au milieu du chapitre donne sa ligne, pas la première.
        assert_eq!(
            find_label(&project, "suite"),
            Some((dir.join("chapitres/02-suite.typ"), 2))
        );

        // Une étiquette qui n'existe pas ne mène nulle part.
        assert_eq!(find_label(&project, "absente"), None);
    }

    /// Une étiquette vide ne doit rien matcher — le marqueur `<>` pourrait
    /// apparaître dans du contenu sans signification.
    #[test]
    fn une_etiquette_vide_ne_matche_rien() {
        let project = Project {
            problemes: Vec::new(),
            dir: PathBuf::new(),
            name: String::new(),
            main: PathBuf::new(),
            document: None,
            document_words: 0,
            chapters: vec![],
            bibliography: None,
            bibliography_entries: 0,
            zotero: None,
        };
        assert_eq!(find_label(&project, ""), None);
    }

    #[test]
    fn slug_makes_a_usable_file_name() {
        assert_eq!(slug("Introduction"), "introduction");
        assert_eq!(slug("Le rôle de l'État"), "le-role-de-l-etat");
        assert_eq!(slug("Chapitre 1 — Ouverture"), "chapitre-1-ouverture");

        // Un titre sans caractère exploitable ne doit pas produire un nom vide.
        assert_eq!(slug("///"), "chapitre");
        assert_eq!(slug(""), "chapitre");
    }

    /// Une clé de citation se retrouve dans `references.bib`, à sa ligne.
    #[test]
    fn a_bibliography_key_is_found_at_its_line() {
        let dir = scratch("bibliographie");
        let bib = dir.join("references.bib");

        std::fs::write(
            &bib,
            "@article{dupont2023,\n  title = {Un titre},\n  author = {Dupont, Alice},\n}\n\n@book{intro,\n  title = {Introduction},\n}\n",
        )
        .unwrap();

        let project = load(&dir);

        assert_eq!(
            find_bibliography_entry(&project, "dupont2023"),
            Some((bib.clone(), 0))
        );
        assert_eq!(
            find_bibliography_entry(&project, "intro"),
            Some((bib.clone(), 5))
        );
        assert_eq!(find_bibliography_entry(&project, "absente"), None);
    }

    /// Un identifiant de fichier ne doit jamais contenir d'espace ni de
    /// ponctuation : c'est ce qui évite les caractères à échapper dans
    /// l'assemblage.
    #[test]
    fn slug_output_is_safe_for_a_path() {
        for title in ["Le rôle de l'État", "Chapitre 1 — Ouverture", "a/b\\c:d"] {
            let slug = slug(title);

            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "« {slug} » contient un caractère à risque"
            );
        }
    }

    #[test]
    fn include_lines_are_parsed_and_ignored_when_malformed() {
        assert_eq!(
            parse_include_line("#include \"chapitres/01-intro.typ\""),
            Some(PathBuf::from("chapitres/01-intro.typ"))
        );
        assert_eq!(
            parse_include_line("  #include \"a/b.typ\"  "),
            Some(PathBuf::from("a/b.typ"))
        );

        // Ce qui n'est pas un include ne doit pas être retenu.
        assert_eq!(parse_include_line("= Titre"), None);
        assert_eq!(parse_include_line("// #include pas un vrai"), None);
        assert_eq!(parse_include_line("#include sans guillemets"), None);
    }

    /// L'ordre des chapitres suit le préfixe numérique, et un fichier sans
    /// préfixe reste un chapitre valide.
    #[test]
    fn chapters_are_listed_in_numeric_order() {
        let dir = scratch("order");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();

        for name in ["02-deux.typ", "10-dix.typ", "01-un.typ", "sans-numero.typ"] {
            std::fs::write(dir.join("chapitres").join(name), "un mot\n").unwrap();
        }

        let project = load(&dir);

        let titles: Vec<&str> = project
            .chapters
            .iter()
            .map(|chapter| chapter.title.as_str())
            .collect();

        assert_eq!(titles, vec!["sans numero", "un", "deux", "dix"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le compte de mots est la somme des chapitres, jamais celle de
    /// l'assemblage — sinon tout serait compté deux fois.
    #[test]
    fn project_word_count_sums_the_chapters() {
        let dir = scratch("words");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "#include \"chapitres/01-un.typ\"\n").unwrap();
        std::fs::write(dir.join("chapitres/01-un.typ"), "un deux trois\n").unwrap();
        std::fs::write(dir.join("chapitres/02-deux.typ"), "quatre cinq\n").unwrap();

        assert_eq!(load(&dir).words(), 5, "3 + 2 mots");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le document intermédiaire est rattaché au projet, compté sans balisage
    /// et reconnu comme un fichier appartenant au projet.
    #[test]
    fn a_project_loads_and_counts_its_writing_document() {
        let dir = scratch("writing-document");
        let path = dir.join("document.tw");
        std::fs::write(&path, "# Titre\n\nDu **gras**.\n").unwrap();

        let project = load(&dir);

        assert_eq!(project.document.as_deref(), Some(path.as_path()));
        assert_eq!(project.document_words, 3);
        assert_eq!(project.words(), 3);
        assert!(project.owns(&path));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une compilation du document unique omet les anciens includes de
    /// chapitres, mais conserve les includes qui ne sont pas des chapitres.
    #[test]
    fn assembly_for_the_writing_document_skips_legacy_chapters() {
        let dir = scratch("assembly-with-document");
        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("chapitres/01-intro.typ"), "= Ancien\n").unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();

        let project = load(&dir);
        let main = "#include \"entete.typ\"\n#include \"chapitres/01-intro.typ\"\n#bibliography(\"references.bib\")\n";
        let result = assembly_without_chapters(&project, main);

        assert_eq!(
            result,
            "#include \"entete.typ\"\n#bibliography(\"references.bib\")\n"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un `main.typ` resté sur un ancien modèle est ramené à l'accroche des
    /// normes : ni page de titre, ni sommaire, ni introduction cachée.
    #[test]
    fn un_ancien_assemblage_de_modele_est_remplace() {
        let dir = scratch("ancien-assemblage");
        let accroche = crate::mise_en_page::gabarit::accroche();
        let ancien = format!("{accroche}{ANCIEN_TRAVAIL_LONG}");
        std::fs::write(dir.join("main.typ"), ancien).unwrap();
        std::fs::write(dir.join("document.tw"), "## Introduction\n").unwrap();

        let project = load(&dir);
        let _ = migrate_to_document_avec_rapport(&project).unwrap();

        let main = std::fs::read_to_string(dir.join("main.typ")).unwrap();
        assert_eq!(main, accroche, "seule l'accroche des normes demeure");
        assert!(
            !main.contains("Page de titre"),
            "la page de titre est partie"
        );
        assert!(!main.contains("#outline"), "le sommaire est parti");
        assert!(!main.contains("= Introduction"), "le titre caché est parti");
        assert!(
            !main.contains("#bibliography"),
            "la bibliographie cachée est partie"
        );

        let copies: Vec<_> = std::fs::read_dir(dir.join("sauvegardes"))
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            copies.iter().any(|nom| nom.starts_with("main.typ.")),
            "l'ancien assemblage est conservé : {copies:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Après le retrait du modèle, aucun style de citation n'est en conflit :
    /// le document ne contient plus de directive que les normes n'ont pas
    /// demandée.
    #[test]
    fn un_ancien_assemblage_ne_provoque_plus_de_conflit_de_style() {
        let dir = scratch("ancien-assemblage-style");
        let accroche = crate::mise_en_page::gabarit::accroche();
        std::fs::write(
            dir.join("main.typ"),
            format!("{accroche}{ANCIEN_TRAVAIL_LONG}"),
        )
        .unwrap();
        std::fs::write(dir.join("document.tw"), "## Introduction\n").unwrap();
        std::fs::write(
            dir.join("mise-en-page.toml"),
            "[bibliographie]\nstyle = \"apa\"\n",
        )
        .unwrap();

        let project = load(&dir);
        let _ = migrate_to_document_avec_rapport(&project).unwrap();
        let main = std::fs::read_to_string(dir.join("main.typ")).unwrap();
        let document = crate::langage::parse("## Introduction\n");
        let (_, _, rapports) = crate::langage::pipeline::compilation_unit_document_avec_diagnostics(
            &dir, &main, &document,
        );

        assert!(
            rapports.is_empty(),
            "aucun conflit de style ne doit rester : {rapports:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une ligne ajoutée par l'auteur suffit à ne pas reconnaître le modèle :
    /// on ne réécrit jamais un assemblage personnalisé.
    #[test]
    fn un_ancien_assemblage_modifie_par_l_auteur_est_preserve() {
        let dir = scratch("ancien-assemblage-modifie");
        let accroche = crate::mise_en_page::gabarit::accroche();
        let personnalise = format!("{accroche}{ANCIEN_TRAVAIL_LONG}\n// Mon ajout personnel.\n");
        std::fs::write(dir.join("main.typ"), &personnalise).unwrap();
        std::fs::write(dir.join("document.tw"), "## Introduction\n").unwrap();

        let project = load(&dir);
        let _ = migrate_to_document_avec_rapport(&project).unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.join("main.typ")).unwrap(),
            personnalise,
            "un fichier personnalisé reste intact"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un fichier `.tw` externe rejoint un projet vide, mais ne remplace jamais
    /// un document qui contient déjà du texte.
    #[test]
    fn importing_a_writing_document_never_overwrites_work() {
        let project_dir = scratch("import-writing-document");
        let source_dir = scratch("source-writing-document");
        let source = source_dir.join("source.tw");
        std::fs::write(&source, "# Importé\n").unwrap();
        let project = load(&project_dir);

        let destination = import_writing_document(&project, &source).unwrap();

        assert_eq!(destination, project_dir.join("document.tw"));
        assert_eq!(
            std::fs::read_to_string(&destination).unwrap(),
            "# Importé\n"
        );

        std::fs::write(&source, "# Remplacement\n").unwrap();
        assert_eq!(
            import_writing_document(&load(&project_dir), &source)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            std::fs::read_to_string(&destination).unwrap(),
            "# Importé\n"
        );

        let _ = std::fs::remove_dir_all(project_dir);
        let _ = std::fs::remove_dir_all(source_dir);
    }

    /// Un fichier Typst importé est converti et ajouté au document unique,
    /// sans remplacer le texte déjà rédigé.
    ///
    /// L'application n'écrit plus l'import sur le disque : il rejoint le
    /// tampon ouvert, prévisualisé avant d'y entrer (voir les épreuves de
    /// l'espace). La conversion elle-même reste éprouvée ici.
    #[test]
    fn importing_typst_appends_to_the_writing_document() {
        let mut document = crate::langage::parse("Texte déjà présent.\n");
        let imported = crate::langage::migration::importer_typst_avec_rapport(
            "= Section importée\n\nTexte *important*.\n",
        )
        .0;
        document.blocs.extend(imported.blocs);
        let relu = crate::langage::parse(&crate::langage::serialiser(&document));

        assert_eq!(
            relu.texte_nu(),
            "Texte déjà présent. Section importée Texte important."
        );
    }

    /// Les anciens chapitres simples sont réunis en respectant l'ordre de
    /// l'assemblage, sans modifier les sources originales.
    #[test]
    fn migrate_chapters_into_one_writing_document() {
        let dir = scratch("migrate-to-document");
        let main = "#set text(lang: \"fr\")\n\n#include \"chapitres/02-suite.typ\"\n\n#include \"chapitres/01-intro.typ\"\n";
        let first = "= Première partie\n\nTexte *important*.\n";
        let second = "== Suite\n\nEncore du texte.\n";

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), main).unwrap();
        std::fs::write(dir.join("chapitres/01-intro.typ"), first).unwrap();
        std::fs::write(dir.join("chapitres/02-suite.typ"), second).unwrap();

        let project = load(&dir);
        let document_path = migrate_to_document(&project).unwrap();
        let content = std::fs::read_to_string(&document_path).unwrap();
        let document = crate::langage::parse(&content);

        assert_eq!(
            document
                .blocs
                .iter()
                .filter_map(|bloc| match bloc {
                    crate::langage::schema::Bloc::Titre { contenu, .. } => {
                        Some(crate::langage::schema::inline_texte_nu(contenu))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec!["Suite", "Première partie"]
        );
        assert_eq!(std::fs::read_to_string(dir.join("main.typ")).unwrap(), main);
        assert_eq!(
            std::fs::read_to_string(dir.join("chapitres/01-intro.typ")).unwrap(),
            first
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("chapitres/02-suite.typ")).unwrap(),
            second
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une destination vide n'empêche pas de récupérer les chapitres d'un
    /// ancien projet ; seul un document non vide est conservé tel quel.
    #[test]
    fn migration_fills_an_existing_empty_writing_document() {
        let dir = scratch("migrate-empty-writing-document");
        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(
            dir.join("main.typ"),
            "#include \"chapitres/01-introduction.typ\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("chapitres/01-introduction.typ"),
            "= Introduction\n\nTexte déjà écrit.\n",
        )
        .unwrap();
        std::fs::write(dir.join("document.tw"), "\n").unwrap();

        let path = migrate_to_document(&load(&dir)).unwrap();
        let document = crate::langage::parse(&std::fs::read_to_string(path).unwrap());

        assert_eq!(document.texte_nu(), "Introduction Texte déjà écrit.");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// La migration ne remplace jamais un document unique déjà rédigé.
    #[test]
    fn migration_preserves_a_nonempty_writing_document() {
        let dir = scratch("migrate-nonempty-writing-document");
        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(
            dir.join("main.typ"),
            "#include \"chapitres/01-ancien.typ\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("chapitres/01-ancien.typ"), "= Archive\n").unwrap();
        std::fs::write(dir.join("document.tw"), "# Source actuelle\n").unwrap();

        let path = migrate_to_document(&load(&dir)).unwrap();

        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            "# Source actuelle\n"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un chapitre absent de l'assemblage doit être signalé : sans quoi il
    /// n'atteindrait jamais le PDF, et l'oubli ne se verrait qu'à l'export.
    #[test]
    fn chapters_missing_from_the_assembly_are_reported() {
        let dir = scratch("orphans");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "#include \"chapitres/01-un.typ\"\n").unwrap();
        std::fs::write(dir.join("chapitres/01-un.typ"), "inclus\n").unwrap();
        std::fs::write(dir.join("chapitres/02-deux.typ"), "oublié\n").unwrap();

        let orphans: Vec<String> = load(&dir)
            .orphan_chapters()
            .map(|chapter| chapter.title.clone())
            .collect();

        assert_eq!(orphans, vec!["deux"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Ajouter puis retirer un chapitre doit laisser l'assemblage exactement
    /// comme avant : c'est ce qui garantit qu'on peut se tromper sans dégât.
    #[test]
    fn adding_and_removing_a_chapter_restores_the_assembly() {
        let dir = scratch("roundtrip");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(
            dir.join("main.typ"),
            format!("{MAIN_TEMPLATE}\n#bibliography(\"references.bib\")\n"),
        )
        .unwrap();

        let before = std::fs::read_to_string(dir.join("main.typ")).unwrap();

        let project = load(&dir);
        let chapter = create_chapter(&project, "Introduction").unwrap();

        let with_chapter = std::fs::read_to_string(dir.join("main.typ")).unwrap();
        assert!(with_chapter.contains("#include \"chapitres/01-introduction.typ\""));
        assert!(
            with_chapter
                .find("#include \"chapitres/01-introduction.typ\"")
                .unwrap()
                < with_chapter.find("#bibliography(").unwrap(),
            "le chapitre doit être inclus avant la bibliographie"
        );
        assert_eq!(load(&dir).chapters.len(), 1);

        remove_chapter(&project, &chapter).unwrap();

        let after = std::fs::read_to_string(dir.join("main.typ")).unwrap();
        assert_eq!(
            after, before,
            "l'assemblage doit revenir à son état initial"
        );
        assert!(!chapter.exists(), "le fichier doit être supprimé");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Ajouter plusieurs chapitres ne doit pas dupliquer les includes.
    #[test]
    fn includes_are_not_duplicated() {
        let dir = scratch("dedup");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).unwrap();

        let project = load(&dir);

        create_chapter(&project, "Unique").unwrap();
        create_chapter(&project, "Autre").unwrap();

        let content = std::fs::read_to_string(dir.join("main.typ")).unwrap();

        assert_eq!(
            content.matches("#include").count(),
            2,
            "un include par chapitre, pas plus"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un projet sans dossier `chapitres/` doit s'afficher vide, sans erreur.
    #[test]
    fn a_project_without_chapters_loads_empty() {
        let dir = scratch("empty");

        let project = load(&dir);

        assert!(project.chapters.is_empty());
        assert_eq!(project.words(), 0);
        assert!(project.bibliography.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// La bibliographie est comptée, pour que le panneau puisse l'afficher.
    #[test]
    fn bibliography_entries_are_counted() {
        let dir = scratch("bib");

        std::fs::write(
            dir.join("references.bib"),
            "@article{un, title = {a}}\n@book{deux, title = {b}}\n",
        )
        .unwrap();

        let project = load(&dir);

        assert!(project.bibliography.is_some());
        assert_eq!(project.bibliography_entries, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un dossier nommé `.bib` n'est pas une bibliographie.
    #[test]
    fn un_dossier_nomme_bib_est_ignore() {
        let dir = scratch("bib-dir");

        std::fs::create_dir_all(dir.join("references.bib")).unwrap();

        let project = load(&dir);

        assert!(project.bibliography.is_none());
        assert_eq!(project.bibliography_entries, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Renommer un chapitre ne doit pas changer l'ordre du document.
    ///
    /// C'est le piège de l'implémentation naïve : retirer l'include puis le
    /// ré-ajouter le déplace en fin d'assemblage, et le chapitre change de
    /// place dans le PDF. L'include doit être réécrit sur place.
    #[test]
    fn renaming_a_chapter_keeps_the_assembly_order() {
        let dir = scratch("rename_order");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).unwrap();

        let project = load(&dir);

        // Trois chapitres, dans l'ordre.
        for title in ["Un", "Deux", "Trois"] {
            create_chapter(&project, title).unwrap();
        }

        let before: Vec<PathBuf> = read_includes(&dir.join("main.typ"));

        // On renomme celui du milieu.
        let middle = dir.join("chapitres/02-deux.typ");
        let renamed = rename_chapter(&load(&dir), &middle, "Introduction").unwrap();

        assert!(
            renamed.ends_with("02-introduction.typ"),
            "obtenu {}",
            renamed.display()
        );
        assert!(!middle.exists(), "l'ancien fichier doit avoir disparu");

        let after: Vec<PathBuf> = read_includes(&dir.join("main.typ"));

        assert_eq!(before.len(), after.len(), "même nombre d'includes");
        assert_eq!(
            after,
            vec![
                PathBuf::from("chapitres/01-un.typ"),
                PathBuf::from("chapitres/02-introduction.typ"),
                PathBuf::from("chapitres/03-trois.typ"),
            ],
            "l'ordre doit être inchangé, seule la deuxième ligne change"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Renommer ne doit pas écraser un chapitre existant.
    #[test]
    fn renaming_onto_an_existing_chapter_fails() {
        let dir = scratch("rename_clash");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).unwrap();

        let project = load(&dir);
        create_chapter(&project, "Un").unwrap();
        let second = create_chapter(&project, "Deux").unwrap();

        let result = rename_chapter(&load(&dir), &second, "Un");

        assert!(result.is_err(), "un titre déjà pris doit être refusé");
        assert!(second.exists(), "le chapitre visé doit rester intact");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le préfixe numérique est conservé : l'ordre est une décision de
    /// structure, indépendante du titre.
    #[test]
    fn renaming_keeps_the_numeric_prefix() {
        let dir = scratch("rename_prefix");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).unwrap();

        let project = load(&dir);
        let chapter = create_chapter(&project, "Chapitre 1").unwrap();

        assert!(chapter.ends_with("01-chapitre-1.typ"));

        let renamed = rename_chapter(&load(&dir), &chapter, "Le rôle de l'État").unwrap();

        assert!(
            renamed.ends_with("01-le-role-de-l-etat.typ"),
            "préfixe attendu 01, obtenu {}",
            renamed.display()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le numéro d'un nouveau chapitre suit le plus grand existant, sans
    /// réordonner les chapitres déjà écrits.
    #[test]
    fn new_chapters_are_numbered_after_the_last_one() {
        let dir = scratch("numbering");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();
        std::fs::write(dir.join("chapitres/07-deja-la.typ"), "x\n").unwrap();

        let project = load(&dir);
        let path = create_chapter(&project, "Suite").unwrap();

        assert!(
            path.to_string_lossy().ends_with("08-suite.typ"),
            "numéro attendu 08, obtenu {}",
            path.display()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
    /// Un fichier venu d'ailleurs est copié dans le projet et rendu atteignable.
    ///
    /// C'est le défaut signalé : le charger ne faisait que l'afficher dans
    /// l'éditeur, sans l'ajouter au projet — donc sans jamais le compter, ni
    /// l'inclure, ni le rendre.
    #[test]
    fn an_outside_file_is_copied_into_the_project() {
        let elsewhere = scratch("import-ailleurs");
        let source = elsewhere.join("mon-article.typ");

        std::fs::write(&source, "= Les réseaux\n\nDu texte.\n").unwrap();

        let dir = scratch("import-projet");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();

        let project = load(&dir);
        let chapter = import_chapter(&project, &source).unwrap();

        // La copie vit dans `chapitres/`, sous un nom tiré du titre du document.
        assert!(
            chapter
                .to_string_lossy()
                .ends_with("chapitres/01-les-reseaux.typ"),
            "chemin obtenu : {}",
            chapter.display()
        );

        assert_eq!(
            std::fs::read_to_string(&chapter).unwrap(),
            "= Les réseaux\n\nDu texte.\n",
            "la copie doit être fidèle"
        );

        // Le fichier d'origine n'est pas touché : importer ne déplace rien.
        assert!(source.exists(), "le fichier d'origine doit rester en place");

        // Et le chapitre est bien inclus dans l'assemblage.
        let reloaded = load(&dir);

        assert_eq!(reloaded.chapters.len(), 1);
        assert_eq!(reloaded.chapters[0].title, "les reseaux");
        assert!(
            reloaded.chapters[0].included,
            "sans inclusion, le chapitre ne serait jamais rendu"
        );
        // « Les réseaux » et « Du texte. » : le `=` du titre n'est pas un mot.
        assert_eq!(reloaded.words(), 4);

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&elsewhere);
    }

    /// Un chapitre déjà installé est inclus, jamais recopié.
    #[test]
    fn an_existing_chapter_is_included_without_being_copied() {
        let dir = scratch("import-deja-la");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();

        // L'assemblage ne mentionne pas le chapitre : il existe sans être rendu.
        std::fs::write(dir.join("main.typ"), "").unwrap();
        std::fs::write(dir.join("chapitres/01-un.typ"), "= Un\n\nTexte.\n").unwrap();

        let project = load(&dir);

        assert!(
            !project.chapters[0].included,
            "le chapitre doit d'abord être orphelin"
        );

        let path = project.chapters[0].path.clone();
        let returned = import_chapter(&project, &path).unwrap();

        assert_eq!(returned, path, "le chapitre ne doit pas être déplacé");

        let reloaded = load(&dir);

        assert_eq!(
            reloaded.chapters.len(),
            1,
            "aucune copie ne doit apparaître"
        );
        assert!(
            reloaded.chapters[0].included,
            "le charger doit le rendre atteignable"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le titre vient du document lui-même, à défaut du nom de fichier.
    #[test]
    fn an_imported_file_takes_its_title_from_its_own_heading() {
        let elsewhere = scratch("import-titres");

        let dir = scratch("import-titres-projet");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();

        let project = load(&dir);

        // Un titre de niveau 1 est le titre du document, même après une entête.
        let with_heading = elsewhere.join("brut.typ");

        std::fs::write(
            &with_heading,
            "#set page(margin: 2cm)\n\n= Le rôle de l'État\n\nTexte.\n",
        )
        .unwrap();

        let chapter = import_chapter(&project, &with_heading).unwrap();

        assert!(
            chapter
                .to_string_lossy()
                .ends_with("01-le-role-de-l-etat.typ"),
            "chemin obtenu : {}",
            chapter.display()
        );

        // Un sous-titre ne fait pas un titre de document : le nom du fichier
        // prend alors le relais.
        let project = load(&dir);
        let only_subheading = elsewhere.join("notes-diverses.typ");

        std::fs::write(&only_subheading, "== Une section\n\nTexte.\n").unwrap();

        let chapter = import_chapter(&project, &only_subheading).unwrap();

        assert!(
            chapter.to_string_lossy().ends_with("02-notes-diverses.typ"),
            "chemin obtenu : {}",
            chapter.display()
        );

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&elsewhere);
    }

    /// L'import se place après les chapitres existants.
    #[test]
    fn an_import_is_numbered_after_the_existing_chapters() {
        let elsewhere = scratch("import-numero-source");
        let source = elsewhere.join("suite.typ");

        std::fs::write(&source, "= Suite\n\nTexte.\n").unwrap();

        let dir = scratch("import-numero");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();
        std::fs::write(dir.join("chapitres/07-deja-la.typ"), "x\n").unwrap();

        let project = load(&dir);
        let chapter = import_chapter(&project, &source).unwrap();

        assert!(
            chapter.to_string_lossy().ends_with("08-suite.typ"),
            "numéro attendu 08, obtenu {}",
            chapter.display()
        );

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&elsewhere);
    }

    /// Deux imports du même fichier donnent deux chapitres.
    ///
    /// Rapprocher les deux demanderait de comparer les contenus, et deux
    /// chapitres peuvent légitimement être identiques — un fichier vide, par
    /// exemple. Un doublon visible se supprime ; un ajout silencieusement
    /// ignoré ferait croire à une panne, ce qui est précisément le défaut qu'on
    /// répare ici.
    #[test]
    fn importing_the_same_file_twice_creates_two_chapters() {
        let elsewhere = scratch("import-double-source");
        let source = elsewhere.join("texte.typ");

        std::fs::write(&source, "= Texte\n\nDu contenu.\n").unwrap();

        let dir = scratch("import-double");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "").unwrap();

        let first = import_chapter(&load(&dir), &source).unwrap();
        let second = import_chapter(&load(&dir), &source).unwrap();

        assert_ne!(first, second);
        assert_eq!(load(&dir).chapters.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&elsewhere);
    }
    /// Les marqueurs de balisage ne sont pas des mots.
    ///
    /// Le `=` d'un titre en ajoutait un à chaque chapitre : sur un projet de
    /// vingt chapitres, cela faisait vingt mots de trop au compteur, et le
    /// chiffre ne voulait plus rien dire.
    #[test]
    fn markup_markers_are_not_words() {
        assert_eq!(count_words("un deux trois"), 3);

        // Le marqueur d'un titre ne compte pas, son texte si.
        assert_eq!(count_words("= Titre"), 1);
        assert_eq!(count_words("= Titre\n\nDu texte."), 3);

        // Les mots accentués sont des mots, apostrophe comprise.
        assert_eq!(count_words("l'État français"), 2);

        // Dans du code, les identifiants comptent, pas les opérateurs.
        assert_eq!(count_words("#let x = 1"), 3);

        // Un texte vide ne compte rien.
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("-- == ~~"), 0);
    }

    /// Deux fichiers réels du même nom, dans deux dossiers différents, ne sont
    /// pas le même fichier.
    ///
    /// La comparaison par les deux derniers composants ne suffisait pas : deux
    /// projets dont le dossier parent porte le même nom auraient été confondus,
    /// et un chapitre aurait pu être rattaché au mauvais projet.
    #[test]
    fn two_real_files_with_the_same_name_are_distinct() {
        let first = scratch("same-file-a");
        let second = scratch("same-file-b");

        std::fs::create_dir_all(first.join("projet")).unwrap();
        std::fs::create_dir_all(second.join("projet")).unwrap();

        std::fs::write(first.join("projet/main.typ"), "").unwrap();
        std::fs::write(second.join("projet/main.typ"), "").unwrap();

        assert!(
            !same_file(
                &first.join("projet/main.typ"),
                &second.join("projet/main.typ")
            ),
            "deux projets distincts ne doivent pas être confondus"
        );

        // Le même fichier, écrit de deux façons, reste le même.
        assert!(same_file(
            &first.join("projet/main.typ"),
            &first.join("projet/./main.typ")
        ));

        // Et la tolérance au chemin relatif, sur des fichiers inexistants, est
        // conservée : c'est le cas de l'assemblage, qui écrit
        // `chapitres/01-un.typ` là où l'analyse produit un chemin absolu.
        assert!(same_file(
            Path::new("chapitres/01-un.typ"),
            &first.join("chapitres/01-un.typ")
        ));

        let _ = std::fs::remove_dir_all(&first);
        let _ = std::fs::remove_dir_all(&second);
    }

    /// Un document extérieur au projet n'est pas un chapitre.
    ///
    /// C'est ce que vérifie le démarrage avant de rouvrir le document de la
    /// session : sans ce contrôle, un chemin venu d'ailleurs — retenu avant
    /// d'être un chapitre, ou déplacé depuis — rouvrirait un fichier fantôme,
    /// affiché sans jamais être rendu.
    #[test]
    fn the_project_owns_its_assembly_and_its_chapters_only() {
        let dir = scratch("owns");

        std::fs::create_dir_all(dir.join("chapitres")).unwrap();
        std::fs::write(dir.join("main.typ"), "#include \"chapitres/01-un.typ\"\n").unwrap();
        std::fs::write(dir.join("chapitres/01-un.typ"), "= Un\n\nTexte.\n").unwrap();

        let project = load(&dir);

        assert!(
            project.owns(&project.main),
            "l'assemblage appartient au projet"
        );
        assert!(project.owns(&dir.join("chapitres/01-un.typ")));
        assert_eq!(project.chapters.len(), 1);

        // Un fichier posé dans le dossier du projet mais hors de `chapitres/`
        // n'est pas un chapitre non plus : l'analyse ne le voit pas.
        assert!(!project.owns(&dir.join("notes.typ")));

        // Ni un fichier d'un autre dossier.
        assert!(!project.owns(std::path::Path::new("/tmp/ailleurs/texte.typ")));

        // Ni l'assemblage d'un autre projet.
        assert!(!project.owns(std::path::Path::new("/tmp/autre-projet/main.typ")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod epreuves_de_modele {
    use super::{BIBLIOGRAPHY_TEMPLATE, DOCUMENT_CLASSIQUE, MAIN_TEMPLATE};

    /// Un dossier de projet fabriqué, sur la base classique. Chaque
    /// appelant a le sien : les tests tournent en parallèle.
    fn projet(essai: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "typst-writer-base-{}-{}",
            essai,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("chapitres")).expect("dossier");
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).expect("assemblage");
        std::fs::write(dir.join("references.bib"), BIBLIOGRAPHY_TEMPLATE).expect("bibliographie");
        std::fs::write(dir.join("document.tw"), DOCUMENT_CLASSIQUE).expect("document");
        let normes = crate::mise_en_page::modele::MiseEnPage {
            page_de_titre: true,
            sommaire: true,
            typographie: crate::mise_en_page::modele::Typographie {
                base: "francaise".to_string(),
                ..Default::default()
            },
            titres: crate::mise_en_page::modele::Titres {
                numerotation: "1.1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let _ = crate::mise_en_page::appliquer(&dir, &normes, true).expect("normes");
        dir
    }

    /// Un projet créé compile réellement — la promesse d’un projet
    /// qui refuse de compiler n’est rien.
    #[test]
    fn un_projet_cree_compile() {
        let dir = projet("compile");
        let source = std::fs::read_to_string(dir.join("main.typ")).expect("assemblage");
        let resultat = crate::typst::render::compile(&dir, &source, &[]);
        assert!(resultat.errors.is_empty(), "{:?}", resultat.errors);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// La promesse est **visible** : ce que le projet écrit se voit
    /// dans l’éditeur, et ses normes portent la page de titre.
    #[test]
    fn le_projet_promet_ce_que_l_editeur_montre() {
        let dir = projet("promesses");
        std::fs::write(
            dir.join("document.tw"),
            "= Introduction\n\nLe corps du devoir.\n\n= Conclusion\n\n= Bibliographie\n",
        )
        .expect("document");

        let document = std::fs::read_to_string(dir.join("document.tw")).expect("document");
        for promesse in ["Introduction", "Conclusion", "Bibliographie"] {
            assert!(
                document.contains(promesse),
                "« {promesse} » manque : {document}"
            );
        }
        let normes = crate::mise_en_page::charger(&dir);
        assert!(normes.page_de_titre, "la page de titre est dans les normes");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
#[cfg(test)]
mod tests_modeles_visibles {
    /// Un modèle est un projet comme un autre : ce qu’il écrit se voit
    /// et se modifie dans l’éditeur — le document porte le contenu, les
    /// normes portent la page de titre, et `main.typ` n’est qu’assemblage
    /// (la promesse de bug-report-7 point 8, portée par les modèles
    /// projets — demande de l’auteur).
    #[test]
    fn un_modele_ecrit_ce_que_l_editeur_montre() {
        let parent = std::env::temp_dir().join(format!("tw-modele-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("dossier");

        // Le modèle : un projet qui porte son contenu et ses normes.
        let base = crate::project::creer_au_nom(&parent, "mon-modele").expect("modèle");
        std::fs::write(base.join("document.tw"), "= Introduction\n\nLe corps.\n").expect("contenu");
        let mut normes = crate::mise_en_page::charger(&base);
        normes.page_de_titre = true;
        let _ = crate::mise_en_page::appliquer(&base, &normes, true).expect("normes");

        // Le projet qui en naît porte la même promesse.
        let dir = crate::project::creer_au_nom_depuis(&parent, "un-devoir", &base).expect("copie");
        let document = std::fs::read_to_string(dir.join("document.tw")).expect("document");
        assert!(document.contains("= Introduction"), "le contenu se voit");

        let normes = crate::mise_en_page::charger(&dir);
        assert!(normes.page_de_titre, "la page de titre est une norme");

        let gabarit = crate::mise_en_page::gabarit::generer(&normes);
        assert!(
            gabarit.contains("page(numbering: none, header: none, footer: none)"),
            "le gabarit compose la page de titre : {gabarit}"
        );

        std::fs::remove_dir_all(&parent).expect("nettoyage");
    }
}
#[cfg(test)]
mod tests_du_branchement {
    use super::*;

    fn racine(nom: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "typst-writer-branchement-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("racine du test");
        root
    }

    #[test]
    fn le_lien_zotero_brance_la_bibliographie() {
        let root = racine("zotero");
        std::fs::write(root.join("references-zotero.bib"), "@book{dupont2023,}").expect("bib");

        // Le document modèle cite la locale — qui n’existe pas.
        let main = concat!(
            "#include \"chapitres/tout.typ\"\n",
            "#bibliography(\"references.bib\", style: \"chicago-notes\")\n"
        );
        let branche = brancher_les_bibliographies(&root, main).expect("branchement attendu");
        assert!(
            branche.contains("\"references-zotero.bib\""),
            "l’export Zotero est cité : {branche}"
        );
        assert!(
            !branche.contains("\"references.bib\""),
            "le fichier manquant ne l’est plus : {branche}"
        );
        assert!(
            branche.contains("chicago-notes"),
            "le style est gardé : {branche}"
        );

        // Déjà branché : rien ne bouge — la compilation ne se réécrit pas.
        assert!(
            brancher_les_bibliographies(&root, &branche).is_none(),
            "le branchement est stable"
        );

        // Les deux bibliothèques vivent ensemble.
        std::fs::write(root.join("references.bib"), "@book{durand2020,}").expect("bib");
        let deux = brancher_les_bibliographies(&root, &branche).expect("la locale rejoint");
        assert!(
            deux.contains("\"references.bib\"") && deux.contains("\"references-zotero.bib\""),
            "les deux sont citées : {deux}"
        );
        assert!(
            brancher_les_bibliographies(&root, &deux).is_none(),
            "stable une fois pour toutes"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sans_appel_le_document_n_en_recoit_pas() {
        let root = racine("sans-appel");
        std::fs::write(root.join("references-zotero.bib"), "").expect("bib");

        assert!(
            brancher_les_bibliographies(&root, "Du texte.\n").is_none(),
            "aucune bibliographie ne s'ajoute sans instruction explicite"
        );

        std::fs::remove_dir_all(&root).ok();
    }
}
