//! Rendu fidèle : compilation par le compilateur Typst en bibliothèque et
//! export SVG vectoriel de chaque page.
//!
//! Contrairement au rendu natif par widgets (sous-ensemble de la syntaxe) ou
//! au binaire CLI, la compilation en bibliothèque évalue le document avec le
//! moteur Typst complet — mêmes polices, même layout, mêmes erreurs que
//! l'export — et produit des pages SVG nettes à tout zoom, en process
//! (aucun spawn de binaire à chaque frappe).

use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult, SourceDiagnostic, Warned};
use typst::foundations::{Bytes, Duration};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, WorldExt};
use typst_layout::PagedDocument;
use typst_svg::{SvgOptions, svg as page_svg};
use typst_syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

/// Un diagnostic (erreur ou avertissement) positionné dans le document.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,

    /// Ligne du diagnostic (1-based), si le compilateur l'a précisée.
    pub line: Option<usize>,

    /// Colonne du diagnostic (1-based), si le compilateur l'a précisée.
    pub column: Option<usize>,
}

/// Résultat du rendu d'un document.
#[derive(Debug, Clone, Default)]
pub struct RenderOutcome {
    /// Contenu rendu : permet de ne relancer que si le document a changé.
    pub content: String,

    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,

    /// Une chaîne SVG par page, vide si la compilation a échoué.
    pub pages: Vec<String>,
}

/// Compile le document et rend chaque page en SVG.
///
/// `root` est le dossier du document actif : les imports et images relatifs
/// s'y résolvent exactement comme à l'export.
pub fn render(root: &Path, content: &str) -> RenderOutcome {
    let world = PreviewWorld::new(root, content);
    let main = world.main_source().clone();

    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);

    let errors = convert_diagnostics(
        output
            .as_ref()
            .err()
            .map(|errors| errors.as_slice())
            .unwrap_or(&[]),
        &world,
        &main,
    );
    let warnings = convert_diagnostics(&warnings, &world, &main);

    let pages = match output {
        Ok(document) => {
            let options = SvgOptions::default();

            document
                .pages()
                .iter()
                .map(|page| page_svg(page, &options))
                .collect()
        }
        Err(_) => Vec::new(),
    };

    RenderOutcome {
        content: content.to_string(),
        errors,
        warnings,
        pages,
    }
}

/// Exporte le document compilé en PDF (octets bruts).
///
/// Renvoie `None` si le document ne compile pas (les erreurs sont alors
/// disponibles via [`render`]).
pub fn render_pdf(root: &Path, content: &str) -> Option<Vec<u8>> {
    let world = PreviewWorld::new(root, content);

    let document = typst::compile::<PagedDocument>(&world).output.ok()?;

    typst_pdf::pdf(&document, &Default::default()).ok()
}

/// Convertit les diagnostics du compilateur en diagnostics positionnés.
fn convert_diagnostics(
    diagnostics: &[SourceDiagnostic],
    world: &PreviewWorld,
    main: &Source,
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            // Seules les positions du fichier principal sont exploitables :
            // un span dans un fichier importé aurait une position fausse.
            let position = world
                .range(diagnostic.span.clone())
                .filter(|range| range.end <= main.text().len())
                .and_then(|range| main.lines().byte_to_line_column(range.start));

            Diagnostic {
                message: diagnostic.message.to_string(),
                line: position.map(|(line, _)| line + 1),
                column: position.map(|(_, column)| column + 1),
            }
        })
        .collect()
}

/// Un monde Typst minimal : le document courant en mémoire, les fichiers
/// importés lus depuis le disque, les polices embarquées de Typst.
pub struct PreviewWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main_id: FileId,
    source: Source,

    /// Dossier du document actif, pour résoudre les imports relatifs.
    root: std::path::PathBuf,
}

