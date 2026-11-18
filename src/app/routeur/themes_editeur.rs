//! L'éditeur de thème : copie de travail, champs et enregistrement.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ThemeEditeurOuvrir => {
            // Le thème actif d'abord — éditer un autre que celui qu'on
            // porte serait un contresens.
            let actif = state.current_theme().map(|t| t.name.clone());
            let premier = actif
                .and_then(|nom| {
                    state
                        .themes
                        .iter()
                        .find(|t| t.name == nom && t.file.is_some())
                        .and_then(|t| t.file.clone())
                })
                .or_else(|| {
                    state
                        .themes
                        .iter()
                        .find(|theme| theme.file.is_some())
                        .and_then(|theme| theme.file.clone())
                });

            match premier {
                Some(fichier) => {
                    state.editeur_theme = Some(fichier);
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(
                        "Aucun thème à éditer : les thèmes natifs n'ont pas de fichier."
                            .to_string(),
                    ));
                }
            }

            Task::none()
        }

        Message::ThemeEditeurChoisir(nom) => {
            let fichier = state
                .themes
                .iter()
                .find(|theme| theme.file.as_ref().is_some_and(|f| f.meta.name == nom))
                .and_then(|theme| theme.file.clone());

            if let Some(fichier) = fichier {
                state.editeur_theme = Some(fichier);
                state.editeur_theme_textes.clear();
            }

            Task::none()
        }

        Message::ThemeEditeurColorPicker(champ) => {
            // La même roue que pour le texte, visée sur le champ du thème
            // (bug-report-7, point 11). Elle s'ouvre par-dessus l'éditeur.
            let valeur = state.editeur_theme_textes.get(champ).cloned();
            let (teinte, saturation, valeur_v) = valeur
                .as_deref()
                .and_then(crate::ui::picker_couleur::hex_vers_hsv)
                .unwrap_or((0.0, 0.0, 0.5));
            state.picker_couleur = Some(crate::app::state::PickerCouleur {
                teinte,
                saturation,
                valeur: valeur_v,
                hex_saisi: valeur,
                champ: Some(champ),
                reglage: None,
                callout: None,
                balise: false,
            });
            state.editor.editor.lose_focus();
            Task::none()
        }

        Message::ThemeEditeurChamp(champ, valeur) => {
            // La saisie brute reste dans le champ, même incomplète : c'est
            // elle que l'utilisateur voit. La couleur n'est mise à jour
            // que si la valeur se parse.
            state
                .editeur_theme_textes
                .insert(champ.to_string(), valeur.clone());

            if let Some(fichier) = state.editeur_theme.as_mut() {
                let Ok(valeur) = crate::theming::HexColor::parse(valeur.trim()) else {
                    return Task::none();
                };

                match champ {
                    "background" => fichier.colors.background = valeur,
                    "surface" => fichier.colors.surface = valeur,
                    "text" => fichier.colors.text = valeur,
                    "accent" => fichier.colors.accent = valeur,
                    "text_muted" => fichier.colors.text_muted = Some(valeur),
                    "border" => fichier.colors.border = Some(valeur),
                    "danger" => fichier.colors.danger = Some(valeur),
                    "warning" => fichier.colors.warning = Some(valeur),
                    "success" => fichier.colors.success = Some(valeur),
                    _ => {}
                }
            }

            // Le thème s'applique en direct : l'éditeur suit le fond de
            // l'application, même pendant l'édition du thème.
            crate::app::themes::retheme(state);

            Task::none()
        }

        Message::ThemeEditeurReset => {
            // Recharge le fichier depuis le disque : les changements
            // non enregistrés sont abandonnés.
            let nom = state.editeur_theme.as_ref().map(|f| f.meta.name.clone());
            if let Some(nom) = nom
                && let Some(theme) = state
                    .themes
                    .iter()
                    .find(|t| t.name == nom && t.file.is_some())
                    .and_then(|t| t.file.clone())
            {
                state.editeur_theme = Some(theme);
                state.editeur_theme_textes.clear();
            }
            Task::none()
        }

        Message::ThemeEditeurFermer => {
            state.editeur_theme = None;
            state.editeur_theme_textes.clear();
            Task::none()
        }

        Message::ThemeEditeurEnregistrer => {
            let Some(fichier) = state.editeur_theme.clone() else {
                return Task::none();
            };

            match crate::theming::enregistrer(&fichier) {
                Ok(avertissements) => {
                    state.editeur_theme = None;
                    // Le thème enregistré s'applique aussitôt : recharger
                    // la liste et re-résoudre le thème courant.
                    crate::app::themes::refresh_themes(state);
                    crate::app::themes::retheme(state);
                    let message = if avertissements.is_empty() {
                        "Le thème est enregistré et appliqué.".to_string()
                    } else {
                        format!(
                            "Le thème est enregistré et appliqué. Avertissement : {}",
                            avertissements.join(" ; ")
                        )
                    };
                    state.note = Some(crate::app::state::Note::info(message));
                    Task::none()
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                    Task::none()
                }
            }
        }

        _ => Task::none(),
    }
}
