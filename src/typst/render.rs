//! Compilation Typst en bibliothèque et rendu raster des pages.
//!
//! Deux temps, volontairement séparés :
//!
//! 1. **Compilation** (`compile`) : le document est compilé par le moteur
//!    Typst complet (polices embarquées, layout identique à `typst compile`).
//!    C'est rapide (~15 ms même pour un gros document) et ne produit aucun
//!    bitmap. Les diagnostics (erreurs, avertissements) en sortent aussi.
//!
//! 2. **Rendu raster** (`render_page`) : une seule page à la fois est
//!    rasterisée en pixels RGBA par `typst-render` (le moteur d'export du CLI,
//!    qui dessine les glyphes directement — pas de SVG, donc pas de bug
//!    d'exposant). Seules les pages visibles à l'écran sont rendues, ce qui
//!    rend l'aperçu d'un document de 5500 mots aussi léger que celui d'une
//!    page.
//!
//!    Les pixels partent bruts vers `Handle::from_rgba`, sans passage par un
//!    PNG : l'encodage puis le décodage coûtaient la moitié du temps de rendu
//!    pour rien, et le backend rechargeait une texture à chaque fois.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use typst::diag::{FileError, FileResult, SourceDiagnostic, Warned};
use typst::foundations::{Bytes, Duration, Label};
use typst::introspection::Introspector;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, WorldExt};
use typst_layout::PagedDocument;
use typst_syntax::{DiagSpan, FileId, RootedPath, Source, VirtualPath, VirtualRoot};

/// Repère inséré dans l'entrée de compilation, devant l'`#include` du chapitre
/// en cours d'écriture.
///
/// Un label Typst n'affiche rien et ne pèse pas dans la mise en page : il sert
/// uniquement à demander au document, une fois compilé, la page où commence le
/// chapitre qu'on écrit. Sans lui, l'aperçu n'a aucun moyen de savoir où il se
/// trouve dans l'assemblage, et s'ouvre donc toujours à la première page — ce
/// qui revenait à voir le chapitre 1 en écrivant le chapitre 7.
const CHAPTER_MARKER: &str = "#metadata(none) <tw-chapitre>";

/// Nom du label du repère, tel qu'on le redemande à l'introspection.
const CHAPTER_LABEL: &str = "tw-chapitre";

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

    /// Le fichier où se situe le diagnostic, résolu sur le disque.
    ///
    /// Le document compilé est l'assemblage, qui `#include` les chapitres : un
    /// diagnostic peut donc tomber dans n'importe lequel. Sans ce champ, une
    /// erreur dans un chapitre serait signalée sans qu'on sache où la
    /// corriger — et « sauter à l'erreur » n'aurait aucun sens.
    pub file: Option<PathBuf>,
}

/// Résultat de la compilation d'un document.
#[derive(Debug, Clone, Default)]
pub struct CompileOutcome {
    /// Contenu compilé : permet de ne relancer que si le document a changé.
    pub content: String,

    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,

    /// Page où commence le chapitre en cours d'écriture (à partir de zéro),
    /// `None` si elle n'a pas pu être déterminée.
    ///
    /// C'est ce qui permet d'amener l'aperçu sur le chapitre qu'on écrit plutôt
    /// que de le laisser en tête de document.
    pub chapter_page: Option<usize>,

    /// Document compilé, `None` si la compilation a échoué.
    ///
    /// Les pages sont gardées en mémoire (`Arc`) : rasteriser une page est
    /// alors instantané à déclencher, et le document peut être réexporté en
    /// PDF sans recompiler.
    pub document: Option<Arc<PagedDocument>>,
}

/// Une page rasterisée : pixels RGBA bruts, tels que le moteur les produit.
///
/// Aucun encodage intermédiaire : ces octets sont donnés directement à
/// [`iced::widget::image::Handle::from_rgba`]. Passer par un PNG (encodage
/// puis décodage par le backend) coûtait la moitié du temps de rendu et
/// faisait travailler un thread de décodage pour rien.
#[derive(Debug, Clone)]
pub struct PageImage {
    /// Largeur en pixels (à la densité de rendu).
    pub width: u32,

    /// Hauteur en pixels (à la densité de rendu).
    pub height: u32,

    /// Pixels RGBA, `width * height * 4` octets.
    pub pixels: Vec<u8>,

    /// Empreinte des pixels.
    ///
    /// Calculée ici, sur le fil de travail, et non à l'affichage : elle sert à
    /// reconnaître une page **identique** et à ne pas la remplacer, ce qui
    /// évite autant de textures neuves — et donc de travail pour le
    /// compositeur. Hacher huit mégaoctets prend une à deux millisecondes, ce
    /// qui n'a rien à faire sur le fil de l'interface.
    pub hash: u64,
}

/// Compile un document (sans rasteriser).
///
/// `root` est la racine du projet : les chemins de l'assemblage s'y résolvent
/// exactement comme à l'export.
///
/// `entry` est le contenu du fichier d'entrée — l'**assemblage** du projet, et
/// non le chapitre ouvert. C'est ce qui fait que l'en-tête Typst (les `#set`,
/// `#show` et autres réglages du document) est toujours appliqué, avec la
/// pagination et les compteurs qui en découlent. Compiler un chapitre seul
/// donnerait un rendu qui ne correspond pas au PDF.
///
/// `overrides` porte les fichiers dont la version en mémoire est plus récente
/// que le disque : en pratique le chapitre en cours d'édition, dont la
/// sauvegarde est différée. Sans eux, l'aperçu aurait jusqu'à une seconde de
/// retard sur la frappe, puisque l'assemblage `#include` les chapitres depuis
/// le disque.
pub fn compile(root: &Path, entry: &str, overrides: &[(PathBuf, String)]) -> CompileOutcome {
    let marked = marked_entry(root, entry, overrides);

    let compiled_entry = marked.as_deref().unwrap_or(entry);

    let world = PreviewWorld::new(root, compiled_entry, overrides);

    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);

    let errors = convert_diagnostics(
        output
            .as_ref()
            .err()
            .map(|errors| errors.as_slice())
            .unwrap_or(&[]),
        &world,
    );
    let warnings = convert_diagnostics(&warnings, &world);

    let document = output.ok().map(Arc::new);
    let chapter_page = document.as_ref().and_then(|doc| chapter_page(doc));

    CompileOutcome {
        // L'empreinte est **celle** que calculera la décision de recompiler.
        // Les faire diverger rendait toute compilation « nouvelle ».
        content: compilation_fingerprint(root, entry, overrides),
        errors,
        warnings,
        chapter_page,
        document,
    }
}

