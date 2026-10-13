use crate::app::{Message, State};
use iced::Element;
use iced::widget::container;

pub fn view(state: &State) -> Element<'_, Message> {
    container(state.editor.editor.view().map(Message::Editor))
        .width(iced::Fill)
        .height(iced::Fill)
        .into()
}
