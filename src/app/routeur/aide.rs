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
        Message::OuvrirCorbeille => {
            crate::project::ouvrir_chemin(&crate::services::stockage::corbeille());
            Task::none()
        }
        Message::OuvrirSauvegardes => {
            let dir = crate::project::sauvegardes_dir(&state.project.dir);
            let _ = std::fs::create_dir_all(&dir);
            crate::project::ouvrir_chemin(&dir);
            Task::none()
        }
        _ => Task::none(),
    }
}
