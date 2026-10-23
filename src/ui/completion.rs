//! La fenêtre de suggestions de citation.
//!
//! Une fenêtre **fixe, au centre de l'écran**, ouverte par un `@`. À la
//! différence d'une bulle accrochée au curseur, elle n'a pas à calculer sa
//! place : elle est au centre, toujours, et rien ne peut la recouvrir ni la
//! faire sortir de l'écran. C'est ce qui la rend simple, et ce qui rend les
//! clics prévisibles.

use iced::widget::{Space, column, container, row};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::ui::editor::{bubble_style, completion_popup};

/// Largeur de la fenêtre de suggestions.
const LARGEUR: f32 = 400.0;

/// La fenêtre, ou sa place vide.
///
/// Sa place est **toujours** dans la page, comme celle de la fenêtre de mise en
/// page : la forme de l'arbre de widgets ne change jamais quand elle s'ouvre.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(open) = state.lsp.completion.as_ref() else {
        return container(Space::new()).into();
    };

    let palette = state.resolved.palette;

    let entete = row![
        palette.label("Choisir une référence").size(15),
        Space::new().width(Length::Fill),
        palette
            .label("Échap pour fermer")
            .size(11)
            .color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let pied = palette
        .label("↑↓ naviguer · Tab ou Entrée insérer")
        .size(11)
        .color(palette.text_muted);

    let fenetre = container(
        column![
            entete,
            completion_popup(open, palette, LARGEUR, state.lsp.awaiting),
            pied,
        ]
        .spacing(10),
    )
    .width(Length::Fixed(LARGEUR + 32.0))
    .padding(16)
    .style(bubble_style(palette));

    // La fenêtre est centrée dans toute la page, et le fond reste la page
    // elle-même : cliquer hors de la fenêtre tombe dans l'éditeur, ce qui
    // déplace le curseur et ferme la liste — le geste naturel de sortie.
    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
