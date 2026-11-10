//! La fenêtre d'édition d'une entrée de glossaire.
//!
//! Elle montre le terme — clé, formes courte et longue, formes
//! plurielles, définition, groupe — et permet de l'écrire ou de le
//! corriger. Une seule règle : **la clé ne bouge pas en édition** — elle
//! est déjà citée par `{glo:clé}` dans les textes, la changer les
//! casserait. Pour une autre clé, une entrée neuve.
//!
//! Le formulaire se conduit au clavier : `Tabulation` circule dans les
//! champs, `Entrée` enregistre, `Échap` ferme.

use crate::app::Message;
use crate::plugins::glossaire::EditionGlossaire;
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 380.0;

/// L'identifiant du premier champ à saisir : la fiche s'ouvre sur lui, et
/// le clavier a déjà un champ à qui parler.
pub fn id_premier_champ() -> iced::widget::Id {
    iced::widget::Id::new("glossaire-premier-champ")
}

/// Largeur des boutons du pied : le libellé reste entier, même quand la
/// fenêtre est étroite.
const LARGEUR_BOUTON: f32 = 104.0;

/// La couche d'édition : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(edition) = state.plugins.glossaire.edition.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(edition, state.resolved.palette)
}

/// La fenêtre d'édition, quand elle est ouverte.
fn fenetre<'a>(edition: &'a EditionGlossaire, palette: Palette) -> Element<'a, Message> {
    let titre = if edition.creation {
        "Nouveau terme"
    } else {
        "Modifier le terme"
    };

    let entete = row![
        palette.label(titre).size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label(if edition.creation {
                "bientôt citable".to_string()
            } else {
                format!("{{glo:{}}}", edition.cle)
            })
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let champ = move |invite: &str,
                      valeur: &'a str,
                      nom: &'static str,
                      premier: bool|
          -> Element<'a, Message> {
        let sur_saisie = move |texte| Message::EditionGlossaireChamp(nom.to_string(), texte);

        // Un champ **capture** `Entrée` sans `on_submit` : la touche
        // n'arrivait au contrat qu'après un Tab.
        if premier {
            text_input(invite, valeur)
                .padding(6)
                .width(Length::Fill)
                .id(id_premier_champ())
                .on_input(sur_saisie)
                .on_submit(Message::EditionGlossaireEnregistrer)
                .into()
        } else {
            text_input(invite, valeur)
                .padding(6)
                .width(Length::Fill)
                .on_input(sur_saisie)
                .on_submit(Message::EditionGlossaireEnregistrer)
                .into()
        }
    };

    let mut formulaire = column![].spacing(theme::jetons::PAS);

    // La clé : choisie une fois, puis citée — elle ne bouge plus.
    if edition.creation {
        formulaire = formulaire.push(champ("Clé", &edition.cle, "cle", true));
    } else {
        formulaire = formulaire.push(
            palette
                .label(format!("Clé — citée : {{glo:{}}}", edition.cle))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    formulaire = formulaire.push(champ(
        "Forme courte (sigle)",
        &edition.court,
        "court",
        false,
    ));
    formulaire = formulaire.push(champ("Forme longue", &edition.long, "long", false));
    formulaire = formulaire.push(champ(
        "Définition",
        &edition.description,
        "description",
        false,
    ));
    formulaire = formulaire.push(champ(
        "Pluriel de la forme courte",
        &edition.pluriel_court,
        "pluriel_court",
        false,
    ));
    formulaire = formulaire.push(champ(
        "Pluriel de la forme longue",
        &edition.pluriel_long,
        "pluriel_long",
        false,
    ));
    formulaire = formulaire.push(champ("Groupe", &edition.groupe, "groupe", false));

    let mut pied = row![
        palette
            .label("Tab · Entrée · Échap")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().width(Length::Fill),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    if !edition.creation {
        pied = pied.push(
            button(palette.label("Supprimer").size(theme::jetons::CORPS))
                .on_press(Message::GlossaireSupprimer(edition.cle.clone()))
                .width(Length::Fixed(LARGEUR_BOUTON))
                .padding([6, 12]),
        );
    }

    pied = pied.push(
        button(palette.label("Fermer").size(theme::jetons::CORPS))
            .on_press(Message::EditionGlossaireFermer)
            .width(Length::Fixed(LARGEUR_BOUTON))
            .padding([6, 12]),
    );
    pied = pied.push(
        button(palette.label("Enregistrer").size(theme::jetons::CORPS))
            .on_press(Message::EditionGlossaireEnregistrer)
            .width(Length::Fixed(LARGEUR_BOUTON))
            .padding([6, 12]),
    );

    let fenetre = container(column![entete, formulaire, pied].spacing(theme::jetons::GRAND_PAS))
        .width(Length::Fixed(LARGEUR))
        .padding(16)
        .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
