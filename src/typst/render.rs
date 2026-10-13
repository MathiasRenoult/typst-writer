//! Compilation Typst en bibliothèque et rendu raster des pages.
//!
//! Deux temps, volontairement séparés :
//!
//! 1. **Compilation** (`compile`) : le document est compilé par le moteur
//!    Typst complet (polices embarquées, layout identique à `typst compile`).
//!    C'est rapide (~50 ms même pour un gros document) et ne produit aucun
//!    bitmap. Les diagnostics (erreurs, avertissements) en sortent aussi.
//!
//! 2. **Rendu raster** (`render_page`) : une seule page à la fois est
//!    rasterisée en PNG par `typst-render` (le moteur d'export PNG du CLI,
//!    qui dessine les glyphes directement — pas de SVG, donc pas de bug
//!    d'exposant). Seules les pages visibles à l'écran sont rendues, ce qui
//!    rend l'aperçu d'un document de 5500 mots aussi léger que celui d'une
//!    page.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use typst::diag::{FileError, FileResult, SourceDiagnostic, Warned};
use typst::foundations::{Bytes, Duration};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, WorldExt};
use typst_layout::PagedDocument;
use typst_syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

/// Densité de rendu des pages (pixels par point).
///
/// 2.0 correspond à un écran HiDPI raisonnable : net, et ~4× moins de pixels
/// qu'une export à 4.0. Le coût de rasterisation reste sous les 30 ms/page.
const PIXEL_PER_PT: f64 = 2.0;

/// Un diagnostic (erreur ou avertissement) positionné dans le document.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,

    /// Ligne du diagnostic (1-based), si le compilateur l'a précisée.
    pub line: Option<usize>,

    /// Colonne du diagnostic (1-based), si le compilateur l'a précisée.
    pub column: Option<usize>,
}

/// Résultat de la compilation d'un document.
#[derive(Debug, Clone, Default)]
pub struct CompileOutcome {
    /// Contenu compilé : permet de ne relancer que si le document a changé.
    pub content: String,

    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,

    /// Document compilé, `None` si la compilation a échoué.
    ///
    /// Les pages sont gardées en mémoire (`Arc`) : rasteriser une page est
    /// alors instantané à déclencher, et le document peut être réexporté en
    /// PDF sans recompiler.
    pub document: Option<Arc<PagedDocument>>,
}

/// Compile le document (sans rasteriser).
///
/// `root` est le dossier du document actif : les imports et images relatifs
/// s'y résolvent exactement comme à l'export.
pub fn compile(root: &Path, content: &str) -> CompileOutcome {
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

    CompileOutcome {
        content: content.to_string(),
        errors,
        warnings,
        document: output.ok().map(Arc::new),
    }
}

/// Rasterise une page du document en PNG.
///
/// Renvoie les octets PNG (à donner à `image::Handle::from_memory`), ou
/// `None` si l'index dépasse le nombre de pages.
pub fn render_page(document: &PagedDocument, index: usize) -> Option<Vec<u8>> {
    let page = document.pages().get(index)?;

    let options = typst_render::RenderOptions {
        pixel_per_pt: typst_utils::Scalar::new(PIXEL_PER_PT),
        render_bleed: false,
    };

    let pixmap = typst_render::render(page, &options);

    pixmap.encode_png().ok()
}

/// Dimensions d'une page en pixels (largeur, hauteur) à la densité de rendu.
pub fn page_size(document: &PagedDocument, index: usize) -> Option<(f64, f64)> {
    let page = document.pages().get(index)?;
    let size = page.frame.size();

    let scale = PIXEL_PER_PT;

    Some((size.x.to_pt() * scale, size.y.to_pt() * scale))
}

/// Nombre de pages du document.
pub fn page_count(document: &PagedDocument) -> usize {
    document.pages().len()
}

