//! Les suppressions de document et de thème, avec leur confirmation.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // Ce qui défait demande son mot — la corbeille est derrière, mais
        // un geste qu'on fait parfois à tort mérite la question.
        Message::DeleteChapter(path) => {
            if state.geste_confirme {
                state.geste_confirme = false;
                return crate::app::fichiers::update(state, Message::DeleteChapter(path));
            }

            state.confirmation = Some(crate::app::state::Confirmation {
                question: "Supprimer ce document ? Il part à la corbeille.".to_string(),
                action: Message::DeleteChapter(path),
            });

            Task::none()
        }

        Message::DeleteTheme(name) => {
            if state.geste_confirme {
                state.geste_confirme = false;
                return crate::app::themes::update(state, Message::DeleteTheme(name));
            }

            state.confirmation = Some(crate::app::state::Confirmation {
                question: "Supprimer ce thème ? Il part à la corbeille.".to_string(),
                action: Message::DeleteTheme(name),
            });

            Task::none()
        }

        _ => Task::none(),
    }
}
