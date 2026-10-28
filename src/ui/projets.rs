//! La vue des projets : tous, classés sur des étagères.
//!
//! Une étagère est un nom — « thèse », « articles », « cours » — qu'on
//! pose à côté d'un projet. Le classement ne touche pas le projet : c'est
//! une vue de travail, et un projet déplacé garde son étagère par son nom.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// La vignette d'un projet : sa première page, quand elle est rendue.
fn vignette<'a>(state: &'a crate::app::State, nom: &str) -> Element<'a, Message> {
    let pixels = state
        .vignettes
        .iter()
        .find(|(cle, ..)| cle == nom)
        .cloned();

    match pixels {
        Some((_, pixels, largeur, hauteur)) => {
            let handle = iced::widget::image::Handle::from_rgba(largeur, hauteur, pixels);

            container(
                iced::widget::image(handle).width(Length::Fixed(56.0)),
            )
            .style(theme::panneau(state.resolved.palette))
            .padding(2)
            .into()
        }
        None => container(
            Space::new().width(Length::Fixed(56.0)).height(Length::Fixed(72.0)),
        )
        .style(theme::panneau(state.resolved.palette))
        .into(),
    }
}

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

        let etagere_bis = etagere.clone();
        let etagere_ter = etagere.clone();

        let bandeau = row![
            palette.label(titre.clone()).size(theme::jetons::SAILLANT),
            Space::new().width(Length::Fill),
            button(
                palette
                    .label("Nouveau projet ici")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ProjetDepuisEtagere(etagere.clone()))
            .padding([4, 10]),
        ]
        .align_y(iced::Alignment::Center);

        corps = corps.push(
            iced::widget::mouse_area(bandeau)
                .on_enter(Message::DragSurvol(etagere_bis))
                .on_release(Message::DragPoser(etagere_ter)),
        );

        for projet in projets {
            let nom = projet.name.clone();
            let nom_bis = projet.name.clone();
            let dir = projet.dir.clone();

            // Le glisser-déposer : la prise à la souris, la pose sur
            // l'étagère (bug 3 du rapport 2).
            let ligne = row![
                vignette(state, &projet.name),
                button(palette.label(&projet.name).size(theme::jetons::CORPS))
                    .on_press(Message::OpenProject(dir))
                    .padding([5, 10]),
                button(
                    palette
                        .label("Poser sur l'étagère")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::EtagereDefinir(
                    nom.clone(),
                    state.etagere_brouillon.clone()
                ))
                .padding([5, 10]),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center);

            corps = corps.push(
                iced::widget::mouse_area(ligne).on_press(Message::DragCommencer(nom_bis)),
            );
        }
    }

    if state.projects.is_empty() {
        corps = corps.push(palette.label("Aucun projet pour l'instant.").size(theme::jetons::DETAIL));
    }

    let fenetre = container(column![
        row![
            palette
                .label("Les projets")
                .size(theme::jetons::SAILLANT),
            Space::new().width(Length::Fill),
            button(theme::icon(
                lucide_icons::iced::icon_x::<iced::Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::ProjetsVue)
            .padding(6),
        ]
        .align_y(iced::Alignment::Center),
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
