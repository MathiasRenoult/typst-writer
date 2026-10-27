pub mod completion;
pub mod editor;
pub mod layout;
pub mod preview;
pub mod reference;
mod reglages;
mod rename;
pub mod sidebar;
pub mod status_bar;
pub mod theme;
pub mod zotero;
pub mod toolbar;

use crate::app::{Message, State};
use iced::widget::{column, stack};
use iced::{Element, Length};

/// Compose l'interface : barre d'outils, espace de travail, barre d'état.
///
/// La barre d'outils et la barre d'état sont fixes ; tout l'espace qui reste
/// appartient à la disposition des panneaux, qui gère seule ce qui est
/// redimensionnable. La fenêtre de suggestions et celle de mise en page sont
/// des **couches** posées par-dessus, toujours en place dans l'arbre.
pub fn view(state: &State) -> Element<'_, Message> {
    // En mode focus, la barre d'outils disparaît : elle n'a plus d'objet, les
    // panneaux étant masqués et les commandes de fichier n'étant pas ce qu'on
    // vient chercher en écrivant. On en sort par F11 ou Échap.
    if state.layout.focus_mode {
        return stack![
            column![layout::view(state), status_bar::view(state)]
                .width(Length::Fill)
                .height(Length::Fill),
            completion::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    // Les fenêtres flottantes sont des **couches** : leur place existe même
    // quand elles sont vides, la forme de la page ne change donc jamais quand
    // elles s'ouvrent — la leçon du défilement perdu à l'ouverture d'une bulle
    // se paie encore.
    stack![
        column![
            toolbar::view(state),
            rename::view(state),
            layout::view(state),
            status_bar::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        completion::view(state),
        reference::view(state),
        zotero::view(state),
        reglages::view(state),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
