//! L'outil de tableau : la grille s'écrit, il n'y a qu'à choisir sa taille.
//!
//! Comme sur les outils d'écriture qui aident sans s'imposer : deux
//! nombres, une action, et la forme markdown se pose au curseur.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// L'identifiant du premier champ : la fenêtre s'ouvre dessus.
pub fn id_du_champ() -> iced::widget::Id {
    iced::widget::Id::new("tableau-rangs")
}

/// La fenêtre, quand un tableau attend sa taille.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(tableau) = state.tableau.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;

    let corps = column![
        palette
            .label("Insérer un tableau")
            .size(theme::jetons::CORPS),
        row![
            palette.label("Rangs").size(theme::jetons::DETAIL),
            text_input("3", &tableau.rangs)
                .id(id_du_champ())
                .on_input(Message::TableauRangs)
                .on_submit(Message::TableauInserer)
                .padding(5)
                .width(Length::Fixed(70.0)),
            palette.label("Colonnes").size(theme::jetons::DETAIL),
            text_input("3", &tableau.colonnes)
                .on_input(Message::TableauColonnes)
                .on_submit(Message::TableauInserer)
                .padding(5)
                .width(Length::Fixed(70.0)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        row![
            button(palette.label("Insérer").size(theme::jetons::DETAIL))
                .on_press(Message::TableauInserer)
                .padding([5, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::TableauAnnuler)
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
