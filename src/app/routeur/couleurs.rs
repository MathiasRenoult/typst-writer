//! Le sélecteur de couleur : roue, code hexadécimal et application.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::PickerCouleurOuvrir => {
            let (teinte, saturation, valeur) = state
                .derniere_couleur
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: None,
                champ: None,
                reglage: None,
                callout: None,
                balise: false,
            });
            state.editor.editor.lose_focus();
            Task::none()
        }
        Message::PickerCouleurFermer => {
            state.picker_couleur = None;
            Task::none()
        }
        Message::PickerCouleurTeinte(teinte) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.teinte = teinte;
            }
            Task::none()
        }
        Message::PickerCouleurSatVal(saturation, valeur) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.saturation = saturation;
                picker.valeur = valeur;
            }
            Task::none()
        }
        Message::PickerCouleurPourReglage(champ) => {
            let courante = state
                .mise_en_page
                .as_ref()
                .map(|dialogue| dialogue.reglages.valeur(champ))
                .unwrap_or_default();
            let hex = crate::ui::reglages::couleur_depuis(&courante).map(|c| {
                format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0).round() as u8,
                    (c.g * 255.0).round() as u8,
                    (c.b * 255.0).round() as u8
                )
            });
            let (teinte, saturation, valeur) = hex
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: None,
                champ: None,
                reglage: Some(champ),
                callout: None,
                balise: false,
            });
            state.editor.editor.lose_focus();
            Task::none()
        }
        Message::PickerCouleurPourBalise => {
            let courante = state
                .balise_dialogue
                .as_ref()
                .and_then(|dialogue| dialogue.champs.iter().find(|champ| champ.cle == "couleur"))
                .map(|champ| champ.valeur.clone())
                .unwrap_or_else(|| "#e74c3c".to_string());
            let (teinte, saturation, valeur) =
                crate::ui::picker_couleur::hex_vers_hsv(&courante).unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: Some(courante),
                champ: None,
                reglage: None,
                callout: None,
                balise: true,
            });
            state.editor.editor.lose_focus();
            Task::none()
        }
        Message::PickerCouleurPourCallout(cible) => {
            let courante = match cible {
                crate::app::state::CibleCouleur::Insertion(champ) => state
                    .callout
                    .as_ref()
                    .map(|dialogue| dialogue.style.valeur(champ))
                    .unwrap_or(""),
                crate::app::state::CibleCouleur::Preset(index, champ) => state
                    .mise_en_page
                    .as_ref()
                    .and_then(|dialogue| {
                        dialogue
                            .reglages
                            .parametres_plugins
                            .callouts
                            .presets
                            .get(index)
                    })
                    .map(|preset| preset.valeur(champ))
                    .unwrap_or(""),
            };
            let hex = crate::ui::reglages::couleur_depuis(courante).map(|c| {
                format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0).round() as u8,
                    (c.g * 255.0).round() as u8,
                    (c.b * 255.0).round() as u8
                )
            });
            let (teinte, saturation, valeur) = hex
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 1.0, 1.0));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur,
                hex_saisi: hex,
                champ: None,
                reglage: None,
                callout: Some(cible),
                balise: false,
            });
            state.editor.editor.lose_focus();
            Task::none()
        }
        Message::PickerCouleurEnregistrer => {
            let Some(picker) = state.picker_couleur.as_ref() else {
                return Task::none();
            };
            let hex = crate::ui::picker_couleur::hsv_vers_hex(
                picker.teinte,
                picker.saturation,
                picker.valeur,
            );
            state.derniere_couleur = Some(hex.clone());
            state.note = Some(crate::app::state::Note::info(format!(
                "« {hex} » est votre dernière couleur — la pastille de la barre en insère la balise."
            )));
            Task::none()
        }
        Message::PickerCouleurAppliquer => {
            let Some(picker) = state.picker_couleur.take() else {
                return Task::none();
            };
            let hex = crate::ui::picker_couleur::hsv_vers_hex(
                picker.teinte,
                picker.saturation,
                picker.valeur,
            );
            state.derniere_couleur = Some(hex.clone());
            if picker.balise {
                if let Some(dialogue) = state.balise_dialogue.as_mut()
                    && let Some(champ) = dialogue
                        .champs
                        .iter_mut()
                        .find(|champ| champ.cle == "couleur")
                {
                    champ.valeur = hex;
                }
                return Task::none();
            }
            if let Some(cible) = picker.callout {
                return match cible {
                    crate::app::state::CibleCouleur::Insertion(champ) => {
                        crate::app::update(state, Message::CalloutChamp(champ, hex))
                    }
                    crate::app::state::CibleCouleur::Preset(index, champ) => {
                        crate::app::update(state, Message::CalloutPresetChamp(index, champ, hex))
                    }
                };
            }
            if let Some(reglage) = picker.reglage {
                return crate::app::update(state, Message::MiseEnPageChamp(reglage, hex));
            }
            match picker.champ {
                Some(champ) => crate::app::update(state, Message::ThemeEditeurChamp(champ, hex)),
                None => crate::app::update(
                    state,
                    Message::Format(crate::app::message::FormatAction::Couleur(hex)),
                ),
            }
        }
        Message::PickerCouleurHex(texte) => {
            if let Some(picker) = state.picker_couleur.as_mut() {
                picker.hex_saisi = Some(texte.clone());
                if let Some((teinte, saturation, valeur)) =
                    crate::ui::picker_couleur::hex_vers_hsv(&texte)
                {
                    picker.teinte = teinte;
                    picker.saturation = saturation;
                    picker.valeur = valeur;
                }
            }
            Task::none()
        }
        Message::PickerCouleurRapide(valeur) => {
            // Une pastille rapide **montre** la couleur ; « Appliquer » seul
            // l'emploie. Le clic ne touche donc ni le texte ni les normes.
            if let Some(picker) = state.picker_couleur.as_mut() {
                if let Some((teinte, saturation, valeur_hsv)) =
                    crate::ui::picker_couleur::hex_vers_hsv(&valeur)
                {
                    picker.teinte = teinte;
                    picker.saturation = saturation;
                    picker.valeur = valeur_hsv;
                }
                picker.hex_saisi = Some(valeur);
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
