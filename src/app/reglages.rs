//! Les réglages du document : la fenêtre de mise en page, et ce qu'elle fait.
//!
//! Le dialogue montre une **copie de travail** : le projet n'est touché qu'à
//! l'application, et fermer la fenêtre ne change rien à personne. L'import et
//! l'export parlent de fichiers TOML — la mise en page est un objet qu'on se
//! prête, pas un réglage qu'on reconstitue à la main.

use crate::app::message::Message;
use crate::app::state::{DialogueMiseEnPage, State};
use crate::mise_en_page;
use iced::Task;

/// Traite les messages de mise en page. L'appelant a déjà vérifié le motif.
pub(super) fn update(state: &mut State, message: Message) -> Option<Task<Message>> {
    match message {
        Message::MiseEnPageOuvrir => {
            let reglages = mise_en_page::charger(&state.project.dir);
            let accroche_manquante = mise_en_page::accroche_manquante(&state.project.dir);
            let apercu_style = crate::typst::styles::apercu_du_style(
                &state.project.dir,
                reglages
                    .bibliographie
                    .style
                    .as_deref()
                    .unwrap_or("chicago-notes"),
            );
            let styles_zotero = crate::zotero::data_dir()
                .map(|dir| crate::zotero::styles::installed(&dir.join("styles")))
                .unwrap_or_default();
            let apercu_titres = crate::mise_en_page::apercu_titres(&state.project.dir, &reglages);

            state.mise_en_page = Some(DialogueMiseEnPage {
                norme: String::new(),
                norme_originale: String::new(),
                modifie: false,
                reglages,
                panneau: 0,
                accrocher: accroche_manquante,
                accroche_manquante,
                regles_main: mise_en_page::regles_ecrites_a_la_main(&state.project.dir),
                apercu_style,
                apercu_titres,
                styles_zotero,
            });

            Task::none()
        }

        Message::MiseEnPageFermer => {
            state.mise_en_page = None;

            Task::none()
        }

        Message::MiseEnPageChamp(champ, valeur) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let concerne_le_style = matches!(
                    champ,
                    crate::mise_en_page::modele::Champ::BibliographieStyle
                );

                dialogue.reglages.modifier(champ, valeur);
                dialogue.modifie = true;

                // Le style change : l'aperçu se refait par le moteur même.
                if concerne_le_style {
                    let dir = state.project.dir.clone();
                    let style = dialogue
                        .reglages
                        .bibliographie
                        .style
                        .clone()
                        .unwrap_or_else(|| "chicago-notes".to_string());
                    dialogue.apercu_style = crate::typst::styles::apercu_du_style(&dir, &style);
                }

                // Les titres suivent : leur aperçu est un vrai rendu.
                let dir = state.project.dir.clone();
                dialogue.apercu_titres =
                    crate::mise_en_page::apercu_titres(&dir, &dialogue.reglages);
            }

            Task::none()
        }

        Message::MiseEnPageMolette(champ, pas) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                let actuel = dialogue.reglages.valeur(champ);
                let nouveau = incrémenter(&actuel, pas);
                dialogue.reglages.modifier(champ, nouveau);
                dialogue.modifie = true;

                let dir = state.project.dir.clone();
                dialogue.apercu_titres =
                    crate::mise_en_page::apercu_titres(&dir, &dialogue.reglages);
            }

            Task::none()
        }

        Message::MiseEnPageBascule(champ) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.basculer(champ);

                let dir = state.project.dir.clone();
                dialogue.apercu_titres =
                    crate::mise_en_page::apercu_titres(&dir, &dialogue.reglages);
            }

            Task::none()
        }

        Message::MiseEnPageAccrocher(accrocher) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.accrocher = accrocher;
            }

            Task::none()
        }

        Message::MiseEnPagePanneau(panneau) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.panneau = panneau;
            }

            Task::none()
        }

        Message::MiseEnPageTypographieDefaut(defaut) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.typographie.charger_le_defaut(&defaut);
            }

            Task::none()
        }

        Message::MiseEnPageAppliquer => appliquer(state),

        Message::MiseEnPageImporter => Task::future(async {
            let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Mise en page", &["toml"])
                .pick_file()
                .await
            else {
                return Message::MiseEnPageNote("Import annulé".to_string());
            };

            match std::fs::read_to_string(file.path()) {
                Ok(texte) => Message::MiseEnPageImportee(texte),
                Err(erreur) => Message::MiseEnPageNote(format!("Lire le fichier : {erreur}")),
            }
        }),

        Message::MiseEnPageImportee(texte) => {
            match mise_en_page::modele::MiseEnPage::lire(&texte) {
                Ok(reglages) => {
                    if let Some(dialogue) = state.mise_en_page.as_mut() {
                        dialogue.reglages = reglages;
                    }

                    state.export_status = Some("Mise en page importée".to_string());
                }

                Err(erreur) => {
                    state.export_status = Some(format!("Fichier illisible : {erreur}"));
                }
            }

            Task::none()
        }

        Message::MiseEnPageExporter => {
            let texte = match state
                .mise_en_page
                .as_ref()
                .map(|dialogue| dialogue.reglages.ecrire())
            {
                Some(Ok(texte)) => texte,
                Some(Err(erreur)) => {
                    return update(
                        state,
                        Message::MiseEnPageNote(format!("Export impossible : {erreur}")),
                    );
                }
                None => return None,
            };

            Task::future(async move {
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Mise en page", &["toml"])
                    .set_file_name("mise-en-page.toml")
                    .save_file()
                    .await
                else {
                    return Message::MiseEnPageNote("Export annulé".to_string());
                };

                Message::MiseEnPageDestination(file.path().to_path_buf(), texte)
            })
        }

        Message::MiseEnPageDestination(chemin, texte) => {
            let resultat = crate::app::fichiers::verifier_destination_export(state, &chemin)
                .and_then(|()| {
                    crate::project::ecrire_atomique(&chemin, &texte)
                        .map_err(|erreur| format!("Écrire le fichier : {erreur}"))
                });
            let note = match resultat {
                Ok(()) => format!("Exporté : {}", chemin.display()),
                Err(erreur) => erreur,
            };
            return update(state, Message::MiseEnPageNote(note));
        }

        Message::MiseEnPageNote(note) => {
            state.export_status = Some(note);

            Task::none()
        }

        _ => return None,
    }
    .into()
}

