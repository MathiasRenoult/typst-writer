//! La page des paramètres propres à l'application.
//!
//! Elle ne touche jamais aux normes du document : celles-ci vivent dans
//! `mise_en_page` et se règlent depuis la fenêtre des normes.

use crate::app::state::State;
use crate::app::{Message, themes};
use iced::Task;

/// Traite les messages de la page Paramètres.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ParametresOuvrir => {
            state.parametres_vue = true;
            // Les deux pages occupent la même place : l'une chasse l'autre.
            state.projets_vue = false;
            Task::none()
        }

        Message::ParametresFermer => {
            state.parametres_vue = false;
            state.editor.editor.request_focus();
            Task::none()
        }

        Message::ParametresPanneau(panneau) => {
            state.parametres_panneau = panneau;
            Task::none()
        }

        Message::ParametresBasculeOutilsMiseEnPage => {
            state.outils_mise_en_page = !state.outils_mise_en_page;
            state.save_session();
            Task::none()
        }

        Message::ParametresBasculeOutilsInsertion => {
            state.outils_insertion = !state.outils_insertion;
            state.save_session();
            Task::none()
        }

        Message::ParametresModeTheme(mode) => {
            state.theme = mode;
            themes::retheme(state);
            state.save_session();
            Task::none()
        }

        _ => Task::none(),
    }
}
