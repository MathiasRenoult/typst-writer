//! Conversion d'un source Typst en modèle de document rendable.
//!
//! Le rendu live passe par ici : le source est parsé en un arbre syntaxique
//! (`typst-syntax`), puis transformé en un modèle de blocs et de contenu en
//! ligne que la vue convertit en widgets iced. Aucune compilation PDF n'est
//! nécessaire : le parse est quasi instantané, ce qui rend la mise à jour
//! réellement « live ».

use typst_syntax::{SyntaxKind, SyntaxNode};

/// Un document parsé, prêt à être rendu.
#[derive(Debug, Clone, Default)]
pub struct Doc {
    /// Blocs de contenu, dans l'ordre du document.
    pub blocks: Vec<Block>,

    /// Messages des erreurs de syntaxe détectées (5 maximum).
    pub errors: Vec<String>,
}

/// Un bloc de contenu.
#[derive(Debug, Clone)]
pub enum Block {
    /// Titre de niveau 1 à 6.
    Heading { level: u8, content: Vec<Inline> },

    /// Paragraphe de texte.
    Paragraph(Vec<Inline>),

    /// Élément de liste (à puces ou numérotée), avec son contenu imbriqué.
    ListItem { marker: Marker, content: Vec<Block> },

    /// Élément de liste de termes : `/ Terme: définition`.
    TermItem {
        term: Vec<Inline>,
        definition: Vec<Inline>,
    },

    /// Bloc de code brut (délimité par trois backticks ou plus).
    CodeBlock { text: String },

    /// Équation mathématique.
    Equation { text: String },
}

/// Marqueur d'élément de liste.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Marker {
    /// `- item`
    Bullet,

    /// `+ item` ou `1. item`, avec son numéro dans l'énumération.
    Enum(usize),
}

/// Contenu en ligne.
#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Code(String),
    Link { text: String, url: String },
    Equation(String),
    LineBreak,
    Fallback(String),
}

/// Parse un source Typst en modèle de document.
pub fn parse(source: &str) -> Doc {
    let root = typst_syntax::parse(source);

    let (errors, _warnings) = root.errors_and_warnings();
    let errors = errors
        .iter()
        .take(5)
        .map(|error| error.message.to_string())
        .collect();

    let mut blocks = Vec::new();
    walk_blocks(&root, &mut blocks);

    Doc { blocks, errors }
}

