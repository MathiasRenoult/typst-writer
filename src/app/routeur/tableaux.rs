//! La grille de tableau : choisir la taille, insérer la forme.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::TableauOuvrir => {
            state.tableau = Some(crate::app::state::TableauDialogue {
                rangs: "3".to_string(),
                colonnes: "3".to_string(),
            });
            Task::none()
        }
        Message::TableauRangs(valeur) => {
            if let Some(tableau) = state.tableau.as_mut() {
                tableau.rangs = valeur;
            }
            Task::none()
        }
        Message::TableauColonnes(valeur) => {
            if let Some(tableau) = state.tableau.as_mut() {
                tableau.colonnes = valeur;
            }
            Task::none()
        }
        Message::TableauAnnuler => {
            state.tableau = None;
            Task::none()
        }
        Message::TableauInserer => {
            let Some(tableau) = state.tableau.take() else {
                return Task::none();
            };

            let rangs: usize = tableau.rangs.trim().parse().unwrap_or(0).clamp(1, 200);
            let colonnes: usize = tableau.colonnes.trim().parse().unwrap_or(0).clamp(1, 20);

            if colonnes == 0 {
                state.note = Some(crate::app::state::Note::erreur(
                    "Un tableau a au moins une colonne.".to_string(),
                ));
                return Task::none();
            }

            let entete = format!("| {} |", vec![" "; colonnes].join(" | "));
            let trait_ = format!("| {} |", vec!["---"; colonnes].join(" | "));
            let rang = format!("| {} |", vec![" "; colonnes].join(" | "));

            let mut grille = vec![entete, trait_];
            grille.extend(std::iter::repeat_n(rang, rangs.saturating_sub(1)));
            let grille = grille.join("\n");

            state.editor.insert_at_cursor(&grille).map(Message::Editor)
        }
        _ => Task::none(),
    }
}
