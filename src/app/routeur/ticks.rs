//! Les ticks : mémoire de l'application, sauvegarde différée.

use crate::app::report_memory;
use crate::app::{Message, State};
use iced::Task;

/// Traite la mémoire et la sauvegarde différée. `SaveTick` n'est pas dans la
/// liste « modifie le projet » : c'est `persist` qui vérifie le verrou avant
/// d'écrire.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::MemoryTick => {
            report_memory(state);

            Task::none()
        }

        Message::SaveTick => {
            // Écriture différée : une seule par tick, et seulement si le
            // document a réellement changé. **La compilation ne décide de
            // rien** : un document en erreur Typst s'écrit quand même, et
            // c'est même là qu'il en a le plus besoin — une coupure ne
            // doit jamais emporter ce qui a été tapé.
            if state.editor.is_modified() {
                let _ecrit = state.persist();
                state.refresh_context();
            }

            Task::none()
        }

        _ => Task::none(),
    }
}
