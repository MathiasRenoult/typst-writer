use crate::app::{Message, State};
use crate::typst::render::Diagnostic;
use iced::widget::svg::{Handle, Svg};
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, scrollable, text};
use iced::{ContentFit, Element, Fill, color};

/// Taille du texte courant.
const BODY_SIZE: f32 = 14.0;

pub fn view(state: &State) -> Element<'_, Message> {
    let render = &state.render;

    let mut content = column![].spacing(12).padding([16, 24]).width(Fill);

    if let Some(status) = &state.export_status {
        content = content.push(text(status).size(13).color(color!(0x8a8a8a)));
    }

    // Diagnostics du compilateur, en tête du panneau.
    if !render.errors.is_empty() {
        content = content.push(diagnostics_panel(&render.errors, true));
    }

    if !render.warnings.is_empty() {
        content = content.push(diagnostics_panel(&render.warnings, false));
    }

    // Les pages du document, rendues en SVG vectoriel.
    for page in &render.pages {
        content = content.push(
            Svg::new(Handle::from_memory(page.clone().into_bytes()))
                .width(Fill)
                .content_fit(ContentFit::Contain),
        );
    }

    // État d'attente avant le premier rendu.
    if render.pages.is_empty() && render.errors.is_empty() && state.export_status.is_none() {
        let message = if render.pending {
            "Rendu en cours…"
        } else if render.done {
            "Aucune page à afficher."
        } else {
            "Écrivez du contenu : le document est compilé automatiquement."
        };

        content = content.push(text(message).size(BODY_SIZE));
    }

    scrollable(content).width(Fill).height(Fill).into()
}

/// Bandeau listant les erreurs (ou avertissements) du compilateur, cliquables
/// pour sauter à la ligne concernée.
fn diagnostics_panel<'a>(diagnostics: &'a [Diagnostic], is_error: bool) -> Element<'a, Message> {
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
