use crate::app::Message;
use iced::widget::{container, text};

pub fn view() -> iced::Element<'static, Message> {
    let preview = container(
        text("PREVIEW\n\n\n\n\nPDF preview")
            .size(18)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .center_x(iced::Fill)
    .center_y(iced::Fill);

    container(preview)
        .width(iced::FillPortion(2))
        .height(iced::Fill)
        .into()
}
