//! Les messages du glossaire.

use crate::app::champ_renseigne;
use crate::app::{Message, State};
use iced::Task;

/// Traite un message du glossaire. La porte d'activation est déjà passée.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::GlossaireSectionInserer => {
            crate::app::fichiers::update(state, Message::GlossaireSectionInserer)
        }
        Message::GlossaireInserer(cle) => {
            crate::app::fichiers::update(state, Message::GlossaireInserer(cle))
        }
        Message::GlossaireFenetreOuvrir => {
            // Une seule fenêtre au premier plan : ouvrir la liste referme
            // la fiche d'édition qui pourrait être derrière elle.
            state.plugins.glossaire.fenetre = true;
            state.plugins.glossaire.edition = None;
            state.editor.editor.lose_focus();

            Task::none()
        }
        Message::GlossaireFenetreFermer => {
            state.plugins.glossaire.fenetre = false;
            state.plugins.glossaire.filtre.clear();

            Task::none()
        }
        Message::GlossaireCreer => {
            // La fiche de création **remplace** la liste : elle est le
            // premier plan, et son champ reçoit le focus.
            state.plugins.glossaire.fenetre = false;
            state.plugins.glossaire.edition =
                Some(crate::plugins::glossaire::EditionGlossaire::nouvelle());

            iced::widget::operation::focus(crate::ui::glossaire::id_premier_champ())
        }
        Message::GlossaireModifier(cle) => {
            let entree = state
                .plugins
                .glossaire
                .entrees
                .iter()
                .find(|entree| entree.key == cle)
                .cloned();

            let Some(entree) = entree else {
                return Task::none();
            };

            state.plugins.glossaire.fenetre = false;
            state.plugins.glossaire.edition = Some(
                crate::plugins::glossaire::EditionGlossaire::depuis_entree(&entree),
            );

            iced::widget::operation::focus(crate::ui::glossaire::id_premier_champ())
        }
        Message::GlossaireSupprimer(cle) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!(
                        "Supprimer le terme « {cle} » ? Le glossaire sera réécrit sans lui."
                    ),
                    action: Message::GlossaireSupprimer(cle),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            let mut glossaire = crate::plugins::glossaire::Glossaire {
                entrees: state.plugins.glossaire.entrees.clone(),
            };

            if glossaire.supprimer(&cle) {
                match glossaire.sauver(&state.project.dir) {
                    Ok(()) => {
                        state.plugins.glossaire.entrees = glossaire.entrees;
                        state.note = Some(crate::app::state::Note::info("Entrée supprimée."));
                    }
                    Err(erreur) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "Suppression impossible : {erreur}"
                        )));
                    }
                }
            }

            Task::none()
        }
        Message::SurvolGlossaire(cle) => {
            state.plugins.glossaire.survol = cle;
            Task::none()
        }
        Message::GlossaireRechercheBascule => {
            state.plugins.glossaire.recherche_visible = !state.plugins.glossaire.recherche_visible;
            Task::none()
        }
        Message::GlossaireReplier => {
            state.plugins.glossaire.replie = !state.plugins.glossaire.replie;
            Task::none()
        }
        Message::GlossaireFiltrer(filtre) => {
            state.plugins.glossaire.filtre = filtre;
            Task::none()
        }
        Message::EditionGlossaireChamp(champ, valeur) => {
            if let Some(edition) = state.plugins.glossaire.edition.as_mut() {
                match champ.as_str() {
                    "cle" => edition.cle = valeur,
                    "court" => edition.court = valeur,
                    "long" => edition.long = valeur,
                    "description" => edition.description = valeur,
                    "pluriel_court" => edition.pluriel_court = valeur,
                    "pluriel_long" => edition.pluriel_long = valeur,
                    "groupe" => edition.groupe = valeur,
                    _ => {}
                }
            }

            Task::none()
        }
        Message::EditionGlossaireFermer => {
            state.plugins.glossaire.edition = None;

            Task::none()
        }
        Message::EditionGlossaireEnregistrer => {
            let Some(edition) = state.plugins.glossaire.edition.clone() else {
                return Task::none();
            };

            // La clé est un identifiant : une clé fausse se refuse, la
            // fiche reste ouverte — elle ne jette jamais le travail.
            if !crate::plugins::glossaire::EntreeGlossaire::cle_valide(&edition.cle) {
                state.note = Some(crate::app::state::Note::erreur(
                    "La clé d'un terme est un identifiant : lettres, chiffres, - _ . — sans espace ni deux-points.",
                ));
                return Task::none();
            }

            // Une création ne remplace jamais un terme déjà cité dans les
            // textes : la collision se dit, la fiche reste ouverte.
            if edition.creation
                && state
                    .plugins
                    .glossaire
                    .entrees
                    .iter()
                    .any(|entree| entree.key == edition.cle)
            {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Le terme « {} » existe déjà — choisissez une autre clé.",
                    edition.cle
                )));
                return Task::none();
            }

            let entree = crate::plugins::glossaire::EntreeGlossaire {
                key: edition.cle.clone(),
                short: champ_renseigne(&edition.court),
                long: champ_renseigne(&edition.long),
                description: champ_renseigne(&edition.description),
                plural: champ_renseigne(&edition.pluriel_court),
                longplural: champ_renseigne(&edition.pluriel_long),
                group: champ_renseigne(&edition.groupe),
            };

            let mut glossaire = crate::plugins::glossaire::Glossaire {
                entrees: state.plugins.glossaire.entrees.clone(),
            };
            glossaire.definir(entree);

            match glossaire.sauver(&state.project.dir) {
                Ok(()) => {
                    state.plugins.glossaire.entrees = glossaire.entrees;
                    state.plugins.glossaire.edition = None;
                    state.note = Some(crate::app::state::Note::info("Entrée enregistrée."));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement impossible : {erreur}"
                    )));
                }
            }

            Task::none()
        }
        _ => Task::none(),
    }
}
