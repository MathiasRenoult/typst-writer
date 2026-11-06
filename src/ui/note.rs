//! Le panneau de la note de bas de page.
//!
//! Le geste de note n'écrit rien dans la ligne : il ouvre ce panneau, la
//! note s'y écrit, et la **seule marque** vient se poser au curseur
//! (bug-report-7, point 15 — « je n'aime pas qu'il y ait le texte de
//! renvoi qui soit affiché »).

use iced::widget::{button, column, container, row, text_input};
use iced::{Element, Length, Theme};

use crate::app::Message;
use crate::app::state::State;
use crate::ui::theme::{self, Palette};

/// Le panneau — vide quand aucune note n'est en cours.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(note) = state.note_dialogue.as_ref() else {
        return iced::widget::Space::new().into();
    };
    let palette: Palette = state.resolved.palette;

    let carte = container(
        column![
            palette
                .label("La note de bas de page")
                .size(theme::jetons::SAILLANT),
            text_input("écrivez la note…", note)
                .on_input(Message::NoteTexte)
                .on_submit(Message::NoteValider)
                .padding(6)
                .width(Length::Fixed(380.0)),
            row![
                button(palette.label("Poser la note").size(theme::jetons::DETAIL),)
                    .on_press(Message::NoteValider)
                    .padding([5, 12]),
                button(palette.label("Annuler").size(theme::jetons::DETAIL))
                    .on_press(Message::NoteAnnuler)
                    .padding([5, 12]),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .padding(18)
    .style(theme::panneau(palette));

    crate::ui::voile(
        container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

// `Theme` est nommé par les génériques des widgets ; inutilisé en surface.
#[allow(dead_code)]
fn _theme_est_nomme(_: Theme) {}
