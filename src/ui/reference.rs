//! La fenêtre d'édition d'une référence.
//!
//! Elle montre ce qui se lit dans une notice — nom, prénom, titre, année —
//! et permet de le corriger. Une seule règle : **seule une bibliothèque
//! locale s'écrit**. Une notice Zotero appartient à Zotero, qui la reprendrait
//! à la prochaine synchronisation : ici, elle se lit, jamais.

use crate::app::Message;
use crate::app::state::EditionReference;
use crate::ui::theme::Palette;
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 380.0;

/// La couche d'édition : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(edition) = state.edition_reference.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(edition, state.resolved.palette)
}

/// La fenêtre d'édition, quand elle est ouverte.
fn fenetre<'a>(
    edition: &'a EditionReference,
    palette: Palette,
) -> Element<'a, Message> {
    let entete = row![
        palette.label("Modifier la référence").size(15),
        Space::new().width(Length::Fill),
        palette
            .label(format!("@{}", edition.cle))
            .size(11)
            .color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let champ = |invite: &str, valeur: &str, nom: &'static str| {
        text_input(invite, valeur)
            .on_input(move |texte| Message::EditionReferenceChamp(nom.to_string(), texte))
            .padding(6)
            .width(Length::Fill)
    };

    let mut formulaire = column![].spacing(8);

    formulaire = formulaire.push(row![
        champ("Nom", &edition.nom, "nom").width(Length::Fill),
        champ("Prénom", &edition.prenom, "prenom").width(Length::Fill),
    ].spacing(8));

    formulaire = formulaire.push(champ("Titre", &edition.titre, "titre"));
    formulaire = formulaire.push(champ("Année", &edition.annee, "annee"));

    let pied = if edition.locale {
        row![
            palette
                .label("Entrée : enregistrer · Échap : fermer")
                .size(11)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
            button(palette.label("Enregistrer").size(12))
                .on_press(Message::EditionReferenceEnregistrer)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    } else {
        // Zotero : la notice se lit, et c'est tout.
        row![
            palette
                .label("Bibliothèque Zotero — lecture seule")
                .size(11)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
            button(palette.label("Fermer").size(12))
                .on_press(Message::EditionReferenceFermer)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    };

    let fenetre = container(column![entete, formulaire, pied].spacing(12))
        .width(Length::Fixed(LARGEUR))
        .padding(16)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            border: iced::Border {
                color: palette.accent,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
