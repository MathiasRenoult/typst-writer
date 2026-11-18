//! Le panneau de création d'un callout.
//!
//! Le panneau choisit un **préréglage**, son titre, et peut modifier son
//! apparence : couleurs, épaisseur, bordure, rayon, marges, largeur et
//! alignement. Les écarts sont écrits dans la source sous forme de propriétés
//! explicites ; le texte de la citation, lui, reste dans le document — le
//! panneau ne le recopie jamais.
//!
//! Quand le curseur est déjà dans une citation, le bouton l'encadre sur
//! place ; sinon il pose une citation vide et donne le curseur à son corps.

use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text_input,
};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::app::state::{CibleCouleur, State};
use crate::langage::callout;
use crate::ui::theme::{self, Palette};

/// Le libellé d'un préréglage.
fn libelle(preset: &callout::CalloutPreset) -> String {
    preset.nom.clone()
}

/// Le préréglage courant, ou le premier de la norme.
fn courant<'a>(
    presets: &'a [callout::CalloutPreset],
    dialogue: &'a crate::app::state::CalloutDialogue,
) -> &'a callout::CalloutPreset {
    presets
        .iter()
        .find(|preset| preset.cle == dialogue.type_)
        .or_else(|| presets.first())
        .unwrap_or(&dialogue.base)
}

/// La phrase qui explique le geste, selon l'endroit du curseur.
fn explication(dialogue: &crate::app::state::CalloutDialogue) -> &'static str {
    if dialogue.sur_citation {
        "La citation sous le curseur sera encadrée ; son texte ne bouge pas."
    } else {
        "Un bloc de citation vide sera posé, et le curseur ira dans son corps."
    }
}