/// Parcourt les enfants d'un nœud markup (racine ou contenu d'item) et
/// produit les blocs. Les nœuds en ligne sont accumulés dans un paragraphe.
fn walk_blocks(node: &SyntaxNode, out: &mut Vec<Block>) {
    let mut paragraph: Vec<Inline> = Vec::new();
    let mut enum_number = 0usize;
    let children: Vec<&SyntaxNode> = node.children().collect();
    let mut index = 0;

    while index < children.len() {
        let child = children[index];
        index += 1;

        // `#fonction(...)` : le `Hash` précède toujours l'appel de fonction.
        if child.kind() == SyntaxKind::Hash {
            if children
                .get(index)
                .is_some_and(|next| next.kind() == SyntaxKind::FuncCall)
            {
                paragraph.extend(inline_func_call(children[index], true));
                index += 1;
            }

            continue;
        }

        match child.kind() {
            SyntaxKind::Parbreak => {
                if flush_paragraph(&mut paragraph, out) {
                    // Un paragraphe interrompu termine l'énumération courante.
                    enum_number = 0;
                }
            }

            SyntaxKind::Heading => {
                flush_paragraph(&mut paragraph, out);
                enum_number = 0;

                let level = heading_level(child);
                let mut content = inline_children(child);
                trim_edges(&mut content);

                out.push(Block::Heading { level, content });
            }

            SyntaxKind::ListItem | SyntaxKind::EnumItem => {
                flush_paragraph(&mut paragraph, out);

                let (marker, mut content) = match child.kind() {
                    SyntaxKind::EnumItem => {
                        enum_number += 1;
                        (Marker::Enum(enum_number), Vec::new())
                    }
                    _ => {
                        enum_number = 0;
                        (Marker::Bullet, Vec::new())
                    }
                };

                for inner in child.children() {
                    if inner.kind() == SyntaxKind::Markup {
                        walk_blocks(inner, &mut content);
                    }
                }

                out.push(Block::ListItem { marker, content });
            }

            SyntaxKind::TermItem => {
                flush_paragraph(&mut paragraph, out);
                enum_number = 0;

                let mut term = Vec::new();
                let mut definition = Vec::new();
                let mut after_colon = false;

                for inner in child.children() {
                    match inner.kind() {
                        SyntaxKind::TermMarker | SyntaxKind::Space => {}
                        SyntaxKind::Colon => after_colon = true,
                        SyntaxKind::Markup => {
                            let inlines = inline_children(inner);

                            if after_colon {
                                definition.extend(inlines);
                            } else {
                                term.extend(inlines);
                            }
                        }
                        _ => {}
                    }
                }

                trim_edges(&mut term);
                trim_edges(&mut definition);

                out.push(Block::TermItem { term, definition });
            }

            SyntaxKind::Raw if is_block_raw(child) => {
                flush_paragraph(&mut paragraph, out);
                enum_number = 0;

                out.push(Block::CodeBlock {
                    text: raw_text(child),
                });
            }

            // Directives de mise en page et expressions de code : ignorées
            // par le rendu (elles n'ont pas d'équivalent visuel direct).
            SyntaxKind::SetRule
            | SyntaxKind::LetBinding
            | SyntaxKind::ShowRule
            | SyntaxKind::ModuleImport
            | SyntaxKind::ImportItems
            | SyntaxKind::ImportItemPath
            | SyntaxKind::RenamedImportItem
            | SyntaxKind::ModuleInclude
            | SyntaxKind::Contextual
            | SyntaxKind::Conditional
            | SyntaxKind::WhileLoop
            | SyntaxKind::ForLoop => {}

            // Tout le reste est du contenu en ligne, accumulé dans le
            // paragraphe courant.
            _ => paragraph.extend(inline_node(child)),
        }
    }

    flush_paragraph(&mut paragraph, out);
}

/// Vide le paragraphe accumulé dans la liste de blocs.
/// Retourne `true` si un paragraphe a effectivement été produit.
fn flush_paragraph(paragraph: &mut Vec<Inline>, out: &mut Vec<Block>) -> bool {
    trim_edges(paragraph);

    if paragraph.is_empty() {
        return false;
    }

    // Une équation seule sur sa ligne devient un bloc centré.
    if let [Inline::Equation(text)] = paragraph.as_slice() {
        out.push(Block::Equation { text: text.clone() });

        paragraph.clear();

        return true;
    }

    out.push(Block::Paragraph(std::mem::take(paragraph)));

    true
}

/// Niveau d'un titre (nombre de `=` du marqueur), borné à 1..=6.
fn heading_level(node: &SyntaxNode) -> u8 {
    node.children()
        .find(|child| child.kind() == SyntaxKind::HeadingMarker)
        .map(|marker| marker.full_text().chars().filter(|c| *c == '=').count() as u8)
        .unwrap_or(1)
        .clamp(1, 6)
}

/// Un `Raw` dont le délimiteur compte trois backticks ou plus est un bloc.
fn is_block_raw(node: &SyntaxNode) -> bool {
    node.children()
        .any(|child| child.kind() == SyntaxKind::RawDelim && child.full_text().chars().count() >= 3)
}

/// Concatène le texte brut d'un nœud `Raw` (hors délimiteurs et langue).
fn raw_text(node: &SyntaxNode) -> String {
    let mut text = String::new();

    for child in node.children() {
        if child.kind() == SyntaxKind::Text {
            text.push_str(&child.full_text());
        }
    }

    text
}

/// Texte d'une équation (contenu du nœud `Math`).
fn equation_text(node: &SyntaxNode) -> String {
    for child in node.children() {
        if child.kind() == SyntaxKind::Math {
            return child.full_text().trim().to_string();
        }
    }

    node.full_text().trim_matches('$').trim().to_string()
}

/// Convertit les enfants d'un nœud en contenu en ligne.
fn inline_children(node: &SyntaxNode) -> Vec<Inline> {
    let mut out = Vec::new();

    for child in node.children() {
        out.extend(inline_node(child));
    }

    out
}

