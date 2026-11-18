//! La saisie : la frappe, les listes, les formats, le composeur de
//! notes et la liste de complétion.
//!
//! Ses bras vivaient dans le conducteur (feuille de route, phase 1,
//! cartographie, geste 3) ; ils sont réunis par famille.

use super::Message;
use super::State;
use iced::Task;

/// Déplace la ligne du curseur d'un niveau de liste, sans passer par le
/// tampon de l'éditeur. `Tab` et `Maj+Tab` arrivent ici par deux chemins :
/// les messages historiques (`ListeIndenter` / `ListeDesindenter`) et les
/// messages que l'éditeur émet lui-même quand il a le focus.
fn decaler_la_liste(state: &mut State, indenter: bool) -> Task<Message> {
    let contenu = state.editor.content();
    let offset = state.editor.cursor_offset();
    let (debut, _, ligne) = super::ligne_du_curseur(&contenu, offset);

    // On ne remplace que le début de la ligne, jamais le document entier :
    // sélectionner tout le texte pour y insérer deux espaces faisait défiler
    // l'éditeur — et, par le suivi de l'aperçu, la page — à chaque Tab.
    let (tache, curseur) = if indenter {
        (state.editor.replace_span(debut, debut, "  "), offset + 2)
    } else {
        let retrait = (ligne.len() - ligne.trim_start().len()).min(2);

        if retrait == 0 {
            (Task::none(), offset)
        } else {
            (
                state.editor.replace_span(debut, debut + retrait, ""),
                offset.saturating_sub(retrait),
            )
        }
    };

    Task::batch([tache, state.editor.set_cursor_offset(curseur)]).map(Message::Editor)
}

