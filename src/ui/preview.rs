use crate::app::{Message, State};
use crate::typst::render::Diagnostic;
use iced::widget::image::{Handle, Viewer};
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{ContentFit, Element, Fill, color};

/// Taille du texte courant.
const BODY_SIZE: f32 = 14.0;

/// Espacement entre les pages affichées.
const PAGE_GAP: f32 = 12.0;

/// Marge intérieure du panneau.
const PANEL_PADDING: f32 = 16.0;

pub fn view(state: &State) -> Element<'_, Message> {
    let render = &state.render;

    let mut content = column![]
        .spacing(PAGE_GAP)
        .padding(PANEL_PADDING)
        .width(Fill);

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

    // Les pages du document : seules les pages rasterisées portent une image ;
    // les autres sont des cadres de la taille exacte de la page, pour que la
    // géométrie du défilement reste stable pendant le rendu.
    let mut first_page_height = None;

    if let Some(document) = render.document.as_ref() {
        let count = crate::typst::render::page_count(document);

        for index in 0..count {
            content = content.push(page_view(state, index));

            if first_page_height.is_none() {
                if let Some((_, height)) = crate::typst::render::page_size(document, index) {
                    first_page_height = Some((height / 2.0) as f32);
                }
            }
        }
    }

    // État d'attente avant la première compilation.
    if render.document.is_none() && render.errors.is_empty() && state.export_status.is_none() {
        let message = if render.pending {
            "Compilation…"
        } else if render.done {
            "Aucune page à afficher."
        } else {
            "Écrivez du contenu : le document est compilé automatiquement."
        };

        content = content.push(text(message).size(BODY_SIZE));
    }

    // Hauteur logique d'une page (première page du document).
    let page_height = first_page_height.unwrap_or(842.0);

    scrollable(content)
        .width(Fill)
        .height(Fill)
        .on_scroll(move |viewport| {
            let (first, last) = visible_pages(&viewport, page_height);

            Message::ViewportChanged(first, last)
        })
        .into()
}

/// Calcule la première et la dernière page visibles dans le viewport, avec une
/// marge d'une page de chaque côté (les hauteurs de pages varient, et le
/// scroll pendant le rendu ne doit jamais découvrir un cadre vide).
fn visible_pages(
    viewport: &iced::widget::scrollable::Viewport,
    page_height: f32,
) -> (usize, usize) {
    if page_height <= 0.0 {
        return (0, 0);
    }

    let stride = page_height + PAGE_GAP;
    let top = viewport.absolute_offset().y - PANEL_PADDING;
    let bottom = top + viewport.bounds().height;

    let first = (top / stride).floor().max(0.0) as usize;
    let last = (bottom / stride).ceil() as usize;

    // Marge d'une page de chaque côté.
    (first.saturating_sub(1), last + 1)
}

/// Une page : l'image PNG si elle est rasterisée, sinon un cadre gris de la
/// même taille (la géométrie du document est connue avant tout rendu).
fn page_view(state: &State, index: usize) -> Element<'_, Message> {
    let render = &state.render;

    let document = match render.document.as_ref() {
        Some(document) => document,
        None => return space().width(Fill).height(400).into(),
    };

    let (width, height) =
        crate::typst::render::page_size(document, index).unwrap_or((595.0, 842.0));

    // Le PNG est rendu à 2 px/pt : on l'affiche à sa taille logique (÷2).
    let logical_width = (width / 2.0) as f32;
    let logical_height = (height / 2.0) as f32;

    // Double tampon : l'image de la page est affichée dès qu'elle existe,
    // même si elle date de la compilation précédente. Une page modifiée se
    // met à jour en place quand son nouveau rendu arrive, sans jamais
    // disparaître de l'écran.
    match render.page_cache.get(&index) {
        Some(png) => row![
            Viewer::new(Handle::from_bytes(png.clone()))
                .width(iced::Length::Fixed(logical_width))
                .height(iced::Length::Fixed(logical_height))
                .content_fit(ContentFit::Contain)
        ]
        .width(Fill)
        .into(),

        None => container(
            text(if render.rendering.contains(&index) {
                "Rendu…"
            } else {
                ""
            })
            .size(13)
            .color(color!(0x666666)),
        )
        .width(iced::Length::Fixed(logical_width))
        .height(iced::Length::Fixed(logical_height))
        .center_x(Fill)
        .center_y(Fill)
        .style(container::rounded_box)
        .into(),
    }
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
