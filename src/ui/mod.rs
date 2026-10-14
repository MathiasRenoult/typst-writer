pub mod editor;
pub mod layout;
pub mod preview;
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
    column![
        toolbar::view(state),
        layout::view(state),
        status_bar::view(state),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