/// Convertit un nœud en contenu en ligne.
fn inline_node(node: &SyntaxNode) -> Vec<Inline> {
    match node.kind() {
        SyntaxKind::Text => vec![Inline::Text(node.full_text().to_string())],

        // Les espaces sont réduits à une espace simple (les sauts de ligne
        // et l'indentation des listes disparaissent).
        SyntaxKind::Space => vec![Inline::Text(" ".to_string())],

        SyntaxKind::Strong => vec![Inline::Bold(inline_children(node))],
        SyntaxKind::Emph => vec![Inline::Italic(inline_children(node))],
        SyntaxKind::Raw => vec![Inline::Code(raw_text(node))],

        SyntaxKind::Escape => vec![Inline::Text(unescape(&node.full_text()))],
        SyntaxKind::Shorthand => vec![Inline::Text(map_shorthand(&node.full_text()))],
        SyntaxKind::SmartQuote => vec![Inline::Text(node.full_text().to_string())],

        SyntaxKind::Link => {
            let url = node.full_text().to_string();

            vec![Inline::Link {
                text: url.clone(),
                url,
            }]
        }

        // Les labels sont invisibles dans le rendu.
        SyntaxKind::Label => Vec::new(),

        // `@cible[texte]` : le texte personnalisé remplace la cible.
        SyntaxKind::Ref => {
            let mut marker = String::new();
            let mut content_block = None;

            for child in node.children() {
                match child.kind() {
                    SyntaxKind::RefMarker => marker = child.full_text().to_string(),
                    SyntaxKind::ContentBlock => content_block = Some(child),
                    _ => {}
                }
            }

            match content_block {
                Some(block) => inline_children(block),
                None => vec![Inline::Text(marker)],
            }
        }

        SyntaxKind::Equation => vec![Inline::Equation(equation_text(node))],
        SyntaxKind::Linebreak => vec![Inline::LineBreak],

        // Les erreurs de syntaxe sont affichées telles quelles.
        SyntaxKind::Error => vec![Inline::Fallback(node.full_text().to_string())],

        SyntaxKind::Hash => Vec::new(),
        SyntaxKind::FuncCall => inline_func_call(node, false),
        SyntaxKind::ContentBlock => inline_children(node),
        SyntaxKind::Markup => inline_children(node),

        _ => Vec::new(),
    }
}

/// Convertit un appel de fonction en ligne.
///
/// Un petit ensemble de fonctions a un rendu natif (`emph`, `strong`,
/// `text`, `link`) ; les autres sont affichées brutes.
fn inline_func_call(node: &SyntaxNode, with_hash: bool) -> Vec<Inline> {
    let name = node
        .children()
        .find(|child| child.kind() == SyntaxKind::Ident)
        .map(|child| child.full_text().to_string())
        .unwrap_or_default();

    let content_block = find_content_block(node);

    let rendered = match name.as_str() {
        "emph" => content_block.map(|block| vec![Inline::Italic(inline_children(&block))]),
        "strong" => content_block.map(|block| vec![Inline::Bold(inline_children(&block))]),
        "text" => content_block.map(|block| inline_children(&block)),
        "link" => {
            let url = find_string(node);

            Some(match (url, content_block) {
                (Some(url), Some(block)) => {
                    let inlines = inline_children(&block);
                    let text = inline_to_string(&inlines);

                    vec![Inline::Link { text, url }]
                }
                (Some(url), None) => vec![Inline::Link {
                    text: url.clone(),
                    url,
                }],
                _ => Vec::new(),
            })
        }
        _ => None,
    };

    match rendered {
        Some(inlines) if !inlines.is_empty() => inlines,
        _ => {
            let mut text = if with_hash {
                "#".to_string()
            } else {
                String::new()
            };

            text.push_str(&node.full_text());

            vec![Inline::Fallback(text)]
        }
    }
}

/// Premier `ContentBlock` (`[...]`) parmi les descendants directs des args.
fn find_content_block(node: &SyntaxNode) -> Option<SyntaxNode> {
    let args = node
        .children()
        .find(|child| child.kind() == SyntaxKind::Args)?;

    args.children()
        .find(|child| child.kind() == SyntaxKind::ContentBlock)
        .cloned()
}

