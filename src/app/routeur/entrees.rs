//! Les entrées : modificateurs tenus, molette.

use crate::app::update;
use crate::app::{Message, State};
use iced::Task;

/// Traite les messages d'entrée — modificateurs et molette ; aucune écriture
/// dans le projet.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ModificateursTenus { controle, maj } => {
            state.controle_tenu = controle;
            state.maj_tenu = maj;
            Task::none()
        }

        // Ctrl + molette : le zoom de l'aperçu, pas à pas. Sans Ctrl, la
        // molette appartient au défilement et ce routeur ne fait rien.
        Message::Molette(pas) => {
            // Sans `Ctrl`, la molette est au défilement — et elle l'est
            // déjà, prise par ce qui défile.
            if !state.controle_tenu {
                return Task::none();
            }

            update(
                state,
                Message::PreviewZoomStep(if pas > 0.0 { 1 } else { -1 }),
            )
        }

        _ => Task::none(),
    }
}
