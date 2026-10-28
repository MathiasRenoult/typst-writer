//! Les styles CSL enregistrés dans Zotero.
//!
//! Zotero range les styles qu'on lui installe dans `styles/`, au format CSL.
//! Le sélecteur de l'application montre ceux-là — la liste de **son**
//! Zotero, pas un catalogue à distance — et le style choisi sert deux fois :
//! à formater ce que le sélecteur montre, et à la bibliographie du document.

use std::path::{Path, PathBuf};

/// Un style CSL enregistré dans Zotero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CslStyle {
    /// L'identifiant CSL : celui que l'API de Zotero et Typst demandent.
    pub id: String,

    /// Le titre lisible, celui que le sélecteur montre.
    pub title: String,

    /// Le fichier.
    pub path: PathBuf,
}

/// Les styles enregistrés dans le dossier `styles/` de Zotero, par titre.
///
/// Un fichier illisible est passé : une bibliothèque ne se vide pas parce
/// qu'un style y est abîmé.
pub fn installed(styles_dir: &Path) -> Vec<CslStyle> {
    let Ok(entries) = std::fs::read_dir(styles_dir) else {
        return Vec::new();
    };

    let mut styles: Vec<CslStyle> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "csl"))
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let (id, title) = parse_header(&text)?;

            Some(CslStyle { id, title, path })
        })
        .collect();

    styles.sort_by_key(|style| style.title.to_lowercase());
    styles
}

/// L'identifiant et le titre d'un fichier CSL.
///
/// Le format est de l'XML, et la spec est simple : `<info>` porte un
/// `<id>` et un `<title>`. Le premier de chacun est celui de la langue par
/// défaut ; `<title-short>` en est un autre élément, qu'on ne prend pas.
fn parse_header(text: &str) -> Option<(String, String)> {
    let id = element_text(text, "id")?;
    let title = element_text(text, "title")?;

    (!id.is_empty() && !title.is_empty()).then_some((id, title))
}

/// Le contenu du premier élément de ce nom.
///
/// `<id>` ne doit pas confondre avec `<identifier>` : seul compte un
/// élément dont le nom s'achève vraiment.
fn element_text(text: &str, name: &str) -> Option<String> {
    let open = format!("<{name}");
    let mut cursor = 0;

    while let Some(found) = text[cursor..].find(&open) {
        let start = cursor + found;
        let after = start + open.len();
        let next = text[after..].chars().next()?;

        if matches!(next, '>' | ' ' | '\n' | '\t') {
            let open_end = text[start..].find('>')? + start;
            let end = text[start..].find(&format!("</{name}>"))? + start;

            return Some(text[open_end + 1..end].trim().to_string());
        }

        cursor = after;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const APA: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<style xmlns="http://purl.org/net/xbiblio/csl" class="in-text" version="1.0">
  <info>
    <id>http://www.zotero.org/styles/apa</id>
    <title>American Psychological Association 7th edition</title>
    <title-short>APA</title-short>
  </info>
</style>"#;

    /// Un style se lit par son identifiant et son titre.
    #[test]
    fn a_style_header_is_its_id_and_its_title() {
        let (id, title) = parse_header(APA).expect("lisible");

        assert_eq!(id, "http://www.zotero.org/styles/apa");
        assert_eq!(title, "American Psychological Association 7th edition");
    }

    /// Un fichier qui n'est pas un style n'ouvre rien.
    #[test]
    fn a_broken_file_opens_nothing() {
        assert_eq!(parse_header("<notes>ceci n'est pas un style"), None);
    }

    /// Le dossier se lit entier, par titre.
    #[test]
    fn installed_styles_are_listed_by_title() {
        let dir = std::env::temp_dir().join(format!("tw-styles-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier");

        std::fs::write(
            dir.join("zoulou.csl"),
            APA.replace("apa", "zoulou").replace(
                "American Psychological Association 7th edition",
                "Style Zoulou",
            ),
        )
        .expect("style");
        std::fs::write(dir.join("a-traduire.txt"), "pas un style").expect("fichier");
        std::fs::write(dir.join("abime.csl"), "<style").expect("fichier abîmé");

        let styles = installed(&dir);

        assert_eq!(styles.len(), 1, "un seul style lisible");
        assert_eq!(styles[0].title, "Style Zoulou");
        assert!(styles[0].path.ends_with("zoulou.csl"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