/// Le panneau — vide quand aucun callout n'est en cours.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(dialogue) = state.callout.as_ref() else {
        return Space::new().into();
    };
    let palette: Palette = state.resolved.palette;
    let largeur = 560.0_f32.min(state.window_size.width - 32.0).max(360.0);
    let hauteur = 660.0_f32.min(state.window_size.height - 32.0).max(380.0);

    let presets = &state.reglages.parametres_plugins.callouts.presets;
    let libelles: Vec<String> = presets.iter().map(libelle).collect();
    let courant = libelle(courant(presets, dialogue));

    let mut carte = column![
        palette
            .label("Encadrer une citation")
            .size(theme::jetons::SAILLANT),
        palette
            .label("Le cadre vient de showybox ; le texte reste une citation du document.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row_libelle(palette, "Préréglage"),
        pick_list(libelles, Some(courant), |choisi: String| {
            let preset = presets
                .iter()
                .find(|preset| preset.nom == choisi)
                .or_else(|| presets.first());
            Message::CalloutType(preset.map(|p| p.cle.clone()).unwrap_or_default())
        })
        .padding(4)
        .width(Length::Fill),
        palette
            .label("Le préréglage vient des paramètres du plugin ; il voyage avec la norme.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row_libelle(palette, "Titre"),
        text_input("Titre (facultatif)", &dialogue.titre)
            .id(crate::app::state::CalloutDialogue::id_titre())
            .on_input(Message::CalloutTitre)
            .on_submit(Message::CalloutInserer)
            .padding([6, 8])
            .width(Length::Fill),
        palette
            .label(explication(dialogue))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label("Apparence — seuls les écarts avec le préréglage sont écrits.")
            .size(theme::jetons::SAILLANT)
            .color(palette.text),
        couleur(
            palette,
            "Bordure",
            callout::CalloutChamp::Bordure,
            &dialogue.style.bordure
        ),
        couleur(
            palette,
            "Fond",
            callout::CalloutChamp::Fond,
            &dialogue.style.fond
        ),
        couleur(
            palette,
            "Fond du titre",
            callout::CalloutChamp::TitreFond,
            &dialogue.style.titre_fond
        ),
        couleur(
            palette,
            "Texte du titre",
            callout::CalloutChamp::TitreTexte,
            &dialogue.style.titre_texte
        ),
        couleur(
            palette,
            "Texte du corps",
            callout::CalloutChamp::CorpsTexte,
            &dialogue.style.corps_texte
        ),
        row![
            champ_mesure(
                palette,
                "Épaisseur",
                callout::CalloutChamp::Epaisseur,
                &dialogue.style.epaisseur
            ),
            champ_mesure(
                palette,
                "Rayon",
                callout::CalloutChamp::Rayon,
                &dialogue.style.rayon
            ),
        ]
        .spacing(8),
        row![
            champ_mesure(
                palette,
                "Marge x",
                callout::CalloutChamp::MargeX,
                &dialogue.style.marge_x
            ),
            champ_mesure(
                palette,
                "Marge y",
                callout::CalloutChamp::MargeY,
                &dialogue.style.marge_y
            ),
            champ_mesure(
                palette,
                "Largeur",
                callout::CalloutChamp::Largeur,
                &dialogue.style.largeur
            ),
        ]
        .spacing(8),
        row![
            liste_bordure(palette, &dialogue.style.bordure_style),
            liste_alignement(palette, &dialogue.style.alignement),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            checkbox(dialogue.style.gras)
                .label("Titre en gras")
                .on_toggle(|actif| Message::CalloutBascule(callout::CalloutChamp::Gras, actif))
                .size(theme::jetons::CORPS),
            checkbox(dialogue.style.rupture)
                .label("Coupure entre pages")
                .on_toggle(|actif| Message::CalloutBascule(callout::CalloutChamp::Rupture, actif))
                .size(theme::jetons::CORPS),
        ]
        .spacing(16),
        button(
            palette
                .label("Revenir au style du préréglage")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::CalloutReinitialiser)
        .padding([4, 8]),
        Space::new().height(Length::Fixed(4.0)),
        iced::widget::row![
            button(
                palette
                    .label(if dialogue.sur_citation {
                        "Encadrer cette citation"
                    } else {
                        "Insérer le callout"
                    })
                    .size(theme::jetons::DETAIL),
            )
            .on_press(Message::CalloutInserer)
            .padding([6, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::CalloutFermer)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8);

    carte = carte.push(
        palette
            .label("Le panneau n'ajoute aucun texte : il pose la balise devant le bloc.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    let corps = scrollable(carte).height(Length::Fixed((hauteur - 72.0).max(280.0)));
    let carte = container(corps)
        .padding(18)
        .style(theme::panneau(palette))
        .width(Length::Fixed(largeur))
        .height(Length::Fixed(hauteur));

    crate::ui::voile(
        container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

/// Un petit libellé de champ, au-dessus de son contrôle.
fn row_libelle<'a>(palette: Palette, texte: &'a str) -> Element<'a, Message> {
    palette
        .label(texte)
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted)
        .into()
}

/// Une couleur : son code hexadécimal, et la roue qui la choisit.
fn couleur<'a>(
    palette: Palette,
    libelle: &'a str,
    champ: callout::CalloutChamp,
    valeur: &'a str,
) -> Element<'a, Message> {
    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fixed(112.0)),
        text_input("#000000", valeur)
            .on_input(move |valeur| Message::CalloutChamp(champ, valeur))
            .padding([4, 6])
            .width(Length::Fixed(110.0)),
        button(palette.label("Roue…").size(theme::jetons::DETAIL))
            .on_press(Message::PickerCouleurPourCallout(CibleCouleur::Insertion(
                champ
            )))
            .padding([3, 7]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// Une mesure, avec son libellé à gauche.
fn champ_mesure<'a>(
    palette: Palette,
    libelle: &'a str,
    champ: callout::CalloutChamp,
    valeur: &'a str,
) -> Element<'a, Message> {
    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fixed(66.0)),
        text_input("", valeur)
            .on_input(move |valeur| Message::CalloutChamp(champ, valeur))
            .padding([4, 6])
            .width(Length::Fixed(76.0)),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .into()
}

/// La liste des traits de bordure.
fn liste_bordure<'a>(palette: Palette, courant: &str) -> Element<'a, Message> {
    let libelles = vec!["Trait plein", "Tirets", "Pointillés"];
    let courant = match courant {
        "dashed" => Some("Tirets"),
        "dotted" => Some("Pointillés"),
        _ => Some("Trait plein"),
    };
    row![
        palette.label("Bordure").size(theme::jetons::DETAIL),
        pick_list(libelles, courant, |choisi: &'static str| {
            Message::CalloutChamp(
                callout::CalloutChamp::BordureStyle,
                match choisi {
                    "Tirets" => "dashed",
                    "Pointillés" => "dotted",
                    _ => "solid",
                }
                .to_string(),
            )
        })
        .padding(4)
        .width(Length::Fixed(130.0)),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// L'alignement du titre.
fn liste_alignement<'a>(palette: Palette, courant: &str) -> Element<'a, Message> {
    let libelles = vec!["Gauche", "Centre", "Droite"];
    let courant = match courant {
        "center" => Some("Centre"),
        "right" => Some("Droite"),
        _ => Some("Gauche"),
    };
    row![
        palette.label("Alignement").size(theme::jetons::DETAIL),
        pick_list(libelles, courant, |choisi: &'static str| {
            Message::CalloutChamp(
                callout::CalloutChamp::Alignement,
                match choisi {
                    "Centre" => "center",
                    "Droite" => "right",
                    _ => "left",
                }
                .to_string(),
            )
        })
        .padding(4)
        .width(Length::Fixed(130.0)),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}
