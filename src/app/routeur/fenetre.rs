//! La fenêtre : barre latérale, taille, ouverture, mode focus.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleSidebar => {
            state.layout.sidebar_visible = !state.layout.sidebar_visible;

            Task::none()
        }

        Message::WindowResized(size) => {
            state.window_size = size;

            Task::none()
        }

        Message::EditeurViewport(size) => {
            state
                .editor
                .editor
                .set_viewport_size(size.width, size.height);
            Task::none()
        }

        Message::WindowOpened(id) => {
            state.window = Some(id);

            // L'échelle de l'écran est redemandée ici : une fenêtre peut
            // s'ouvrir sur un écran dont l'échelle diffère de celle des autres,
            // et c'est elle qui règle la densité du rendu.
            iced::window::scale_factor(id).map(Message::ScaleFactor)
        }

        Message::ToggleFocusMode => {
            state.layout.focus_mode = !state.layout.focus_mode;

            // Le mode focus est un plein écran : masquer les panneaux ne suffit
            // pas, la barre de titre et le reste du bureau resteraient dans le
            // champ de vision.
            let mode = if state.layout.focus_mode {
                iced::window::Mode::Fullscreen
            } else {
                iced::window::Mode::Windowed
            };

            match state.window {
                Some(id) => iced::window::set_mode(id, mode),
                // La fenêtre n'est pas encore connue : on la demande, puis on
                // appliquera le mode. Sans cela, la première bascule au tout
                // début du démarrage resterait sans effet.
                None => iced::window::latest().and_then(move |id| iced::window::set_mode(id, mode)),
            }
        }

        _ => Task::none(),
    }
}
