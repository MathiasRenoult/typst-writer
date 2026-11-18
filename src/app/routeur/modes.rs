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
            quitter_la_vue(state, ancien);
            state.mode = match state.mode {
                crate::app::state::ModeVue::Edition => crate::app::state::ModeVue::Rendu,
                crate::app::state::ModeVue::Rendu => crate::app::state::ModeVue::Edition,
                crate::app::state::ModeVue::Typst => crate::app::state::ModeVue::Edition,
            };
            fermer_la_recherche_de_l_autre_mode(state);
            let reprise = entrer_dans_la_vue(state);

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
                    Task::batch([pdf, reprise])
                } else {
                    state.render.pending = true;
                    Task::batch([pdf, compile_task(state), reprise])
                };
            }

            reprise
        }

        Message::ModeTypst => {
            let ancien = state.mode;
            quitter_la_vue(state, ancien);
            state.mode = if state.mode == crate::app::state::ModeVue::Typst {
                crate::app::state::ModeVue::Edition
            } else {
                crate::app::state::ModeVue::Typst
            };
            fermer_la_recherche_de_l_autre_mode(state);
            let reprise = entrer_dans_la_vue(state);

            // Le source Typst n'est généré qu'au moment où on le regarde.
            if state.mode == crate::app::state::ModeVue::Typst {
                state.actualiser_typst();
            }

            reprise
        }

        Message::NumerosDeLigneBascule => {
            state.basculer_les_numeros_de_ligne();
            Task::none()
        }

        _ => Task::none(),
    }
}

/// Retient l'ancre de la vue qu'on quitte.
///
/// Le widget de défilement est détruit avec la vue ; son décalage doit donc
/// être recopié dans l'état qui, lui, survit au changement de mode.
fn quitter_la_vue(state: &mut State, mode: crate::app::state::ModeVue) {
    if mode == crate::app::state::ModeVue::Edition {
        state.editor.memoriser_defilement();
    }
}

/// Ramène la vue qui s'ouvre à l'endroit qu'elle occupait.
///
/// Chaque mode garde son ancre : l'édition revient à son défilement exact, le
/// rendu à sa position exacte. Une première ouverture du rendu part de la page
/// du sommaire qui couvre le haut de l'édition — c'est la seule correspondance
/// que Typst donne entre une ligne et une page.
fn entrer_dans_la_vue(state: &mut State) -> Task<Message> {
    match state.mode {
        crate::app::state::ModeVue::Edition => {
            state.editor.restaurer_defilement().map(Message::Editor)
        }
        crate::app::state::ModeVue::Rendu => {
            if state.render.fenetre.is_some() {
                state.render.reprise_apercu = true;
                crate::app::restaurer_l_apercu(state)
            } else if let Some(page) = crate::app::page_de_lecture(state) {
                // Avant la première composition, la page est retenue et sera
                // appliquée par `RenderDone` quand le sommaire existera.
                state.render.reprise_page = Some(page);
                state.render.reprise_apres_compilation = false;
                crate::app::scroll_preview_to_page(state, page)
            } else {
                // Aucun sommaire encore : la correspondance sera calculée dès
                // la compilation terminée, à partir de la ligne de lecture.
                state.render.reprise_apres_compilation = true;
                Task::none()
            }
        }
        crate::app::state::ModeVue::Typst => Task::none(),
    }
}

/// Une seule recherche à la fois : celle du mode actif.
///
/// Le panneau de recherche du document et la barre de recherche du PDF sont
/// deux outils distincts ; les laisser ouverts ensemble donnait deux barres
/// concurrentes.
fn fermer_la_recherche_de_l_autre_mode(state: &mut State) {
    if state.mode == crate::app::state::ModeVue::Rendu {
        if let Some(etat) = state.recherche.as_mut() {
            etat.jeton
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        state.recherche = None;
        state.editor.editor.effacer_recherche_silencieuse();
    } else if state.render.recherche_apercu.visible {
        state.render.recherche_apercu.fermer();
    }
}
