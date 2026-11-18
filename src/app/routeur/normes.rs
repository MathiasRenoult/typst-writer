//! Les normes : appliquer, écraser, enregistrer, supprimer.

use crate::app::{Message, State};
use iced::Task;

/// Traite les messages de ce domaine. La porte d'activation et la garde
/// du verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NormesNom(nom) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.norme = nom;
            }

            Task::none()
        }

        Message::NormesAppliquer(nom) => {
            match crate::project::norme(&nom) {
                Some(mise) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = mise;
                        dialogue.norme = nom.clone();
                        dialogue.norme_originale = nom.clone();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est chargée dans la fenêtre — « Appliquer » pour l'écrire."
                    )));
                }
                None => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "La norme « {nom} » n'existe pas."
                    )));
                }
            }

            Task::none()
        }

        // Écraser la norme chargée : le premier temps de l'enregistrement,
        // quand le document en porte une (bug-report-9).
        Message::NormesEcraser => {
            let Some((nom, reglages)) = state
                .mise_en_page
                .as_ref()
                .filter(|dialogue| !dialogue.norme_originale.trim().is_empty())
                .map(|dialogue| {
                    (
                        dialogue.norme_originale.trim().to_string(),
                        dialogue.reglages.clone(),
                    )
                })
            else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Aucune norme chargée à écraser.".to_string(),
                ));
                return Task::none();
            };

            match crate::project::enregistrer_norme(&nom, &reglages) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est remplacée."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesEnregistrer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(
                    "Une copie a un nom.".to_string(),
                ));
                return Task::none();
            }

            if crate::project::norme(nom).is_some() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "« {nom} » existe déjà — donnez un autre nom à la copie, ou écrasez la norme chargée."
                )));
                return Task::none();
            }

            match crate::project::enregistrer_norme(
                nom,
                &state
                    .mise_en_page
                    .as_ref()
                    .map(|d| d.reglages.clone())
                    .unwrap_or_default(),
            ) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.norme = nom.to_string();
                        dialogue.norme_originale = nom.to_string();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La copie « {nom} » est dans la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        Message::NormesSupprimerDemande(nom) => {
            let nom = nom.trim().to_string();
            if nom.is_empty() {
                return Task::none();
            }
            state.confirmation = Some(crate::app::state::Confirmation {
                question: format!(
                    "Supprimer la norme « {nom} » de la bibliothèque ? Les documents qui \
                     l'utilisent gardent leur propre fichier de normes ; seule l'entrée de la \
                     bibliothèque part."
                ),
                action: Message::NormesSupprimer(nom),
            });
            Task::none()
        }

        Message::NormesSupprimer(nom) => {
            let nom = nom.trim();

            if nom.is_empty() {
                return Task::none();
            }

            match crate::project::supprimer_norme(nom) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut()
                        && dialogue.norme_originale.trim() == nom
                    {
                        // La norme n'existe plus : le document redevient un
                        // porteur de normes personnalisées.
                        dialogue.norme_originale.clear();
                        dialogue.norme.clear();
                        dialogue.modifie = true;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » a quitté la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // Appliquer une norme directement au document : c'est le geste du
        // panneau de gestion des fichiers, sans passer par la copie de
        // travail. Le fichier de normes du projet, son gabarit et l'état des
        // plugins suivent dans le même mouvement.
        Message::NormesAppliquerAuDocument(nom) => {
            let Some(norme) = crate::project::norme(&nom) else {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "La norme « {nom} » n'existe pas."
                )));
                return Task::none();
            };
            let problemes = norme.problemes();
            if !problemes.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "La norme « {nom} » ne s'applique pas : {}",
                    problemes
                        .iter()
                        .map(|(champ, probleme)| format!("{champ} : {probleme}"))
                        .collect::<Vec<_>>()
                        .join(" ; ")
                )));
                return Task::none();
            }

            let accrocher = crate::mise_en_page::accroche_manquante(&state.project.dir);
            match crate::mise_en_page::appliquer(&state.project.dir, &norme, accrocher) {
                Ok(_) => {
                    state.reglages = norme.clone();
                    state.configurer_les_plugins_selon_les_normes();
                    state.render.pending = true;
                    state.rafraichir_statistiques_bibliques();
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = norme;
                        dialogue.norme = nom.clone();
                        dialogue.norme_originale = nom.clone();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est appliquée au document."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        // Exporter une entrée de la bibliothèque dans un fichier `.toml`.
        Message::NormesExporter(nom) => {
            let Some(norme) = crate::project::norme(&nom) else {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "La norme « {nom} » n'existe pas."
                )));
                return Task::none();
            };
            let texte = match norme.ecrire() {
                Ok(texte) => texte,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Export impossible : {erreur}"
                    )));
                    return Task::none();
                }
            };
            let suggere = crate::services::stockage::slug(&nom);

            Task::future(async move {
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Fichier de normes", &["toml"])
                    .set_file_name(format!("{suggere}.toml"))
                    .save_file()
                    .await
                else {
                    return Message::MiseEnPageNote("Export annulé".to_string());
                };
                let chemin = chemin_libre_pour_export(file.path());
                match crate::services::stockage::ecrire_atomique(&chemin, &texte) {
                    Ok(()) => Message::MiseEnPageNote(format!(
                        "Norme exportée dans « {} »",
                        chemin.display()
                    )),
                    Err(erreur) => Message::MiseEnPageNote(format!("Export impossible : {erreur}")),
                }
            })
        }

        // Importer un fichier dans la bibliothèque interne. Le nom du
        // fichier devient le nom de la norme, suffixé si ce nom est déjà pris.
        Message::NormesFichierImporter => Task::future(async {
            let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Fichier de normes", &["toml"])
                .pick_file()
                .await
            else {
                return Message::MiseEnPageNote("Import annulé".to_string());
            };
            Message::NormesFichierImporte(Some(file.path().to_path_buf()))
        }),

        Message::NormesFichierImporte(chemin) => {
            let Some(chemin) = chemin else {
                return Task::none();
            };
            let texte = match std::fs::read_to_string(&chemin) {
                Ok(texte) => texte,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Lire le fichier de normes : {erreur}"
                    )));
                    return Task::none();
                }
            };
            let norme = match crate::mise_en_page::modele::MiseEnPage::lire(&texte) {
                Ok(norme) => norme,
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Fichier de normes illisible : {erreur}"
                    )));
                    return Task::none();
                }
            };
            let problemes = norme.problemes();
            if !problemes.is_empty() {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Fichier de normes invalide : {}",
                    problemes
                        .iter()
                        .map(|(champ, probleme)| format!("{champ} : {probleme}"))
                        .collect::<Vec<_>>()
                        .join(" ; ")
                )));
                return Task::none();
            }
            let base = chemin
                .file_stem()
                .and_then(|nom| nom.to_str())
                .unwrap_or("Norme importée");
            let nom = nom_de_norme_libre(base);
            match crate::project::enregistrer_norme(&nom, &norme) {
                Ok(()) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = norme;
                        dialogue.norme = nom.clone();
                        dialogue.norme_originale = nom.clone();
                        dialogue.modifie = false;
                    }
                    state.note = Some(crate::app::state::Note::info(format!(
                        "La norme « {nom} » est entrée dans la bibliothèque."
                    )));
                }
                Err(message) => {
                    state.note = Some(crate::app::state::Note::erreur(message));
                }
            }

            Task::none()
        }

        _ => Task::none(),
    }
}

