//! La fenêtre de projet : créer, ou renommer.
//!
//! Un champ, une action, et `Échap` qui annule — le panneau de gauche
//! garde un seul bouton, et c'est ici que le nom s'écrit.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// L'identifiant du champ du nom : la fenêtre s'ouvre dessus.
pub fn id_du_nom() -> iced::widget::Id {
    iced::widget::Id::new("projet-nom")
}

/// La fenêtre, quand un projet attend son nom.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(dialogue) = state.projet_dialogue.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;
    let creation = dialogue.cible.is_none();

    let corps = column![
        palette
            .label(if creation {
                "Nouveau projet"
            } else {
                "Renommer le projet"
            })
            .size(theme::jetons::CORPS),
        text_input("nom du projet", &dialogue.nom)
            .id(id_du_nom())
            .on_input(Message::ProjetNom)
            .on_submit(Message::ProjetValider)
            .padding(6)
            .width(Length::Fixed(260.0)),
        row![
            button(
                palette
                    .label(if creation { "Créer" } else { "Renommer" })
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ProjetValider)
            .padding([5, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::ProjetAnnuler)
                .padding([5, 12]),
        ]
        .spacing(theme::jetons::PAS),
    ]
    .spacing(theme::jetons::PAS);

    let fenetre = container(corps).padding(16).style(theme::panneau(palette));

    crate::ui::voile(container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        )
}