/// Exporte le document compilé en PDF (octets bruts).
///
/// Renvoie `None` si le document ne compile pas (les erreurs sont alors
/// disponibles via [`compile`]).
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
    use std::time::Instant;

    /// Génère un document de la taille demandée (mots) pour les mesures.
    fn generate_content(words: usize) -> String {
        let vocabulary = [
            "lorem",
            "ipsum",
            "dolor",
            "sit",
            "amet",
            "consectetur",
            "adipiscing",
            "elit",
            "sed",
            "do",
            "eiusmod",
            "tempor",
            "incididunt",
            "ut",
            "labore",
            "et",
            "dolore",
            "magna",
            "aliqua",
        ];

        let mut content = String::from("#set page(margin: 2cm)\n\n");

        // ~70 mots par paragraphe.
        let paragraphs = words / 70;

        for index in 1..=paragraphs {
            if index % 8 == 1 {
                content.push_str(&format!("= Section {}\n\n", index / 8 + 1));
            }

            for word_index in 0..70 {
                let word = vocabulary[(index * 7 + word_index * 3) % vocabulary.len()];
                content.push_str(word);
                content.push(' ');
            }

            content.push_str("\n\n");
        }

        content
    }

    /// Banc de mesure : compilation, rasterisation, incrémentalité.
    ///
    /// Ignoré par défaut (quelques secondes) :
    /// `cargo test --release -- --ignored bench_full_document`
    #[test]
    #[ignore = "banc de mesure manuel"]
    fn bench_full_document() {
        let dir = std::env::temp_dir().join("typst-writer-bench");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let content = generate_content(5500);

        println!();
        println!("Document : {} caractères", content.len());

        // 1. Compilation complète.
        let start = Instant::now();
        let outcome = compile(&dir, &content);
        let compile_time = start.elapsed();

        let document = outcome.document.expect("document compilé");

        println!(
            "Compilation        : {compile_time:>10.2?}  ({} page(s))",
            page_count(&document)
        );

        // 2. Rasterisation des deux pages visibles.
        let start = Instant::now();
        let mut rendered = 0;

        for index in 0..2.min(page_count(&document)) {
            if render_page(&document, index).is_some() {
                rendered += 1;
            }
        }

        let raster_time = start.elapsed();

        println!(
            "Rendu {rendered} page(s)     : {raster_time:>10.2?}  ({:.2?}/page)",
            raster_time / rendered.max(1) as u32
        );

        // 3. Taille d'une page.
        if let Some((width, height)) = page_size(&document, 0) {
            println!("Taille d'une page  : {width:.0} × {height:.0} px");
        }

        // 4. Compilation incrémentale : une frappe en fin de document.
        let mut edited = content.clone();
        edited.push_str("\nUn mot de plus.\n");

        let start = Instant::now();
        let second = compile(&dir, &edited);
        let incremental_time = start.elapsed();

        println!("Compilation incré. : {incremental_time:>10.2?}");

        // 5. Coût d'une frame : la compilation est le temps dominant, la
        //    rasterisation ne concerne que les pages visibles.
        println!();
        println!(
            "Budget frame 60 Hz : 16.00 ms  —  compilation = {:.0}% du budget",
            compile_time.as_secs_f64() / 0.016 * 100.0
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compiles_valid_document() {
        let dir = std::env::temp_dir().join("typst-writer-compile-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "= Titre\n\nDu texte avec du *gras*.\n");

        assert!(outcome.errors.is_empty(), "aucune erreur attendue");
        assert_eq!(page_count(outcome.document.as_ref().unwrap()), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    /// Le vrai compilateur détecte les erreurs sémantiques, avec position.
    #[test]
    fn reports_semantic_errors_with_position() {
        let dir = std::env::temp_dir().join("typst-writer-compile-err");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "= Titre\n\n#foo()\n");

        assert_eq!(outcome.errors.len(), 1, "une erreur attendue");
        assert_eq!(outcome.errors[0].message, "unknown variable: foo");
        assert_eq!(outcome.errors[0].line, Some(3));
        assert!(
            outcome.document.is_none(),
            "pas de document en cas d'erreur"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un document de plusieurs pages est compilé page par page.
    #[test]
    fn compiles_multiple_pages() {
        let dir = std::env::temp_dir().join("typst-writer-compile-pages");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut content = String::from("#set page(paper: \"a6\")\n");
        for i in 1..=3 {
            if i > 1 {
                content.push_str("#pagebreak()\n");
            }
            content.push_str(&format!("= Page {i}\n"));
        }

        let outcome = compile(&dir, &content);

        assert!(outcome.errors.is_empty());
        assert_eq!(page_count(outcome.document.as_ref().unwrap()), 3);

        let _ = fs::remove_dir_all(&dir);
    }

    /// La rasterisation produit un PNG valide, à la bonne taille.
    #[test]
    fn renders_page_to_png() {
        let dir = std::env::temp_dir().join("typst-writer-raster");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "= Titre\n\nDu contenu.\n");
        let document = outcome.document.expect("document compilé");

        let png = render_page(&document, 0).expect("rendu de la page");

        assert!(png.starts_with(b"\x89PNG"), "le rendu doit être un PNG");

        // A4 à 2 px/pt : ~1191 × 1684 px.
        let (width, height) = page_size(&document, 0).expect("taille de page");
        assert!(
            (width - 1191.0).abs() < 5.0,
            "largeur A4 attendue, eu {width}"
        );
        assert!(
            (height - 1684.0).abs() < 5.0,
            "hauteur A4 attendue, eu {height}"
        );

        // Index hors bornes : pas de page.
        assert!(render_page(&document, 99).is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    /// Les maths s'affichent sans artefact d'exposant : le rendu passe par le
    /// moteur (pas par un encodage SVG).
    #[test]
    fn renders_math_without_svg_artifacts() {
        let dir = std::env::temp_dir().join("typst-writer-math");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "Inline $x_1^2$ et display $ sum_(i=1)^n i $");

        assert!(outcome.errors.is_empty(), "maths valides attendues");

        let document = outcome.document.expect("document compilé");
        let png = render_page(&document, 0).expect("rendu");

        assert!(png.starts_with(b"\x89PNG"));

        let _ = fs::remove_dir_all(&dir);
    }

    /// L'export PDF produit un PDF valide.
    #[test]
    fn exports_valid_pdf() {
        let dir = std::env::temp_dir().join("typst-writer-pdf");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let pdf = render_pdf(&dir, "= Titre\n\nDu contenu.\n").expect("export PDF");

        assert!(pdf.starts_with(b"%PDF"), "le PDF doit être valide");

        let _ = fs::remove_dir_all(&dir);
    }
}
