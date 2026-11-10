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
        _ => Task::none(),
    }
}
