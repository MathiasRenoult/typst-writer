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
pub fn create_chapter(project: &Project, title: &str) -> std::io::Result<PathBuf> {
    let chapters_dir = project.dir.join("chapitres");

    std::fs::create_dir_all(&chapters_dir)?;

    let order = next_order(&project.chapters);

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
/// `main.typ` écrit `chapitres/01-intro.typ` — un chemin relatif — alors que
/// l'analyse produit des chemins absolus. On compare donc les deux derniers
/// composants, ce qui suffit ici et évite de canoniser (qui échoue sur un
/// chemin inexistant).
fn same_file(a: &Path, b: &Path) -> bool {
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

/// Ajoute un chapitre à un assemblage, s'il n'y est pas déjà.
///
/// Exposé pour la reprise des documents existants, qui doivent être inscrits
/// dans un assemblage sans passer par la création d'un chapitre.
pub fn add_chapter_to_assembly(main: &Path, chapter: &Path) -> std::io::Result<()> {
    add_include(main, chapter)
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
    content.split_whitespace().count()
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
}