/// L'entrée réellement compilée : l'assemblage, un repère posé devant le
/// chapitre en cours.
///
/// Le chapitre en cours est celui dont `overrides` porte la version en mémoire :
/// c'est exactement le fichier qu'on est en train d'écrire. Quand l'assemblage
/// lui-même est ouvert, `overrides` est vide et aucun repère n'est posé.
fn marked_entry(root: &Path, entry: &str, overrides: &[(PathBuf, String)]) -> Option<String> {
    overrides
        .first()
        .and_then(|(path, _)| path.strip_prefix(root).ok())
        .and_then(|chapter| mark_chapter(entry, chapter))
}

/// L'empreinte de ce qui serait compilé.
///
/// **C'est la même valeur que celle que [`compile`] rapporte**, et c'est le
/// point essentiel : la décision de recompiler et le résultat d'une compilation
/// se comparent à la même aune. Les faire diverger — l'un sur l'entrée repérée,
/// l'autre sur l'entrée brute — rendait toute compilation « nouvelle » :
/// l'aperçu recompilait alors à chaque tick, plus de trois fois par seconde,
/// sans qu'on ait écrit un seul caractère, et reprenait toutes les pages
/// visibles à chaque fois. C'est ce qui se voyait comme un clignotement
/// permanent.
pub fn compilation_fingerprint(
    root: &Path,
    entry: &str,
    overrides: &[(PathBuf, String)],
) -> String {
    let marked = marked_entry(root, entry, overrides);

    fingerprint(marked.as_deref().unwrap_or(entry), overrides)
}

/// Insère le repère devant l'`#include` du chapitre visé.
///
/// Renvoie `None` si l'assemblage n'inclut pas ce chapitre. Poser le repère
/// ailleurs désignerait la page d'un autre texte — mieux vaut ne rien savoir
/// que savoir faux, l'aperçu restant alors simplement où il est.
fn mark_chapter(entry: &str, chapter: &Path) -> Option<String> {
    // Les chemins de l'assemblage s'écrivent en relatif, avec des barres
    // obliques, quels que soient la plateforme et le chemin d'origine.
    let chapter = chapter.to_string_lossy().replace('\\', "/");
    let include = format!("#include \"{chapter}\"");

    let include_at = entry.find(&include)?;

    // Le repère se pose en début de ligne : l'insérer en plein milieu
    // décalerait tout le reste de la ligne.
    let line_start = entry[..include_at].rfind('\n').map_or(0, |index| index + 1);

    let mut marked = String::with_capacity(entry.len() + CHAPTER_MARKER.len() + 1);
    marked.push_str(&entry[..line_start]);
    marked.push_str(CHAPTER_MARKER);
    marked.push('\n');
    marked.push_str(&entry[line_start..]);

    Some(marked)
}

/// Page où commence le chapitre repéré.
///
/// Le repère est un élément labellisé du document ; l'introspection rend sa
/// position, dont le numéro de page — ramené **à partir de zéro** pour rester
/// cohérent avec le reste de l'application.
fn chapter_page(document: &PagedDocument) -> Option<usize> {
    let introspector = document.introspector();

    // Le label se reconstruit par `construct`, seul constructeur public depuis
    // une chaîne — `Label::new` attend une chaîne internée.
    let label = Label::construct(CHAPTER_LABEL.into()).ok()?;

    let content = introspector.query_label(label).ok()?;
    let location = content.location()?;
    let position = introspector.position(location)?;

    Some(position.page.get() - 1)
}

/// Empreinte de ce qui a été compilé.
///
/// Sert à ne relancer la compilation que si le document a réellement changé.
/// Elle couvre l'assemblage **et** les fichiers fournis en mémoire : ne
/// comparer que l'assemblage rendrait l'aperçu sourd à toute frappe, puisque
/// l'assemblage ne change pas quand on écrit dans un chapitre.
pub fn fingerprint(entry: &str, overrides: &[(PathBuf, String)]) -> String {
    let mut fingerprint = String::with_capacity(entry.len() + 64);

    fingerprint.push_str(entry);

    for (path, content) in overrides {
        fingerprint.push('\u{0}');
        fingerprint.push_str(&path.to_string_lossy());
        fingerprint.push('\u{0}');
        fingerprint.push_str(content);
    }

    fingerprint
}

/// Rasterise une page du document en pixels RGBA bruts.
///
/// Renvoie `None` si l'index dépasse le nombre de pages.
///
/// C'est la seule opération coûteuse de l'aperçu (~30 ms pour une page A4) :
/// elle ne doit donc être déclenchée que pour les pages réellement
/// nécessaires, et jamais depuis le thread d'affichage.
pub fn render_page(document: &PagedDocument, index: usize) -> Option<PageImage> {
    let page = document.pages().get(index)?;

    let options = typst_render::RenderOptions {
        pixel_per_pt: typst_utils::Scalar::new(PIXEL_PER_PT),
        render_bleed: false,
    };

    let pixmap = typst_render::render(page, &options);

    // Les dimensions se lisent **avant** `take`, qui vide le tampon.
    let (width, height) = (pixmap.width(), pixmap.height());

    // `take` cède le tampon sans le recopier : les pixels partent tels quels
    // vers le Handle, sans repasser par un PNG.
    let pixels = pixmap.take();

    let hash = pixels_hash(&pixels);

    Some(PageImage {
        width,
        height,
        pixels,
        hash,
    })
}

/// Empreinte d'un tampon de pixels.
///
/// Sert à reconnaître deux rendus **identiques** de la même page. Ce n'est pas
/// une empreinte cryptographique : on ne cherche pas à résister à une collision
/// fabriquée, seulement à constater qu'une page n'a pas changé.
fn pixels_hash(pixels: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    pixels.hash(&mut hasher);

    hasher.finish()
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
pub fn render_pdf(root: &Path, entry: &str, overrides: &[(PathBuf, String)]) -> Option<Vec<u8>> {
    let world = PreviewWorld::new(root, entry, overrides);

    let document = typst::compile::<PagedDocument>(&world).output.ok()?;

    typst_pdf::pdf(&document, &Default::default()).ok()
}

/// Convertit les diagnostics du compilateur en diagnostics positionnés.
///
/// Chaque position est calculée **dans le fichier auquel appartient le span** :
/// l'assemblage inclut les chapitres, donc un diagnostic peut tomber n'importe
/// où. Rapporter une position calculée sur un autre fichier produirait un saut
/// au mauvais endroit — pire que pas de saut du tout.
fn convert_diagnostics(diagnostics: &[SourceDiagnostic], world: &PreviewWorld) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let (file, line, column) = locate(diagnostic.span, world);

            Diagnostic {
                message: diagnostic.message.to_string(),
                line,
                column,
                file,
            }
        })
        .collect()
}

