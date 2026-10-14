use crate::app::{Message, State};

use crate::app::state::Completion;
use crate::ui::theme::Palette;

use iced::widget::{button, column, container, row, space};
use iced::{Element, Fill, Length, Padding, alignment};

/// Mesure de lecture : la largeur maximale du texte, en pixels.
///
/// Environ 75 caractères à la police monospace de l'éditeur (un caractère
/// occupe ≈ 0,6 fois la taille du texte). Au-delà, retrouver le début de la
/// ligne suivante demande à l'œil un repérage qui coûte à la lecture ; en
/// deçà, la ligne se coupe trop souvent.
const MEASURE: f32 = 760.0;

/// Marge latérale minimale.
///
/// Elle n'a d'effet que si le panneau est plus étroit que la mesure ; sinon
/// c'est la mesure elle-même qui laisse du blanc de chaque côté.
const SIDE_MARGIN: f32 = 40.0;

/// Marge en haut du texte : respirer avant la première ligne.
const TOP_MARGIN: f32 = 28.0;

/// Largeur de la liste de complétion.
const COMPLETION_WIDTH: f32 = 320.0;

/// Hauteur approximative d'une ligne de la liste, en pixels.
///
/// C'est une **estimation**, et elle suffit : la place calculée est bornée au
/// panneau, donc une imprécision ne peut pas faire sortir la liste de l'écran.
const COMPLETION_ROW: f32 = 28.0;

/// Combien d'éléments la liste montre d'un coup.
const COMPLETION_VISIBLE: usize = 8;

/// Marge en bas : de quoi voir venir la suite sans que le texte touche le bord.
const BOTTOM_MARGIN: f32 = 64.0;

/// La zone de rédaction.
///
/// Le texte est borné par la mesure et centré dans le panneau : c'est ce qui
/// donne à l'éditeur des marges latérales sur une fenêtre large, et ce qui
/// borne le retour à la ligne à une longueur lisible.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    // La largeur demandée est fixe, mais iced la borne à l'espace disponible :
    // sur une fenêtre étroite le texte se réduit au lieu de déborder, et les
    // marges latérales prennent le relais.
    let editor = state.editor.editor.view().map(Message::Editor);

    let page = match state.lsp.completion.as_ref() {
        // La liste se dessine **par-dessus** le texte, à l'endroit du curseur :
        // c'est là que le regard est déjà, et l'on choisit sans quitter la ligne
        // qu'on écrit.
        Some(open) if !open.shown.is_empty() => {
            // La place disponible est la fenêtre moins les deux barres et les
            // marges du panneau : c'est le seul cadre où le curseur peut être.
            let panel = (state.window_size.height
                - crate::ui::toolbar::BAR_HEIGHT
                - crate::ui::status_bar::BAR_HEIGHT
                - TOP_MARGIN
                - BOTTOM_MARGIN)
                .max(0.0);

            let menu = open.shown.len().min(COMPLETION_VISIBLE) as f32 * COMPLETION_ROW + 2.0;

            let top = crate::lsp::completion::placement(open.anchor.y, panel, menu);

            let popup = container(completion_popup(open, palette))
                .width(Fill)
                .height(Fill)
                .align_x(alignment::Horizontal::Left)
                .align_y(alignment::Vertical::Top)
                // Le décalage se donne par une marge intérieure : c'est ainsi
                // qu'un élément de pile se pose ailleurs qu'au coin.
                .padding(Padding {
                    top,
                    left: open.anchor.x,
                    ..Padding::default()
                });

            container(iced::widget::stack![editor, popup])
        }
        _ => container(editor),
    }
    .width(Fill)
    .height(Fill)
    .max_width(MEASURE);

    // La zone de rédaction se détache sur le fond de l'application : sans
    // cette différence de teinte, la marge existe en géométrie mais reste
    // invisible — on ne voit pas de marge sur un fond uniforme.
    container(page)
        .width(Fill)
        .height(Fill)
        .align_x(alignment::Horizontal::Center)
        .padding(Padding {
            top: TOP_MARGIN,
            right: SIDE_MARGIN,
            bottom: BOTTOM_MARGIN,
            left: SIDE_MARGIN,
        })
        // Le tour est au fond de l'application, la page au ton du panneau : la
        // différence est ce qui rend la marge visible.
        .style(move |_| container::Style {
            background: Some(palette.background.into()),
            ..Default::default()
        })
        .into()
}

/// La liste de complétion, posée à l'endroit du curseur.
fn completion_popup<'a>(open: &'a Completion, palette: Palette) -> Element<'a, Message> {
    let total = open.shown.len();

    // La fenêtre suit la sélection : on doit pouvoir choisir au clavier plus bas
    // que ce qui tient à l'écran.
    let first = open
        .selected
        .saturating_sub(COMPLETION_VISIBLE.saturating_sub(1))
        .min(total.saturating_sub(COMPLETION_VISIBLE));

    let mut list = column![]
        .spacing(0.0)
        .width(Length::Fixed(COMPLETION_WIDTH));

    for (position, index) in open
        .shown
        .iter()
        .enumerate()
        .skip(first)
        .take(COMPLETION_VISIBLE)
    {
        let Some(item) = open.items.get(*index) else {
            continue;
        };

        let selected = position == open.selected;

        let mut line = row![]
            .width(Fill)
            .padding([5, 8])
            .spacing(8)
            .align_y(alignment::Vertical::Center);

        line = line.push(palette.label(&item.label).size(13).color(if selected {
            palette.background
        } else {
            palette.text
        }));

        if let Some(kind) = item.kind
            && !kind.label().is_empty()
        {
            line = line.push(space::horizontal().width(Length::Fill));
            line = line.push(palette.label(kind.label()).size(11).color(if selected {
                palette.background
            } else {
                palette.text_muted
            }));
        }

        list = list.push(
            button(line)
                .width(Fill)
                .padding(0)
                .style(move |_, _| button::Style {
                    background: selected.then(|| palette.accent.into()),
                    text_color: if selected {
                        palette.background
                    } else {
                        palette.text
                    },
                    ..Default::default()
                })
                .on_press(Message::CompletionPick(position)),
        );
    }

    container(list)
        .width(Length::Fixed(COMPLETION_WIDTH))
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La mesure doit rester dans la fourchette de lisibilité : entre 45 et
    /// 90 caractères. À la police de l'éditeur, un caractère vaut environ 0,6
    /// fois la taille du texte.
    #[test]
    fn measure_is_within_readable_bounds() {
        let character_width = crate::editor::BODY_SIZE * 0.6;
        let characters = MEASURE / character_width;

        assert!(
            (45.0..=90.0).contains(&characters),
            "{characters:.0} caractères par ligne : hors de la fourchette lisible"
        );
    }

    /// Les marges doivent laisser de la place au texte : une mesure plus étroite
    /// que les deux marges réunies ne laisserait rien.
    #[test]
    fn margins_leave_room_for_the_text() {
        assert!(MEASURE > SIDE_MARGIN * 2.0);
        assert!(TOP_MARGIN > 0.0, "le texte ne doit pas toucher le haut");
        assert!(BOTTOM_MARGIN > 0.0, "le texte ne doit pas toucher le bas");
    }
}
