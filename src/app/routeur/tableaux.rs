//! La grille de tableau : choisir la taille, insérer la forme.

use crate::app::{Message, State};
use crate::langage::tableau::{FormatageTableau, SensDeplacement};
use iced::Task;

/// Applique un remplacement de tableau, sans toucher au reste du document.
pub(crate) fn appliquer_formatage(state: &mut State, formatage: FormatageTableau) -> Task<Message> {
    let contenu = state.editor.content();
    let change = contenu[formatage.plage.clone()] != formatage.texte;
    if change {
        state.rafraichir_statistiques_bibliques();
        state.mots_a_recalculer();
    }
    let remplacement = if change {
        state
            .editor
            .replace_span(formatage.plage.start, formatage.plage.end, &formatage.texte)
            .map(Message::Editor)
    } else {
        Task::none()
    };

    Task::batch([
        remplacement,
        state
            .editor
            .set_cursor_offset(formatage.curseur)
            .map(Message::Editor),
    ])
}

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::TableauOuvrir => {
            state.tableau = Some(crate::app::state::TableauDialogue {
                rangs: "3".to_string(),
                colonnes: "3".to_string(),
                titre: String::new(),
            });
            Task::none()
        }
        Message::TableauTitre(valeur) => {
            if let Some(tableau) = state.tableau.as_mut() {
                tableau.titre = valeur;
            }
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
        Message::TableauNaviguer(sens) => {
            let contenu = state.editor.content();
            let Some(formatage) = crate::langage::tableau::naviguer_et_formater(
                &contenu,
                state.editor.cursor_offset(),
                sens,
            ) else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauInsererColonne => {
            let contenu = state.editor.content();
            let Some(formatage) =
                crate::langage::tableau::inserer_colonne(&contenu, state.editor.cursor_offset())
            else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauInsererLigne => {
            let contenu = state.editor.content();
            let Some(formatage) =
                crate::langage::tableau::inserer_ligne(&contenu, state.editor.cursor_offset())
            else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauSupprimerLigne => {
            let contenu = state.editor.content();
            let Some(formatage) =
                crate::langage::tableau::supprimer_ligne(&contenu, state.editor.cursor_offset())
            else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauSupprimerColonne => {
            let contenu = state.editor.content();
            let Some(formatage) =
                crate::langage::tableau::supprimer_colonne(&contenu, state.editor.cursor_offset())
            else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauDeplacer(sens) => {
            let contenu = state.editor.content();
            let formatage = match sens {
                SensDeplacement::Haut | SensDeplacement::Bas => {
                    crate::langage::tableau::deplacer_ligne(
                        &contenu,
                        state.editor.cursor_offset(),
                        sens,
                    )
                }
                SensDeplacement::Gauche | SensDeplacement::Droite => {
                    crate::langage::tableau::deplacer_colonne(
                        &contenu,
                        state.editor.cursor_offset(),
                        sens,
                    )
                }
            };
            let Some(formatage) = formatage else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::TableauAlignerEntete => {
            let contenu = state.editor.content();
            let Some(formatage) =
                crate::langage::tableau::aligner_colonne(&contenu, state.editor.cursor_offset())
            else {
                return Task::none();
            };
            appliquer_formatage(state, formatage)
        }
        Message::BeautifierTableaux => {
            let contenu = state.editor.content();
            let mut formate = crate::langage::tableau::embellir(&contenu);
            if formate == contenu {
                return Task::none();
            }

            // Le saut de ligne final reste dans la source : le remplacer
            // aussi ajoutait une ligne vide à la fin du document.
            let fin = if contenu.ends_with('\n') && formate.ends_with('\n') {
                formate.pop();
                contenu.len() - 1
            } else {
                contenu.len()
            };

            state.rafraichir_statistiques_bibliques();
            state.mots_a_recalculer();
            state
                .editor
                .replace_span(0, fin, &formate)
                .map(Message::Editor)
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

            let titre = tableau.titre.trim();
            let ouverture = if titre.is_empty() {
                "{{t}}".to_string()
            } else {
                let titre = titre.replace('\\', "\\\\").replace('"', "\\\"");
                format!("{{{{t \"{titre}\"}}}}")
            };
            let grille = format!("{ouverture}\n{}", grille.join("\n"));

            state.editor.insert_at_cursor(&grille).map(Message::Editor)
        }
        _ => Task::none(),
    }
}
