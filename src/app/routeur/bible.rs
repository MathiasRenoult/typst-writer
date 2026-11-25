//! Les messages du plugin biblique.
//!
//! Le panneau de construction, les statistiques et la consultation vivent
//! ici : le routeur central n'a plus à connaître leurs détails.

use crate::app::{Message, State};
use crate::plugins::bible::{
    BiblePanneau, ElementCitation, FormeLivre, Reference, ReferenceCliquee, ReferenceEdition,
    Traduction, afficher_groupe, afficher_groupe_forme, depuis_texte, extrait, filtrer_livres,
    livres, position_reference, references_depuis_texte,
};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ReferenceBibliqueInserer => ouvrir_le_panneau(state),

        Message::BiblePanneauRecherche(recherche) => {
            let tous = livres(state.plugins.bible.traduction);
            let filtres = filtrer_livres(&tous, &recherche);
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.recherche = recherche;
                panneau.saisie_modifiee = true;
                let choix_valide = panneau
                    .livre
                    .and_then(|index| tous.get(index))
                    .is_some_and(|livre| filtres.iter().any(|filtre| filtre.osis == livre.osis));
                if !choix_valide {
                    panneau.livre = None;
                }
            }
            Task::none()
        }
        Message::BiblePanneauLivre(index) => {
            let tous = livres(state.plugins.bible.traduction);
            let filtre = state
                .plugins
                .bible
                .panneau
                .as_ref()
                .map(|panneau| panneau.recherche.clone())
                .unwrap_or_default();
            let filtres = filtrer_livres(&tous, &filtre);
            if let Some(panneau) = state.plugins.bible.panneau.as_mut()
                && let Some(livre) = tous.get(index)
                && filtres.iter().any(|filtre| filtre.osis == livre.osis)
            {
                panneau.livre = Some(index);
                panneau.saisie_modifiee = true;
            }
            Task::none()
        }
        Message::BiblePanneauChapitre(chapitre) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.chapitre = chapitre;
                panneau.saisie_modifiee = true;
            }
            Task::none()
        }
        Message::BiblePanneauVersets(versets) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.versets = versets;
                panneau.saisie_modifiee = true;
            }
            Task::none()
        }
        Message::BiblePanneauTexte(texte) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.texte_libre = texte;
            }
            Task::none()
        }
        Message::BiblePanneauAjouter => ajouter_au_panneau(state),
        Message::BiblePanneauRetirer(index) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut()
                && index < panneau.references.len()
            {
                panneau.references.remove(index);
            }
            Task::none()
        }
        Message::BiblePanneauForme(forme) => {
            let tous = livres(state.plugins.bible.traduction);
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.forme = forme;
                panneau.renouveler_choix(&tous, forme);
            }
            Task::none()
        }
        Message::BiblePanneauInserer => inserer_la_citation(state),
        Message::BiblePanneauFermer => {
            state.plugins.bible.panneau = None;
            state.editor.editor.request_focus();
            Task::none()
        }

        Message::BibleRechercheBascule => {
            let etat = &mut state.plugins.bible;
            etat.recherche_active = !etat.recherche_active;
            if !etat.recherche_active {
                etat.recherche.clear();
                state.editor.editor.request_focus();
                Task::none()
            } else {
                state.editor.editor.lose_focus();
                iced::widget::operation::focus(crate::ui::sidebar::id_recherche_biblique())
            }
        }
        Message::BibleFiltrer(recherche) => {
            state.plugins.bible.recherche = recherche;
            Task::none()
        }
        Message::BibleTri(tri) => {
            state.plugins.bible.tri = tri;
            Task::none()
        }
        Message::BibleAllerReference(reference) => aller_a_la_reference(state, &reference),
        Message::BibleTraduction(traduction) => {
            appliquer_traduction(state, traduction);
            Task::none()
        }
        Message::BibleVersionImporter => Task::perform(
            async {
                let reponse = rfd::AsyncMessageDialog::new()
                    .set_title("Ajouter une version biblique")
                    .set_level(rfd::MessageLevel::Warning)
                    .set_description(
                        "N'installez qu'une version dont vous détenez les droits. \
                         L'application ne fournit ni ne télécharge aucun texte biblique.",
                    )
                    .set_buttons(rfd::MessageButtons::OkCancel)
                    .show()
                    .await;
                if reponse != rfd::MessageDialogResult::Ok {
                    return None;
                }
                rfd::AsyncFileDialog::new()
                    .add_filter("Version biblique (JSON)", &["json"])
                    .pick_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::BibleVersionImported,
        ),
        Message::BibleVersionImported(Some(chemin)) => Task::perform(
            async move { crate::langage::versions::installer(&chemin) },
            Message::BibleVersionInstallee,
        ),
        Message::BibleVersionInstallee(resultat) => {
            match resultat {
                Ok(version) => {
                    let traduction = Traduction::depuis_code(&version.id).unwrap_or_default();
                    appliquer_traduction(state, traduction);
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Version « {} » installée : {} livre(s), {} chapitre(s).",
                        version.nom,
                        version.livres.len(),
                        version
                            .livres
                            .iter()
                            .map(|livre| livre.chapitres.len())
                            .sum::<usize>()
                    )));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Version biblique non installée : {erreur} Corrigez le fichier JSON ou \
                         choisissez-en un autre ; le dossier des versions s'ouvre par « Dossier »."
                    )));
                }
            }
            Task::none()
        }
        Message::BibleVersionImported(None) => Task::none(),
        Message::BibleVersionDossier => {
            let dossier = crate::langage::versions::VersionBiblique::dossier();
            let _ = std::fs::create_dir_all(&dossier);
            crate::project::ouvrir_chemin(&dossier);
            Task::none()
        }
        Message::BibleVersionSupprimer(code) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!(
                        "Supprimer la version biblique « {code} » ? Son fichier part à la corbeille."
                    ),
                    action: Message::BibleVersionSupprimer(code),
                });
                return Task::none();
            }
            state.geste_confirme = false;
            supprimer_la_version(state, &code)
        }
        Message::BibleVersionExporter(code) => Task::perform(
            async move {
                let Some(fichier) = rfd::AsyncFileDialog::new()
                    .add_filter("Version biblique (JSON)", &["json"])
                    .set_file_name(format!("{code}.json"))
                    .save_file()
                    .await
                else {
                    return Ok(None);
                };
                crate::langage::versions::exporter(&code, fichier.path()).map(|()| {
                    Some(format!(
                        "Version exportée vers {}. Joignez ce fichier à votre archive si vous \
                         voulez transporter les textes : l'archive seule ne les contient pas.",
                        fichier.path().display()
                    ))
                })
            },
            Message::BibleVersionExportee,
        ),
        Message::BibleVersionExportee(resultat) => {
            state.note = Some(match resultat {
                Ok(Some(message)) => crate::app::state::Note::info(message),
                Ok(None) => return Task::none(),
                Err(erreur) => crate::app::state::Note::erreur(format!(
                    "Version biblique non exportée : {erreur}"
                )),
            });
            Task::none()
        }
        Message::BibleVersionModele => Task::perform(
            async {
                let Some(fichier) = rfd::AsyncFileDialog::new()
                    .add_filter("Version biblique (JSON)", &["json"])
                    .set_file_name("modele-version-biblique.json")
                    .save_file()
                    .await
                else {
                    return Ok(None);
                };
                crate::services::stockage::ecrire_atomique(
                    fichier.path(),
                    &crate::langage::versions::modele(),
                )
                .map_err(|erreur| format!("Modèle non écrit : {erreur}"))?;
                Ok(Some(format!(
                    "Modèle écrit vers {}. Remplissez-le, puis installez-le avec « + ».",
                    fichier.path().display()
                )))
            },
            Message::BibleVersionModeleEcrit,
        ),
        Message::BibleVersionModeleEcrit(resultat) => {
            state.note = Some(match resultat {
                Ok(Some(message)) => crate::app::state::Note::info(message),
                Ok(None) => return Task::none(),
                Err(erreur) => crate::app::state::Note::erreur(erreur),
            });
            Task::none()
        }
        Message::BibleReferenceModifier(index) => preparer_edition(state, index),
        Message::BibleEditionLivre(index) => {
            if let Some(edition) = state.plugins.bible.edition.as_mut() {
                edition.livre = Some(index);
            }
            Task::none()
        }
        Message::BibleEditionChapitre(chapitre) => {
            if let Some(edition) = state.plugins.bible.edition.as_mut() {
                edition.chapitre = chapitre;
            }
            Task::none()
        }
        Message::BibleEditionVersets(versets) => {
            if let Some(edition) = state.plugins.bible.edition.as_mut() {
                edition.versets = versets;
            }
            Task::none()
        }
        Message::BibleEditionEnregistrer => enregistrer_edition(state),
        Message::BibleEditionAnnuler => {
            state.plugins.bible.edition = None;
            state.editor.editor.request_focus();
            Task::none()
        }

        // L'index et le repli sont des gestes de la même famille ; ils
        // restent dans `fichiers`, qui connaît déjà le dialecte du tampon.
        Message::BibleIndexInserer | Message::StatsReplier => {
            crate::app::fichiers::update(state, message)
        }

        _ => Task::none(),
    }
}

