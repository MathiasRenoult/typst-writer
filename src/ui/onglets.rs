//! La barre des onglets : les projets ouverts, chacun avec son monde.
//!
//! Un onglet est un projet vivant — sa bibliographie, ses normes, sa
//! session — même quand un autre prend l'écran. Choisir un onglet
//! persiste le monde courant et charge l'autre tout entier. Fermer
//! l'onglet courant ouvre le premier qui reste.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, row};
use iced::Element;

/// La barre, quand au moins un projet est ouvert.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    if state.onglets.is_empty() {
        return Space::new().height(0.0).into();
    }

    let mut barre = row![].spacing(6).align_y(iced::Alignment::Center);

    for onglet in &state.onglets {
        let nom = onglet
            .file_name()
            .map(|nom| nom.to_string_lossy().to_string())
            .unwrap_or_else(|| onglet.display().to_string());

        let courant = *onglet == state.project.dir;
        let cible = onglet.clone();
        let fermer = onglet.clone();

        barre = barre.push(
            row![
                button(
                    palette
                        .label(nom.clone())
                        .size(theme::jetons::DETAIL)
                        .style(move |_theme| iced::widget::text::Style {
                            color: Some(if courant {
                                palette.accent
                            } else {
                                palette.text
                            }),
                        }),
                )
                    .on_press(Message::OngletChoisir(cible))
                    .padding([4, 10])
                    ,
                button(theme::icon(
                    lucide_icons::iced::icon_x::<iced::Theme>(),
                    10.0,
                    palette.text_muted,
                ))
                .on_press(Message::OngletFermer(fermer))
                .padding(4),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
        );
    }

    barre.into()
}
