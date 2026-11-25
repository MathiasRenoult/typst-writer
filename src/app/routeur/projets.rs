//! La liste des projets : vignettes, étagères, glisser-déposer, suppression.

use crate::app::message::FormatExport;
use crate::app::{Message, State};
use iced::Task;
use std::path::{Path, PathBuf};

/// Les projets dans l'ordre où la bibliothèque les montre : étagères dans
/// leur ordre, puis les projets sans étagère. Les sélections par Maj+clic
/// suivent cet ordre, jamais l'ordre alphabétique des dossiers.
fn ordre_librairie(state: &State) -> Vec<PathBuf> {
    let mut ordre = Vec::new();
    let mut vus = std::collections::HashSet::new();

    for etagere in &state.etageres_ordre {
        for projet in &state.projects {
            if state.etageres.get(&projet.name) == Some(etagere) && vus.insert(projet.dir.clone()) {
                ordre.push(projet.dir.clone());
            }
        }
    }
    for projet in &state.projects {
        if vus.insert(projet.dir.clone()) {
            ordre.push(projet.dir.clone());
        }
    }

    ordre
}

/// Les dossiers sélectionnés, dans l'ordre de la bibliothèque.
fn selection_dossiers(state: &State) -> Vec<PathBuf> {
    ordre_librairie(state)
        .into_iter()
        .filter(|dir| state.selection_projets.contains(dir))
        .collect()
}

/// La plage entre deux documents, dans l'ordre de la bibliothèque — le
/// geste de Maj+clic. Si l'un des deux manque, la cible est seule.
fn plage_selection(ordre: &[PathBuf], ancre: &Path, cible: &Path) -> Vec<PathBuf> {
    let Some(debut) = ordre.iter().position(|chemin| chemin == ancre) else {
        return vec![cible.to_path_buf()];
    };
    let Some(fin) = ordre.iter().position(|chemin| chemin == cible) else {
        return vec![cible.to_path_buf()];
    };
    let (debut, fin) = (debut.min(fin), debut.max(fin));
    ordre[debut..=fin].to_vec()
}

/// Une destination qui n'écrase jamais un fichier existant.
fn chemin_libre(dossier: &Path, nom: &str, extension: &str) -> PathBuf {
    let mut numero = 1;
    loop {
        let fichier = if numero == 1 {
            format!("{nom}.{extension}")
        } else {
            format!("{nom}-{numero}.{extension}")
        };
        let chemin = dossier.join(fichier);
        if !chemin.exists() {
            return chemin;
        }
        numero += 1;
    }
}

fn source_du_projet(dir: &Path) -> Result<String, String> {
    if let Ok(source) = std::fs::read_to_string(dir.join("document.tw")) {
        return Ok(source);
    }
    std::fs::read_to_string(dir.join("main.typ"))
        .map_err(|erreur| format!("La source du projet ne se lit pas : {erreur}"))
}

/// L'assemblage Typst que le PDF compose — jamais la source Markdown.
fn assemblage_du_projet(dir: &Path) -> Result<String, String> {
    let principal = std::fs::read_to_string(dir.join("main.typ")).unwrap_or_default();
    if !principal.trim().is_empty() {
        return Ok(principal);
    }
    let source = std::fs::read_to_string(dir.join("document.tw"))
        .map_err(|erreur| format!("La source du projet ne se lit pas : {erreur}"))?;
    Ok(crate::langage::generer::generer(&crate::langage::parse(
        &source,
    )))
}

