//! La barre de recherche et de remplacement.
//!
//! Elle flotte, comme les autres fenêtres de l'application, et se ferme
//! d'`Échap`. Le motif est une expression régulière : `$1` dans le
//! remplacement rappelle un groupe.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

/// L'identifiant du champ du motif : la recherche s'ouvre sur lui.
pub fn id_du_motif() -> iced::widget::Id {
    iced::widget::Id::new("recherche-motif")
}

/// La barre, quand la recherche est ouverte.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(etat) = state.recherche.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;

    // `Entrée` dans le motif cherche le suivant ; dans le remplacement,
    // elle remplace l'occurrence courante. Les deux gestes sont ceux qu'on
    // attend en tapant — ils ne se cherchent pas.
    let motif = text_input("motif — expression régulière", &etat.motif)
        .on_submit(Message::RechercheSuivant)
        .id(id_du_motif())
        .on_input(Message::RechercheMotif)
        .padding(6)
        .width(Length::Fixed(240.0));

    let remplacement = text_input("remplacement — $1 rappelle un groupe", &etat.remplacement)
        .on_submit(Message::RechercheRemplacer)
        .on_input(Message::RechercheRemplacement)
        .padding(6)
        .width(Length::Fixed(240.0));

    let compte = if etat.occurrences.is_empty() {
        if etat.motif.is_empty() {
            String::new()
        } else {
            "aucune".to_string()
        }
    } else {
        format!("{}/{}", etat.courante + 1, etat.occurrences.len())
    };

    let mut haut = row![motif, remplacement]
        .spacing(theme::jetons::PAS)
        .align_y(Alignment::Center);

    haut = haut.push(
        palette
            .label(compte)
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    let mut actions = row![].spacing(6);

    for (libelle, message) in [
        ("Précédent", Message::RecherchePrecedent),
        ("Suivant", Message::RechercheSuivant),
        ("Remplacer", Message::RechercheRemplacer),
        ("Tout remplacer", Message::RechercheRemplacerTout),
    ] {
        actions = actions.push(
            button(text(libelle).size(11))
                .on_press(message)
                .padding([4, 8]),
        );
    }

    actions = actions.push(
        button(text("Fermer").size(11))
            .on_press(Message::RechercheFermer)
            .padding([4, 8]),
    );

    let mut corps = column![haut, actions].spacing(8);

    if let Some(erreur) = etat.erreur.as_ref() {
        corps = corps.push(
            palette
                .label(erreur)
                .size(theme::jetons::DETAIL)
                .color(palette.danger),
        );
    }

    let fenetre = container(corps).padding(12).style(theme::panneau(palette));

    crate::ui::voile(container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        )
}
