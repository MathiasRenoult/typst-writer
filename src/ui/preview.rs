use crate::app::{Message, State};
use crate::typst::check::CheckError;
use crate::typst::renderer::{Block, Inline, Marker};
use iced::widget::text::{Rich, Span, Wrapping};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Font, Padding, color, font};

/// Taille du texte courant.
const BODY_SIZE: f32 = 14.5;

/// Taille du code (brut et mathématique).
const CODE_SIZE: f32 = 13.0;

pub fn view(state: &State) -> Element<'_, Message> {
    let doc = &state.doc;

    let mut content = column![].spacing(12).padding([16, 24]).width(Fill);

    if let Some(status) = &state.export_status {
        content = content.push(text(status).size(13).color(color!(0x8a8a8a)));
    }

    // Diagnostics du vrai compilateur, en tête du panneau.
    if !state.check.errors.is_empty() {
        content = content.push(diagnostics_panel(&state.check.errors, true));
    }

    if !state.check.warnings.is_empty() {
        content = content.push(diagnostics_panel(&state.check.warnings, false));
    }

    for block in &doc.blocks {
        content = content.push(render_block(block, 0));
    }

    if doc.blocks.is_empty() && state.export_status.is_none() {
        content = content.push(
            text("Le document est vide.\n\nÉcrivez du contenu pour voir le rendu en direct.")
                .size(BODY_SIZE),
        );
    }

    scrollable(content).width(Fill).height(Fill).into()
}

/// Bandeau listant les erreurs (ou avertissements) du compilateur, cliquables
/// pour sauter à la ligne concernée.
fn diagnostics_panel<'a>(diagnostics: &'a [CheckError], is_error: bool) -> Element<'a, Message> {
    let title = if is_error {
        format!(
            "⚠ {} erreur(s) — le document ne compilera pas en PDF",
            diagnostics.len()
        )
    } else {
        format!("⚠ {} avertissement(s)", diagnostics.len())
    };

    let accent = if is_error {
        color!(0xff8a8a)
    } else {
        color!(0xe8c07a)
    };

    let mut list = column![text(title).size(13).color(accent)].spacing(4);

    for diagnostic in diagnostics {
        let location = match (diagnostic.line, diagnostic.column) {
            (Some(line), Some(column)) => format!("ligne {line}, colonne {column}"),
            (Some(line), None) => format!("ligne {line}"),
            _ => String::new(),
        };

        let label = if location.is_empty() {
            diagnostic.message.clone()
        } else {
            format!("{location} — {}", diagnostic.message)
        };

        let jump = diagnostic
            .line
            .zip(diagnostic.column)
            .map(|(line, column)| Message::GotoError(line, column));

        list = list.push(
            button(text(label).size(13).wrapping(Wrapping::Word).width(Fill))
                .on_press_maybe(jump)
                .padding([6, 10])
                .width(Fill)
                .style(button::text),
        );
    }

    container(list)
        .padding(10)
        .width(Fill)
        .style(panel_style(is_error))
        .into()
}

/// Fond et bordure du bandeau de diagnostics.
fn panel_style(is_error: bool) -> impl Fn(&iced::Theme) -> container::Style {
    let (background, border) = if is_error {
        (color!(0x3b2026), color!(0x9c4444))
    } else {
        (color!(0x332b1e), color!(0x8a7233))
    };

    move |_theme| container::Style {
        background: Some(background.into()),
        border: iced::Border {
            color: border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Rend un bloc à la profondeur d'indentation donnée.
fn render_block(block: &Block, depth: usize) -> Element<'_, Message> {
    match block {
        Block::Heading { level, content } => {
            let size = match level {
                1 => 26.0,
                2 => 21.0,
                3 => 17.5,
                _ => 15.5,
            };

            let spans = inline_spans(content, size, bold(Font::DEFAULT));

            Rich::with_spans(spans)
                .size(size)
                .width(Fill)
                .wrapping(Wrapping::Word)
                .on_link_click(Message::OpenLink)
                .into()
        }

        Block::Paragraph(content) => rich_paragraph(content, BODY_SIZE, Font::DEFAULT),

        Block::ListItem { marker, content } => {
            let label = match marker {
                Marker::Bullet => "•".to_string(),
                Marker::Enum(number) => format!("{number}."),
            };

            let mut inner = column![].spacing(6);

            for block in content {
                inner = inner.push(render_block(block, depth + 1));
            }

            row![text(label).size(BODY_SIZE), inner.width(Fill)]
                .spacing(8)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 20.0 * depth as f32,
                })
                .into()
        }

        Block::TermItem { term, definition } => {
            let term_spans = inline_spans(term, BODY_SIZE, bold(Font::DEFAULT));
            let definition_spans = inline_spans(definition, BODY_SIZE, Font::DEFAULT);

            row![
                Rich::with_spans(term_spans)
                    .size(BODY_SIZE)
                    .on_link_click(Message::OpenLink),
                text(" — ").size(BODY_SIZE),
                Rich::with_spans(definition_spans)
                    .size(BODY_SIZE)
                    .on_link_click(Message::OpenLink),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
        }

        Block::CodeBlock { text: code } => {
            container(text(code).font(Font::MONOSPACE).size(CODE_SIZE).width(Fill))
                .padding(12)
                .width(Fill)
                .style(container::secondary)
                .into()
        }

        Block::Equation { text: equation } => container(
            text(equation)
                .font(Font::MONOSPACE)
                .size(BODY_SIZE)
                .width(Fill)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .padding([8, 16])
        .width(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .style(container::secondary)
        .into(),
    }
}

/// Rend un paragraphe avec styles en ligne.
fn rich_paragraph(content: &[Inline], size: f32, base_font: Font) -> Element<'_, Message> {
    let spans = inline_spans(content, size, base_font);

    Rich::with_spans(spans)
        .size(size)
        .width(Fill)
        .wrapping(Wrapping::Word)
        .on_link_click(Message::OpenLink)
        .into()
}

/// Convertit le contenu en ligne en segments de texte riche.
fn inline_spans<'a>(
    content: &'a [Inline],
    size: f32,
    base_font: Font,
) -> Vec<Span<'a, String, Font>> {
    let mut spans = Vec::new();

    for inline in content {
        match inline {
            Inline::Text(text) => spans.push(Span::new(text.as_str()).size(size).font(base_font)),

            Inline::Bold(inner) => spans.extend(inline_spans(inner, size, bold(base_font))),

            Inline::Italic(inner) => spans.extend(inline_spans(inner, size, italic(base_font))),

            Inline::Code(code) => spans.push(
                Span::new(code.as_str())
                    .size(size * 0.92)
                    .font(monospace(base_font)),
            ),

            Inline::Link { text: label, url } => spans.push(
                Span::new(label.as_str())
                    .size(size)
                    .font(base_font)
                    .link(url.clone()),
            ),

            Inline::Equation(equation) => spans.push(
                Span::new(equation.as_str())
                    .size(size)
                    .font(monospace(base_font)),
            ),

            Inline::LineBreak => spans.push(Span::new("\n").size(size).font(base_font)),

            Inline::Fallback(fallback) => spans.push(
                Span::new(fallback.as_str())
                    .size(size * 0.9)
                    .font(monospace(base_font))
                    .color(color!(0x8a8a8a)),
            ),
        }
    }

    spans
}

fn bold(font: Font) -> Font {
    Font {
        weight: font::Weight::Bold,
        ..font
    }
}

fn italic(font: Font) -> Font {
    Font {
        style: font::Style::Italic,
        ..font
    }
}

fn monospace(font: Font) -> Font {
    Font {
        family: font::Family::Monospace,
        ..font
    }
}