/// Première chaîne de caractères littérale dans les arguments.
fn find_string(node: &SyntaxNode) -> Option<String> {
    fn search(node: &SyntaxNode) -> Option<String> {
        for child in node.children() {
            if child.kind() == SyntaxKind::Str {
                return Some(child.full_text().trim_matches('"').to_string());
            }

            // Le contenu entre crochets peut lui-même contenir des chaînes :
            // on ne le traverse pas pour trouver l'URL du lien.
            if child.kind() != SyntaxKind::ContentBlock {
                if let Some(found) = search(child) {
                    return Some(found);
                }
            }
        }

        None
    }

    search(node)
}

/// Aplatit le contenu en ligne en texte brut (pour le libellé d'un lien).
fn inline_to_string(inlines: &[Inline]) -> String {
    let mut text = String::new();

    for inline in inlines {
        match inline {
            Inline::Text(t) | Inline::Code(t) | Inline::Equation(t) => text.push_str(t),
            Inline::Bold(inner) | Inline::Italic(inner) => text.push_str(&inline_to_string(inner)),
            Inline::Link { text: t, .. } => text.push_str(t),
            Inline::LineBreak => text.push(' '),
            Inline::Fallback(t) => text.push_str(t),
        }
    }

    text
}

/// Retire les espaces et sauts de ligne en bordure de contenu en ligne.
fn trim_edges(inlines: &mut Vec<Inline>) {
    let is_blank = |inline: &Inline| match inline {
        Inline::Text(t) => t.trim().is_empty(),
        Inline::LineBreak => true,
        _ => false,
    };

    while inlines.first().is_some_and(is_blank) {
        inlines.remove(0);
    }

    while inlines.last().is_some_and(is_blank) {
        inlines.pop();
    }
}

/// Interprète une séquence d'échappement (`\#`, `\u{1F600}`, ...).
fn unescape(text: &str) -> String {
    let rest = text.strip_prefix('\\').unwrap_or(text);

    if let Some(hex) = rest.strip_prefix("u{").and_then(|r| r.strip_suffix('}')) {
        if let Ok(code) = u32::from_str_radix(hex, 16) {
            if let Some(ch) = char::from_u32(code) {
                return ch.to_string();
            }
        }
    }

    rest.to_string()
}

