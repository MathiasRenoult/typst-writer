//! La fenêtre du glossaire : la liste des termes, et les gestes qui
//! les conduisent — consulter, ajouter, modifier, supprimer, et
//! insérer au curseur (bug-report-10, 5.3).
//!
//! Elle remplace la liste déroulante de la barre : un terme se
//! cherche, se lit, se corrige — la barre garde un seul geste,
//! ouvrir la fenêtre.

use crate::app::Message;
use crate::app::state::State;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 420.0;

/// La couche : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &State) -> Element<'_, Message> {
    if !state.plugins.glossaire.fenetre {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    let palette = state.resolved.palette;
    let filtre = state.plugins.glossaire.filtre.trim().to_lowercase();

    let mut liste = column![].spacing(4);

    for entree in &state.plugins.glossaire.entrees {
        let cherche = filtre.is_empty()
            || entree.key.to_lowercase().contains(&filtre)
            || entree
                .short
                .as_deref()
                .is_some_and(|forme| forme.to_lowercase().contains(&filtre))
            || entree
                .long
                .as_deref()
                .is_some_and(|forme| forme.to_lowercase().contains(&filtre));

        if !cherche {
            continue;
        }

        let formes = [entree.short.as_deref(), entree.long.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" — ");

        let mut rang = row![
            column![
                palette.label(entree.key.as_str()).size(13),
                palette.label(formes).size(11).color(palette.text_muted),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            button(palette.label("Insérer").size(theme::jetons::DETAIL))
                .on_press(Message::GlossaireInserer(entree.key.clone()))
                .padding([5, 10]),
            button(theme::icon(
                lucide_icons::iced::icon_pencil::<iced::Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::GlossaireModifier(entree.key.clone()))
            .padding(6),
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<iced::Theme>(),
                12.0,
                palette.danger,
            ))
            .on_press(Message::GlossaireSupprimer(entree.key.clone()))
            .padding(6),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        // Rangée pleine largeur : la liste lit d'un bout à l'autre.
        rang = rang.width(Length::Fill);
        liste = liste.push(rang);
    }

    let gestes = row![
        button(
            palette
                .label("Ajouter une entrée")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::GlossaireCreer)
        .padding([5, 12]),
        button(palette.label("Fermer").size(theme::jetons::DETAIL))
            .on_press(Message::GlossaireFenetreFermer)
            .padding([5, 12]),
    ]
    .spacing(8);

    let carte = column![
        palette.label("Glossaire").size(theme::jetons::TITRE),
        text_input("chercher un terme", &state.plugins.glossaire.filtre)
            .on_input(Message::GlossaireFiltrer)
            .padding(6),
        iced::widget::scrollable(liste).height(Length::Fixed(300.0)),
        gestes,
    ]
    .spacing(12)
    .width(Length::Fixed(LARGEUR));

    // Centré dans la fenêtre — jamais rejeté dans un coin.
    crate::ui::voile(
        container(container(carte).padding(18).style(theme::panneau(palette)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
