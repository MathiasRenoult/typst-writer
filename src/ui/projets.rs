//! La vue des projets : tous, classés sur des étagères.
//!
//! Une étagère est un nom — « thèse », « articles », « cours » — qu'on
//! pose à côté d'un projet. Le classement ne touche pas le projet : c'est
//! une vue de travail, et un projet déplacé garde son étagère par son nom.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// La fenêtre, quand la vue des projets est ouverte.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    if !state.projets_vue {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    let etageres = crate::project::etageres();

    // Les projets par étagère, dans l'ordre des noms.
    let mut par_etagere: std::collections::BTreeMap<String, Vec<&crate::project::Project>> =
        std::collections::BTreeMap::new();

    for projet in &state.projects {
        let etagere = etageres.get(&projet.name).cloned().unwrap_or_default();
        par_etagere.entry(etagere).or_default().push(projet);
    }

    let mut corps = column![].spacing(14);

    // Le nom de l'étagère se donne une fois, et se pose à chaque projet.
    corps = corps.push(
        row![
            text_input("nom de l'étagère", &state.etagere_brouillon)
                .on_input(Message::EtagereBrouillon)
                .padding(5)
                .width(Length::Fixed(200.0)),
            palette
                .label("Entrée : l'étagère vide retire le projet du classement.")
                .size(theme::jetons::DETAIL),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    );

    for (etagere, projets) in par_etagere {
        let titre = if etagere.is_empty() {
            "Sans étagère".to_string()
        } else {
            etagere.clone()
        };

        corps = corps.push(palette.label(titre).size(theme::jetons::SAILLANT));

        for projet in projets {
            corps = corps.push(
                row![
                    button(palette.label(&projet.name).size(theme::jetons::CORPS))
                        .on_press(Message::OpenProject(projet.dir.clone()))
                        .padding([5, 10]),
                    button(
                        palette
                            .label("Poser sur l'étagère")
                            .size(theme::jetons::DETAIL)
                    )
                    .on_press(Message::EtagereDefinir(
                        projet.name.clone(),
                        state.etagere_brouillon.clone()
                    ))
                    .padding([5, 10]),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            );
        }
    }

    if state.projects.is_empty() {
        corps = corps.push(palette.label("Aucun projet pour l'instant.").size(theme::jetons::DETAIL));
    }

    let fenetre = container(column![
        palette.label("Les projets").size(theme::jetons::SAILLANT),
        Space::new().height(6),
        corps,
    ])
    .padding(18)
    .width(Length::Fixed(420.0))
    .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
