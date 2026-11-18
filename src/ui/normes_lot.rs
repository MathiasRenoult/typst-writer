//! Changer les normes de plusieurs documents sélectionnés dans la
//! bibliothèque : une norme du catalogue, appliquée d'un seul geste.
//!
//! La fenêtre complète des normes règle un document ; ici, le geste est
//! volontairement court — choisir une norme, décider de l'accroche, appliquer.

use crate::app::{Message, State};
use crate::ui::theme;
use iced::widget::{Space, button, checkbox, column, container, pick_list, row, text};
use iced::{Element, Length};

pub fn view(state: &State) -> Element<'_, Message> {
    let Some(lot) = state.normes_lot.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };
    let palette = state.resolved.palette;
    let nombre = lot.cibles.len();

    let mut documents = column![].spacing(2);
    for dir in &lot.cibles {
        documents = documents.push(
            palette
                .label(
                    dir.file_name()
                        .map(|nom| nom.to_string_lossy().to_string())
                        .unwrap_or_else(|| "document".to_string()),
                )
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    let choix = (!lot.norme.is_empty()).then(|| lot.norme.clone());
    let corps = column![
        text("Changer les normes").size(18),
        palette
            .label(format!(
                "Une norme, appliquée à {nombre} document(s) sélectionné(s)."
            ))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        documents,
        row![
            palette.label("Norme :").size(theme::jetons::DETAIL),
            pick_list(
                crate::project::noms_de_normes(),
                choix,
                Message::NormesLotChoisir
            )
            .width(Length::Fixed(240.0)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        checkbox(lot.accrocher)
            .label("Écrire l'accroche des normes dans les main.typ qui ne l'ont pas encore")
            .on_toggle(Message::NormesLotAccrocher),
        row![
            button(text("Annuler").size(12))
                .on_press(Message::NormesLotFermer)
                .padding([7, 12]),
            Space::new().width(Length::Fill),
            button(text("Appliquer aux documents").size(12))
                .on_press(Message::NormesLotAppliquer)
                .padding([7, 12]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(14);

    let fenetre = container(corps)
        .padding(20)
        .width(Length::Fixed(520.0))
        .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