/// Exporte les projets sélectionnés dans un dossier, un fichier par projet.
/// Un échec n'arrête pas les autres : le compte rendu nomme les deux.
fn exporter_selection(
    dossiers: &[PathBuf],
    destination: &Path,
    format: FormatExport,
) -> Result<String, String> {
    let mut ecrits = 0;
    let mut erreurs = Vec::new();

    for dir in dossiers {
        let nom = dir
            .file_name()
            .map(|nom| nom.to_string_lossy().to_string())
            .unwrap_or_else(|| "projet".to_string());

        let resultat = match format {
            FormatExport::Pdf => assemblage_du_projet(dir).and_then(|source| {
                let bytes = crate::typst::render::render_pdf(dir, &source, &[])
                    .ok_or_else(|| "la compilation a échoué".to_string())?;
                let chemin = chemin_libre(destination, &nom, "pdf");
                crate::services::stockage::ecrire_atomique_octets(&chemin, &bytes)
                    .map(|_| ())
                    .map_err(|erreur| format!("écriture du PDF : {erreur}"))
            }),
            FormatExport::Tw => crate::project::portable::exporter_tw(dir).and_then(|texte| {
                let chemin = chemin_libre(destination, &nom, "tw");
                crate::services::stockage::ecrire_atomique(&chemin, &texte)
                    .map(|_| ())
                    .map_err(|erreur| format!("écriture du .tw : {erreur}"))
            }),
            FormatExport::Markdown => source_du_projet(dir).and_then(|source| {
                let chemin = chemin_libre(destination, &nom, "md");
                let texte = crate::langage::export::vers_markdown(&source);
                crate::services::stockage::ecrire_atomique(&chemin, &texte)
                    .map(|_| ())
                    .map_err(|erreur| format!("écriture du Markdown : {erreur}"))
            }),
            FormatExport::Docx => source_du_projet(dir).and_then(|source| {
                let chemin = chemin_libre(destination, &nom, "docx");
                crate::langage::export::vers_docx(&source, &chemin)
            }),
        };

        match resultat {
            Ok(()) => ecrits += 1,
            Err(erreur) => erreurs.push(format!("{nom} : {erreur}")),
        }
    }

    if erreurs.is_empty() {
        Ok(format!(
            "{ecrits} document(s) exporté(s) dans {}.",
            destination.display()
        ))
    } else {
        Err(format!(
            "{ecrits} document(s) exporté(s) ; échec(s) : {}.",
            erreurs.join(" ; ")
        ))
    }
}

