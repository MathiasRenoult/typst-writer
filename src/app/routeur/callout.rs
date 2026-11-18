//! Le panneau des callouts : le préréglage, le titre, et la citation à
//! habiller.
//!
//! Le panneau ne demande pas de recopier le texte : si le curseur est dans
//! une citation, il pose la balise devant elle ; sinon il insère une citation
//! vide et laisse le curseur dans son corps. Les deux gestes passent par
//! l'éditeur, jamais par une copie du document.
//!
//! Les couleurs et les mesures du panneau ne sont pas un rendu parallèle :
//! elles deviennent des propriétés de la source (`bordure=…`, `largeur=…`)
//! quand elles s'écartent du préréglage. Le document reste lisible et le
//! fichier de normes n'est pas modifié par une simple insertion.

use crate::app::{Message, State};
use crate::langage::callout;
use iced::Task;

/// Traite les messages du plugin. La porte d'activation est appliquée par
/// `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::CalloutOuvrir => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (_, _, ligne) = super::super::ligne_du_curseur(&contenu, offset);
            let sur_citation = ligne.trim_start().starts_with('>');
            let presets = state.reglages.parametres_plugins.callouts.presets.clone();

            if state.callout.is_none() {
                state.callout = Some(crate::app::state::CalloutDialogue::depuis(
                    &presets,
                    sur_citation,
                ));
            }

            iced::widget::operation::focus(crate::app::state::CalloutDialogue::id_titre())
        }

        Message::CalloutFermer => {
            state.callout = None;
            Task::none()
        }

        Message::CalloutType(type_) => {
            let preset = state
                .reglages
                .parametres_plugins
                .callouts
                .preset(&type_)
                .cloned()
                .or_else(|| {
                    callout::presets_defaut()
                        .into_iter()
                        .find(|preset| preset.cle == type_)
                })
                .unwrap_or_else(|| callout::preset_defaut(&type_));
            if let Some(dialogue) = state.callout.as_mut() {
                dialogue.choisir(preset);
            }
            Task::none()
        }

        Message::CalloutTitre(titre) => {
            if let Some(dialogue) = state.callout.as_mut() {
                dialogue.titre = titre;
            }
            Task::none()
        }

        Message::CalloutChamp(champ, valeur) => {
            if let Some(dialogue) = state.callout.as_mut() {
                dialogue.style.modifier(champ, valeur);
            }
            Task::none()
        }

        Message::CalloutBascule(champ, actif) => {
            if let Some(dialogue) = state.callout.as_mut() {
                match champ {
                    callout::CalloutChamp::Gras => dialogue.style.gras = actif,
                    callout::CalloutChamp::Rupture => dialogue.style.rupture = actif,
                    _ => {}
                }
            }
            Task::none()
        }

        Message::CalloutReinitialiser => {
            if let Some(dialogue) = state.callout.as_mut() {
                dialogue.style = dialogue.base.clone();
            }
            Task::none()
        }

        Message::CalloutInserer => {
            let Some(dialogue) = state.callout.take() else {
                return Task::none();
            };

            let problemes = dialogue.style.problemes();
            if !problemes.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Le callout n'a pas été inséré : {}",
                    problemes.join(" ; ")
                )));
                state.callout = Some(dialogue);
                return Task::none();
            }

            let balise = dialogue.vers_source();
            let offset = state.editor.cursor_offset();
            let contenu = state.editor.content();
            let (debut, _, ligne) = super::super::ligne_du_curseur(&contenu, offset);
            let sur_citation = dialogue.sur_citation && ligne.trim_start().starts_with('>');

            if sur_citation {
                // La balise se glisse devant la citation existante. Le
                // curseur reste dans son texte, décalé de ce qui a été posé.
                let insertion = format!("{balise}\n");
                let nouveau = offset + insertion.len();
                let tache = state.editor.replace_span(debut, debut, &insertion);
                Task::batch([tache, state.editor.set_cursor_offset(nouveau)]).map(Message::Editor)
            } else {
                // Un bloc neuf : la citation vide est le prochain texte que
                // l'auteur écrit, et le curseur est déjà dans son corps.
                let insertion = format!("{balise}\n> ");
                let nouveau = offset + insertion.len();
                let tache = state.editor.insert_at_cursor(&insertion);
                Task::batch([tache, state.editor.set_cursor_offset(nouveau)]).map(Message::Editor)
            }
        }

        _ => Task::none(),
    }
}