impl PreviewWorld {
    pub fn new(root: &Path, content: &str) -> Self {
        // Les polices embarquées sont celles du CLI Typst : le rendu est donc
        // identique à celui de `typst compile`.
        let fonts: Vec<Font> = typst_assets::fonts()
            .enumerate()
            .filter_map(|(index, data)| Font::new(Bytes::new(data.to_vec()), index as u32))
            .collect();

        let book = FontBook::from_fonts(fonts.iter());

        let main_id = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").expect("chemin valide"),
        ));

        let source = Source::new(main_id, content.to_string());

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            main_id,
            source,
            root: root.to_path_buf(),
        }
    }

    /// Le source du fichier principal.
    fn main_source(&self) -> &Source {
        &self.source
    }

    /// Chemin sur disque d'un fichier référencé par un FileId.
    fn path_of(&self, id: FileId) -> PathBuf {
        self.root.join(id.vpath().get_without_slash())
    }
}

impl World for PreviewWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            return Ok(self.source.clone());
        }

        let path = self.path_of(id);

        let text = std::fs::read_to_string(&path).map_err(|_| FileError::NotFound(path))?;

        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main_id {
            return Ok(Bytes::from_string(self.source.text().to_owned()));
        }

        let path = self.path_of(id);

        let bytes = std::fs::read(&path).map_err(|_| FileError::NotFound(path))?;

        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<typst::foundations::Datetime> {
        // Sans date, `datetime.today()` renvoie une erreur — acceptable pour
        // un aperçu (et rarement utilisé dans les documents).
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn renders_valid_document_to_svg_pages() {
        let dir = std::env::temp_dir().join("typst-writer-render-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = render(&dir, "= Titre\n\nDu texte avec du *gras*.\n");

        assert!(outcome.errors.is_empty(), "aucune erreur attendue");
        assert_eq!(outcome.pages.len(), 1, "une page SVG attendue");

        let page = &outcome.pages[0];
        assert!(page.contains("<svg"), "le rendu doit être un SVG");
        // Typst encode les glyphes en chemins vectoriels (comme le PDF) :
        // aucune police système n'intervient, le rendu est fidèle.
        assert!(
            page.contains("<path"),
            "les glyphes doivent être vectorisés"
        );
        assert!(page.contains("<symbol"), "les glyphes doivent être définis");

        let _ = fs::remove_dir_all(&dir);
    }

    /// Le vrai compilateur détecte les erreurs sémantiques, avec position.
    #[test]
    fn reports_semantic_errors_with_position() {
        let dir = std::env::temp_dir().join("typst-writer-render-err");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = render(&dir, "= Titre\n\n#foo()\n");

        assert_eq!(outcome.errors.len(), 1, "une erreur attendue");
        assert_eq!(outcome.errors[0].message, "unknown variable: foo");
        assert_eq!(outcome.errors[0].line, Some(3));
        assert!(outcome.pages.is_empty(), "pas de page en cas d'erreur");

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un document de plusieurs pages produit un SVG par page.
    #[test]
    fn renders_multiple_pages() {
        let dir = std::env::temp_dir().join("typst-writer-render-pages");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut content = String::from("#set page(paper: \"a6\")\n");
        for i in 1..=3 {
            if i > 1 {
                content.push_str("#pagebreak()\n");
            }
            content.push_str(&format!("= Page {i}\n"));
        }

        let outcome = render(&dir, &content);

        assert!(outcome.errors.is_empty());
        assert_eq!(outcome.pages.len(), 3, "trois pages SVG attendues");

        let _ = fs::remove_dir_all(&dir);
    }

    /// L'export PDF produit un PDF valide.
    #[test]
    fn exports_valid_pdf() {
        let dir = std::env::temp_dir().join("typst-writer-render-pdf");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let pdf = render_pdf(&dir, "= Titre\n\nDu contenu.\n").expect("export PDF");

        assert!(pdf.starts_with(b"%PDF"), "le PDF doit être valide");

        let _ = fs::remove_dir_all(&dir);
    }
}
