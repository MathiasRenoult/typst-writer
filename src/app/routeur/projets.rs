//! La liste des projets : vignettes, étagères, glisser-déposer, suppression.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ProjetsVue => {
            state.projets_vue = !state.projets_vue;

            if state.projets_vue {
                let dirs: Vec<std::path::PathBuf> = state
                    .projects
                    .iter()
                    .map(|projet| projet.dir.clone())
                    .collect();

                return Task::perform(
                    async move { crate::project::vignettes(&dirs) },
                    Message::VignettesFaites,
                );
            }

            Task::none()
        }

        Message::VignettesFaites(vignettes) => {
            state.vignettes = vignettes;
            Task::none()
        }

        // Le glisser-déposer : prendre un projet, survoler une étagère,
        // poser. Le geste dit où il en est — une ombre suit le projet.
        Message::DragCommencer(nom) => {
            state.drag_projet = Some(nom);
            Task::none()
        }

        // Les deux recherches ne se montrent que sur leur icône, et
        // l'étagère se choisit au menu (bug-report 2026-11-03).
        Message::ProjetsRechercheBascule => {
            state.projets_recherche_visible = !state.projets_recherche_visible;
            Task::none()
        }

        Message::ProjetsEtageChoisie(choix) => {
            state.projets_etage = choix;
            Task::none()
        }

        // La liste des projets : étagère courante par défaut, tous sur
        // demande — et la recherche filtre par nom (bug-report-7, 18).
        Message::ProjetsFiltreBascule => {
            state.projets_tous = !state.projets_tous;
            Task::none()
        }

        Message::ProjetsRecherche(valeur) => {
            state.projets_recherche = valeur;
            Task::none()
        }

        Message::DragSurvol(etagere) => {
            state.drag_etagere = Some(etagere);
            Task::none()
        }

        Message::DragPoser(etagere) => {
            if let Some(projet) = state.drag_projet.take() {
                state.drag_etagere = None;

                match crate::project::definir_etagere(&projet, &etagere) {
                    Ok(()) => {
                        state.etageres = crate::project::etageres();
                        state.note = Some(crate::app::state::Note::info(format!(
                            "« {projet} » rejoint « {etagere} »."
                        )));
                    }
                    Err(message) => {
                        state.note = Some(crate::app::state::Note::erreur(message));
                    }
                }
            }

            Task::none()
        }

        Message::EtagereBrouillon(valeur) => {
            state.etagere_brouillon = valeur;
            Task::none()
        }

        Message::EtagereDefinir(projet, etagere) => {
            match crate::project::definir_etagere(&projet, &etagere) {
                Ok(()) => {
                    state.etageres = crate::project::etageres();
                    state.note = Some(crate::app::state::Note::info(format!(
                        "« {projet} » rejoint « {etagere} »."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // La suppression d'un projet, et la bascule vers un autre.
        Message::ProjetSupprimer(dir) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: "Supprimer ce projet ? Il part à la corbeille.".to_string(),
                    action: Message::ProjetSupprimer(dir),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            // La carte peut viser un projet autre que la vue courante. Un
            // verrou propre à cette cible reste tenu jusqu'au déplacement.
            let deja_verrouille = state.espace.verrous.contains_key(&dir)
                || (state.project.dir == dir && state.verrou.is_some());
            let mut temporaire = if deja_verrouille {
                None
            } else {
                match crate::project::Verrou::prendre(&dir) {
                    Ok(verrou) => Some(verrou),
                    Err(probleme) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "Suppression refusée : {probleme}"
                        )));
                        return Task::none();
                    }
                }
            };

            if let Err(probleme) = state.sauvegarder_avant_deplacer(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Corrigez la sauvegarde et réessayez."
                )));
                return Task::none();
            }

            if let Err(probleme) = state.preparer_suppression_projet(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Le projet reste en place."
                )));
                return Task::none();
            }

            match crate::project::supprimer(&dir) {
                Ok(corbeille) => {
                    if let Some(mut verrou) = temporaire.take() {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if let Some(mut verrou) = state.espace.verrous.remove(&dir) {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if state.project.dir == dir
                        && let Some(mut verrou) = state.verrou.take()
                    {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Le projet part à la corbeille : {}",
                        corbeille.display()
                    )));

                    state.projects.retain(|projet| projet.dir != dir);
                    state.retirer_projet_supprime(&dir);
                    Task::none()
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        _ => Task::none(),
    }
}
