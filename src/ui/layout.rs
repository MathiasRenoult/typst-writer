use crate::app::{Message, State};
use iced::widget::{column, row};

use super::{editor, preview, sidebar, status_bar, toolbar};

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let content = row![
        sidebar::view(state),
        editor::view(state),
        preview::view(state),
    ]
    .spacing(1)
    .height(iced::Fill);

    column![toolbar::view(), content, status_bar::view(state),]
        .height(iced::Fill)
        .into()
}