/// Résout un span en (fichier, ligne, colonne).
///
/// Renvoie des valeurs absentes pour un span synthétique, sans fichier
/// d'origine : certains diagnostics du moteur n'en portent pas.
fn locate(span: DiagSpan, world: &PreviewWorld) -> (Option<PathBuf>, Option<usize>, Option<usize>) {
    let Some(id) = span.id() else {
        return (None, None, None);
    };

    let Some(range) = world.range(span) else {
        return (None, None, None);
    };

    let Ok(source) = world.source(id) else {
        return (None, None, None);
    };

    let file = world.path_of(id);

    match source.lines().byte_to_line_column(range.start) {
        // Les positions de typst sont 0-based, celles de l'interface 1-based.
        Some((line, column)) => (Some(file), Some(line + 1), Some(column + 1)),
        None => (Some(file), None, None),
    }
}

/// Un monde Typst minimal : le fichier d'entrée en mémoire, les autres lus
/// depuis le disque, les polices embarquées de Typst.
pub struct PreviewWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main_id: FileId,
    source: Source,

    /// Racine du projet, pour résoudre les chemins relatifs.
    root: std::path::PathBuf,

    /// Fichiers servis depuis la mémoire plutôt que depuis le disque.
    ///
    /// L'assemblage `#include` les chapitres : sans cela, l'aperçu afficherait
    /// le document tel qu'il était à la dernière sauvegarde, et la frappe
    /// n'apparaîtrait qu'avec un temps de retard.
    overrides: Vec<(PathBuf, String)>,
}

impl PreviewWorld {
    pub fn new(root: &Path, content: &str, overrides: &[(PathBuf, String)]) -> Self {
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
            overrides: overrides.to_vec(),
        }
    }

    /// Chemin sur disque d'un fichier référencé par un FileId.
    fn path_of(&self, id: FileId) -> PathBuf {
        self.root.join(id.vpath().get_without_slash())
    }

    /// Contenu en mémoire d'un fichier, s'il y en a un.
    ///
    /// La comparaison se fait sur le chemin canonique quand c'est possible :
    /// l'assemblage nomme ses includes en relatif, l'application connaît ses
    /// fichiers en absolu, et les deux doivent désigner le même fichier.
    fn override_for(&self, path: &Path) -> Option<&str> {
        self.overrides
            .iter()
            .find(|(candidate, _)| same_path(candidate, path))
            .map(|(_, content)| content.as_str())
    }
}

