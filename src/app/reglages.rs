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

            state.mise_en_page = Some(DialogueMiseEnPage {
                reglages,
                accrocher: accroche_manquante,
                accroche_manquante,
                regles_main: mise_en_page::regles_ecrites_a_la_main(&state.project.dir),
            });

            Task::none()
        }

        Message::MiseEnPageFermer => {
            state.mise_en_page = None;

            Task::none()
        }

        Message::MiseEnPageChamp(champ, valeur) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.modifier(champ, valeur);
            }

            Task::none()
        }

        Message::MiseEnPageBascule(champ) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.basculer(champ);
            }

            Task::none()
        }

        Message::MiseEnPageAccrocher(accrocher) => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.accrocher = accrocher;
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
            let texte = state
                .mise_en_page
                .as_ref()
                .map(|dialogue| dialogue.reglages.ecrire())
                .unwrap_or_default();

            Task::future(async move {
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Mise en page", &["toml"])
                    .set_file_name("mise-en-page.toml")
                    .save_file()
                    .await
                else {
                    return Message::MiseEnPageNote("Export annulé".to_string());
                };

                match std::fs::write(file.path(), texte) {
                    Ok(()) => {
                        Message::MiseEnPageNote(format!("Exporté : {}", file.path().display()))
                    }
                    Err(erreur) => Message::MiseEnPageNote(format!("Écrire le fichier : {erreur}")),
                }
            })
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
