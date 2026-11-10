//! L'overlay d'édition d'une note de bas de page.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NoteOverlayChamp(texte) => {
            if let Some(note) = state.note_overlay.as_mut() {
                note.contenu = texte;
            }
            Task::none()
        }

        Message::NoteOverlayEnregistrer => {
            if let Some(note) = state.note_overlay.take() {
                let nouveau = format!("^[{}]", note.contenu);
                let tache = state
                    .editor
                    .replace_span(note.debut, note.fin, &nouveau)
                    .map(Message::Editor);
                return tache;
            }
            Task::none()
        }

        Message::NoteOverlayFermer => {
            state.note_overlay = None;
            Task::none()
        }

        _ => Task::none(),
    }
}