/// Deux chemins désignent-ils le même fichier ?
///
/// On compare les chemins canoniques lorsque c'est possible, et à défaut les
/// composants : un chemin inexistant ne peut pas être canonisé, et une erreur de
/// résolution ne doit pas faire perdre le contenu en mémoire.
fn same_path(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }

    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a.ends_with(b) || b.ends_with(a),
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

        // Le contenu en mémoire prime sur le disque : c'est ce qui permet à
        // l'aperçu de suivre la frappe malgré la sauvegarde différée.
        if let Some(text) = self.override_for(&path) {
            return Ok(Source::new(id, text.to_string()));
        }

        let text = std::fs::read_to_string(&path).map_err(|_| FileError::NotFound(path))?;

        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main_id {
            return Ok(Bytes::from_string(self.source.text().to_owned()));
        }

        let path = self.path_of(id);

        if let Some(text) = self.override_for(&path) {
            return Ok(Bytes::from_string(text.to_string()));
        }

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
        let outcome = compile(&dir, &content, &[]);
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
        let _second = compile(&dir, &edited, &[]);
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

        let outcome = compile(&dir, "= Titre\n\nDu texte avec du *gras*.\n", &[]);

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

        let outcome = compile(&dir, "= Titre\n\n#foo()\n", &[]);

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

        let outcome = compile(&dir, &content, &[]);

        assert!(outcome.errors.is_empty());
        assert_eq!(page_count(outcome.document.as_ref().unwrap()), 3);

        let _ = fs::remove_dir_all(&dir);
    }

    /// La rasterisation produit des pixels RGBA cohérents avec la géométrie
    /// de la page.
    #[test]
    fn renders_page_to_rgba() {
        let dir = std::env::temp_dir().join("typst-writer-raster");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "= Titre\n\nDu contenu.\n", &[]);
        let document = outcome.document.expect("document compilé");

        let page = render_page(&document, 0).expect("rendu de la page");

        // A4 à 2 px/pt : ~1191 × 1684 px.
        assert!(
            (page.width as f64 - 1191.0).abs() < 5.0,
            "largeur A4 attendue, eu {}",
            page.width
        );
        assert!(
            (page.height as f64 - 1684.0).abs() < 5.0,
            "hauteur A4 attendue, eu {}",
            page.height
        );

        // Le tampon doit être exactement un RGBA par pixel : `from_rgba`
        // l'exige, et un écart ferait planter le backend graphique.
        assert_eq!(
            page.pixels.len(),
            page.width as usize * page.height as usize * 4,
            "tampon RGBA de taille width * height * 4 attendu"
        );

        // La géométrie annoncée par `page_size` doit correspondre aux pixels
        // produits : c'est ce rapport qui règle la taille d'affichage. Les
        // deux voies de calcul ne s'arrondissent pas de la même façon, d'où
        // la tolérance d'un pixel — sans effet visible puisque l'image est
        // affichée en `ContentFit::Contain`.
        let (width, height) = page_size(&document, 0).expect("taille de page");

        assert!(
            (width - page.width as f64).abs() <= 1.0,
            "largeur cohérente entre la géométrie et le rendu : {width} vs {}",
            page.width
        );
        assert!(
            (height - page.height as f64).abs() <= 1.0,
            "hauteur cohérente entre la géométrie et le rendu : {height} vs {}",
            page.height
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

        let outcome = compile(&dir, "Inline $x_1^2$ et display $ sum_(i=1)^n i $", &[]);

        assert!(outcome.errors.is_empty(), "maths valides attendues");

        let document = outcome.document.expect("document compilé");
        let page = render_page(&document, 0).expect("rendu");

        assert_eq!(
            page.pixels.len(),
            page.width as usize * page.height as usize * 4
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un projet minimal : un assemblage qui règle la page et inclut un
    /// chapitre.
    fn project(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("typst-writer-{name}"));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        let chapter = dir.join("chapitres/01-intro.typ");
        fs::write(&chapter, "= Introduction\n\nDu texte.\n").unwrap();

        fs::write(
            dir.join("main.typ"),
            "#set page(paper: \"a6\")\n#set text(size: 11pt)\n\n#include \"chapitres/01-intro.typ\"\n",
        )
        .unwrap();

        (dir, chapter)
    }

    /// L'en-tête Typst est appliqué au rendu.
    ///
    /// C'est la raison d'être de compiler l'assemblage : composé seul, le
    /// chapitre ne verrait pas le `#set page` de l'en-tête et serait rendu au
    /// format par défaut — un aperçu qui ne correspond pas au PDF.
    #[test]
    fn the_assembly_header_is_applied_to_the_rendering() {
        let (dir, chapter) = project("header");

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();
        let chapter_source = fs::read_to_string(&chapter).unwrap();

        // Le document, dont l'en-tête impose un format A6.
        let document = compile(&dir, &assembly, &[]).document.expect("rendu");
        let (width, height) = page_size(&document, 0).expect("taille de page");

        // A6 : 105 × 148 mm, soit ≈ 595 × 839 px à 2 px/pt.
        assert!(
            (width - 595.0).abs() < 5.0,
            "largeur A6 attendue, eu {width}"
        );
        assert!(
            (height - 839.0).abs() < 5.0,
            "hauteur A6 attendue, eu {height}"
        );

        // Le même contenu sans l'en-tête : format par défaut, donc A4. C'est
        // l'écart entre les deux qui prouve que l'en-tête est bien appliqué.
        let alone = compile(&dir, &chapter_source, &[]).document.expect("rendu");
        let (plain_width, _) = page_size(&alone, 0).expect("taille de page");

        assert!(
            (plain_width - 1191.0).abs() < 5.0,
            "largeur A4 attendue sans en-tête, eu {plain_width}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Le contenu en mémoire prime sur le disque.
    ///
    /// L'assemblage `#include` les chapitres depuis le disque, alors que la
    /// sauvegarde est différée : sans cette priorité, l'aperçu aurait jusqu'à
    /// une seconde de retard sur la frappe.
    #[test]
    fn the_in_memory_version_wins_over_the_disk() {
        let (dir, chapter) = project("override");

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        // Sur le disque, le chapitre est valide.
        assert!(
            compile(&dir, &assembly, &[]).errors.is_empty(),
            "aucune erreur attendue depuis le disque"
        );

        // En mémoire, une faute que le disque ignore encore.
        let overrides = vec![(
            chapter.clone(),
            "= Introduction\n\n#inconnu()\n".to_string(),
        )];

        let outcome = compile(&dir, &assembly, &overrides);

        assert_eq!(
            outcome.errors.len(),
            1,
            "la version en mémoire doit être celle qui est compilée"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un diagnostic dans un chapitre inclus indique ce chapitre, et la bonne
    /// ligne.
    ///
    /// Calculer la position sur l'assemblage donnerait un saut au mauvais
    /// endroit — pire que pas de saut du tout.
    #[test]
    fn diagnostics_inside_a_chapter_point_to_that_chapter() {
        let (dir, chapter) = project("diag-file");

        fs::write(&chapter, "= Introduction\n\nDu texte.\n\n#inconnu()\n").unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        let outcome = compile(&dir, &assembly, &[]);

        assert_eq!(outcome.errors.len(), 1);

        let diagnostic = &outcome.errors[0];

        assert_eq!(
            diagnostic.file.as_deref(),
            Some(chapter.as_path()),
            "le diagnostic doit désigner le chapitre, pas l'assemblage"
        );
        assert_eq!(
            diagnostic.line,
            Some(5),
            "l'erreur est à la ligne 5 du chapitre"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Un diagnostic dans l'assemblage lui-même désigne l'assemblage.
    #[test]
    fn diagnostics_in_the_assembly_point_to_the_assembly() {
        let dir = std::env::temp_dir().join("typst-writer-diag-main");

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, "= Titre\n\n#inconnu()\n", &[]);

        assert_eq!(outcome.errors.len(), 1);
        assert_eq!(
            outcome.errors[0].file.as_deref(),
            Some(dir.join("main.typ").as_path())
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// L'empreinte couvre le fichier fourni en mémoire.
    ///
    /// Sinon l'aperçu serait sourd à la frappe : l'assemblage ne change pas
    /// quand on écrit dans un chapitre.
    #[test]
    fn the_fingerprint_notices_in_memory_changes() {
        let first = fingerprint("assemblage", &[]);
        let same = fingerprint("assemblage", &[]);

        assert_eq!(first, same, "une même entrée doit donner la même empreinte");

        let with_override = fingerprint(
            "assemblage",
            &[(PathBuf::from("/tmp/chapitre.typ"), "un".to_string())],
        );

        assert_ne!(
            first, with_override,
            "un apport en mémoire change l'empreinte"
        );

        let changed_override = fingerprint(
            "assemblage",
            &[(PathBuf::from("/tmp/chapitre.typ"), "deux".to_string())],
        );

        assert_ne!(
            with_override, changed_override,
            "changer le contenu en mémoire change l'empreinte"
        );
    }

    /// L'export PDF produit un PDF valide.
    #[test]
    fn exports_valid_pdf() {
        let dir = std::env::temp_dir().join("typst-writer-pdf");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let pdf = render_pdf(&dir, "= Titre\n\nDu contenu.\n", &[]).expect("export PDF");

        assert!(pdf.starts_with(b"%PDF"), "le PDF doit être valide");

        let _ = fs::remove_dir_all(&dir);
    }

    /// RH : deux chapitres dans l'assemblage, les deux doivent être rendus.
    #[test]
    fn both_chapters_render() {
        let dir = std::env::temp_dir().join("typst-writer-repro-chapitres");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        fs::write(dir.join("chapitres/01-un.typ"), "= Premier\n\nALPHA.\n").unwrap();
        fs::write(dir.join("chapitres/02-deux.typ"), "= Second\n\nBETA.\n").unwrap();
        fs::write(
            dir.join("main.typ"),
            "#set page(margin: 2cm)\n\n#include \"chapitres/01-un.typ\"\n\n#include \"chapitres/02-deux.typ\"\n",
        )
        .unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();
        let outcome = compile(&dir, &assembly, &[]);

        println!(
            "erreurs = {:?}",
            outcome
                .errors
                .iter()
                .map(|d| &d.message)
                .collect::<Vec<_>>()
        );
        let document = outcome.document.expect("document");
        println!("pages = {}", page_count(&document));

        let _ = fs::remove_dir_all(&dir);
    }
    /// Le repère se pose sur sa propre ligne, juste avant l'`#include` visé.
    #[test]
    fn the_marker_is_inserted_on_its_own_line() {
        let entry = "#set page(margin: 2cm)\n\n#include \"chapitres/01-un.typ\"\n\n#include \"chapitres/02-deux.typ\"\n";

        let marked = mark_chapter(entry, Path::new("chapitres/02-deux.typ")).expect("repère posé");

        assert_eq!(
            marked,
            "#set page(margin: 2cm)\n\n#include \"chapitres/01-un.typ\"\n\n\
             #metadata(none) <tw-chapitre>\n#include \"chapitres/02-deux.typ\"\n"
        );

        // Retirer le repère doit rendre l'assemblage exactement tel qu'il était :
        // le repère ne doit rien changer d'autre.
        let restored = marked.replace(&format!("{CHAPTER_MARKER}\n"), "");

        assert_eq!(restored, entry, "le repère ne doit rien modifier d'autre");

        // Un chapitre absent de l'assemblage n'a pas de page à désigner.
        assert_eq!(
            mark_chapter(entry, Path::new("chapitres/09-absent.typ")),
            None
        );
    }

    /// Le repère doit désigner la page du chapitre en cours, pas la première.
    ///
    /// C'est ce qui permet d'amener l'aperçu sur le chapitre qu'on écrit : sans
    /// lui il s'ouvre en tête d'assemblage, si bien qu'écrire le chapitre 7
    /// revenait à regarder le chapitre 1.
    #[test]
    fn the_marker_locates_the_current_chapter() {
        let dir = std::env::temp_dir().join("typst-writer-repere");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        // Le premier chapitre est poussé sur sa propre page par un saut
        // explicite : sans cela les deux tiendraient sur la même, et le test ne
        // pourrait pas distinguer l'un de l'autre.
        let first_text = "= Premier\n\nALPHA.\n\n#pagebreak()\n";

        fs::write(dir.join("chapitres/01-un.typ"), first_text).unwrap();
        fs::write(dir.join("chapitres/02-deux.typ"), "= Second\n\nBETA.\n").unwrap();
        fs::write(
            dir.join("main.typ"),
            "#include \"chapitres/01-un.typ\"\n\n#include \"chapitres/02-deux.typ\"\n",
        )
        .unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        // On écrit dans le second chapitre : sa version en mémoire est plus
        // fraîche que le disque, c'est donc elle que `overrides` porte.
        let second = dir.join("chapitres/02-deux.typ");
        let outcome = compile(
            &dir,
            &assembly,
            &[(second, "= Second\n\nBETA.\n".to_string())],
        );

        assert!(
            outcome.errors.is_empty(),
            "l'assemblage doit compiler : {:?}",
            outcome.errors
        );

        let pages = page_count(outcome.document.as_ref().expect("document"));

        assert!(
            pages >= 2,
            "il faut au moins deux pages, or il y en a {pages}"
        );

        // Deuxième page, comptée à partir de zéro.
        assert_eq!(outcome.chapter_page, Some(1));

        // Et le premier chapitre reste bien en tête.
        let first = dir.join("chapitres/01-un.typ");
        let outcome = compile(&dir, &assembly, &[(first, first_text.to_string())]);

        assert_eq!(outcome.chapter_page, Some(0));

        let _ = fs::remove_dir_all(&dir);
    }

    /// L'assemblage ouvert lui-même ne porte aucun repère : il n'y a pas de
    /// chapitre en cours dont on chercherait la page.
    #[test]
    fn no_marker_when_the_assembly_is_open() {
        let dir = std::env::temp_dir().join("typst-writer-repere-vide");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("main.typ"), "= Titre\n\nDu contenu.\n").unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();
        let outcome = compile(&dir, &assembly, &[]);

        assert_eq!(outcome.chapter_page, None);

        let _ = fs::remove_dir_all(&dir);
    }
    /// Le texte réellement composé sur les pages d'un document.
    ///
    /// Descend dans les groupes : le texte d'une page est rangé dans des
    /// sous-cadres, et s'arrêter au premier niveau ne lirait rien.
    fn composed_text(document: &PagedDocument) -> String {
        fn frame_text(frame: &typst::layout::Frame, out: &mut String) {
            for (_, item) in frame.items() {
                match item {
                    typst::layout::FrameItem::Text(text) => out.push_str(&text.text),
                    typst::layout::FrameItem::Group(group) => {
                        frame_text(&group.frame, out);
                    }
                    _ => {}
                }
            }
        }

        let mut text = String::new();

        for page in document.pages() {
            frame_text(&page.frame, &mut text);
        }

        text
    }

    /// Un fichier chargé dans le projet doit finir **sur la page**.
    ///
    /// C'est le défaut signalé, vu de bout en bout : le fichier s'affichait
    /// dans l'éditeur sans jamais être rendu. Le test ne se contente donc pas
    /// de vérifier que l'assemblage mentionne un `#include` — il lit le texte
    /// réellement composé sur les pages.
    #[test]
    fn a_loaded_file_ends_up_on_the_page() {
        let elsewhere = std::env::temp_dir().join("typst-writer-charge-source");
        let _ = fs::remove_dir_all(&elsewhere);
        fs::create_dir_all(&elsewhere).unwrap();

        let source = elsewhere.join("mon-texte.typ");

        fs::write(&source, "= Chapitre Chargé\n\nBILLET DOUZE.\n").unwrap();

        let dir = std::env::temp_dir().join("typst-writer-charge-projet");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        // Un projet vide : l'assemblage n'inclut encore rien.
        fs::write(dir.join("main.typ"), "#set page(margin: 2cm)\n").unwrap();

        let project = crate::project::load(&dir);
        let chapter = crate::project::import_chapter(&project, &source).expect("import du fichier");

        assert!(
            chapter.exists(),
            "le chapitre importé doit exister : {}",
            chapter.display()
        );

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        assert!(
            assembly.contains("#include \"chapitres/01-chapitre-charge.typ\""),
            "l'assemblage doit inclure le chapitre chargé, or il contient :\n{assembly}"
        );

        // Compilation de l'assemblage, exactement comme le fait l'aperçu.
        let outcome = compile(&dir, &assembly, &[]);

        assert!(
            outcome.errors.is_empty(),
            "erreurs de compilation : {:?}",
            outcome
                .errors
                .iter()
                .map(|diagnostic| &diagnostic.message)
                .collect::<Vec<_>>()
        );

        let document = outcome.document.expect("document compilé");
        let text = composed_text(&document);

        assert!(
            text.contains("BILLET DOUZE"),
            "le texte du fichier chargé doit être composé sur la page, or on lit : {text:?}"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&elsewhere);
    }
    /// DIAGNOSTIC : les tailles de texte réellement composées.
    ///
    /// Ignoré par défaut : il compile un projet réel de la machine, pas une
    /// fixture. Sert à distinguer ce qu'on **croit** écrit de ce qui est
    /// vraiment composé — un chiffre en exposant, par exemple, se voit ici
    /// comme un texte de taille réduite, et nulle part ailleurs.
    ///
    ///     cargo test -- --ignored diagnose_the_real_project --nocapture
    #[test]
    #[ignore]
    fn diagnose_the_real_project() {
        let home = std::env::var("HOME").expect("HOME");
        let name = std::env::var("PROJET").unwrap_or_else(|_| "projet".to_string());
        let dir = PathBuf::from(home).join(format!(".typst-writer/projets/{name}"));

        let Ok(assembly) = fs::read_to_string(dir.join("main.typ")) else {
            println!("pas de projet à {}", dir.display());
            return;
        };

        let outcome = compile(&dir, &assembly, &[]);

        println!(
            "erreurs = {:?}",
            outcome
                .errors
                .iter()
                .map(|diagnostic| &diagnostic.message)
                .collect::<Vec<_>>()
        );

        let Some(document) = outcome.document else {
            return;
        };

        println!("pages = {}", page_count(&document));

        let mut runs: Vec<(f32, String)> = Vec::new();

        for page in document.pages() {
            collect_runs(&page.frame, &mut runs);
        }

        // Les tailles distinctes, avec un exemple de texte pour chacune : c'est
        // ce qui dit si quelque chose est composé plus petit que le corps.
        let mut sizes: Vec<(u32, Vec<(f32, String)>)> = Vec::new();

        for (size, text) in runs {
            if !text.trim().is_empty() {
                let key = (size * 100.0).round() as u32;

                match sizes.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, group)) => {
                        if group.len() < 6 {
                            group.push((size, text));
                        }
                    }
                    None => sizes.push((key, vec![(size, text)])),
                }
            }
        }

        sizes.sort_by_key(|(key, _)| *key);

        for (_, group) in &sizes {
            let size = group[0].0;

            println!("\n=== taille {size:.2} pt ({} exemples)", group.len());

            for (_, text) in group {
                let text: String = text.chars().take(60).collect();

                println!("    {text:?}");
            }
        }
    }

    /// Rassemble les textes composés, avec leur taille.
    fn collect_runs(frame: &typst::layout::Frame, out: &mut Vec<(f32, String)>) {
        for (_, item) in frame.items() {
            match item {
                typst::layout::FrameItem::Text(text) => {
                    out.push((text.size.to_pt() as f32, text.text.to_string()));
                }
                typst::layout::FrameItem::Group(group) => collect_runs(&group.frame, out),
                _ => {}
            }
        }
    }
    /// DIAGNOSTIC : d'où viennent les chiffres en exposant ?
    ///
    /// Ignoré par défaut. Compare des documents minimaux pour isoler la cause :
    /// chacun n'ajoute qu'une chose au précédent.
    #[test]
    #[ignore]
    fn diagnose_superscripts() {
        let variants: &[(&str, &str)] = &[
            ("texte nu", "Texte ordinaire, page 1.\n"),
            (
                "avec numbering de page",
                "#set page(numbering: \"1/1\")\n\nTexte ordinaire, page 1.\n",
            ),
            (
                "numbering + set super",
                "#set super(baseline: -0.5em, size: 0.5em, typographic: false)\n#set page(numbering: \"1/1\")\n\nTexte ordinaire, page 1.\n",
            ),
            (
                "set super seul",
                "#set super(baseline: -0.5em, size: 0.5em, typographic: false)\n\nTexte ordinaire, page 1.\n",
            ),
            (
                "une note de bas de page",
                "Texte avec une note#footnote[Le contenu de la note.] suite.\n",
            ),
        ];

        for (name, content) in variants {
            let dir = std::env::temp_dir().join("typst-writer-diag-exposant");
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();

            let outcome = compile(&dir, content, &[]);

            println!("\n=== {name} (erreurs: {:?})", outcome.errors.len());

            let Some(document) = outcome.document else {
                println!("    pas de document");
                continue;
            };

            let mut runs: Vec<(f32, String)> = Vec::new();

            for page in document.pages() {
                collect_runs(&page.frame, &mut runs);
            }

            let mut sizes: Vec<(u32, f32, String)> = Vec::new();

            for (size, text) in runs {
                if text.trim().is_empty() {
                    continue;
                }

                let key = (size * 100.0).round() as u32;

                if !sizes.iter().any(|(k, _, _)| *k == key) {
                    sizes.push((key, size, text.chars().take(28).collect()));
                }
            }

            sizes.sort_by_key(|(key, _, _)| *key);

            for (_, size, sample) in sizes {
                println!("    {size:>6.2} pt  {sample:?}");
            }

            let _ = fs::remove_dir_all(&dir);
        }
    }

    /// DIAGNOSTIC : les textes d'une page, dans l'ordre, avec leur taille.
    ///
    /// Ignoré par défaut. Sert à voir **où** tombe un texte anormal : un
    /// chiffre en exposant au milieu d'une phrase n'est pas la même chose qu'un
    /// numéro de page en pied de page.
    #[test]
    #[ignore]
    fn diagnose_one_page_in_order() {
        let home = std::env::var("HOME").expect("HOME");
        let name = std::env::var("PROJET").unwrap_or_else(|_| "mon-travail".to_string());
        let page_wanted: usize = std::env::var("PAGE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);

        let dir = PathBuf::from(home).join(format!(".typst-writer/projets/{name}"));

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();
        let outcome = compile(&dir, &assembly, &[]);

        let Some(document) = outcome.document else {
            println!("pas de document");
            return;
        };

        let Some(page) = document.pages().get(page_wanted) else {
            println!("pas de page {page_wanted}");
            return;
        };

        let mut runs: Vec<(f32, String)> = Vec::new();

        collect_runs(&page.frame, &mut runs);

        for (size, text) in runs {
            if text.trim().is_empty() {
                continue;
            }

            let text: String = text.chars().take(48).collect();

            // Les tailles anormalement petites sont mises en évidence : c'est
            // là que se cachent les exposants.
            let mark = if size < 9.0 { "  <-- PETIT" } else { "" };

            println!("{size:>6.2}  {text:?}{mark}");
        }
    }

    /// DIAGNOSTIC : enregistre une page en PNG, pour la regarder.
    ///
    /// Ignoré par défaut. Un artefact d'affichage ne se déduit pas des tailles
    /// composées : il faut voir l'image telle que l'aperçu la produit.
    #[test]
    #[ignore]
    fn diagnose_save_page_png() {
        let home = std::env::var("HOME").expect("HOME");
        let name = std::env::var("PROJET").unwrap_or_else(|_| "mon-travail".to_string());
        let page_wanted: usize = std::env::var("PAGE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);

        let dir = PathBuf::from(home).join(format!(".typst-writer/projets/{name}"));

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();
        let outcome = compile(&dir, &assembly, &[]);

        let Some(document) = outcome.document else {
            println!("pas de document");
            return;
        };

        let Some(page) = document.pages().get(page_wanted) else {
            println!("pas de page {page_wanted}");
            return;
        };

        // Exactement les réglages de l'aperçu.
        let options = typst_render::RenderOptions {
            pixel_per_pt: typst_utils::Scalar::new(PIXEL_PER_PT),
            render_bleed: false,
        };

        let pixmap = typst_render::render(page, &options);

        let out = std::env::temp_dir().join(format!("typst-writer-page-{page_wanted}.png"));

        match pixmap.save_png(&out) {
            Ok(()) => println!("écrit : {}", out.display()),
            Err(error) => println!("échec de l'écriture : {error}"),
        }

        println!(
            "taille : {} × {} px, à {PIXEL_PER_PT} px/pt",
            pixmap.width(),
            pixmap.height()
        );
    }

    /// DIAGNOSTIC : les glyphes d'un texte, un par un.
    ///
    /// Ignoré par défaut. L'exposant ne se voit **pas** dans la taille du texte
    /// — qui reste celle du corps — mais dans le décalage vertical et l'avance
    /// de chaque glyphe. C'est là qu'il faut regarder, et nulle part ailleurs.
    #[test]
    #[ignore]
    fn diagnose_glyphs() {
        fn dig(frame: &typst::layout::Frame, needle: &str, found: &mut Vec<(f32, Vec<String>)>) {
            for (_, item) in frame.items() {
                match item {
                    typst::layout::FrameItem::Text(text) => {
                        if text.text.contains(needle) {
                            let chars: Vec<char> = text.text.chars().collect();

                            let glyphs = text
                                .glyphs
                                .iter()
                                .map(|glyph| {
                                    let range = glyph.range.start as usize
                                        ..(glyph.range.end as usize).min(chars.len());

                                    let piece: String = chars[range].iter().collect();

                                    format!(
                                        "{piece:?} avance {:.3}em décalage {:+.3}em",
                                        glyph.x_advance.get(),
                                        glyph.y_offset.get()
                                    )
                                })
                                .collect();

                            found.push((text.size.to_pt() as f32, glyphs));
                        }
                    }
                    typst::layout::FrameItem::Group(group) => dig(&group.frame, needle, found),
                    _ => {}
                }
            }
        }

        fn probe(name: &str, content: &str) {
            let dir = std::env::temp_dir().join("typst-writer-diag-glyphes");
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();

            let outcome = compile(&dir, content, &[]);

            let mut found = Vec::new();

            if let Some(document) = outcome.document.as_ref() {
                for page in document.pages() {
                    dig(&page.frame, "Août", &mut found);
                }
            }

            println!("\n=== {name}");

            for (size, glyphs) in found {
                println!("    taille du texte : {size:.2} pt");

                for glyph in glyphs {
                    println!("        {glyph}");
                }
            }

            let _ = fs::remove_dir_all(&dir);
        }

        probe("nu", "Août 2026\n");
        probe(
            "avec fractions",
            "#set text(fractions: true)\n\nAoût 2026\n",
        );
        probe(
            "avec set super",
            "#set super(baseline: -0.5em, size: 0.5em, typographic: false)\n\nAoût 2026\n",
        );
        probe(
            "les deux",
            "#set text(fractions: true)\n#set super(baseline: -0.5em, size: 0.5em, typographic: false)\n\nAoût 2026\n",
        );
    }

    /// DIAGNOSTIC : `fractions: true` sur les chiffres, en image.
    ///
    /// Ignoré par défaut. Les deux réglages sur la même page, pour comparer
    /// d'un seul coup d'œil : au-dessus avec la fonction activée, en dessous
    /// sans.
    #[test]
    #[ignore]
    fn diagnose_fractions_visually() {
        let content = "#set text(size: 20pt)\n\
             #set text(fractions: true)\n\
             AVEC fractions : Août 2026, 1994, 7 et 15.\n\
             \n\
             #set text(fractions: false)\n\
             SANS fractions : Août 2026, 1994, 7 et 15.\n";

        let dir = std::env::temp_dir().join("typst-writer-diag-fractions");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let outcome = compile(&dir, content, &[]);

        println!("erreurs : {:?}", outcome.errors.len());

        let Some(document) = outcome.document else {
            println!("pas de document");
            return;
        };

        let options = typst_render::RenderOptions {
            pixel_per_pt: typst_utils::Scalar::new(PIXEL_PER_PT),
            render_bleed: false,
        };

        let pixmap = typst_render::render(&document.pages()[0], &options);

        // Recadrage sur le haut de la page : c'est là qu'est le texte.
        let out = std::env::temp_dir().join("typst-writer-fractions.png");

        match pixmap.save_png(&out) {
            Ok(()) => println!("écrit : {}", out.display()),
            Err(error) => println!("échec : {error}"),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    /// L'empreinte qui décide de recompiler doit être **celle** que la
    /// compilation rapporte.
    ///
    /// Divergentes, elles rendaient toute compilation « nouvelle » : l'aperçu
    /// recompilait à chaque tick — plus de trois fois par seconde — sans qu'on
    /// ait écrit un seul caractère, et reprenait toutes les pages visibles à
    /// chaque fois. C'est le défaut qui se voyait comme un clignotement
    /// permanent, et que le repos n'arrivait pas à éteindre puisqu'il croyait
    /// toujours qu'il venait d'y avoir un changement.
    #[test]
    fn the_fingerprint_that_decides_is_the_one_compilation_returns() {
        let dir = std::env::temp_dir().join("typst-writer-empreinte");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        fs::write(dir.join("chapitres/01-un.typ"), "= Un\n\nALPHA.\n").unwrap();
        fs::write(dir.join("chapitres/02-deux.typ"), "= Deux\n\nBETA.\n").unwrap();
        fs::write(
            dir.join("main.typ"),
            "#include \"chapitres/01-un.typ\"\n\n#include \"chapitres/02-deux.typ\"\n",
        )
        .unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        // Les trois cas qui comptent : aucun chapitre ouvert, un chapitre
        // inclus — donc un repère posé —, et un chapitre absent de
        // l'assemblage, donc aucun repère.
        let cases: Vec<(&str, Vec<(PathBuf, String)>)> = vec![
            ("assemblage ouvert", Vec::new()),
            (
                "chapitre inclus",
                vec![(
                    dir.join("chapitres/02-deux.typ"),
                    "= Deux\n\nBETA.\n".to_string(),
                )],
            ),
            (
                "chapitre absent de l'assemblage",
                vec![(
                    dir.join("chapitres/99-ailleurs.typ"),
                    "= Ailleurs\n".to_string(),
                )],
            ),
        ];

        for (name, overrides) in &cases {
            let outcome = compile(&dir, &assembly, overrides);

            assert_eq!(
                outcome.content,
                compilation_fingerprint(&dir, &assembly, overrides),
                "« {name} » : l'empreinte du tick doit être exactement celle que la                  compilation rapporte, sinon l'aperçu se croit toujours dépassé"
            );
        }

        // Le piège, montré. Dès qu'un chapitre est ouvert, le repère fait
        // partie de l'empreinte : qui la calculerait sur l'entrée **brute**
        // verrait un changement à chaque tick, indéfiniment — c'était
        // exactement l'état du tick. L'écrire ici rend le piège explicite,
        // plutôt que de laisser le prochain lecteur le redécouvrir.
        let marked = &cases[1].1;

        assert_ne!(
            fingerprint(&assembly, marked),
            compilation_fingerprint(&dir, &assembly, marked),
            "avec un chapitre ouvert, le repère doit entrer dans l'empreinte"
        );

        // Et sans chapitre ouvert, aucun repère : les deux écritures donnent
        // la même empreinte, donc le piège ne se referme que dans le premier
        // cas.
        assert_eq!(
            fingerprint(&assembly, &[]),
            compilation_fingerprint(&dir, &assembly, &[]),
            "sans chapitre ouvert, aucune divergence possible"
        );

        // Et l'empreinte du chapitre inclus porte bien le repère, sans quoi
        // changer de chapitre ne serait pas remarqué.
        let first = vec![(
            dir.join("chapitres/01-un.typ"),
            "= Un\n\nALPHA.\n".to_string(),
        )];
        let second = vec![(
            dir.join("chapitres/02-deux.typ"),
            "= Deux\n\nBETA.\n".to_string(),
        )];

        assert_ne!(
            compilation_fingerprint(&dir, &assembly, &first),
            compilation_fingerprint(&dir, &assembly, &second),
            "changer de chapitre doit se remarquer"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Après une compilation, plus aucune ne doit être due — même longtemps
    /// après, et sans qu'on ait rien écrit.
    ///
    /// C'est le bout de la chaîne que l'on voyait à l'usage : l'aperçu
    /// recompilait **plusieurs fois par seconde au repos**, parce que
    /// l'empreinte rapportée par la compilation n'était jamais celle
    /// qu'observait le tick. Le repos n'y pouvait rien : à chaque tick, le
    /// texte semblait avoir changé.
    #[test]
    fn after_compiling_nothing_more_is_due() {
        let dir = std::env::temp_dir().join("typst-writer-repos");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("chapitres")).unwrap();

        fs::write(dir.join("chapitres/01-un.typ"), "= Un\n\nALPHA.\n").unwrap();
        fs::write(dir.join("main.typ"), "#include \"chapitres/01-un.typ\"\n").unwrap();

        let assembly = fs::read_to_string(dir.join("main.typ")).unwrap();

        // Un chapitre est ouvert : c'est le cas d'usage, et celui où le repère
        // entre dans l'empreinte.
        let overrides = vec![(
            dir.join("chapitres/01-un.typ"),
            "= Un\n\nALPHA.\n".to_string(),
        )];

        // 1. Ce que le tick observe, juste avant de décider.
        let observed = compilation_fingerprint(&dir, &assembly, &overrides);

        // 2. La compilation, et ce qu'elle rapporte.
        let outcome = compile(&dir, &assembly, &overrides);

        // 3. L'état retient ce que la compilation a rapporté.
        let mut render = crate::app::state::RenderState::default();
        render.content = outcome.content.clone();

        // 4. Le tick réobserve : plus rien ne doit être dû, indéfiniment.
        let start = std::time::Instant::now();

        for tick in 0..50u64 {
            let later = start + std::time::Duration::from_millis(tick * 300);

            assert!(
                !render.compile_due(&observed, false, later),
                "tick {tick} : au repos, l'aperçu ne doit plus rien recompiler"
            );
        }

        // Et la fin de paragraphe ne doit pas rouvrir la porte : le texte n'a
        // pas changé, donc il n'y a rien à recompiler, quoi qu'il arrive.
        assert!(
            !render.compile_due(&observed, true, start + std::time::Duration::from_secs(5)),
            "un paragraphe terminé ne fait pas recompiler un texte inchangé"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
