pub mod editor;
pub mod layout;
pub mod preview;
pub mod sidebar;
pub mod status_bar;
pub mod toolbar;

use crate::app::{Message, State};
use iced::Element;

pub fn view(state: &State) -> Element<'_, Message> {
    layout::view(state)
}