/// Interprète un raccourci typographique (`~`, `--`, `...`).
fn map_shorthand(text: &str) -> String {
    match text {
        "~" => " ".to_string(),
        "-?" => String::new(),
        "--" => "–".to_string(),
        "---" => "—".to_string(),
        "..." => "…".to_string(),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inline_text(inlines: &[Inline]) -> String {
        inline_to_string(inlines)
    }

    #[test]
    fn parses_headings_and_inline_styles() {
        let doc = parse("= Titre\n\n== Sous-titre\n\nUn *gras*, de l'_italique_ et `code`.");

        assert_eq!(doc.blocks.len(), 3);

        match &doc.blocks[0] {
            Block::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(inline_text(content), "Titre");
            }
            other => panic!("titre attendu, obtenu {other:?}"),
        }

        match &doc.blocks[1] {
            Block::Heading { level, content } => {
                assert_eq!(*level, 2);
                assert_eq!(inline_text(content), "Sous-titre");
            }
            other => panic!("titre attendu, obtenu {other:?}"),
        }

        match &doc.blocks[2] {
            Block::Paragraph(content) => {
                assert!(content.iter().any(|i| matches!(i, Inline::Bold(_))));
                assert!(content.iter().any(|i| matches!(i, Inline::Italic(_))));
                assert!(
                    content
                        .iter()
                        .any(|i| matches!(i, Inline::Code(t) if t == "code"))
                );
            }
            other => panic!("paragraphe attendu, obtenu {other:?}"),
        }
    }

    #[test]
    fn parses_lists_with_nesting_and_numbering() {
        let doc = parse("- un\n- deux\n  - imbriqué\n\n+ premier\n+ second");

        let bullets: Vec<_> = doc
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::ListItem {
                    marker: Marker::Bullet,
                    ..
                } => Some(()),
                _ => None,
            })
            .collect();
        assert_eq!(bullets.len(), 2, "deux items à puces");

        // L'item imbriqué est dans le contenu du deuxième item.
        match &doc.blocks[1] {
            Block::ListItem { content, .. } => {
                assert!(
                    content
                        .iter()
                        .any(|block| matches!(block, Block::ListItem { .. })),
                    "item imbriqué attendu dans le contenu"
                );
            }
            other => panic!("item de liste attendu, obtenu {other:?}"),
        }

        let enums: Vec<_> = doc
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::ListItem {
                    marker: Marker::Enum(n),
                    ..
                } => Some(*n),
                _ => None,
            })
            .collect();
        assert_eq!(enums, vec![1, 2], "numérotation des énumérations");
    }

    #[test]
    fn parses_code_blocks_and_equations() {
        let doc = parse("```rust\nfn main() {}\n```\n\nDu texte $x^2 + 1$ inline.");

        match &doc.blocks[0] {
            Block::CodeBlock { text } => {
                assert_eq!(text, "fn main() {}");
            }
            other => panic!("bloc de code attendu, obtenu {other:?}"),
        }

        match &doc.blocks[1] {
            Block::Paragraph(content) => {
                assert!(
                    content
                        .iter()
                        .any(|i| matches!(i, Inline::Equation(t) if t == "x^2 + 1")),
                    "équation inline attendue"
                );
            }
            other => panic!("paragraphe attendu, obtenu {other:?}"),
        }
    }

    #[test]
    fn parses_links_and_known_functions() {
        let doc = parse("Lien auto https://typst.org et #link(\"https://ex.com\")[texte].");

        let links: Vec<_> = doc
            .blocks
            .iter()
            .flat_map(|block| match block {
                Block::Paragraph(content) => content.iter().collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .filter_map(|inline| match inline {
                Inline::Link { text, url } => Some((text.clone(), url.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(
            links,
            vec![
                (
                    "https://typst.org".to_string(),
                    "https://typst.org".to_string()
                ),
                ("texte".to_string(), "https://ex.com".to_string()),
            ]
        );
    }

    #[test]
    fn unknown_functions_fall_back_to_raw_text() {
        let doc = parse("#figure(\n  [Légende]\n)\n");

        match &doc.blocks[0] {
            Block::Paragraph(content) => {
                assert!(
                    content
                        .iter()
                        .any(|i| matches!(i, Inline::Fallback(t) if t.starts_with("#figure"))),
                    "repli brut attendu pour #figure"
                );
            }
            other => panic!("paragraphe attendu, obtenu {other:?}"),
        }
    }

    #[test]
    fn layout_directives_are_skipped() {
        let doc = parse("#set page(margin: 2cm)\n#set text(size: 12pt)\n\nDu texte.");

        assert_eq!(doc.blocks.len(), 1, "les règles set sont ignorées");

        match &doc.blocks[0] {
            Block::Paragraph(content) => {
                assert_eq!(inline_text(content), "Du texte.");
            }
            other => panic!("paragraphe attendu, obtenu {other:?}"),
        }
    }

    #[test]
    fn reports_syntax_errors() {
        let doc = parse("Du *texte non fermé");

        assert!(!doc.errors.is_empty(), "erreur de syntaxe attendue");
    }

    #[test]
    fn parses_term_lists_and_escapes() {
        let doc = parse("/ Terme: définition\n\nÉchappement : \\# pas un titre.");

        match &doc.blocks[0] {
            Block::TermItem { term, definition } => {
                assert_eq!(inline_text(term), "Terme");
                assert_eq!(inline_text(definition), "définition");
            }
            other => panic!("item de terme attendu, obtenu {other:?}"),
        }

        match &doc.blocks[1] {
            Block::Paragraph(content) => {
                assert!(
                    content
                        .iter()
                        .any(|i| matches!(i, Inline::Text(t) if t == "#")),
                    "échappement `\\#` attendu"
                );
            }
            other => panic!("paragraphe attendu, obtenu {other:?}"),
        }
    }
}
