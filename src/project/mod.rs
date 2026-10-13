//! Gestion du projet : les documents de l'application, la session de travail
//! et la bibliographie locale.
//!
//! L'application travaille toujours sur des fichiers locaux appartenant à
//! l'app : le dossier de travail (`~/.typst-writer/`) est la bibliothèque
//! personnelle de l'utilisateur. Aucune boîte de dialogue « où
//! sauvegarder ? » n'intervient pendant l'édition — elle ne sert qu'à
//! l'export (PDF, partage d'un document).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Dossier de travail de l'application : tous les documents actifs y vivent.
pub fn work_dir() -> PathBuf {
    let dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".typst-writer"),
        Err(_) => PathBuf::from(".typst-writer"),
    };

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Un document de la bibliothèque personnelle.
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    /// Chemin absolu du fichier.
    pub path: PathBuf,

    /// Nom du fichier (avec l'extension).
    pub name: String,

    /// Taille en octets.
    pub size: u64,

    /// Horodatage de dernière modification (secondes epoch).
    pub modified: u64,
}

impl DocumentInfo {
    /// Nom sans l'extension `.typ` : le titre affiché dans la bibliothèque.
    pub fn title(&self) -> &str {
        self.name.strip_suffix(".typ").unwrap_or(&self.name)
    }
}

/// Liste les documents Typst du dossier de travail, du plus récemment
/// modifié au plus ancien.
///
/// Renvoie une liste vide si le dossier n'existe pas ou est vide.
pub fn list_documents() -> Vec<DocumentInfo> {
    let Ok(entries) = std::fs::read_dir(work_dir()) else {
        return Vec::new();
    };

    let mut documents: Vec<DocumentInfo> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|extension| extension == "typ")
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().ok()?;
            let name = path.file_name()?.to_str()?.to_string();

            Some(DocumentInfo {
                path,
                name,
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0),
            })
        })
        .collect();

    documents.sort_by(|a, b| b.modified.cmp(&a.modified));

    documents
}

/// Crée un nouveau document dans le dossier de travail.
///
/// Le nom est `document-N.typ` avec N le premier entier libre, pour ne jamais
/// écraser un travail existant.
pub fn create_document(content: &str) -> std::io::Result<PathBuf> {
    let dir = work_dir();

    for n in 1..=1000 {
        let path = dir.join(format!("document-{n}.typ"));

        if !path.exists() {
            std::fs::write(&path, content)?;

            return Ok(path);
        }
    }

    std::fs::write(dir.join("document.typ"), content)?;

    Ok(dir.join("document.typ"))
}

/// Supprime un document de la bibliothèque.
pub fn delete_document(path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

/// Session de travail : le document à rouvrir au démarrage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Session {
    /// Chemin du dernier document ouvert.
    #[serde(default)]
    pub last_document: Option<PathBuf>,
}

fn session_path() -> PathBuf {
    work_dir().join("session.json")
}

/// Charge la session enregistrée (dernier document ouvert).
pub fn load_session() -> Session {
    std::fs::read_to_string(session_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Enregistre la session (dernier document ouvert).
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

/// Cherche un fichier `.bib` dans le dossier du document.
///
/// La convention est la même que dans Typst : `#bibliography("refs.bib")`.
pub fn find_bibliography(document_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(document_dir).ok()?;

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().map(|ext| ext == "bib").unwrap_or(false))
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

/// Modèle de document : le squelette d'un nouveau devoir.
pub const DOCUMENT_TEMPLATE: &str = r#"#set page(margin: 2cm)
#set text(font: "New Computer Modern", size: 11pt)

= Introduction

Bienvenue dans Typst Writer.

Ceci est notre premier document Typst.
"#;

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

    /// `DocumentInfo::title` retire l'extension pour l'affichage.
    #[test]
    fn title_strips_extension() {
        let info = DocumentInfo {
            path: PathBuf::from("/tmp/devoir-3.typ"),
            name: "devoir-3.typ".to_string(),
            size: 0,
            modified: 0,
        };

        assert_eq!(info.title(), "devoir-3");
    }
}
