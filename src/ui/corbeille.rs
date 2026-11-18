//! L'écran de la corbeille : voir ce qui a été supprimé, le remettre en place.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, scrollable};
use iced::{Alignment, Element, Length};

pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(corbeille) = state.corbeille.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };
    let palette = state.resolved.palette;

    let mut liste = column![].spacing(2);
    for (index, entree) in corbeille.entrees.iter().enumerate() {
        let choisi = corbeille.choisie == Some(index);
        let genre = if entree.est_dossier {
            "dossier".to_string()
        } else {
            format!("{} o", entree.taille)
        };
        let origine = entree
            .origine
            .as_ref()
            .map(|origine| origine.display().to_string())
            .unwrap_or_else(|| "origine inconnue".to_string());
        let libelle = format!("{} — {} — {} — {}", entree.date, entree.nom, genre, origine);
        let bouton = button(
            palette
                .label(if choisi {
                    format!("▸ {libelle}")
                } else {
                    format!("  {libelle}")
                })
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::CorbeilleChoisir(index))
        .padding([4, 6])
        .width(Length::Fill)
        .style(button::text);
        liste = liste.push(bouton);
    }
    if corbeille.entrees.is_empty() {
        liste = liste.push(
            palette
                .label("La corbeille est vide.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }
    if corbeille.illisibles > 0 {
        liste = liste.push(
            palette
                .label(format!("{} entrée(s) illisible(s).", corbeille.illisibles))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    let choisie = corbeille
        .choisie
        .and_then(|index| corbeille.entrees.get(index));
    let restaurer = button(
        palette
            .label("Restaurer à l'emplacement d'origine")
            .size(theme::jetons::DETAIL),
    )
    .padding([6, 12]);
    let restaurer = if choisie.is_some_and(|entree| entree.origine.is_some()) {
        restaurer.on_press(Message::CorbeilleRestaurer)
    } else {
        restaurer
    };

    let mut corps = column![
        palette.label("Corbeille").size(theme::jetons::TITRE),
        palette
            .label("Une suppression reste réversible ; rien n'est écrasé au retour.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().height(4),
        scrollable(liste).height(Length::Fixed(230.0)),
    ]
    .spacing(8);

    corps = corps.push(
        row![
            restaurer,
            button(palette.label("Dossier").size(theme::jetons::DETAIL),)
                .on_press(Message::CorbeilleDossier)
                .padding([6, 12]),
            button(palette.label("Fermer").size(theme::jetons::DETAIL),)
                .on_press(Message::CorbeilleFermer)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    let fenetre = container(corps)
        .width(Length::Fixed(860.0))
        .padding(16)
        .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill),
    )
}
