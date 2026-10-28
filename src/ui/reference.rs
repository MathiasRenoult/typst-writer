//! La fenêtre d'édition d'une référence.
//!
//! Elle montre la notice — type, auteur, titre, titre court, année, éditeur,
//! lieu — et permet de la corriger, ou de l'écrire pour la première fois. Une
//! seule règle : **seule une bibliothèque locale s'écrit**. Une notice Zotero
//! appartient à Zotero, qui la reprendrait à la prochaine synchronisation :
//! ici, elle se lit, jamais.
//!
//! Le formulaire se conduit au clavier : `Tabulation` circule dans les
//! champs, `Entrée` enregistre, `Échap` ferme.

use crate::app::Message;
use crate::app::state::EditionReference;
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, container, pick_list, row, text_input};
use iced::{Alignment, Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 380.0;

/// L'identifiant du premier champ à saisir : la fiche s'ouvre sur lui, et
/// le clavier a déjà un champ à qui parler.
pub fn id_premier_champ() -> iced::widget::Id {
    iced::widget::Id::new("reference-premier-champ")
}

/// Les types de document, du plus courant au plus rare.
const TYPES: &[&str] = &[
    "book",
    "article",
    "incollection",
    "inproceedings",
    "phdthesis",
    "mastersthesis",
    "techreport",
    "online",
    "misc",
];

/// La couche d'édition : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(edition) = state.edition_reference.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(edition, state.resolved.palette)
}

/// La fenêtre d'édition, quand elle est ouverte.
fn fenetre<'a>(edition: &'a EditionReference, palette: Palette) -> Element<'a, Message> {
    let titre = if edition.creation {
        "Nouvelle référence"
    } else {
        "Modifier la référence"
    };

    let entete = row![
        palette.label(titre).size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label(if edition.creation {
                "bientôt citable".to_string()
            } else {
                format!("@{}", edition.cle)
            })
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    // Seule une bibliothèque locale s'écrit : une notice Zotero se lit,
    // et ses champs ne sont pas des champs de saisie.
    let lecture_seule = !edition.locale;

    let champ = move |invite: &str, valeur: &str, nom: &'static str| -> Element<'static, Message> {
        let saisie = text_input(invite, valeur).padding(6).width(Length::Fill);

        // Un champ sans `on_input` ne prend rien : c'est iced qui le dit,
        // et la notice Zotero reste lue telle qu'elle est.
        if lecture_seule {
            saisie.into()
        } else {
            saisie
                .on_input(move |texte| Message::EditionReferenceChamp(nom.to_string(), texte))
                // Un champ **capture** `Entrée` sans `on_submit` : la touche
                // n'arrivait au contrat qu'après un Tab, ce qui forçait à
                // tabuler avant d'enregistrer.
                .on_submit(Message::EditionReferenceEnregistrer)
                .into()
        }
    };

    let mut formulaire = column![].spacing(theme::jetons::PAS);

    // La clé : choisie une fois, puis citée — elle ne bouge plus.
    if edition.creation {
        formulaire = formulaire.push(
            row![
                {
                    // L'identifiant du premier champ : la fiche s'ouvre sur
                    // lui, et lui seul — un identifiant partagé viserait deux
                    // cibles à la fois.
                    let saisie = text_input("Clé", &edition.cle)
                        .padding(6)
                        .width(Length::Fill)
                        .id(id_premier_champ());

                    let champ: Element<'static, Message> = if lecture_seule {
                        saisie.into()
                    } else {
                        saisie
                            .on_input(|texte| {
                                Message::EditionReferenceChamp("cle".to_string(), texte)
                            })
                            .on_submit(Message::EditionReferenceEnregistrer)
                            .into()
                    };

                    champ
                },
                pick_list(
                    TYPES
                        .iter()
                        .map(|type_| type_.to_string())
                        .collect::<Vec<_>>(),
                    Some(edition.kind.clone()),
                    |choisi: String| Message::EditionReferenceChamp("kind".to_string(), choisi),
                )
                .width(Length::Fixed(130.0)),
            ]
            .spacing(theme::jetons::PAS),
        );
    } else {
        formulaire = formulaire.push(pick_list(
            TYPES
                .iter()
                .map(|type_| type_.to_string())
                .collect::<Vec<_>>(),
            Some(edition.kind.clone()),
            |choisi: String| Message::EditionReferenceChamp("kind".to_string(), choisi),
        ));
    }

    formulaire = formulaire.push(
        row![
            champ("Nom", &edition.nom, "nom"),
            champ("Prénom", &edition.prenom, "prenom"),
        ]
        .spacing(theme::jetons::PAS),
    );

    formulaire = formulaire.push(champ("Titre", &edition.titre, "titre"));
    formulaire = formulaire.push(champ("Titre court", &edition.titre_court, "titre_court"));
    formulaire = formulaire.push(
        row![
            champ("Année", &edition.annee, "annee"),
            champ("Éditeur", &edition.editeur, "editeur"),
        ]
        .spacing(theme::jetons::PAS),
    );
    formulaire = formulaire.push(champ("Lieu", &edition.lieu, "lieu"));

    let pied = if edition.locale {
        row![
            palette
                .label("Tab : suivant · Entrée : enregistrer · Échap : fermer")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
            button(palette.label("Supprimer").size(theme::jetons::CORPS))
                .on_press(Message::ReferenceSupprimer(edition.cle.clone()))
                .padding([6, 12]),
            button(palette.label("Enregistrer").size(theme::jetons::CORPS))
                .on_press(Message::EditionReferenceEnregistrer)
                .padding([6, 12]),
        ]
        .spacing(theme::jetons::PAS)
        .align_y(Alignment::Center)
    } else {
        // Zotero : la notice se lit, et c'est tout.
        row![
            palette
                .label("Bibliothèque Zotero — lecture seule")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
            button(palette.label("Fermer").size(theme::jetons::CORPS))
                .on_press(Message::EditionReferenceFermer)
                .padding([6, 12]),
        ]
        .spacing(theme::jetons::PAS)
        .align_y(Alignment::Center)
    };

    let fenetre = container(column![entete, formulaire, pied].spacing(theme::jetons::GRAND_PAS))
        .width(Length::Fixed(LARGEUR))
        .padding(16)
        .style(theme::panneau(palette));

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
