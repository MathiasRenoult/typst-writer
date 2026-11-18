//! L'écran de restauration d'une version antérieure.
//!
//! Les copies viennent de `sauvegardes/`; chacune dit sa date, sa taille, son
//! empreinte et le début de son texte. Rien ne s'écrit avant le mot de
//! confirmation, et le texte remplacé est lui-même sauvegardé.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, scrollable};
use iced::{Alignment, Element, Length};

pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(restauration) = state.restauration.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };
    let palette = state.resolved.palette;

    let mut liste = column![].spacing(2);
    for (index, version) in restauration.versions.iter().enumerate() {
        let choisi = restauration.choisie == Some(index);
        let libelle = format!(
            "{} — {} — {} o — {}",
            version.date, version.nom, version.taille, version.empreinte
        );
        let bouton = button(
            palette
                .label(if choisi {
                    format!("▸ {libelle}")
                } else {
                    format!("  {libelle}")
                })
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::RestaurationChoisir(index))
        .padding([4, 6])
        .width(Length::Fill)
        .style(button::text);
        liste = liste.push(bouton);
    }
    if restauration.versions.is_empty() {
        liste = liste.push(
            palette
                .label("Aucune copie antérieure pour ce document.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }
    if restauration.illisibles > 0 {
        liste = liste.push(
            palette
                .label(format!(
                    "{} copie(s) n'ont pas pu être lues.",
                    restauration.illisibles
                ))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    let apercu = restauration
        .choisie
        .and_then(|index| restauration.versions.get(index))
        .map(|version| {
            column![
                palette
                    .label(format!("Aperçu de {}", version.nom))
                    .size(theme::jetons::CORPS),
                palette
                    .label(if version.apercu.is_empty() {
                        "(texte vide)".to_string()
                    } else {
                        version.apercu.clone()
                    })
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
            ]
            .spacing(6)
        });

    let mut corps = column![
        palette
            .label("Restaurer une version")
            .size(theme::jetons::TITRE),
        palette
            .label(format!("Document : {}", restauration.cible.display()))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().height(4),
        scrollable(liste).height(Length::Fixed(190.0)),
    ]
    .spacing(8);

    if let Some(apercu) = apercu {
        corps = corps.push(apercu);
    }

    let restaurer = button(
        palette
            .label("Restaurer cette version")
            .size(theme::jetons::DETAIL),
    )
    .padding([6, 12]);
    let restaurer = if restauration.choisie.is_some() {
        restaurer.on_press(Message::RestaurationRestaurer)
    } else {
        restaurer
    };

    corps = corps.push(
        row![
            restaurer,
            button(palette.label("Fermer").size(theme::jetons::DETAIL),)
                .on_press(Message::RestaurationFermer)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    let fenetre = container(corps)
        .width(Length::Fixed(760.0))
        .padding(16)
        .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill),
    )
}
