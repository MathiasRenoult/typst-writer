//! Les messages de la fiche de référence (bibliographie).

use crate::app::{Message, State};
use iced::Task;

/// Traite la fiche ouverte. La porte d'activation est déjà passée.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        // Le geste de citation ouvre le composeur au curseur : c'est un
        // message de la bibliographie, et son routeur le reçoit comme tel.
        Message::CitationOuvrir => crate::app::composeur::ouvrir_au_curseur(state),

        // L'édition d'une référence : une fenêtre, des champs, et une
        // bibliothèque locale à écrire — jamais une notice Zotero.
        Message::ModifierReference(cle) => {
            let entree = state
                .bibliography
                .iter()
                .find(|entree| entree.key == cle)
                .cloned();

            let Some(entree) = entree else {
                return Task::none();
            };

            // La bibliographie vient-elle de Zotero ? Seule une bibliothèque
            // locale s'écrit : la règle tient dans le champ lui-même.
            let locale = state.zotero.source == crate::zotero::Source::Locale;

            state.plugins.bibliographie.edition = Some(
                crate::plugins::bibliographie::EditionReference::depuis_entree(&entree, locale),
            );

            // La fiche s'ouvre sur son premier champ : le clavier a déjà
            // quelqu'un à qui parler.
            iced::widget::operation::focus(crate::ui::reference::id_premier_champ())
        }
        Message::BibliographieSectionInserer => {
            crate::app::fichiers::update(state, Message::BibliographieSectionInserer)
        }
        Message::ReferencesReplier => {
            state.plugins.bibliographie.repliees = !state.plugins.bibliographie.repliees;
            Task::none()
        }
        Message::ReferencesFiltrer(filtre) => {
            state.plugins.bibliographie.filtre = filtre;
            Task::none()
        }
        Message::SurvolReference(cle) => {
            state.plugins.bibliographie.survol = cle;
            Task::none()
        }
        Message::ReferencesRechercheBascule => {
            state.plugins.bibliographie.recherche_visible =
                !state.plugins.bibliographie.recherche_visible;
            Task::none()
        }
        Message::EditionReferenceChamp(champ, valeur) => {
            if let Some(edition) = state.plugins.bibliographie.edition.as_mut() {
                match champ.as_str() {
                    "cle" => edition.cle = valeur,
                    "kind" => edition.kind = valeur,
                    "nom" => edition.nom = valeur,
                    "prenom" => edition.prenom = valeur,
                    "titre" => edition.titre = valeur,
                    "titre_court" => edition.titre_court = valeur,
                    "annee" => edition.annee = valeur,
                    "editeur" => edition.editeur = valeur,
                    "lieu" => edition.lieu = valeur,
                    _ => {}
                }
            }

            Task::none()
        }
        Message::EditionReferenceNouveau => {
            // Seule une bibliothèque locale s'écrit : la création suppose
            // d'écrire, donc un fichier à soi.
            let locale = state.zotero.source == crate::zotero::Source::Locale;

            if !locale {
                return Task::none();
            }

            state.plugins.bibliographie.edition = Some(
                crate::plugins::bibliographie::EditionReference::nouvelle(locale),
            );

            iced::widget::operation::focus(crate::ui::reference::id_premier_champ())
        }
        Message::EditionReferenceFermer => {
            state.plugins.bibliographie.edition = None;

            Task::none()
        }
        // L'enregistrement écrit la bibliothèque locale — jamais une notice
        // Zotero.
        Message::EditionReferenceEnregistrer => {
            let Some(edition) = state.plugins.bibliographie.edition.clone() else {
                return Task::none();
            };

            if !edition.locale {
                // Une notice Zotero est à Zotero : on ne l'écrit pas ici.
                state.plugins.bibliographie.edition = None;

                return Task::none();
            }

            let Some(chemin) = state.project.bibliography.clone() else {
                state.plugins.bibliographie.edition = None;

                return Task::none();
            };

            let champs = crate::project::ReferenceChamps {
                cle: edition.cle.clone(),
                kind: edition.kind.clone(),
                nom: edition.nom.clone(),
                prenom: edition.prenom.clone(),
                titre: edition.titre.clone(),
                titre_court: edition.titre_court.clone(),
                annee: edition.annee.clone(),
                editeur: edition.editeur.clone(),
                lieu: edition.lieu.clone(),
            };

            let ecrit: Result<(), String> = if edition.creation {
                crate::project::nouvelle_reference(&chemin, &champs)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            } else {
                crate::project::maj_reference(&chemin, &edition.cle, &champs)
                    .map_err(|e| e.to_string())
            };

            match ecrit {
                Err(erreur) => {
                    // La saisie reste : un formulaire qui se ferme en
                    // échec jette le travail de l'utilisateur.
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement de la référence impossible : {erreur}"
                    )));
                }
                Ok(()) => {
                    // La clé retenue se voit aussitôt : elle vient d'être
                    // citable, et c'est celle-là qu'on insérera.
                    state.plugins.bibliographie.edition = None;
                    state.bibliography = crate::project::parse_bibliography(&chemin);
                    state.note = Some(crate::app::state::Note::info("Référence enregistrée."));
                }
            }

            Task::none()
        }

        Message::ReferenceSupprimer(cle) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!("Supprimer la notice « {cle} » ? Elle part à la corbeille."),
                    action: Message::ReferenceSupprimer(cle),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            // Une notice Zotero ne se supprime pas ici : elle est à Zotero.
            if state.zotero.source == crate::zotero::Source::Zotero {
                state.note = Some(crate::app::state::Note::erreur(
                    "Une notice Zotero se supprime dans Zotero.".to_string(),
                ));
                return Task::none();
            }

            let Some(bibliotheque) = state.project.bibliography.clone() else {
                return Task::none();
            };

            match crate::project::supprimer_reference(&bibliotheque, &cle) {
                Ok(copie) => {
                    state.bibliography = crate::project::parse_bibliography(&bibliotheque);
                    state.plugins.bibliographie.edition = None;
                    state.derniere_notice_supprimee = Some(copie);
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La notice « {cle} » part à la corbeille."
                    )));
                    state.refresh_context();
                    Task::none()
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        Message::ReferenceRestaurer(copie) => {
            let Some(bibliotheque) = state.project.bibliography.clone() else {
                return Task::none();
            };

            match crate::project::restaurer_reference(&bibliotheque, &copie) {
                Ok(cle) => {
                    state.bibliography = crate::project::parse_bibliography(&bibliotheque);
                    state.derniere_notice_supprimee = None;
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La notice « {cle} » est de retour."
                    )));
                    state.refresh_context();
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }
            Task::none()
        }

        Message::BibliographieCreer => {
            let dir = state.project.dir.clone();
            let chemin = dir.join("references.bib");

            if chemin.is_file() {
                state.note = Some(crate::app::state::Note::info(
                    "Le projet a déjà sa bibliographie.".to_string(),
                ));
                return Task::none();
            }

            match crate::services::stockage::ecrire_atomique(
                &chemin,
                crate::project::model::BIBLIOGRAPHY_TEMPLATE,
            ) {
                Ok(()) => {
                    state.project = crate::project::load(&dir);
                    state.bibliography = crate::project::parse_bibliography(&chemin);
                    state.note = Some(crate::app::state::Note::info(
                        "La bibliographie locale est créée — references.bib.".to_string(),
                    ));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "La bibliographie n'a pas pu naître : {erreur}"
                    )));
                }
            }

            Task::none()
        }

        // L'alternative sans plugin : Zotero sait exporter un `.bib` tout
        // seul, et ce fichier rejoint la bibliothèque locale sans rien
        // écraser.
        Message::BibliographieImporter => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Bibliographie BibLaTeX", &["bib"])
                    .pick_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::BibImporte,
        ),

        Message::BibImporte(result) => {
            let Some(source) = result else {
                return Task::none();
            };

            match crate::project::importer_bib(&state.project.dir, &source) {
                Ok(rapport) => {
                    let dir = state.project.dir.clone();
                    let chemin = dir.join("references.bib");
                    state.project = crate::project::load(&dir);
                    state.bibliography = crate::project::parse_bibliography(&chemin);

                    // Le rapport se lit entièrement : ce qui est entré, ce
                    // qui était déjà là, ce qui s'est heurté.
                    let mut parties = Vec::new();
                    if !rapport.ajoutees.is_empty() {
                        parties.push(format!(
                            "{} entrée(s) : {}",
                            rapport.ajoutees.len(),
                            rapport.ajoutees.join(", ")
                        ));
                    }
                    if !rapport.deja_la.is_empty() {
                        parties.push(format!(
                            "{} déjà là : {}",
                            rapport.deja_la.len(),
                            rapport.deja_la.join(", ")
                        ));
                    }
                    if !rapport.conflits.is_empty() {
                        parties.push(format!(
                            "{} conflit(s), la première tient : {}",
                            rapport.conflits.len(),
                            rapport.conflits.join(", ")
                        ));
                    }

                    let message = format!("Import BibLaTeX — {}.", parties.join(" ; "));
                    state.note = Some(if rapport.conflits.is_empty() {
                        crate::app::state::Note::info(message)
                    } else {
                        crate::app::state::Note::erreur(message)
                    });
                }
                Err(probleme) => {
                    // Rien n'a changé — la notice le dit déjà.
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }

            Task::none()
        }

        _ => Task::none(),
    }
}
