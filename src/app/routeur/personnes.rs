//! Les messages du registre des personnes citées.

use crate::app::{Message, State, champ_renseigne};
use crate::project::personnes::{EntreePersonne, Personnes};
use iced::Task;

/// Traite un message des personnes. La porte d'activation est déjà passée.
/// Ouvre le panneau sur une balise existante : la clé et l'expression sont
/// reprises, et la validation remplacera la balise au lieu d'en ajouter une.
pub(crate) fn ouvrir_panneau_sur(
    state: &mut State,
    plage: std::ops::Range<usize>,
    cle: &str,
    expression: &str,
) -> Task<Message> {
    let mut panneau =
        crate::plugins::personnes::PersonnePanneau::nouveau(&state.plugins.personnes.entrees);
    if let Some(entree) = state
        .plugins
        .personnes
        .entrees
        .iter()
        .find(|entree| entree.cle == cle)
        .cloned()
    {
        panneau.choisir(&entree);
    } else {
        panneau.edition.cle = cle.to_string();
    }
    panneau.expression = expression.to_string();
    panneau.remplacement = Some(plage);
    state.plugins.personnes.fenetre = false;
    state.plugins.personnes.edition = None;
    state.plugins.personnes.panneau = Some(panneau);
    state.balise_dialogue = None;
    state.editor.editor.lose_focus();
    iced::widget::operation::focus(crate::ui::personne_panneau::id_recherche())
}

pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::PersonnesFenetreOuvrir => {
            state.plugins.personnes.fenetre = true;
            state.plugins.personnes.edition = None;
            state.editor.editor.lose_focus();
            Task::none()
        }
        Message::PersonnesFenetreFermer => {
            state.plugins.personnes.fenetre = false;
            state.plugins.personnes.filtre.clear();
            Task::none()
        }
        Message::PersonnesCreer => {
            state.plugins.personnes.fenetre = false;
            state.plugins.personnes.edition =
                Some(crate::plugins::personnes::EditionPersonne::nouvelle());
            iced::widget::operation::focus(crate::ui::personnes::id_premier_champ())
        }
        Message::PersonnesModifier(cle) => {
            let entree = state
                .plugins
                .personnes
                .entrees
                .iter()
                .find(|entree| entree.cle == cle)
                .cloned();

            let Some(entree) = entree else {
                return Task::none();
            };

            state.plugins.personnes.fenetre = false;
            state.plugins.personnes.edition = Some(
                crate::plugins::personnes::EditionPersonne::depuis_entree(&entree),
            );

            iced::widget::operation::focus(crate::ui::personnes::id_premier_champ())
        }
        Message::PersonnesSupprimer(cle) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: format!(
                        "Supprimer la personne « {cle} » ? Les balises qui la citent resteront, mais l'index sera réécrit sans elle."
                    ),
                    action: Message::PersonnesSupprimer(cle),
                });
                return Task::none();
            }

            state.geste_confirme = false;
            let mut personnes = Personnes {
                entrees: state.plugins.personnes.entrees.clone(),
            };

            if personnes.supprimer(&cle) {
                match personnes.sauver(&state.project.dir) {
                    Ok(()) => {
                        state.plugins.personnes.entrees = personnes.entrees;
                        state.plugins.personnes.edition = None;
                        state.note = Some(crate::app::state::Note::info("Personne supprimée."));
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
        Message::PersonnesFiltrer(filtre) => {
            state.plugins.personnes.filtre = filtre;
            Task::none()
        }
        Message::PersonnesInserer(cle) => {
            let texte = format!("{{{{p {cle}}}}}");
            state.editor.insert_at_cursor(&texte).map(Message::Editor)
        }
        Message::PersonnesIndexInserer => state
            .editor
            .insert_at_cursor("{{index-personnes}}")
            .map(Message::Editor),
        Message::EditionPersonneChamp(champ, valeur) => {
            if let Some(edition) = state.plugins.personnes.edition.as_mut() {
                modifier_edition(edition, &champ, valeur);
            }
            Task::none()
        }
        Message::EditionPersonneFermer => {
            state.plugins.personnes.edition = None;
            Task::none()
        }
        Message::EditionPersonneEnregistrer => {
            let Some(edition) = state.plugins.personnes.edition.clone() else {
                return Task::none();
            };

            let entree = match valider_edition(state, &edition, edition.creation) {
                Ok(entree) => entree,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                    return Task::none();
                }
            };

            let mut personnes = Personnes {
                entrees: state.plugins.personnes.entrees.clone(),
            };
            personnes.definir(entree);

            match personnes.sauver(&state.project.dir) {
                Ok(()) => {
                    state.plugins.personnes.entrees = personnes.entrees;
                    state.plugins.personnes.edition = None;
                    state.note = Some(crate::app::state::Note::info("Personne enregistrée."));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement impossible : {erreur}"
                    )));
                }
            }

            Task::none()
        }
        Message::PersonnePanneauOuvrir => {
            let panneau = crate::plugins::personnes::PersonnePanneau::nouveau(
                &state.plugins.personnes.entrees,
            );
            state.plugins.personnes.fenetre = false;
            state.plugins.personnes.edition = None;
            state.plugins.personnes.panneau = Some(panneau);
            state.balise_dialogue = None;
            state.editor.editor.lose_focus();
            iced::widget::operation::focus(crate::ui::personne_panneau::id_recherche())
        }
        Message::PersonnePanneauFermer => {
            state.plugins.personnes.panneau = None;
            crate::app::palette::redonner_le_clavier(state)
        }
        Message::PersonnePanneauRecherche(texte) => {
            if let Some(panneau) = state.plugins.personnes.panneau.as_mut() {
                panneau.recherche = texte;
            }
            Task::none()
        }
        Message::PersonnePanneauValiderRecherche => {
            let cle = state
                .plugins
                .personnes
                .panneau
                .as_ref()
                .and_then(|panneau| {
                    panneau
                        .options(&state.plugins.personnes.entrees)
                        .first()
                        .map(|choix| choix.cle.clone())
                });
            if let Some(cle) = cle {
                return traiter(state, Message::PersonnePanneauChoisir(cle));
            }
            Task::none()
        }
        Message::PersonnePanneauChoisir(cle) => {
            let entree = state
                .plugins
                .personnes
                .entrees
                .iter()
                .find(|entree| entree.cle == cle)
                .cloned();

            if let (Some(panneau), Some(entree)) =
                (state.plugins.personnes.panneau.as_mut(), entree)
            {
                panneau.choisir(&entree);
            }

            Task::none()
        }
        Message::PersonnePanneauNouvelle => {
            if let Some(panneau) = state.plugins.personnes.panneau.as_mut() {
                panneau.nouvelle(&state.plugins.personnes.entrees);
            }
            Task::none()
        }
        Message::PersonnePanneauChamp(champ, valeur) => {
            if let Some(panneau) = state.plugins.personnes.panneau.as_mut() {
                modifier_edition(&mut panneau.edition, &champ, valeur);
            }
            Task::none()
        }
        Message::PersonnePanneauExpression(expression) => {
            if let Some(panneau) = state.plugins.personnes.panneau.as_mut() {
                panneau.expression = expression;
            }
            Task::none()
        }
        Message::PersonnePanneauEnregistrer => {
            let Some(panneau) = state.plugins.personnes.panneau.as_ref() else {
                return Task::none();
            };
            let edition = panneau.edition.clone();
            let creation = edition.creation;

            let entree = match valider_edition(state, &edition, creation) {
                Ok(entree) => entree,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                    return Task::none();
                }
            };

            let mut personnes = Personnes {
                entrees: state.plugins.personnes.entrees.clone(),
            };
            personnes.definir(entree);

            match personnes.sauver(&state.project.dir) {
                Ok(()) => {
                    state.plugins.personnes.entrees = personnes.entrees;
                    if let Some(panneau) = state.plugins.personnes.panneau.as_mut() {
                        panneau.selection = Some(panneau.edition.cle.trim().to_string());
                        panneau.edition.creation = false;
                    }
                    state.note = Some(crate::app::state::Note::info("Personne enregistrée."));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Enregistrement impossible : {erreur}"
                    )));
                }
            }

            Task::none()
        }
        Message::PersonnePanneauInserer => {
            let Some(panneau) = state.plugins.personnes.panneau.as_ref() else {
                return Task::none();
            };
            let texte = match panneau.texte() {
                Ok(texte) => texte,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(erreur));
                    return Task::none();
                }
            };
            if !state
                .plugins
                .personnes
                .entrees
                .iter()
                .any(|entree| entree.cle == panneau.edition.cle.trim())
            {
                state.note = Some(crate::app::state::Note::erreur(
                    "Enregistrez la fiche avant d'insérer sa balise.".to_string(),
                ));
                return Task::none();
            }
            let remplacement = panneau.remplacement.clone();
            state.plugins.personnes.panneau = None;

            let insertion = if let Some(plage) = remplacement {
                state
                    .editor
                    .replace_span(plage.start, plage.end, &texte)
                    .map(Message::Editor)
            } else {
                state.editor.insert_at_cursor(&texte).map(Message::Editor)
            };

            Task::batch([insertion, crate::app::palette::redonner_le_clavier(state)])
        }
        _ => Task::none(),
    }
}

