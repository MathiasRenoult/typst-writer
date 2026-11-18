//! Les modes de la fenêtre : édition, rendu, Typst.

use crate::app::{Message, State};
use crate::app::{compile_task, generer_le_pdf_du_rendu};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ModeBasculer => {
            let ancien = state.mode;
            state.mode = match state.mode {
                crate::app::state::ModeVue::Edition => crate::app::state::ModeVue::Rendu,
                crate::app::state::ModeVue::Rendu => crate::app::state::ModeVue::Edition,
                crate::app::state::ModeVue::Typst => crate::app::state::ModeVue::Edition,
            };

            // La bascule vers la vue rendu régénère systématiquement le PDF
            // (bug-report-9) : l'aperçu affiche le fichier livré, il doit
            // donc être neuf — jamais l'export d'une session précédente.
            if state.mode == crate::app::state::ModeVue::Rendu
                && ancien != crate::app::state::ModeVue::Rendu
            {
                let pdf = generer_le_pdf_du_rendu(state);

                // Le texte affiché suit l'édition : au basculement, on
                // compile d'entrée si le rendu est périmé — jamais l'image
                // d'une frappe d'avant (bug-report-10, 6.2).
                let empreinte = state.compilation_fingerprint();
                let frache = state.render.pending || state.render.content == empreinte;

                return if frache {
                    pdf
                } else {
                    state.render.pending = true;
                    Task::batch([pdf, compile_task(state)])
                };
            }

            Task::none()
        }

        Message::ModeTypst => {
            state.mode = if state.mode == crate::app::state::ModeVue::Typst {
                crate::app::state::ModeVue::Edition
            } else {
                crate::app::state::ModeVue::Typst
            };

            // Le source Typst n'est généré qu'au moment où on le regarde.
            if state.mode == crate::app::state::ModeVue::Typst {
                state.actualiser_typst();
            }

            Task::none()
        }

        _ => Task::none(),
    }
}