/// Supprime une version et remet la consultation sur la traduction par défaut
/// si c'était elle qu'on lisait.
fn supprimer_la_version(state: &mut State, code: &str) -> Task<Message> {
    // L'identité d'une version supprimée disparaît du registre : son indice
    // ne rend plus son code. La comparaison doit donc être faite avant le
    // retrait, pour remettre la consultation sur Louis Segond si c'était
    // elle qu'on lisait.
    let etait_lue = state
        .plugins
        .bible
        .traduction
        .code()
        .eq_ignore_ascii_case(code);
    match crate::langage::versions::supprimer(code) {
        Ok(nombre) => {
            if etait_lue {
                appliquer_traduction(state, Traduction::default());
            }
            state.note = Some(crate::app::state::Note::info(format!(
                "Version biblique supprimée ({nombre} fichier(s) à la corbeille)."
            )));
        }
        Err(erreur) => {
            state.note = Some(crate::app::state::Note::erreur(format!(
                "Version biblique non supprimée : {erreur}"
            )));
        }
    }
    Task::none()
}

/// Applique une nouvelle traduction à toute la consultation : le panneau
/// garde sa forme et son brouillon, la référence cliquée se relit.
fn appliquer_traduction(state: &mut State, traduction: Traduction) {
    let tous = livres(traduction);
    state.plugins.bible.traduction = traduction;
    if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
        let forme = panneau.forme;
        panneau.renouveler_choix(&tous, forme);
        panneau.livre = None;
        panneau.saisie_modifiee = true;
    }
    if let Some(cliquee) = state.plugins.bible.reference_cliquee.as_mut() {
        cliquee.extrait = extrait(&cliquee.references, traduction);
    }
}

