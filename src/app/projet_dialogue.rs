//! Le dialogue de projet : création, renommage, normes, base.
//!
//! Ses bras vivaient dans le conducteur (2 818 lignes) ; la feuille de
//! route (phase 1, cartographie, geste 3) les réunit par famille.

use super::Message;
use super::State;
use iced::Task;

pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ProjetNom(nom) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                dialogue.nom = nom;
            }

            Task::none()
        }
        Message::ProjetMeta(champ, valeur) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                match champ.as_str() {
                    "titre" => dialogue.titre = valeur,
                    "auteur" => dialogue.auteur = valeur,
                    _ => {}
                }
            }

            Task::none()
        }
        Message::ProjetDepuisEtagere(etagere) => {
            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                base: String::new(),
                bases: Vec::new(),
                fonctionnalites: Default::default(),
                norme: String::new(),
                etagere: etagere.clone(),
                nom: String::new(),
                titre: String::new(),
                auteur: String::new(),
                cible: None,
            });

            state.note = Some(crate::app::state::Note::info(format!(
                "Le projet rejoindra « {etagere} »."
            )));

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }
        Message::ProjetNorme(nom) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                dialogue.norme = nom;
            }

            Task::none()
        }
        Message::ProjetCreer => {
            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                base: String::new(),
                bases: Vec::new(),
                fonctionnalites: Default::default(),
                norme: String::new(),
                etagere: String::new(),
                nom: String::new(),
                titre: String::new(),
                auteur: String::new(),
                cible: None,
            });

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }
        Message::ProjetRenommer(dir) => {
            let nom = state
                .projects
                .iter()
                .find(|projet| projet.dir == dir)
                .map(|projet| projet.name.clone())
                .unwrap_or_default();

            state.projet_dialogue = Some(crate::app::state::ProjetDialogue {
                base: String::new(),
                bases: Vec::new(),
                fonctionnalites: Default::default(),
                norme: String::new(),
                etagere: String::new(),
                nom,
                titre: String::new(),
                auteur: String::new(),
                cible: Some(dir),
            });

            iced::widget::operation::focus(crate::ui::projet::id_du_nom())
        }
        Message::ProjetValider => {
            let Some(dialogue) = state.projet_dialogue.clone() else {
                return Task::none();
            };

            let nom = dialogue.nom.trim().to_string();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Un projet porte un nom.".to_string(),
                ));
                return Task::none();
            }

            // Le verrou suit le dossier pendant tout le renommage. Les vues
            // modifiées sont enregistrées avant que leurs chemins changent.
            let mut verrou_renommage = None;
            let mut verrou_deja_detenu = false;
            if let Some(dir) = &dialogue.cible {
                verrou_deja_detenu = state.espace.verrous.contains_key(dir)
                    || (state.project.dir == *dir && state.verrou.is_some());
                if !verrou_deja_detenu {
                    match crate::project::Verrou::prendre(dir) {
                        Ok(verrou) => verrou_renommage = Some(verrou),
                        Err(probleme) => {
                            state.note = Some(crate::app::state::Note::erreur(format!(
                                "Renommage refusé : {probleme}"
                            )));
                            return Task::none();
                        }
                    }
                }
                if let Err(probleme) = state.sauvegarder_avant_deplacer(dir) {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Renommage annulé : {probleme}. Corrigez la sauvegarde et réessayez."
                    )));
                    return Task::none();
                }
                if verrou_renommage.is_none() {
                    verrou_renommage = state.espace.verrous.remove(dir);
                    if verrou_renommage.is_none() && state.project.dir == *dir {
                        verrou_renommage = state.verrou.take();
                    }
                }
            }

            let rendu = match &dialogue.cible {
                None if !dialogue.base.is_empty() => {
                    // Le projet naît de sa base : un projet-modèle, copié
                    // sans son verrou ni ses sauvegardes (feuille de route).
                    crate::project::creer_au_nom_depuis(
                        &crate::project::projects_dir(),
                        &nom,
                        &crate::project::projects_dir().join(&dialogue.base),
                    )
                }
                None => crate::project::creer_au_nom(&crate::project::projects_dir(), &nom),
                Some(dir) => crate::project::renommer(dir, &nom),
            };

            match rendu {
                Ok(dir) => {
                    // Les métadonnées choisies à la création : le projet
                    // naît avec les siennes (bug-report-7, point 9).
                    if dialogue.cible.is_none() {
                        let mut meta = crate::project::metadonnees::Metadonnees::base();
                        if !dialogue.titre.trim().is_empty() {
                            meta.definir(
                                "",
                                "titre".to_string(),
                                dialogue.titre.trim().to_string(),
                            );
                        }
                        if !dialogue.auteur.trim().is_empty() {
                            meta.definir(
                                "",
                                "auteur".to_string(),
                                dialogue.auteur.trim().to_string(),
                            );
                        }
                        let _ = meta.sauver(&dir);
                    }

                    if let Some(mut verrou) = verrou_renommage.take() {
                        verrou.suivre_deplacement(&dir);
                        state.espace.verrous.insert(dir.clone(), verrou);
                    }
                    if let Some(ancien) = &dialogue.cible
                        && ancien != &dir
                    {
                        state.suivre_projet_renomme(ancien, &dir);
                        state.save_session();
                    }
                    state.projet_dialogue = None;
                    // Le nom a changé : la liste des projets se relit pour
                    // que l'onglet et le panneau portent le nouveau nom.
                    state.projects = crate::project::list_projects();

                    // La norme choisie à la création devient celle du
                    // projet : elle s'y pose par la même voie que le
                    // panneau, et le modèle du document ne change pas.
                    if dialogue.cible.is_none()
                        && !dialogue.norme.is_empty()
                        && let Some(mise) = crate::project::norme(&dialogue.norme)
                    {
                        let _ = crate::mise_en_page::appliquer(&dir, &mise, false);
                    }

                    if dialogue.cible.is_none() && !dialogue.etagere.is_empty() {
                        let nom = dir
                            .file_name()
                            .map(|nom| nom.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let _ = crate::project::definir_etagere(&nom, &dialogue.etagere);
                    }

                    crate::app::update(state, Message::OpenProject(dir))
                }
                Err(probleme) => {
                    if verrou_deja_detenu
                        && let (Some(dir), Some(verrou)) =
                            (&dialogue.cible, verrou_renommage.take())
                    {
                        state.espace.verrous.insert(dir.clone(), verrou);
                    }
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }
        Message::ProjetAnnuler => {
            state.projet_dialogue = None;
            Task::none()
        }
        Message::ProjetBase(nom) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                dialogue.base = nom;
            }
            Task::none()
        }
        Message::ProjetFonctionnalite(nom, actif) => {
            if let Some(dialogue) = state.projet_dialogue.as_mut() {
                let f = &mut dialogue.fonctionnalites;
                match nom.as_str() {
                    "bibliographie" => f.bibliographie = actif,
                    "glossaire" => f.glossaire = actif,
                    "graphiques" => f.graphiques = actif,
                    _ => {}
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
