//! Les onglets de projets : choisir, fermer.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::OngletChoisir(dir) => {
            if state.project.dir == dir {
                return Task::none();
            }

            state.persist();
            crate::app::update(state, Message::OpenProject(dir))
        }
        Message::OngletFermer(dir) => {
            state.onglets.retain(|onglet| *onglet != dir);

            if state.project.dir == dir {
                match state.onglets.first().cloned() {
                    Some(suivant) => {
                        state.persist();
                        return crate::app::update(state, Message::OpenProject(suivant));
                    }
                    None => {
                        state.note = Some(crate::app::state::Note::info(
                            "Dernier onglet fermé.".to_string(),
                        ));
                    }
                }
            }

            Task::none()
        }
        _ => Task::none(),
    }
}
