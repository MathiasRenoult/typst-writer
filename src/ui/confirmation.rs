//! La confirmation d'un geste qui défait quelque chose.
//!
//! Une question, deux réponses, et `Échap` qui est une réponse — le
//! geste part ou s'efface, jamais entre les deux.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row};
use iced::{Element, Length};

/// La fenêtre, quand un geste attend son mot.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(confirmation) = state.confirmation.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;

    let corps = column![
        palette
            .label(&confirmation.question)
            .size(theme::jetons::CORPS),
        row![
            button(palette.label("Confirmer").size(theme::jetons::DETAIL))
                .on_press(Message::ConfirmationConfirmer)
                .padding([5, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::ConfirmationAnnuler)
                .padding([5, 12]),
        ]
        .spacing(theme::jetons::PAS),
    ]
    .spacing(theme::jetons::PAS);

    let fenetre = container(corps).padding(16).style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