/// Applique un champ de la fiche au formulaire du panneau.
fn modifier_edition(
    edition: &mut crate::plugins::personnes::EditionPersonne,
    champ: &str,
    valeur: String,
) {
    match champ {
        "cle" => edition.cle = valeur,
        "nom" => edition.nom = valeur,
        "nom_simple" => edition.nom_simple = valeur,
        "alias" => edition.alias = valeur,
        "tri" => edition.tri = valeur,
        "dates" => edition.dates = valeur,
        "naissance" => edition.naissance = valeur,
        "mort" => edition.mort = valeur,
        "qualite" => edition.qualite = valeur,
        "variantes" => edition.variantes = valeur,
        _ => {}
    }
}

/// Valide et prépare une entrée, sans écrire le fichier.
fn valider_edition(
    state: &State,
    edition: &crate::plugins::personnes::EditionPersonne,
    creation: bool,
) -> Result<EntreePersonne, String> {
    let cle = edition.cle.trim();
    if !EntreePersonne::cle_valide(cle) {
        return Err(
            "La clé d'une personne est un identifiant : lettres, chiffres, - _ . — sans espace ni deux-points."
                .to_string(),
        );
    }
    if creation
        && state
            .plugins
            .personnes
            .entrees
            .iter()
            .any(|entree| entree.cle == cle)
    {
        return Err(format!(
            "La personne « {cle} » existe déjà — choisissez une autre clé."
        ));
    }
    if champ_renseigne(&edition.nom).is_none() {
        return Err("Le nom complet d'une personne ne peut pas être vide.".to_string());
    }

    Ok(edition.entree())
}
