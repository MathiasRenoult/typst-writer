//! La fenêtre d'édition d'une personne citée.
//!
//! Le nom canonique, la clé de classement, les dates et la qualité vivent dans
//! `personnes.toml`. La clé ne bouge pas en édition : elle est déjà citée par
//! `{{p clé}}` dans les textes. Pour une autre clé, une fiche neuve.

use crate::app::Message;
use crate::plugins::personnes::EditionPersonne;
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Element, Length};

/// La largeur de la fenêtre.
const LARGEUR: f32 = 400.0;

/// La largeur des boutons du pied.
const LARGEUR_BOUTON: f32 = 104.0;

/// L'identifiant du premier champ : la fiche s'ouvre prête à écrire.
pub fn id_premier_champ() -> iced::widget::Id {
    iced::widget::Id::new("personnes-premier-champ")
}

/// La couche d'édition : la fenêtre quand elle est ouverte, rien sinon.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(edition) = state.plugins.personnes.edition.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(edition, state.resolved.palette)
}

fn fenetre<'a>(edition: &'a EditionPersonne, palette: Palette) -> Element<'a, Message> {
    let titre = if edition.creation {
        "Nouvelle personne"
    } else {
        "Modifier la personne"
    };

    let entete = row![
        palette.label(titre).size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label(if edition.creation {
                "bientôt citable".to_string()
            } else {
                format!("{{{{p {}}}}}", edition.cle)
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
        let sur_saisie = move |texte| Message::EditionPersonneChamp(nom.to_string(), texte);

        if premier {
            text_input(invite, valeur)
                .padding(6)
                .width(Length::Fill)
                .id(id_premier_champ())
                .on_input(sur_saisie)
                .on_submit(Message::EditionPersonneEnregistrer)
                .into()
        } else {
            text_input(invite, valeur)
                .padding(6)
                .width(Length::Fill)
                .on_input(sur_saisie)
                .on_submit(Message::EditionPersonneEnregistrer)
                .into()
        }
    };

    let mut formulaire = column![].spacing(theme::jetons::PAS);

    if edition.creation {
        formulaire = formulaire.push(champ("Clé", &edition.cle, "cle", true));
    } else {
        formulaire = formulaire.push(
            palette
                .label(format!("Clé — citée : {{{{p {}}}}}", edition.cle))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    formulaire = formulaire.push(champ("Nom complet", &edition.nom, "nom", !edition.creation));
    formulaire = formulaire.push(
        palette
            .label("Ex. Augustin d'Hippone")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    formulaire = formulaire.push(champ(
        "Nom simple / nom court (facultatif)",
        &edition.nom_simple,
        "nom_simple",
        false,
    ));
    formulaire = formulaire.push(
        palette
            .label("Ex. Augustin — utilisé aux mentions suivantes selon les normes.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    formulaire = formulaire.push(champ(
        "Alias / nom de plume (facultatif)",
        &edition.alias,
        "alias",
        false,
    ));
    formulaire = formulaire.push(
        palette
            .label("Ex. Saint Augustin — forme de reprise possible.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    formulaire = formulaire.push(champ(
        "Clé de classement (facultatif)",
        &edition.tri,
        "tri",
        false,
    ));
    formulaire = formulaire.push(
        palette
            .label("Ex. Aquin, pour classer Thomas d'Aquin à la lettre A.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    formulaire = formulaire.push(champ(
        "Année de naissance (facultatif)",
        &edition.naissance,
        "naissance",
        false,
    ));
    formulaire = formulaire.push(champ(
        "Année de mort (facultatif)",
        &edition.mort,
        "mort",
        false,
    ));
    formulaire = formulaire.push(champ(
        "Dates libres (facultatif)",
        &edition.dates,
        "dates",
        false,
    ));
    formulaire = formulaire.push(
        palette
            .label("Le texte libre sert de repli quand naissance et mort manquent.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    formulaire = formulaire.push(champ(
        "Qualité (facultatif)",
        &edition.qualite,
        "qualite",
        false,
    ));
    formulaire = formulaire.push(champ(
        "Variantes cherchables, séparées par des virgules",
        &edition.variantes,
        "variantes",
        false,
    ));

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
                .on_press(Message::PersonnesSupprimer(edition.cle.clone()))
                .width(Length::Fixed(LARGEUR_BOUTON))
                .padding([6, 12]),
        );
    }

    pied = pied.push(
        button(palette.label("Fermer").size(theme::jetons::CORPS))
            .on_press(Message::EditionPersonneFermer)
            .width(Length::Fixed(LARGEUR_BOUTON))
            .padding([6, 12]),
    );
    pied = pied.push(
        button(palette.label("Enregistrer").size(theme::jetons::CORPS))
            .on_press(Message::EditionPersonneEnregistrer)
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