/// Un nom de fichier libre, sans jamais écraser un export existant.
fn chemin_libre_pour_export(chemin: &std::path::Path) -> std::path::PathBuf {
    if !chemin.exists() {
        return chemin.to_path_buf();
    }
    let parent = chemin.parent().unwrap_or_else(|| std::path::Path::new("."));
    let nom = chemin
        .file_stem()
        .and_then(|nom| nom.to_str())
        .unwrap_or("norme");
    let extension = chemin
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("toml");
    for numero in 2.. {
        let candidat = parent.join(format!("{nom}-{numero}.{extension}"));
        if !candidat.exists() {
            return candidat;
        }
    }
    chemin.to_path_buf()
}

/// Le nom d'une norme importée, distinct des noms déjà présents.
fn nom_de_norme_libre(base: &str) -> String {
    let base = base.trim();
    let base = if base.is_empty() {
        "Norme importée"
    } else {
        base
    };
    let noms = crate::project::noms_de_normes();
    if !noms.iter().any(|nom| nom == base) {
        return base.to_string();
    }
    for numero in 2.. {
        let candidat = format!("{base}-{numero}");
        if !noms.iter().any(|nom| nom == &candidat) {
            return candidat;
        }
    }
    base.to_string()
}

#[cfg(test)]
mod tests {
    use super::chemin_libre_pour_export;

    /// Un export ne remplace jamais un fichier déjà présent : il prend le
    /// premier nom libre à côté.
    #[test]
    fn un_export_de_norme_necrase_jamais() {
        let dossier =
            std::env::temp_dir().join(format!("typst-writer-normes-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).expect("dossier de test");
        let choisi = dossier.join("norme.toml");
        std::fs::write(&choisi, "ancien").expect("fichier existant");

        let libre = chemin_libre_pour_export(&choisi);

        assert_eq!(libre, dossier.join("norme-2.toml"));
        assert_eq!(std::fs::read_to_string(&choisi).unwrap(), "ancien");
        let _ = std::fs::remove_dir_all(&dossier);
    }
}
