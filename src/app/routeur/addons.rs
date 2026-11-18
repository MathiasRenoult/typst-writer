//! L'activation des add-ons depuis les normes.

use crate::app::{Message, State};
use iced::Task;

/// Applique une bascule d'add-on. La porte d'activation et la garde du
/// verrou sont déjà passées ; une clé inconnue est ignorée par `basculer`.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ReglagesFonctionnalite(nom, actif) => {
            // La fenêtre des normes montre une copie de travail : c'est elle
            // qu'il faut modifier, sinon « Appliquer » réécrit l'ancienne
            // valeur. Sans fenêtre ouverte, le réglage courant fait foi.
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.fonctionnalites.basculer(&nom, actif);
                dialogue.modifie = true;
            } else {
                state.reglages.fonctionnalites.basculer(&nom, actif);
            }

            Task::none()
        }

        Message::BasculerFonctionnalite(nom) => {
            // Depuis une commande, il n'y a pas forcément de fenêtre ouverte :
            // la bascule doit être écrite tout de suite dans les normes du
            // projet, sinon elle ne serait qu'un état d'écran.
            let actif = state.reglages.fonctionnalites.actif(&nom);
            let nouveau = !actif;

            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.fonctionnalites.basculer(&nom, nouveau);
                dialogue.modifie = true;
                state.note = Some(crate::app::state::Note::info(format!(
                    "Le plugin « {nom} » est {} dans les normes — « Appliquer » pour l'écrire.",
                    if nouveau { "activé" } else { "désactivé" }
                )));
                return Task::none();
            }

            state.reglages.fonctionnalites.basculer(&nom, nouveau);
            let accrocher = crate::mise_en_page::accroche_manquante(&state.project.dir);
            match crate::mise_en_page::appliquer(&state.project.dir, &state.reglages, accrocher) {
                Ok(_) => {
                    state.configurer_les_plugins_selon_les_normes();
                    state.render.pending = true;
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Le plugin « {nom} » est {}.",
                        if nouveau { "activé" } else { "désactivé" }
                    )));
                }
                Err(erreur) => {
                    state.reglages.fonctionnalites.basculer(&nom, actif);
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Le plugin « {nom} » n'a pas pu être modifié : {erreur}"
                    )));
                }
            }

            Task::none()
        }
        _ => Task::none(),
    }
}