/// Fusionne des couvertures arrivées dans `state.vignettes`, sans jeter
/// celles d'un projet qui n'a pas encore de cache : la bibliothèque garde
/// l'ancienne image jusqu'à la nouvelle, jamais un cadre vide entre les deux.
fn fusionner_vignettes(state: &mut State, vignettes: Vec<(String, Vec<u8>, u32, u32)>) {
    state
        .vignettes
        .retain(|(nom, ..)| state.projects.iter().any(|projet| projet.name == *nom));

    for (nom, pixels, largeur, hauteur) in vignettes {
        if let Some(entree) = state.vignettes.iter_mut().find(|(cle, ..)| *cle == nom) {
            *entree = (nom, pixels, largeur, hauteur);
        } else {
            state.vignettes.push((nom, pixels, largeur, hauteur));
        }
    }
}

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ProjetsVue => {
            // La bibliothèque et les paramètres sont deux pages de même
            // rang : ouvrir l'une referme l'autre.
            if state.parametres_vue {
                state.parametres_vue = false;
            }
            state.projets_vue = !state.projets_vue;

            if state.projets_vue {
                let dirs: Vec<std::path::PathBuf> = state
                    .projects
                    .iter()
                    .map(|projet| projet.dir.clone())
                    .collect();

                // **Tout de suite** : la couverture choisie ou la
                // première page déjà rendue. Aucune compilation ne retarde
                // l'ouverture de la bibliothèque.
                let immediates = crate::project::vignettes_immediates(&dirs);
                fusionner_vignettes(state, immediates);

                // Le projet qu'on est en train d'écrire n'est pas recompilé
                // pour une vignette : sa couverture en cache reste affichée.
                // C'est le travail lourd qui, sinon, continuait après le
                // retour à l'écriture.
                let a_rafraichir: Vec<std::path::PathBuf> = dirs
                    .into_iter()
                    .filter(|dir| dir != &state.project.dir)
                    .collect();
                let generation = state.vignettes_generation.clone();
                let depart = generation
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    .wrapping_add(1);

                // Ensuite seulement, les premières pages changées se
                // rendent en arrière-plan, et la boucle s'arrête si la
                // bibliothèque est quittée.
                return Task::perform(
                    async move {
                        crate::project::vignettes_a_rafraichir_avec_annulation(
                            &a_rafraichir,
                            Some((generation.as_ref(), depart)),
                        )
                    },
                    Message::VignettesRafraichies,
                );
            }

            // Quitter la bibliothèque efface la sélection : elle ne
            // survivra pas à un geste d'écriture.
            state
                .vignettes_generation
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            state.selection_projets.clear();
            state.dernier_projet_selectionne = None;

            Task::none()
        }

        Message::VignettesRafraichies(vignettes) => {
            if state.projets_vue {
                fusionner_vignettes(state, vignettes);
            }
            Task::none()
        }

        Message::ProjetCliquer(dir) => {
            if state.controle_tenu {
                if !state.selection_projets.remove(&dir) {
                    state.selection_projets.insert(dir.clone());
                }
                state.dernier_projet_selectionne = Some(dir);
                return Task::none();
            }

            if state.maj_tenu {
                let ordre = ordre_librairie(state);
                let ancre = state.dernier_projet_selectionne.clone();
                state.selection_projets.clear();
                if let Some(ancre) = ancre {
                    state
                        .selection_projets
                        .extend(plage_selection(&ordre, &ancre, &dir));
                } else {
                    state.selection_projets.insert(dir.clone());
                }
                state.dernier_projet_selectionne = Some(dir);
                return Task::none();
            }

            state.selection_projets.clear();
            state.dernier_projet_selectionne = None;
            crate::app::update(state, Message::OpenProject(dir))
        }

        Message::SelectionVider => {
            state.selection_projets.clear();
            state.dernier_projet_selectionne = None;
            Task::none()
        }

        Message::EtagereCreer => {
            let nom = state.etagere_brouillon.trim().to_string();
            match crate::project::creer_etagere(&nom) {
                Ok(()) => {
                    state.etageres_ordre = crate::project::etageres_ordonnees();
                    state.etagere_brouillon.clear();
                    state.note = Some(crate::app::state::Note::info(format!(
                        "L'étagère « {nom} » est créée."
                    )));
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }
            Task::none()
        }

        Message::EtagereMonter(nom) => {
            match crate::project::deplacer_etagere(&nom, true) {
                Ok(()) => state.etageres_ordre = crate::project::etageres_ordonnees(),
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }
            Task::none()
        }

        Message::EtagereDescendre(nom) => {
            match crate::project::deplacer_etagere(&nom, false) {
                Ok(()) => state.etageres_ordre = crate::project::etageres_ordonnees(),
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }
            Task::none()
        }

        Message::SelectionNormes => {
            let cibles = selection_dossiers(state);
            if cibles.is_empty() {
                state.note = Some(crate::app::state::Note::info(
                    "Sélectionnez au moins un document.".to_string(),
                ));
                return Task::none();
            }
            let norme =
                crate::project::norme_qui_correspond(&crate::mise_en_page::charger(&cibles[0]))
                    .or_else(|| crate::project::noms_de_normes().into_iter().next())
                    .unwrap_or_default();
            state.normes_lot = Some(crate::app::state::NormesLot {
                cibles,
                norme,
                accrocher: true,
            });
            Task::none()
        }

        Message::NormesLotChoisir(norme) => {
            if let Some(lot) = state.normes_lot.as_mut() {
                lot.norme = norme;
            }
            Task::none()
        }

        Message::NormesLotAccrocher(accrocher) => {
            if let Some(lot) = state.normes_lot.as_mut() {
                lot.accrocher = accrocher;
            }
            Task::none()
        }

        Message::NormesLotFermer => {
            state.normes_lot = None;
            Task::none()
        }

        Message::NormesLotAppliquer => {
            let Some(lot) = state.normes_lot.take() else {
                return Task::none();
            };
            let Some(reglages) = crate::project::norme(&lot.norme) else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Choisissez une norme avant d'appliquer.".to_string(),
                ));
                state.normes_lot = Some(lot);
                return Task::none();
            };

            let mut appliquees = 0;
            let mut erreurs = Vec::new();
            for dir in &lot.cibles {
                let temporaire = if state.projet_verrouille(dir) {
                    None
                } else {
                    match crate::project::Verrou::prendre(dir) {
                        Ok(verrou) => Some(verrou),
                        Err(probleme) => {
                            erreurs.push(format!(
                                "{} : {probleme}",
                                dir.file_name().unwrap_or_default().to_string_lossy()
                            ));
                            continue;
                        }
                    }
                };

                match crate::mise_en_page::appliquer(dir, &reglages, lot.accrocher) {
                    Ok(_) => {
                        appliquees += 1;
                        if *dir == state.project.dir {
                            state.reglages = reglages.clone();
                            state.norme_document = lot.norme.clone();
                            state.render.pending = true;
                        }
                    }
                    Err(probleme) => erreurs.push(format!(
                        "{} : {probleme}",
                        dir.file_name().unwrap_or_default().to_string_lossy()
                    )),
                }
                drop(temporaire);
            }

            if erreurs.is_empty() {
                state.note = Some(crate::app::state::Note::info(format!(
                    "Normes « {} » appliquées à {appliquees} document(s).",
                    lot.norme
                )));
            } else {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "{appliquees} document(s) mis à jour ; échec(s) : {}.",
                    erreurs.join(" ; ")
                )));
            }
            Task::none()
        }

        Message::SelectionExporter(format) => {
            let dossiers = selection_dossiers(state);
            if dossiers.is_empty() {
                state.note = Some(crate::app::state::Note::info(
                    "Sélectionnez au moins un document.".to_string(),
                ));
                return Task::none();
            }

            Task::future(async move {
                let Some(destination) = rfd::AsyncFileDialog::new()
                    .set_title("Choisir le dossier d'export")
                    .pick_folder()
                    .await
                else {
                    return Message::SelectionExportee(Ok("Export annulé".to_string()));
                };
                let resultat = exporter_selection(&dossiers, destination.path(), format);
                Message::SelectionExportee(resultat)
            })
        }

        Message::SelectionExportee(resultat) => {
            state.note = Some(match resultat {
                Ok(message) => crate::app::state::Note::info(message),
                Err(probleme) => crate::app::state::Note::erreur(probleme),
            });
            Task::none()
        }

        // La miniature d'un projet : le sélecteur natif lit le JPG ou le
        // PNG, le recadre au format A4 et le pose dans le projet.
        Message::MiniatureChoisir(dir) => Task::future(async move {
            let Some(fichier) = rfd::AsyncFileDialog::new()
                .set_title("Choisir une miniature")
                .add_filter("Images", &["png", "jpg", "jpeg"])
                .pick_file()
                .await
            else {
                return Message::MiniaturePrete(dir, Ok(None));
            };

            let chemin = fichier.path().to_path_buf();
            let resultat = std::fs::read(&chemin)
                .map_err(|erreur| format!("Cette image ne peut pas être lue : {erreur}."))
                .and_then(|octets| crate::project::preparer_couverture(&octets))
                .map(Some);

            Message::MiniaturePrete(dir, resultat)
        }),

        Message::MiniaturePrete(dir, resultat) => {
            let couverture = match resultat {
                Ok(Some(couverture)) => couverture,
                Ok(None) => return Task::none(),
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    return Task::none();
                }
            };

            if !state.projects.iter().any(|projet| projet.dir == dir) {
                state.note = Some(crate::app::state::Note::erreur(
                    "Ce projet n'est plus dans la bibliothèque.".to_string(),
                ));
                return Task::none();
            }

            // L'écriture d'une couverture appartient au projet : si une
            // autre instance le tient, on ne l'écrase pas. Quand ce n'est
            // pas le projet courant, un verrou court suffit.
            let temporaire = if state.projet_verrouille(&dir) {
                None
            } else {
                match crate::project::Verrou::prendre(&dir) {
                    Ok(verrou) => Some(verrou),
                    Err(probleme) => {
                        state.note = Some(crate::app::state::Note::erreur(probleme));
                        return Task::none();
                    }
                }
            };

            let ecriture = crate::project::ecrire_couverture(&dir, &couverture.png);
            drop(temporaire);

            match ecriture {
                Ok(()) => {
                    let nom = state
                        .projects
                        .iter()
                        .find(|projet| projet.dir == dir)
                        .map(|projet| projet.name.clone())
                        .unwrap_or_else(|| {
                            dir.file_name()
                                .map(|nom| nom.to_string_lossy().to_string())
                                .unwrap_or_default()
                        });
                    fusionner_vignettes(
                        state,
                        vec![(
                            nom,
                            couverture.pixels,
                            couverture.largeur,
                            couverture.hauteur,
                        )],
                    );
                    state.note = Some(crate::app::state::Note::info(
                        "La miniature du projet est mise à jour.".to_string(),
                    ));
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }

            Task::none()
        }

        // Le glisser-déposer : prendre un projet, survoler une étagère,
        // poser. Le geste dit où il en est — une ombre suit le projet.
        Message::DragCommencer(nom) => {
            // Ctrl/Maj appartiennent à la sélection, pas au déplacement :
            // un clic de sélection ne doit pas armer un glisser-déposer.
            if state.controle_tenu || state.maj_tenu {
                return Task::none();
            }
            state.drag_projet = Some(nom);
            Task::none()
        }

        // Les deux recherches ne se montrent que sur leur icône, et
        // l'étagère se choisit au menu (bug-report 2026-11-03).
        Message::ProjetsRechercheBascule => {
            state.projets_recherche_visible = !state.projets_recherche_visible;
            Task::none()
        }

        Message::ProjetsEtageChoisie(choix) => {
            state.projets_etage = choix;
            Task::none()
        }

        // La liste des projets : étagère courante par défaut, tous sur
        // demande — et la recherche filtre par nom (bug-report-7, 18).
        Message::ProjetsFiltreBascule => {
            state.projets_tous = !state.projets_tous;
            Task::none()
        }

        Message::ProjetsRecherche(valeur) => {
            state.projets_recherche = valeur;
            Task::none()
        }

        Message::DragSurvol(etagere) => {
            state.drag_etagere = Some(etagere);
            Task::none()
        }

        Message::DragPoser(etagere) => {
            if let Some(projet) = state.drag_projet.take() {
                state.drag_etagere = None;

                match crate::project::definir_etagere(&projet, &etagere) {
                    Ok(()) => {
                        state.etageres = crate::project::etageres();
                        state.etageres_ordre = crate::project::etageres_ordonnees();
                        state.note = Some(crate::app::state::Note::info(format!(
                            "« {projet} » rejoint « {etagere} »."
                        )));
                    }
                    Err(message) => {
                        state.note = Some(crate::app::state::Note::erreur(message));
                    }
                }
            }

            Task::none()
        }

        Message::EtagereBrouillon(valeur) => {
            state.etagere_brouillon = valeur;
            Task::none()
        }

        Message::EtagereDefinir(projet, etagere) => {
            match crate::project::definir_etagere(&projet, &etagere) {
                Ok(()) => {
                    state.etageres = crate::project::etageres();
                    state.etageres_ordre = crate::project::etageres_ordonnees();
                    state.note = Some(crate::app::state::Note::info(format!(
                        "« {projet} » rejoint « {etagere} »."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // La suppression d'un projet, et la bascule vers un autre.
        Message::ProjetSupprimer(dir) => {
            if !state.geste_confirme {
                state.confirmation = Some(crate::app::state::Confirmation {
                    question: "Supprimer ce projet ? Il part à la corbeille.".to_string(),
                    action: Message::ProjetSupprimer(dir),
                });

                return Task::none();
            }

            state.geste_confirme = false;

            // La carte peut viser un projet autre que la vue courante. Un
            // verrou propre à cette cible reste tenu jusqu'au déplacement.
            let deja_verrouille = state.espace.verrous.contains_key(&dir)
                || (state.project.dir == dir && state.verrou.is_some());
            let mut temporaire = if deja_verrouille {
                None
            } else {
                match crate::project::Verrou::prendre(&dir) {
                    Ok(verrou) => Some(verrou),
                    Err(probleme) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "Suppression refusée : {probleme}"
                        )));
                        return Task::none();
                    }
                }
            };

            if let Err(probleme) = state.sauvegarder_avant_deplacer(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Corrigez la sauvegarde et réessayez."
                )));
                return Task::none();
            }

            if let Err(probleme) = state.preparer_suppression_projet(&dir) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Suppression annulée : {probleme}. Le projet reste en place."
                )));
                return Task::none();
            }

            match crate::project::supprimer(&dir) {
                Ok(corbeille) => {
                    if let Some(mut verrou) = temporaire.take() {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if let Some(mut verrou) = state.espace.verrous.remove(&dir) {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    if state.project.dir == dir
                        && let Some(mut verrou) = state.verrou.take()
                    {
                        verrou.suivre_deplacement(&corbeille);
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Le projet part à la corbeille : {}",
                        corbeille.display()
                    )));

                    state.projects.retain(|projet| projet.dir != dir);
                    state.selection_projets.remove(&dir);
                    if state.dernier_projet_selectionne.as_ref() == Some(&dir) {
                        state.dernier_projet_selectionne = None;
                    }
                    state.retirer_projet_supprime(&dir);
                    Task::none()
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    Task::none()
                }
            }
        }

        _ => Task::none(),
    }
}

