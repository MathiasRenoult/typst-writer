pub mod editor;
pub mod layout;
pub mod preview;
mod reglages;
mod rename;
pub mod sidebar;
pub mod status_bar;
pub mod theme;
pub mod toolbar;

use crate::app::{Message, State};
use iced::widget::column;
use iced::{Element, Length};

/// Compose l'interface : barre d'outils, espace de travail, barre d'état.
///
/// La barre d'outils et la barre d'état sont fixes ; tout l'espace qui reste
/// appartient à la disposition des panneaux, qui gère seule ce qui est
/// redimensionnable.
pub fn view(state: &State) -> Element<'_, Message> {
    // En mode focus, la barre d'outils disparaît : elle n'a plus d'objet, les
    // panneaux étant masqués et les commandes de fichier n'étant pas ce qu'on
    // vient chercher en écrivant. On en sort par F11 ou Échap.
    if state.layout.focus_mode {
        return column![layout::view(state), status_bar::view(state)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // La fenêtre de mise en page est une **couche** : elle flotte au-dessus de
    // tout, et sa place existe même quand elle est vide. La forme de la page
    // ne change donc jamais quand elle s'ouvre — la leçon du défilement perdu
    // à l'ouverture d'une bulle se paie encore.
    iced::widget::stack![
        column![
            toolbar::view(state),

            // La case de renommage, toujours en place — vide quand elle ne
            // sert pas.
            rename::view(state),

            layout::view(state),
            status_bar::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        reglages::view(state),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