/// L'éditeur a-t-il capté une tabulation qui appartient à une liste ?
///
/// Le widget de l'éditeur transforme `Tab` en `Message::Tab` et
/// `Maj+Tab` en `Message::FocusNavigationShiftTab` **avant** que les
/// raccourcis de l'application ne voient l'événement. Sans cette porte, la
/// tabulation insérait ses espaces et la liste ne changeait jamais de
/// niveau — le geste existait dans le plan de touches, mais jamais atteint.
fn tabulation_sur_liste(
    state: &mut State,
    message: &iced_code_editor::Message,
) -> Option<Task<Message>> {
    let indenter = match message {
        iced_code_editor::Message::Tab => true,
        iced_code_editor::Message::FocusNavigationShiftTab => false,
        _ => return None,
    };

    if super::pointeur_bloque(state) {
        return None;
    }

    let contenu = state.editor.content();
    let offset = state.editor.cursor_offset();
    if !super::est_sur_une_liste(&contenu, offset) {
        return None;
    }

    Some(decaler_la_liste(state, indenter))
}

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
        Message::ListeDesindenter => decaler_la_liste(state, false),
        Message::ListeIndenter => decaler_la_liste(state, true),
        Message::ComposeurSaisie(_)
        | Message::ComposeurAvancer
        | Message::ComposeurTerminer
        | Message::ComposeurFermer
        | Message::ComposeurDeplacer(_)
        | Message::ComposeurPrendre(_)
        | Message::ComposeurChoisir(_) => crate::app::composeur::composeur_update(state, message),

        Message::CompletionDismiss => {
            state.completion = None;
            Task::none()
        }

        Message::CompletionPick(index) => {
            if let Some(open) = state.completion.as_mut()
                && index < open.shown.len()
            {
                open.selected = index;
            }

            Task::none()
        }
        Message::CompletionChercher(t) => {
            // La recherche : la liste se filtre à la frappe.
            if let Some(open) = state.completion.as_mut() {
                open.typed = t;
                let cherche = open.typed.to_lowercase();
                open.shown = open
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.filter().to_lowercase().contains(&cherche))
                    .map(|(i, _)| i)
                    .collect();
                open.selected = 0;
            }
            Task::none()
        }

        Message::CompletionAccept => {
            // L'insertion de l'élément choisi : `insert` porte la forme à
            // écrire, le libellé n'est que l'affichage de la liste.
            let insertion = {
                let Some(open) = state.completion.as_ref() else {
                    return Task::none();
                };
                let Some(element) = open.selected_item() else {
                    return Task::none();
                };
                crate::editor::completion::insertion(element, &open.typed, None)
            };
            let recul = if insertion.text.ends_with("}}") { 2 } else { 0 };

            let tache = state
                .editor
                .replace_before(insertion.backspaces, &insertion.text)
                .map(Message::Editor);
            let curseur = if recul > 0 {
                let position = state.editor.cursor_offset().saturating_sub(recul);
                state
                    .editor
                    .set_cursor_offset(position)
                    .map(Message::Editor)
            } else {
                Task::none()
            };
            state.completion = None;

            Task::batch([tache, curseur])
        }

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

            // Les statistiques des livres cités suivent le document (le
            // panneau, l add-on biblique).
            state.rafraichir_statistiques_bibliques();
            state.mots_a_recalculer();
            state.editor.apply_format(action).map(Message::Editor)
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
            // Un clic dans le texte ferme le menu contextuel et rend la main
            // à l'éditeur : la bulle ne bloque ni la lecture ni le curseur.
            if state.composeur.is_some()
                && matches!(
                    message,
                    iced_code_editor::Message::MouseClick(..)
                        | iced_code_editor::Message::DoubleClick(_)
                        | iced_code_editor::Message::TripleClick(_)
                )
            {
                return Task::batch([
                    crate::app::composeur::fermer_composeur(state, None),
                    state.editor.update(message).map(Message::Editor),
                ]);
            }

            // `Tab` sur une citation en cours d'écriture ouvre le menu
            // contextuel sans rien consommer ; ailleurs, la tabulation garde
            // son geste ordinaire.
            if matches!(message, iced_code_editor::Message::Tab)
                && let Some(tache) = crate::app::composeur::ouvrir_si_citation_au_curseur(state)
            {
                return tache;
            }

            // L'éditeur capte `Tab` et `Maj+Tab` pour ses propres gestes.
            // Sur une liste, c'est le document qui commande : le niveau
            // change, et la tabulation n'insère pas ses espaces.
            if let Some(tache) = tabulation_sur_liste(state, &message) {
                return tache;
            }

            let content_changed = crate::editor::EditorState::message_may_change_content(&message);
            // Le texte a peut-être changé : le compte de mots est à refaire, au
            // prochain tic, et non à chaque image.
            // Le survol des références bibliques : ce qui est sous la souris
            // devient la cible de la bulle (le système de références).
            if let iced_code_editor::Message::MouseClick(_, offset) = message
                && state
                    .reglages
                    .fonctionnalites
                    .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
                && !super::pointeur_bloque(state)
            {
                // Le clic sur une référence : sa place, ses parties, son texte.
                let texte = state.editor.content();
                let nouvelle = crate::plugins::bible::reference_au(&texte, offset).and_then(
                    |(debut, fin, _)| {
                        let contenu = &texte[debut + 2..fin - 2];
                        crate::plugins::bible::lire_groupe_complet(contenu).map(
                            |(elements, forme)| {
                                let references = elements
                                    .iter()
                                    .filter_map(|element| element.reference().cloned())
                                    .collect::<Vec<_>>();
                                let extrait = crate::plugins::bible::extrait(
                                    &references,
                                    state.plugins.bible.traduction,
                                );
                                crate::plugins::bible::ReferenceCliquee {
                                    intervalle: (debut, fin),
                                    elements,
                                    references,
                                    forme,
                                    extrait,
                                }
                            },
                        )
                    },
                );
                state.plugins.bible.reference_cliquee = nouvelle;
                state.plugins.bible.edition = None;
            }

            if let iced_code_editor::Message::MouseHover(point, offset) = message
                && state
                    .reglages
                    .fonctionnalites
                    .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
                && !super::pointeur_bloque(state)
                && state.composeur.is_none()
            {
                let texte = state.editor.content();
                if let Some(reference) = crate::plugins::bible::reference_sous(&texte, offset) {
                    let change = state
                        .plugins
                        .bible
                        .survol
                        .as_ref()
                        .is_some_and(|(_, actuel)| actuel != &reference);
                    if change {
                        state.plugins.bible.survol = None;
                    }
                    let deja_jeune = state
                        .plugins
                        .bible
                        .survol_candidat
                        .as_ref()
                        .is_some_and(|(_, actuel, _)| actuel == &reference);
                    if !deja_jeune {
                        state.plugins.bible.survol_candidat =
                            Some((point, reference, std::time::Instant::now()));
                    }
                } else {
                    state.plugins.bible.survol = None;
                    state.plugins.bible.survol_candidat = None;
                }
            }

            // Le texte défile sous le pointeur : la référence survolée n'est
            // plus celle qu'on voit, et sa bulle doit disparaître.
            if matches!(message, iced_code_editor::Message::Scrolled(_)) {
                state.plugins.bible.survol = None;
                state.plugins.bible.survol_candidat = None;
            }

            state.mots_a_recalculer();

            // L'aperçu suit ce qu'on lit — mais seulement quand le suivi est
            // demandé : le bouton de la barre d'outils en décide, et écrire
            // ailleurs dans l'assemblage ne doit pas arracher l'aperçu.
            let suivre = matches!(message, iced_code_editor::Message::Scrolled(_))
                && state.layout.follow_preview;
            let citation_tapee = matches!(&message, iced_code_editor::Message::CharacterInput('@'));
            let task = state.editor.update(message).map(Message::Editor);

            // La frappe vient peut-être de poser `..` ou `/` : la palette
            // s'ouvre alors sans voler le texte, et son champ prend le relais.
            let palette = if content_changed {
                crate::app::palette::ouvrir_si_declencheur(state)
            } else {
                Task::none()
            };

            // `@` ouvre le composeur de citation dès la frappe. Le `@` d'une
            // adresse électronique ne passe pas la frontière, et le composeur
            // ne s'ouvre pas dans une bande de code.
            let citation = if citation_tapee {
                crate::app::composeur::ouvrir_si_citation_au_curseur(state)
            } else {
                None
            };

            let suivi = if suivre {
                super::follow_editor_scroll(state)
            } else {
                Task::none()
            };

            Task::batch(vec![
                task,
                palette,
                citation.unwrap_or_else(Task::none),
                suivi,
            ])
        }
        _ => Task::none(),
    }
}
