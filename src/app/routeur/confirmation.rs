//! La confirmation d'un geste : confirmer ou annuler.

use crate::app::update;
use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ConfirmationConfirmer => match state.confirmation.take() {
            Some(confirmation) => {
                state.geste_confirme = true;
                let tache = update(state, confirmation.action);
                // Le drapeau est à usage unique : après le rejeu, il retombe,
                // même si l'action n'était pas destructrice. Sans cela, une
                // sauvegarde forcée confirmée armait la suppression suivante.
                state.geste_confirme = false;
                tache
            }
            None => Task::none(),
        },

        Message::ConfirmationAnnuler => {
            state.geste_confirme = false;
            // Refuser d'écraser un changement extérieur est une réponse pour ce
            // changement-ci : un autre en sera une question neuve.
            if let Some(crate::app::state::Confirmation {
                action: Message::SauvegardeForcee(empreinte),
                ..
            }) = state.confirmation.as_ref()
            {
                state.conflit_refuse = Some(*empreinte);
            }
            // Un import refusé ne laisse rien derrière lui.
            state.importation = None;
            state.confirmation = None;
            Task::none()
        }

        _ => Task::none(),
    }
}
