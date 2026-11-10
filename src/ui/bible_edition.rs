//! Le panneau flottant d'édition d'une référence biblique.
//!
//! Il est ouvert depuis « La référence cliquée » du panneau gauche. Le
//! formulaire est une fenêtre à part : le panneau latéral garde sa liste, et
//! le clavier ne peut pas continuer à écrire dans l'éditeur derrière elle.

use crate::app::Message;
use crate::plugins::bible::{ReferenceEdition, Traduction, livres};
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, container, pick_list, row, text_input};
use iced::{Alignment, Element, Length};

/// Le premier champ du formulaire : le chapitre, qui reçoit le clavier.
pub fn id_premier_champ() -> iced::widget::Id {
    iced::widget::Id::new("bible-edition-chapitre")
}

/// La couche flottante : la fenêtre quand une édition est ouverte, sinon rien.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(edition) = state.plugins.bible.edition.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(edition, state.resolved.palette)
}

fn fenetre<'a>(edition: &'a ReferenceEdition, palette: Palette) -> Element<'a, Message> {
    let tous = livres(Traduction::Lsg);
    let options: Vec<String> = tous.iter().map(|livre| livre.nom.clone()).collect();
    let courant = edition
        .livre
        .and_then(|index| tous.get(index))
        .map(|livre| livre.nom.clone());

    let entete = row![
        palette
            .label("Modifier la référence")
            .size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label("La balise sera réécrite")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let formulaire = column![
        pick_list(options, courant, move |nom| {
            let index = tous.iter().position(|livre| livre.nom == nom).unwrap_or(0);
            Message::BibleEditionLivre(index)
        })
        .width(Length::Fill)
        .placeholder("Choisir un livre"),
        row![
            text_input("Chapitre (1-11)", &edition.chapitre)
                .id(id_premier_champ())
                .on_input(Message::BibleEditionChapitre)
                .on_submit(Message::BibleEditionEnregistrer)
                .padding(6)
                .width(Length::Fixed(130.0)),
            text_input("Versets (16, 16-18)", &edition.versets)
                .on_input(Message::BibleEditionVersets)
                .on_submit(Message::BibleEditionEnregistrer)
                .padding(6)
                .width(Length::Fill),
        ]
        .spacing(8),
    ]
    .spacing(10);

    let pied = row![
        palette
            .label("Tab · Entrée pour enregistrer · Échap")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().width(Length::Fill),
        button(palette.label("Annuler").size(theme::jetons::CORPS))
            .on_press(Message::BibleEditionAnnuler)
            .padding([6, 12]),
        button(palette.label("Enregistrer").size(theme::jetons::CORPS))
            .on_press(Message::BibleEditionEnregistrer)
            .padding([6, 12]),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let fenetre = container(column![entete, formulaire, pied].spacing(16))
        .width(Length::Fixed(460.0))
        .padding(18)
        .style(crate::ui::editor::bubble_style(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