/// Applique la copie de travail au projet.
///
/// Ce qui ne va pas n'est **jamais** écrit : les réglages fautifs se disent,
/// champ par champ, et la fenêtre reste ouverte pour les corriger.
fn appliquer(state: &mut State) -> Task<Message> {
    let Some(dialogue) = state.mise_en_page.as_ref() else {
        return Task::none();
    };

    let problemes = dialogue.reglages.problemes();

    if !problemes.is_empty() {
        state.export_status = Some(format!(
            "{} réglage(s) à corriger dans la fenêtre",
            problemes.len()
        ));

        return Task::none();
    }

    let reglages = dialogue.reglages.clone();
    let accrocher = dialogue.accrocher;

    match mise_en_page::appliquer(&state.project.dir, &reglages, accrocher) {
        Ok(mise_en_page::Applique::AccrocheAjoutee) => {
            state.mise_en_page = None;
            state.export_status =
                Some("Mise en page appliquée — main.typ l'accueille maintenant".to_string());
        }

        Ok(mise_en_page::Applique::RienAjoute) => {
            state.mise_en_page = None;
            state.export_status = Some("Mise en page appliquée".to_string());
        }

        Err(erreur) => {
            state.export_status = Some(erreur);
        }
    }

    Task::none()
}

/// Incrémente la valeur numérique d'un champ — la molette et les flèches.
///
/// `11pt` → `12pt`, `2cm` → `3cm`, `1.5` → `2.5`. L'unité est préservée.
fn incrémenter(valeur: &str, pas: i8) -> String {
    // Séparer le nombre et le suffixe (unité).
    let fin_nombre = valeur
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != ',')
        .unwrap_or(valeur.len());
    let (nombre, suffixe) = valeur.split_at(fin_nombre);
    let nombre_normalise = nombre.replace(',', ".");
    let Ok(n) = nombre_normalise.parse::<f64>() else {
        return valeur.to_string();
    };
    let nouveau = n + pas as f64;
    // Pas de décimale inutile : 2.0 → 2, 2.5 → 2.5.
    if nouveau.fract() == 0.0 {
        format!("{}{suffixe}", nouveau as i64)
    } else {
        format!("{nouveau}{suffixe}")
    }
}