/// Ouvre le panneau d'insertion et prend le clavier.
fn ouvrir_le_panneau(state: &mut State) -> Task<Message> {
    state.plugins.bible.panneau = Some(BiblePanneau::avec_livres(&livres(
        state.plugins.bible.traduction,
    )));
    state.editor.editor.lose_focus();
    Task::none()
}

/// Ouvre le panneau sur une balise déjà écrite : ses références sont reprises
/// et l'insertion remplace la plage au lieu d'ajouter une seconde balise.
pub(crate) fn ouvrir_le_panneau_sur(
    state: &mut State,
    plage: std::ops::Range<usize>,
    references: Vec<Reference>,
) {
    let mut panneau = BiblePanneau::avec_livres(&livres(state.plugins.bible.traduction));
    panneau.references = references;
    panneau.remplacement = Some(plage);
    state.plugins.bible.panneau = Some(panneau);
    state.editor.editor.lose_focus();
}

/// Prépare l'édition d'une référence de la consultation : le livre, le
/// chapitre et les versets sont repris de la référence cliquée.
fn preparer_edition(state: &mut State, index: usize) -> Task<Message> {
    let Some(cliquee) = state.plugins.bible.reference_cliquee.as_ref() else {
        return Task::none();
    };
    let Some(ElementCitation::Reference(reference)) = cliquee.elements.get(index) else {
        return Task::none();
    };
    let tous = livres(state.plugins.bible.traduction);
    let livre = tous.iter().position(|livre| livre.osis == reference.livre);
    let chapitre = if reference.chapitre_fin > reference.chapitre {
        format!("{}-{}", reference.chapitre, reference.chapitre_fin)
    } else {
        reference.chapitre.to_string()
    };
    let versets = if reference.verset_fin == u32::MAX {
        String::new()
    } else if reference.verset_fin > reference.verset_debut {
        format!("{}-{}", reference.verset_debut, reference.verset_fin)
    } else {
        reference.verset_debut.to_string()
    };

    state.plugins.bible.edition = Some(ReferenceEdition {
        index,
        livre,
        chapitre,
        versets,
        traduction: reference.traduction,
    });
    state.editor.editor.lose_focus();
    iced::widget::operation::focus(crate::ui::bible_edition::id_premier_champ())
}