#[cfg(test)]
mod tests {
    use super::{chemin_libre, plage_selection};
    use std::path::PathBuf;

    fn chemin(nom: &str) -> PathBuf {
        PathBuf::from(format!("/bibliotheque/{nom}"))
    }

    /// Maj+clic prend la plage dans l'ordre de la bibliothèque, pas dans
    /// l'ordre des dossiers ni dans celui de la sélection.
    #[test]
    fn maj_clic_prend_la_plage_dans_l_ordre_affiche() {
        let ordre = vec![
            chemin("un"),
            chemin("deux"),
            chemin("trois"),
            chemin("quatre"),
        ];

        let plage = plage_selection(&ordre, &chemin("deux"), &chemin("quatre"));
        assert_eq!(
            plage,
            vec![chemin("deux"), chemin("trois"), chemin("quatre")]
        );

        let retour = plage_selection(&ordre, &chemin("quatre"), &chemin("deux"));
        assert_eq!(retour, plage, "la plage se lit dans les deux sens");
    }

    #[test]
    fn une_ancre_disparue_ne_selectionne_que_la_cible() {
        let ordre = vec![chemin("un"), chemin("deux")];
        assert_eq!(
            plage_selection(&ordre, &chemin("absent"), &chemin("deux")),
            vec![chemin("deux")]
        );
    }

