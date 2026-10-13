use crate::app::Message;
use iced::widget::{button, column, container, text};

pub fn view() -> iced::Element<'static, Message> {
    let project = column![
        text("PROJECT").size(14),
        button("📁 Mon projet"),
        text(""),
        button("📄 main.typ"),
        button("📁 chapters"),
        button("📁 images"),
        button("📄 references.bib"),
    ]
    .spacing(6)
    .padding(12);

    container(project).width(220).height(iced::Fill).into()
}
