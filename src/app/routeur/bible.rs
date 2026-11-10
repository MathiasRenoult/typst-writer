//! Les messages du plugin biblique.
//!
//! Le panneau de construction, les statistiques et la consultation vivent
//! ici : le routeur central n'a plus à connaître leurs détails.

use crate::app::{Message, State};
use crate::plugins::bible::{
    BiblePanneau, ElementCitation, FormeLivre, Reference, ReferenceCliquee, ReferenceEdition,
    Traduction, afficher_groupe, afficher_groupe_forme, extrait, livres, position_reference,
    references_depuis_texte,
};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ReferenceBibliqueInserer => ouvrir_le_panneau(state),

        Message::BiblePanneauRecherche(recherche) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.recherche = recherche;
            }
            Task::none()
        }
        Message::BiblePanneauLivre(index) => {
            let total = livres(Traduction::Lsg).len();
            if let Some(panneau) = state.plugins.bible.panneau.as_mut()
                && index < total
            {
                panneau.livre = Some(index);
            }
            Task::none()
        }
        Message::BiblePanneauChapitre(chapitre) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.chapitre = chapitre;
            }
            Task::none()
        }
        Message::BiblePanneauVersets(versets) => {
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.versets = versets;
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
            if let Some(panneau) = state.plugins.bible.panneau.as_mut() {
                panneau.forme = forme;
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
            state.plugins.bible.traduction = traduction;
            if let Some(cliquee) = state.plugins.bible.reference_cliquee.as_mut() {
                cliquee.extrait = extrait(&cliquee.references, traduction);
            }
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

/// Ouvre le panneau d'insertion et prend le clavier.
fn ouvrir_le_panneau(state: &mut State) -> Task<Message> {
    state.plugins.bible.panneau = Some(BiblePanneau::default());
    state.editor.editor.lose_focus();
    iced::widget::operation::focus(crate::ui::bible::id_champ())
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
    let tous = livres(Traduction::Lsg);
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
        .and_then(|index| livres(Traduction::Lsg).get(index).cloned())
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
    let Some(panneau) = state.plugins.bible.panneau.as_mut() else {
        return Task::none();
    };
    let Some(index) = panneau.livre else {
        state.note = Some(crate::app::state::Note::info(
            "Choisissez d'abord un livre biblique.",
        ));
        return Task::none();
    };
    let livres = livres(Traduction::Lsg);
    let Some(livre) = livres.get(index) else {
        return Task::none();
    };
    let nouvelles = references_depuis_texte(&livre.osis, &panneau.chapitre, &panneau.versets);
    if nouvelles.is_empty() {
        state.note = Some(crate::app::state::Note::info(
            "Indiquez un chapitre et des versets valides.",
        ));
        return Task::none();
    }

    panneau.references.extend(nouvelles);
    panneau.versets.clear();
    Task::none()
}

/// Pose la citation construite au curseur et referme le panneau.
fn inserer_la_citation(state: &mut State) -> Task<Message> {
    let vide = state
        .plugins
        .bible
        .panneau
        .as_ref()
        .is_none_or(|panneau| panneau.references.is_empty());
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
        });
    }

    let texte = format!("{{{{{corps}}}}}");
    let tache = state
        .editor
        .insert_avec_curseur(&texte, 2)
        .map(Message::Editor);
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
