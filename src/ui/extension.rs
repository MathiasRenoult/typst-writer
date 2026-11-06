//! Le composeur d'extension (bug-report-7, point 4).
//!
//! Les champs **naissent de `parametres()`** : une extension de plus au
//! registre, une GUI de plus — sans une ligne de vue écrite pour elle.
//! « J'aimerais qu'une GUI lui permette de paramétrer son truc comme il
//! le veut » : voici la maison.

use iced::widget::{button, column, container, pick_list, row, text_input};
use iced::{Element, Length, Theme};

use crate::app::Message;
use crate::app::state::State;
use crate::ui::theme::{self, Palette};

/// Le composeur — vide quand aucune extension n'est en cours.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(dialogue) = state.extension.as_ref() else {
        return iced::widget::Space::new().into();
    };
    let palette: Palette = state.resolved.palette;
    let exts = crate::extensions::extensions();
    let Some(ext) = exts.get(dialogue.choix) else {
        return iced::widget::Space::new().into();
    };

    let mut carte = column![
        palette
            .label("Insérer une extension")
            .size(theme::jetons::SAILLANT),
    ]
    .spacing(10);

    if exts.len() > 1 {
        let noms: Vec<&'static str> = exts.iter().map(|e| e.nom()).collect();
        carte = carte.push(pick_list(noms, Some(ext.nom()), |choisi: &'static str| {
            let rang = crate::extensions::extensions()
                .iter()
                .position(|e| e.nom() == choisi)
                .unwrap_or_default();
            Message::ExtensionChoisir(rang)
        }));
    }

    for (cle, defaut) in ext.parametres() {
        let valeur = dialogue
            .valeurs
            .get(*cle)
            .map(String::as_str)
            .unwrap_or(defaut);
        let pour_cle = (*cle).to_string();
        carte = carte.push(
            row![
                palette.label(*cle).size(theme::jetons::DETAIL),
                text_input(defaut, valeur)
                    .on_input(move |v| Message::ExtensionParam(pour_cle.clone(), v))
                    .padding([4, 8])
                    .width(Length::Fill),
            ]
            .spacing(8),
        );
    }

    carte = carte.push(
        row![
            button(
                palette
                    .label("Poser dans le document")
                    .size(theme::jetons::DETAIL),
            )
            .on_press(Message::ExtensionInserer)
            .padding([5, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::ExtensionAnnuler)
                .padding([5, 12]),
        ]
        .spacing(8),
    );

    let carte = container(carte).padding(18).style(theme::panneau(palette));

    crate::ui::voile(
        container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

// `Theme` est nommé par les génériques des widgets.
#[allow(dead_code)]
fn _theme_est_nomme(_: Theme) {}