/// Applique l'édition à la balise cliquée, puis laisse la consultation à jour.
fn enregistrer_edition(state: &mut State) -> Task<Message> {
    let Some(edition) = state.plugins.bible.edition.clone() else {
        return Task::none();
    };
    let Some(cliquee) = state.plugins.bible.reference_cliquee.clone() else {
        return Task::none();
    };
    let Some(livre) = edition
        .livre
        .and_then(|index| livres(state.plugins.bible.traduction).get(index).cloned())
    else {
        state.note = Some(crate::app::state::Note::info(
            "Choisissez le livre de la référence.",
        ));
        return Task::none();
    };
    let nouvelles = references_depuis_texte(&livre.osis, &edition.chapitre, &edition.versets);
    if nouvelles.is_empty() {
        state.note = Some(crate::app::state::Note::info(
            "Indiquez un chapitre et des versets valides.",
        ));
        return Task::none();
    }
    let nouvelles: Vec<_> = nouvelles
        .into_iter()
        .map(|mut reference| {
            reference.traduction = edition.traduction;
            reference
        })
        .collect();
    if edition.index >= cliquee.elements.len() {
        state.plugins.bible.edition = None;
        return Task::none();
    }

    let mut elements = cliquee.elements.clone();
    elements.splice(
        edition.index..edition.index + 1,
        nouvelles.iter().cloned().map(ElementCitation::Reference),
    );
    let mut corps =
        crate::langage::references::ecrire_elements(&elements, &state.reglages.references);
    if let Some(forme) = cliquee.forme {
        corps.push_str(match forme {
            FormeLivre::FrancaiseLongue => "|long",
            FormeLivre::FrancaiseCourte => "|court",
            FormeLivre::Osis => "|osis",
            FormeLivre::SansPoint => "|sanspoint",
            FormeLivre::Tob => "|tob",
        });
    }
    let texte = format!("{{{{{corps}}}}}");
    let (debut, fin) = cliquee.intervalle;
    let tache = state.editor.replace_span(debut, fin, &texte);

    let references: Vec<Reference> = elements
        .iter()
        .filter_map(|element| element.reference().cloned())
        .collect();
    let extrait = extrait(&references, state.plugins.bible.traduction);
    state.plugins.bible.reference_cliquee = Some(ReferenceCliquee {
        intervalle: (debut, debut + texte.len()),
        elements,
        references,
        forme: cliquee.forme,
        extrait,
    });
    state.plugins.bible.edition = None;
    state.editor.editor.request_focus();
    tache.map(Message::Editor)
}

/// Ajoute au panneau la référence décrite par le livre, le chapitre et les
/// versets. Le livre et le chapitre restent en place : on en ajoute une
/// autre du même chapitre d'un geste.
fn ajouter_au_panneau(state: &mut State) -> Task<Message> {
    let tous = livres(state.plugins.bible.traduction);
    let Some(panneau) = state.plugins.bible.panneau.as_mut() else {
        return Task::none();
    };
    let Some(_) = livre_du_panneau(panneau, &tous) else {
        state.note = Some(crate::app::state::Note::info(
            "Choisissez d'abord un livre biblique.",
        ));
        return Task::none();
    };
    let nouvelles = references_du_panneau(panneau, &tous);
    if nouvelles.is_empty() {
        state.note = Some(crate::app::state::Note::info(
            "Indiquez un chapitre et des versets valides.",
        ));
        return Task::none();
    }

    panneau.references.extend(nouvelles);
    panneau.versets.clear();
    panneau.saisie_modifiee = false;
    Task::none()
}

/// Le livre désigné par la liste, ou par le texte encore dans le champ.
///
/// Le widget ne rend un livre que lorsqu'on choisit une option ; si l'auteur
/// a tapé un nom exact puis a inséré sans cliquer, on le résout ici. Un texte
/// ambigu ne désigne aucun livre : il ne faut pas prendre le premier venu.
fn livre_du_panneau(
    panneau: &BiblePanneau,
    tous: &[crate::plugins::bible::LivreConnu],
) -> Option<usize> {
    if let Some(index) = panneau.livre
        && index < tous.len()
    {
        return Some(index);
    }

    let recherche = panneau.recherche.trim();
    if recherche.is_empty() {
        return None;
    }

    let code = crate::langage::references::code_du_livre(recherche)
        .or_else(|| depuis_texte(recherche).map(|reference| reference.livre));
    if let Some(code) = code {
        return tous.iter().position(|livre| livre.osis == code);
    }

    let filtres = filtrer_livres(tous, recherche);
    if filtres.len() == 1 {
        tous.iter().position(|livre| livre.osis == filtres[0].osis)
    } else {
        None
    }
}

/// Les références décrites par les champs courants du panneau.
fn references_du_panneau(
    panneau: &BiblePanneau,
    tous: &[crate::plugins::bible::LivreConnu],
) -> Vec<Reference> {
    let Some(index) = livre_du_panneau(panneau, tous) else {
        return Vec::new();
    };
    let Some(livre) = tous.get(index) else {
        return Vec::new();
    };
    references_depuis_texte(&livre.osis, &panneau.chapitre, &panneau.versets)
}

