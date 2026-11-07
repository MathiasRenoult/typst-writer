//! La saisie : la frappe, les listes, les formats, le composeur de
//! notes et la liste de complétion.
//!
//! Ses bras vivaient dans le conducteur (feuille de route, phase 1,
//! cartographie, geste 3) ; ils sont réunis par famille.

use super::Message;
use super::State;
use iced::Task;

pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::BlocDescendre => state.editor.deplacer_le_bloc(false).map(Message::Editor),

        Message::BlocMonter => state.editor.deplacer_le_bloc(true).map(Message::Editor),
        Message::ListeMonter | Message::ListeDescendre => {
            let haut = matches!(message, Message::ListeMonter);
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = super::deplacer_la_ligne(&contenu, offset, haut);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        Message::ListeDesindenter => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = super::desindenter_la_ligne(&contenu, offset);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        Message::ListeIndenter => {
            let contenu = state.editor.content();
            let offset = state.editor.cursor_offset();
            let (neuf, curseur) = super::indenter_la_ligne(&contenu, offset);

            Task::batch([
                state.editor.replace_span(0, contenu.len(), &neuf),
                state.editor.set_cursor_offset(curseur),
            ])
            .map(Message::Editor)
        }
        Message::ComposeurSaisie(_)
        | Message::ComposeurAvancer
        | Message::ComposeurTerminer
        | Message::ComposeurFermer
        | Message::ComposeurDeplacer(_)
        | Message::ComposeurPrendre(_)
        | Message::ComposeurChoisir(_) => crate::app::composeur::composeur_update(state, message),

        Message::CompletionDismiss => Task::none(),

        Message::CompletionPick(index) => {
            if let Some(open) = state.completion.as_mut()
                && index < open.shown.len()
            {
                open.selected = index;
            }

            Task::none()
        }
        Message::CompletionAccept => Task::none(),

        Message::CompletionMove(delta) => {
            if let Some(open) = state.completion.as_mut() {
                open.move_selection(delta);
            }

            Task::none()
        }
        Message::Format(action) => {
            if !state.editor.is_intermediate() {
                return Task::none();
            }

            // TOPBAR-04-07 : toute couleur choisie devient la couleur courante.
            if let crate::app::message::FormatAction::Couleur(ref nom) = action {
                state.derniere_couleur = Some(nom.clone());
            }

            // Le bouton « citation » est un `@` tapé par la main de l'outil.
            let _declencheur_tape = matches!(action, crate::app::message::FormatAction::Citation);

            state.render.words_pending = true;
            let edit = state.editor.apply_format(action).map(Message::Editor);
            let completion = super::declencher_si_besoin(state);

            Task::batch([edit, completion])
        }
        Message::TitreInserer(niveau) => {
            // L'insertion simple au curseur : `#` × n + espace. Pas de
            // reformatage de bloc — on tape, comme à la main.
            let marque = "#".repeat(niveau.clamp(1, 6) as usize);
            state
                .editor
                .insert_at_cursor(&format!("{marque} "))
                .map(Message::Editor)
        }
        Message::Editor(message) => {
            let _content_changed = crate::editor::EditorState::message_may_change_content(&message);

            // Un `@` qui vient d'être tapé est le déclencheur du composeur ;
            // les autres frappes peuvent être l'édition d'une citation.
            let _declencheur_tape =
                matches!(message, iced_code_editor::Message::CharacterInput('@'));

            // Un clic dans le texte a le dernier mot sur le composeur.
            let clic_dans_le_texte = matches!(
                message,
                iced_code_editor::Message::MouseClick(_)
                    | iced_code_editor::Message::DoubleClick(_)
                    | iced_code_editor::Message::TripleClick(_)
            );

            // Le texte a peut-être changé : le compte de mots est à refaire, au
            // prochain tic, et non à chaque image.
            state.render.words_pending = true;

            // L'aperçu suit ce qu'on lit — mais seulement quand le suivi est
            // demandé : le bouton de la barre d'outils en décide, et écrire
            // ailleurs dans l'assemblage ne doit pas arracher l'aperçu.
            let suivre = matches!(message, iced_code_editor::Message::Scrolled(_))
                && state.layout.follow_preview;

            let task = state.editor.update(message).map(Message::Editor);

            if clic_dans_le_texte {
                if state.composeur.is_some() {
                    // La main est rendue au document, et la note en cours ne
                    // s'écrit pas — le même geste qu'Échap.
                    return Task::batch([
                        task,
                        crate::app::composeur::fermer_composeur(state, None),
                    ]);
                }

                // **Le clic sur un glyphe est la seule porte d'édition d'une
                // note.** Une citation ouvre son composeur ; un pied de page se
                // déplie sous le curseur. Ailleurs, rien ne bouge — et surtout
                // plus jamais l'édition d'une citation au fil de la frappe.
                let contenu = state.editor.content();
                let notes = crate::langage::marques::notes(&contenu);

                if let Some(rang) = state.editor.note_au_clic(&notes) {
                    let note = notes[rang].clone();

                    return match note.nature {
                        crate::langage::marques::Nature::Appel => {
                            let ouverture =
                                crate::langage::parse::appel_autour(&contenu, note.debut + 1)
                                    .map(|(references, debut, fin)| {
                                        crate::app::composeur::ouvrir_edition(
                                            state, references, debut, fin,
                                        )
                                    })
                                    .unwrap_or_else(Task::none);

                            Task::batch([task, ouverture])
                        }
                        crate::langage::marques::Nature::PiedDePage => {
                            // L'overlay de note : le texte s'édite en bas
                            // de l'écran, sans déplier la note sous le
                            // curseur ni déranger le fil du document.
                            let texte = &contenu[note.debut..note.fin];
                            let nu = texte
                                .strip_prefix("^[")
                                .and_then(|s| s.strip_suffix(']'))
                                .unwrap_or(texte);
                            state.note_overlay = Some(crate::app::state::NoteOverlay {
                                contenu: nu.to_string(),
                                debut: note.debut,
                                fin: note.fin,
                            });
                            Task::none()
                        }
                    };
                }
            }

            let suivi = if suivre {
                super::follow_editor_scroll(state)
            } else {
                Task::none()
            };

            // La frappe filtre la liste ; la navigation au curseur la ferme.
            // Le défilement, lui, la garde ouverte et met à jour son ancre.
            let completion = super::declencher_si_besoin(state);

            crate::app::epreuves::actif("lsp");

            Task::batch(vec![task, completion, suivi])
        }
        _ => Task::none(),
    }
}
