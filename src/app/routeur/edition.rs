//! Les gestes d'édition : liste, titres, sommaire.

use crate::app::{Message, State};
use crate::app::{marque_suivante, sauter_titre, scroll_preview_to_page};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ListeEnchainer => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();

            match marque_suivante(&contenu, offset) {
                Some(marque) => state.editor.insert_at_cursor(&marque).map(Message::Editor),
                None => Task::none(),
            }
        }

        Message::TitreSuivant => sauter_titre(state, 1),
        Message::TitrePrecedent => sauter_titre(state, -1),

        Message::GotoHeading(index) => {
            let Some(entry) = state.render.outline.get(index).cloned() else {
                return Task::none();
            };

            // Le sommaire sert à se repérer dans **les deux** vues : le curseur
            // va à la ligne, l'aperçu à la page.
            state.render.followed_page = Some(entry.page);

            let aller = match entry.file {
                Some(path) => {
                    crate::app::composeur::jump_to(state, &path, entry.line.saturating_sub(1), 0)
                }
                None => Task::none(),
            };

            Task::batch([aller, scroll_preview_to_page(state, entry.page)])
        }

        Message::GotoError(line, column) => {
            // Les positions de typst sont 1-based ; l'éditeur attend du
            // 0-based.
            let _ = state.editor.update(iced_code_editor::Message::GotoPosition(
                line.saturating_sub(1),
                column.saturating_sub(1),
            ));

            Task::none()
        }

        _ => Task::none(),
    }
}