    #[test]
    fn un_export_necrase_jamais_un_fichier_existant() {
        let dossier =
            std::env::temp_dir().join(format!("tw-export-selection-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).expect("dossier");

        std::fs::write(dossier.join("projet.pdf"), b"deja la").expect("fichier");
        assert_eq!(
            chemin_libre(&dossier, "projet", "pdf"),
            dossier.join("projet-2.pdf")
        );

        let _ = std::fs::remove_dir_all(&dossier);
    }

    /// Chaque projet sélectionné produit son fichier, et deux projets ne se
    /// marchent pas dessus : le nom du dossier les distingue.
    #[test]
    fn une_selection_exporte_un_fichier_par_document() {
        let racine = std::env::temp_dir().join(format!("tw-export-lot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&racine);
        let premier = racine.join("premier");
        let second = racine.join("second");
        let destination = racine.join("sortie");
        for dir in [&premier, &second] {
            std::fs::create_dir_all(dir).expect("projet");
            std::fs::write(dir.join("main.typ"), "= Un titre\n\nDu texte.\n").expect("main");
            std::fs::write(dir.join("document.tw"), "# Un titre\n\nDu texte.\n").expect("document");
        }
        std::fs::create_dir_all(&destination).expect("sortie");

        let compte = super::exporter_selection(
            &[premier.clone(), second.clone()],
            &destination,
            super::FormatExport::Markdown,
        )
        .expect("export");
        assert!(compte.contains("2 document"), "compte rendu : {compte}");
        assert!(destination.join("premier.md").is_file());
        assert!(destination.join("second.md").is_file());

        let _ = std::fs::remove_dir_all(&racine);
    }
}
