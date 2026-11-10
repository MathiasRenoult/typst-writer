//! Les normes : appliquer, écraser, enregistrer, supprimer.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NormesNom(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.norme = nom;
            }

            Task::none()
        }

        Message::NormesAppliquer(nom) => {
            match crate::project::norme(&nom) {
                Some(mise) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = mise;
                        dialogue.norme = nom.clone();
                        dialogue.norme_originale = nom.clone();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est chargée dans la fenêtre — « Appliquer » pour l'écrire."
                    )));
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "La norme « {nom} » n'existe pas."
                    )));
                }
            }

            Task::none()
        }

        // Écraser la norme chargée : le premier temps de l'enregistrement,
        // quand le document en porte une (bug-report-9).
        Message::NormesEcraser => {
            let Some((nom, reglages)) = state
                .mise_en_page
                .as_ref()
                .filter(|dialogue| !dialogue.norme_originale.trim().is_empty())
                .map(|dialogue| {
                    (
                        dialogue.norme_originale.trim().to_string(),
                        dialogue.reglages.clone(),
                    )
                })
            else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Aucune norme chargée à écraser.".to_string(),
                ));
                return Task::none();
            };

            match crate::project::enregistrer_norme(&nom, &reglages) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est remplacée."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesEnregistrer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Une copie a un nom.".to_string(),
                ));
                return Task::none();
            }

            if crate::project::norme(nom).is_some() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "« {nom} » existe déjà — donnez un autre nom à la copie, ou écrasez la norme chargée."
                )));
                return Task::none();
            }

            match crate::project::enregistrer_norme(
                nom,
                &state
                    .mise_en_page
                    .as_ref()
                    .map(|d| d.reglages.clone())
                    .unwrap_or_default(),
            ) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.norme = nom.to_string();
                        dialogue.norme_originale = nom.to_string();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La copie « {nom} » est dans la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesSupprimer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                return Task::none();
            }

            match crate::project::supprimer_norme(nom) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut()
                        && dialogue.norme_originale.trim() == nom
                    {
                        // La norme n'existe plus : le document redevient un
                        // porteur de normes personnalisées.
                        dialogue.norme_originale.clear();
                        dialogue.norme.clear();
                        dialogue.modifie = true;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » a quitté la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        _ => Task::none(),
    }
}
