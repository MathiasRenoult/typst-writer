//! La fenêtre du registre des personnes citées : la liste, le filtre, les
//! gestes d'insertion, de modification et de suppression.
//!
//! Elle est ouverte par la palette ou par la commande du plugin. Une personne
//! n'entre jamais dans un document par une devinette : elle est choisie ici,
//! puis la balise `{{p clé}}` est posée au curseur.

use crate::app::Message;
use crate::app::state::State;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 460.0;

/// La couche : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &State) -> Element<'_, Message> {
    if !state.plugins.personnes.fenetre {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    let palette = state.resolved.palette;
    let filtre = state.plugins.personnes.filtre.trim().to_lowercase();

    let mut liste = column![].spacing(4);

    for entree in &state.plugins.personnes.entrees {
        let cherche = filtre.is_empty()
            || entree.cle.to_lowercase().contains(&filtre)
            || entree.nom.to_lowercase().contains(&filtre)
            || entree
                .tri
                .as_deref()
                .is_some_and(|tri| tri.to_lowercase().contains(&filtre))
            || entree
                .variantes
                .iter()
                .any(|variante| variante.to_lowercase().contains(&filtre));

        if !cherche {
            continue;
        }

        let dates =
            entree.dates_pour(crate::mise_en_page::modele::FormeDatesPersonne::NaissanceMort);
        let details = [dates.as_deref(), entree.qualite.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" — ");

        let rang = row![
            column![
                palette.label(entree.nom_affiche()).size(13),
                palette
                    .label(if details.is_empty() {
                        entree.cle.clone()
                    } else {
                        details
                    })
                    .size(11)
                    .color(palette.text_muted),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            button(palette.label("Insérer").size(theme::jetons::DETAIL))
                .on_press(Message::PersonnesInserer(entree.cle.clone()))
                .padding([5, 10]),
            button(theme::icon(
                lucide_icons::iced::icon_pencil::<iced::Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::PersonnesModifier(entree.cle.clone()))
            .padding(6),
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<iced::Theme>(),
                12.0,
                palette.danger,
            ))
            .on_press(Message::PersonnesSupprimer(entree.cle.clone()))
            .padding(6),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

        liste = liste.push(rang);
    }

    let gestes = row![
        button(
            palette
                .label("Ajouter une personne")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::PersonnesCreer)
        .padding([5, 12]),
        button(palette.label("Insérer l'index").size(theme::jetons::DETAIL))
            .on_press(Message::PersonnesIndexInserer)
            .padding([5, 12]),
        button(palette.label("Fermer").size(theme::jetons::DETAIL))
            .on_press(Message::PersonnesFenetreFermer)
            .padding([5, 12]),
    ]
    .spacing(8);

    let mut carte = column![
        palette.label("Personnes citées").size(theme::jetons::TITRE),
        palette
            .label("Le registre du projet — aucune mention n'est devinée dans le texte.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        text_input("chercher un nom", &state.plugins.personnes.filtre)
            .on_input(Message::PersonnesFiltrer)
            .padding(6),
        iced::widget::scrollable(liste).height(Length::Fixed(300.0)),
        gestes,
    ]
    .spacing(12)
    .width(Length::Fixed(LARGEUR));

    if state.plugins.personnes.entrees.is_empty() {
        carte = carte.push(
            palette
                .label("Aucune personne encore : ajoutez la première, puis citez-la par {{p clé}}.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    crate::ui::voile(
        container(container(carte).padding(18).style(theme::panneau(palette)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
