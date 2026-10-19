//! Le projet : un dossier de travail contenant un fichier par chapitre, une
//! bibliographie et un fichier d'assemblage.
//!
//! ```text
//! ~/.typst-writer/projets/<nom>/
//!   main.typ              l'assemblage : inclut les chapitres, cite la biblio
//!   chapitres/
//!     01-introduction.typ
//!     02-....
//!   references.bib
//! ```
//!
//! Un chapitre est un fichier comme un autre : Typst n'a pas besoin de plus
//! pour `#include`. La structure existe pour deux raisons — s'y retrouver en
//! écrivant, et pouvoir constater où en est le travail sans ouvrir chaque
//! fichier.

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

    /// Nom du projet (nom du dossier).
    pub name: String,

    /// Le fichier d'assemblage.
    pub main: PathBuf,

    /// Les chapitres, dans l'ordre de numérotation.
    pub chapters: Vec<Chapter>,

    /// La bibliographie, si un fichier `.bib` est présent.
    pub bibliography: Option<PathBuf>,

    /// Nombre d'entrées de la bibliographie.
    pub bibliography_entries: usize,
}

impl Project {
    /// Nombre total de mots du projet.
    ///
    /// La somme des chapitres, et non celle de `main.typ` : compter
    /// l'assemblage reviendrait à compter deux fois, puisque son contenu est
    /// fait des `#include`.
    pub fn words(&self) -> usize {
        self.chapters.iter().map(|chapter| chapter.words).sum()
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
        .map(|entry| load(&entry.path()))
        .collect();

    projects.sort_by_key(|project| std::cmp::Reverse(last_modified(project)));

    projects
}

/// Horodatage de la modification la plus récente d'un projet.
fn last_modified(project: &Project) -> u64 {
    project
        .chapters
        .iter()
        .map(|chapter| chapter.modified)
        .max()
        .unwrap_or(0)
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

    let included = read_includes(&main);

    let chapters = list_chapters(dir, &included);

    let bibliography = find_bibliography(dir);

    let bibliography_entries = bibliography
        .as_ref()
        .map(|path| super::parse_bibliography(path).len())
        .unwrap_or(0);

    Project {
        dir: dir.to_path_buf(),
        name,
        main,
        chapters,
        bibliography,
        bibliography_entries,
    }
}

/// Crée un projet vide : ses dossiers, son assemblage et sa bibliographie.
///
/// La bibliographie est créée même vide : un fichier absent serait à deviner,
/// alors qu'un fichier avec un exemple commenté montre quoi mettre dedans.
pub fn create(name: &str) -> std::io::Result<PathBuf> {
    let dir = projects_dir().join(slug(name));

    std::fs::create_dir_all(dir.join("chapitres"))?;

    let main = dir.join("main.typ");

    if !main.exists() {
        std::fs::write(&main, MAIN_TEMPLATE)?;
    }

    let bibliography = dir.join("references.bib");

    if !bibliography.exists() {
        std::fs::write(&bibliography, BIBLIOGRAPHY_TEMPLATE)?;
    }

    Ok(dir)
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

/// Retire un chapitre de l'assemblage, puis du disque.
pub fn remove_chapter(project: &Project, path: &Path) -> std::io::Result<()> {
    remove_include(&project.main, path)?;

    std::fs::remove_file(path)
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
fn read_includes(main: &Path) -> Vec<PathBuf> {
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

    if !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str(&format!("\n#include \"{}\"\n", relative.display()));

    std::fs::write(main, content)
}

/// Retire le `#include` correspondant à un chapitre.
///
/// Retire aussi la ligne vide qui le précédait : c'est celle qu'`add_include`
/// avait insérée comme séparateur. Sans cela, chaque ajout suivi d'un retrait
/// laisserait une ligne blanche de plus dans l'assemblage.
fn remove_include(main: &Path, chapter: &Path) -> std::io::Result<()> {
    let Ok(content) = std::fs::read_to_string(main) else {
        return Ok(());
    };

    let wanted = relative_to_chapters(chapter);

    let lines: Vec<&str> = content.lines().collect();

    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());

    for (index, line) in lines.iter().enumerate() {
        if parse_include_line(line).as_ref() == Some(&wanted) {
            // Le séparateur inséré juste avant l'include part avec lui.
            if index > 0 && lines[index - 1].trim().is_empty() {
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

/// Trouve la bibliographie d'un projet.
pub fn find_bibliography(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().map(|ext| ext == "bib").unwrap_or(false))
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

/// Transforme un titre en nom de fichier : minuscules, tirets, sans accent.
pub fn slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());

    let mut pending_dash = false;

    for character in title.chars() {
        // La casse est traitée avant le repli : sinon « État » perdrait sa
        // majuscule accentuée, qui n'est ni alphanumérique ASCII ni repliée.
        let folded = character
            .to_lowercase()
            .next()
            .map(fold_accent)
            .unwrap_or(character);

        if folded.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }

            slug.push(folded.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }

    if slug.is_empty() {
        slug.push_str("chapitre");
    }

    slug
}

/// Réduit les lettres accentuées à leur équivalent sans accent.
fn fold_accent(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        'œ' => 'o',
        other => other,
    }
}

/// Assemblage d'un nouveau projet.
pub const MAIN_TEMPLATE: &str = r#"#set page(margin: 2cm)
#set text(font: "New Computer Modern", size: 11pt)

// L'assemblage : les chapitres sont inclus ici, dans l'ordre.
// Ajouter un chapitre depuis le panneau de travail écrit la ligne pour vous.
"#;

/// Bibliographie d'un nouveau projet.
pub const BIBLIOGRAPHY_TEMPLATE: &str = r#"// Une entrée par référence, au format BibTeX.
// La clé (ici « exemple2026 ») est ce qui se cite dans le texte : @exemple2026.
//
// @article{exemple2026,
//   title = {Un titre},
//   author = {Nom, Prénom},
//   year = {2026},
// }
"#;

#[cfg(test)]
mod tests {
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
    fn an_empty_label_matches_nothing() {
        let project = Project {
            dir: PathBuf::new(),
            name: String::new(),
            main: PathBuf::new(),
            chapters: vec![],
            bibliography: None,
            bibliography_entries: 0,
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
        std::fs::write(dir.join("main.typ"), MAIN_TEMPLATE).unwrap();

        let before = std::fs::read_to_string(dir.join("main.typ")).unwrap();

        let project = load(&dir);
        let chapter = create_chapter(&project, "Introduction").unwrap();

        let with_chapter = std::fs::read_to_string(dir.join("main.typ")).unwrap();
        assert!(with_chapter.contains("#include \"chapitres/01-introduction.typ\""));
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
