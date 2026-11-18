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

        Message::ChapitreChoisir(index) => {
            let chapitre = index.and_then(|index| state.render.chapitres.get(index).cloned());

            if state.render.chapitre == chapitre {
                return Task::none();
            }

            let chapitre_pour_curseur = chapitre.clone();
            state.render.chapitre = chapitre;
            state.render.visible = None;
            state.render.visible_exact = None;
            state.render.scroll = (0.0, 0.0);
            state.render.scroll_direction = 0;
            state.render.followed_page = None;
            state.render.followed_chapter = None;
            state
                .render
                .vue_generation
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            state.render.bound_page_cache();

            // Le chapitre choisi est aussi un lieu du texte : le curseur
            // rejoint sa tête, sans que l'éditeur cesse de montrer le document
            // entier. La vue de rendu, elle, reste instantanée — elle ne
            // recompile pas.
            chapitre_pour_curseur
                .zip(state.editor.path.clone())
                .map(|(chapitre, chemin)| {
                    crate::app::composeur::jump_to(
                        state,
                        &chemin,
                        chapitre.ligne.saturating_sub(1),
                        0,
                    )
                })
                .unwrap_or_else(Task::none)
        }

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
