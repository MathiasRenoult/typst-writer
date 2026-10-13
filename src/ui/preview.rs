use crate::app::{Message, State};
use iced::widget::{button, container, row, text};

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let content = match state.preview.as_ref() {
        None => container(
            text("Prévisualisation\n\nCliquez sur « Actualiser » pour compiler le document.")
                .size(18)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .center_x(iced::Fill)
        .center_y(iced::Fill),

        Some(preview) => container(
            row![
                text(preview.pdf.display().to_string()).size(13),
                button("Ouvrir le PDF").on_press(Message::OpenPreview),
                text(&preview.status).size(14)
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        )
        .width(iced::FillPortion(2))
        .height(iced::Fill),
    };

    content.into()
}
