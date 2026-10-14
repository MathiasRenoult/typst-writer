//! Gestion du projet : les chapitres, la bibliographie, la session de travail.
//!
//! L'application travaille toujours sur des fichiers locaux : aucun dialogue
//! « où sauvegarder ? » n'intervient pendant l'édition — il ne sert qu'à
//! l'export.
//!
//! Un travail se présente comme un **projet** : un dossier dans
//! `~/.typst-writer/projets/`, avec un fichier par chapitre, sa bibliographie
//! et son assemblage. Voir [`project`] pour la structure exacte.

pub mod project;

pub use project::*;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Dossier de travail de l'application : les projets y vivent.
pub fn work_dir() -> PathBuf {
    let dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".typst-writer"),
        Err(_) => PathBuf::from(".typst-writer"),
    };

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Session de travail : ce qu'il faut rouvrir au démarrage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session {
    /// Dossier du projet ouvert.
    #[serde(default)]
    pub project: Option<PathBuf>,

    /// Chemin du chapitre ouvert dans ce projet.
    #[serde(default)]
    pub last_document: Option<PathBuf>,
}

fn session_path() -> PathBuf {
    work_dir().join("session.json")
}

/// Charge la session enregistrée.
pub fn load_session() -> Session {
    std::fs::read_to_string(session_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Enregistre la session.
pub fn save_session(session: &Session) {
    let text = match serde_json::to_string_pretty(session) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Sérialisation de la session : {error}");
            return;
        }
    };

    if let Err(error) = std::fs::write(session_path(), text) {
        eprintln!("Enregistrement de la session : {error}");
    }
}

/// Une entrée de bibliographie (fichier `.bib`).
#[derive(Debug, Clone)]
pub struct BibliographyEntry {
    /// La clé de citation, celle qui suit `@type{`.
    pub key: String,

    /// Le type BibTeX (`article`, `book`, `inproceedings`…).
    pub kind: String,
}

/// Lit les entrées d'un fichier `.bib`.
///
/// Parseur volontairement tolérant : il ne cherche que les débuts d'entrée
/// (`@type{clé,`), ce qui suffit pour lister les clés citables. Un fichier
/// malformé renvoie les entrées reconnues jusqu'au problème.
pub fn parse_bibliography(path: &Path) -> Vec<BibliographyEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(|line| parse_bib_entry_line(line))
        .collect()
}

/// Analyse une ligne de fichier BibTeX : `@article{clé,` → entrée.
fn parse_bib_entry_line(line: &str) -> Option<BibliographyEntry> {
    let line = line.trim();

    let rest = line.strip_prefix('@')?;
    let (kind, rest) = rest.split_once('{')?;

    let key = rest.split(',').next()?.trim().to_string();

    if key.is_empty() {
        return None;
    }

    Some(BibliographyEntry {
        kind: kind.trim().to_string(),
        key,
    })
}

/// Le nom du projet créé lors de la reprise des documents existants.
const MIGRATED_PROJECT: &str = "mon-travail";

/// Reprend les documents à plat dans un premier projet.
///
/// Les versions précédentes rangeaient les documents directement à la racine de
/// `~/.typst-writer/`. Puisqu'un travail est désormais un projet, ces fichiers
/// deviennent les chapitres d'un projet « mon-travail ».
///
/// Ne fait rien si un projet existe déjà, ou s'il n'y a rien à reprendre :
/// la migration ne s'exécute qu'une fois, sur un dossier vierge de projets.
///
/// Renvoie le dossier du projet créé, le cas échéant.
pub fn migrate_flat_documents() -> Option<PathBuf> {
    // Un projet existe déjà : le dossier a été migré, ou l'utilisateur a
    // commencé à travailler dans la nouvelle organisation.
    if !project::list_projects().is_empty() {
        return None;
    }

    let flat = locate_flat_documents();

    if flat.is_empty() {
        return None;
    }

    let dir = project::create(MIGRATED_PROJECT).ok()?;

    // Une bibliographie restée à la racine doit rejoindre le projet : sans
    // cela, un chapitre qui la cite perdrait sa référence.
    adopt_stray_bibliography(&dir);

    let chapters_dir = dir.join("chapitres");

    let session = load_session();
    let mut moved_session = session.project.is_none();

    for (index, path) in flat.iter().enumerate() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("chapitre.typ");

        let destination = chapters_dir.join(format!("{:02}-{}", index + 1, name));

        if std::fs::rename(path, &destination).is_err() {
            // Un fichier qu'on n'a pas pu déplacer est laissé sur place plutôt
            // que perdu : la migration reprendra au prochain démarrage.
            continue;
        }

        let _ = project::add_chapter_to_assembly(&dir.join("main.typ"), &destination);

        if session.last_document.as_deref() == Some(path.as_path()) {
            moved_session = true;
        }
    }

    // La session pointait vers un fichier à plat : elle pointe maintenant vers
    // le projet, sinon le démarrage suivant chercherait un document disparu.
    if moved_session {
        save_session(&Session {
            project: Some(dir.clone()),
            last_document: None,
        });
    }

    Some(dir)
}

/// Fait rejoindre au projet une bibliographie laissée à la racine.
///
/// Le fichier créé par `project::create` est un modèle vide : on ne le remplace
/// que s'il est effectivement vide, pour ne jamais écraser des références
/// existantes par un exemple commenté.
fn adopt_stray_bibliography(project_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(work_dir()) else {
        return;
    };

    let Some(stray) = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().map(|ext| ext == "bib").unwrap_or(false))
    else {
        return;
    };

    let destination = project_dir.join("references.bib");

    let template_is_untouched = std::fs::read_to_string(&destination)
        // Le modèle n'a que des lignes vides ou commentées : aucun `@`.
        .map(|content| !content.contains('@'))
        .unwrap_or(false);

    if template_is_untouched {
        let _ = std::fs::rename(&stray, &destination);
    }
}

/// Les documents `.typ` restés à la racine du dossier de travail.
fn locate_flat_documents() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(work_dir()) else {
        return Vec::new();
    };

    let mut documents: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().map(|ext| ext == "typ").unwrap_or(false))
        .collect();

    // L'ordre de lecture du dossier n'est pas garanti : on trie pour que la
    // numérotation des chapitres soit reproductible.
    documents.sort();

    documents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bib_entries() {
        let dir = std::env::temp_dir().join("typst-writer-bib");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{smith2020,\n  title = {Un titre},\n}\n@book{doe2019, author = {Doe}}\n",
        )
        .unwrap();

        let entries = parse_bibliography(&bib);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "smith2020");
        assert_eq!(entries[0].kind, "article");
        assert_eq!(entries[1].key, "doe2019");
        assert_eq!(entries[1].kind, "book");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une ligne sans `@` ou sans clé est ignorée, pas une erreur.
    #[test]
    fn ignores_malformed_bib_lines() {
        assert!(parse_bib_entry_line("ceci n'est pas une entrée").is_none());
        assert!(parse_bib_entry_line("@article{,").is_none());
        assert!(parse_bib_entry_line("@article{clé,").is_some());
    }

    /// Une session d'une version précédente, qui ne connaissait que
    /// `last_document`, doit se lire sans erreur.
    #[test]
    fn an_older_session_still_loads() {
        let session: Session =
            serde_json::from_str(r#"{"last_document": "/tmp/ancien.typ"}"#).unwrap();

        assert_eq!(
            session.last_document,
            Some(PathBuf::from("/tmp/ancien.typ"))
        );
        assert!(session.project.is_none());
    }
}
