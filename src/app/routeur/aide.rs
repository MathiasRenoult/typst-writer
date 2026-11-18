//! L'aide et l'accès aux dossiers de récupération.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AideBasculer => {
            state.aide = !state.aide;
            if state.aide {
                state.editor.editor.lose_focus();
            }
            Task::none()
        }
        Message::OuvrirDossierDonnees => {
            crate::project::ouvrir_chemin(&crate::services::stockage::work_dir());
            Task::none()
        }
        Message::OuvrirSauvegardes => {
            let dir = crate::project::sauvegardes_dir(&state.project.dir);
            let _ = std::fs::create_dir_all(&dir);
            crate::project::ouvrir_chemin(&dir);
            Task::none()
        }
        Message::RestaurationOuvrir => {
            let cible = state
                .editor
                .path
                .clone()
                .unwrap_or_else(|| state.project.dir.join("document.tw"));
            match crate::app::restauration::ouvrir(&state.project.dir, &cible) {
                Ok(restauration) => {
                    state.restauration = Some(restauration);
                    state.aide = false;
                    state.editor.editor.lose_focus();
                }
                Err(erreur) => {
                    state.restauration = None;
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Versions antérieures illisibles : {erreur}"
                    )));
                }
            }
            Task::none()
        }
        Message::RestaurationFermer => {
            state.restauration = None;
            state.editor.editor.request_focus();
            Task::none()
        }
        Message::RestaurationChoisir(index) => {
            if let Some(restauration) = state.restauration.as_mut()
                && index < restauration.versions.len()
            {
                restauration.choisie = Some(index);
            }
            Task::none()
        }
        Message::RestaurationRestaurer => {
            if !state.geste_confirme {
                let choix = state.restauration.as_ref().and_then(|restauration| {
                    restauration
                        .choisie
                        .and_then(|index| restauration.versions.get(index))
                });
                let Some(version) = choix else {
                    return Task::none();
                };
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!(
                        "Restaurer la version du {} ? Le texte actuel sera d'abord conservé dans « sauvegardes/ ».",
                        version.date
                    ),
                    action: Message::RestaurationRestaurer,
                });
                return Task::none();
            }
            state.geste_confirme = false;
            state.confirmation = None;

            let Some(restauration) = state.restauration.as_ref() else {
                return Task::none();
            };
            let Some(index) = restauration.choisie else {
                return Task::none();
            };
            let Some(version) = restauration.versions.get(index) else {
                return Task::none();
            };
            let cible = restauration.cible.clone();
            let nom = version.nom.clone();
            let date = version.date.clone();
            let Ok(octets) = std::fs::read(&version.chemin) else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Cette copie n'est plus lisible : elle a peut-être été retirée.".to_string(),
                ));
                return Task::none();
            };
            let Ok(texte) = String::from_utf8(octets) else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Cette copie n'est pas un document texte.".to_string(),
                ));
                return Task::none();
            };
            if let Some(actuel) = state.editor.path.clone() {
                if let Err(erreur) = crate::project::sauvegarder_version(
                    &state.project.dir,
                    &actuel,
                    &state.editor.content(),
                    true,
                ) {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Le texte actuel n'a pas pu être conservé : {erreur} La restauration \
                         est annulée."
                    )));
                    return Task::none();
                }
            } else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Le document courant n'a pas encore de fichier : enregistrez-le avant de \
                     restaurer une version."
                        .to_string(),
                ));
                return Task::none();
            }
            match crate::services::stockage::ecrire_atomique(&cible, &texte) {
                Ok(()) => {
                    state.restauration = None;
                    let tache = state.recharger_document(cible, texte);
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Version du {date} restaurée ({nom}) ; le texte précédent reste dans « sauvegardes/ »."
                    )));
                    return tache;
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Restauration impossible : {erreur}"
                    )));
                }
            }
            Task::none()
        }
        Message::CorbeilleOuvrir => {
            match crate::app::corbeille::ouvrir() {
                Ok(corbeille) => {
                    state.corbeille = Some(corbeille);
                    state.aide = false;
                    state.editor.editor.lose_focus();
                }
                Err(erreur) => {
                    state.corbeille = None;
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                }
            }
            Task::none()
        }
        Message::CorbeilleFermer => {
            state.corbeille = None;
            state.editor.editor.request_focus();
            Task::none()
        }
        Message::CorbeilleChoisir(index) => {
            if let Some(corbeille) = state.corbeille.as_mut()
                && index < corbeille.entrees.len()
            {
                corbeille.choisie = Some(index);
            }
            Task::none()
        }
        Message::CorbeilleRestaurer => {
            let Some(entree) = state
                .corbeille
                .as_ref()
                .and_then(|corbeille| {
                    corbeille
                        .choisie
                        .and_then(|index| corbeille.entrees.get(index))
                })
                .cloned()
            else {
                return Task::none();
            };
            match crate::services::stockage::restaurer_depuis_la_corbeille(&entree.chemin) {
                Ok(origine) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "« {} » restauré vers {}.",
                        entree.nom,
                        origine.display()
                    )));
                    match crate::app::corbeille::ouvrir() {
                        Ok(corbeille) => state.corbeille = Some(corbeille),
                        Err(erreur) => state.note = Some(crate::app::state::Note::erreur(erreur)),
                    }
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                }
            }
            Task::none()
        }
        Message::CorbeilleDossier => {
            crate::project::ouvrir_chemin(&crate::services::stockage::corbeille());
            Task::none()
        }
        _ => Task::none(),
    }
}