/// Pose la citation construite au curseur et referme le panneau.
fn inserer_la_citation(state: &mut State) -> Task<Message> {
    let tous = livres(state.plugins.bible.traduction);
    let vide = {
        let Some(panneau) = state.plugins.bible.panneau.as_mut() else {
            return Task::none();
        };
        let nouvelles = if panneau.saisie_modifiee {
            references_du_panneau(panneau, &tous)
        } else {
            Vec::new()
        };
        for reference in nouvelles {
            if !panneau.references.contains(&reference) {
                panneau.references.push(reference);
            }
        }
        panneau.references.is_empty()
    };
    if vide {
        state.note = Some(crate::app::state::Note::info(
            "Ajoutez au moins une référence avant d'insérer.",
        ));
        return Task::none();
    }

    let Some(panneau) = state.plugins.bible.panneau.take() else {
        return Task::none();
    };

    let normes = &state.reglages.references;
    let mut corps = match panneau.forme {
        Some(forme) => afficher_groupe_forme(&panneau.references, normes, Some(forme)),
        None => afficher_groupe(&panneau.references, normes),
    };
    if !panneau.texte_libre.trim().is_empty() {
        if !corps.is_empty() {
            corps.push_str(normes.separateur_livre.as_str());
        }
        corps.push_str(&crate::langage::references::texte_libre_source(
            &panneau.texte_libre,
        ));
    }
    if let Some(forme) = panneau.forme {
        corps.push_str(match forme {
            FormeLivre::FrancaiseLongue => "|long",
            FormeLivre::FrancaiseCourte => "|court",
            FormeLivre::Osis => "|osis",
            FormeLivre::SansPoint => "|sanspoint",
            FormeLivre::Tob => "|tob",
        });
    }

    let texte = format!("{{{{{corps}}}}}");
    let tache = if let Some(remplacement) = panneau.remplacement.clone() {
        state
            .editor
            .replace_span(remplacement.start, remplacement.end, &texte)
            .map(Message::Editor)
    } else {
        state
            .editor
            .insert_avec_curseur(&texte, 2)
            .map(Message::Editor)
    };
    state.editor.editor.request_focus();
    tache
}

/// Va à la première balise qui porte la référence cliquée.
fn aller_a_la_reference(state: &mut State, reference: &str) -> Task<Message> {
    let contenu = state.editor.content();
    let Some((debut, fin)) = position_reference(&contenu, reference, &state.reglages.references)
    else {
        state.note = Some(crate::app::state::Note::info(
            "Cette référence n'est plus dans le document.",
        ));
        return Task::none();
    };
    state.editor.editor.request_focus();
    state
        .editor
        .selectionner_plage(debut, fin)
        .map(Message::Editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genese(tous: &[crate::plugins::bible::LivreConnu]) -> usize {
        tous.iter()
            .position(|livre| livre.osis == "Gen")
            .expect("la Genèse")
    }

    #[test]
    fn les_champs_forment_une_reference_sans_cliquer_ajouter() {
        let tous = livres(Traduction::Lsg);
        let mut panneau = BiblePanneau::avec_livres(&tous);
        assert_eq!(
            panneau.choix.options().len(),
            tous.len(),
            "la liste cherchable porte tous les livres"
        );
        panneau.livre = Some(genese(&tous));
        panneau.chapitre = "3".to_string();
        panneau.versets = "16".to_string();

        let references = references_du_panneau(&panneau, &tous);
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].livre, "Gen");
        assert_eq!(references[0].chapitre, 3);
        assert_eq!(references[0].verset_debut, 16);
    }

    #[test]
    fn un_nom_tape_resout_le_livre_meme_sans_selection() {
        let tous = livres(Traduction::Lsg);
        let mut panneau = BiblePanneau::avec_livres(&tous);
        panneau.recherche = "Genèse".to_string();
        panneau.chapitre = "1".to_string();

        let references = references_du_panneau(&panneau, &tous);
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].livre, "Gen");
        assert_eq!(references[0].chapitre, 1);
    }

    #[test]
    fn un_texte_ambigu_ne_designe_aucun_livre() {
        let tous = livres(Traduction::Lsg);
        let mut panneau = BiblePanneau::avec_livres(&tous);
        panneau.recherche = "a".to_string();
        panneau.chapitre = "1".to_string();

        assert!(
            references_du_panneau(&panneau, &tous).is_empty(),
            "un filtre qui laisse plusieurs livres ne doit pas en choisir un"
        );
    }
}
