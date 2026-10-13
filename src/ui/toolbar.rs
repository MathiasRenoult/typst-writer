use crate::app::Message;
use iced::widget::{Space, button, row, text};

pub fn view() -> iced::Element<'static, Message> {
    row![
        text("Typst Writer").size(20),
        button("Nouveau").on_press(Message::NewFile),
        button("Ouvrir").on_press(Message::OpenFile),
        button("Sauvegarder").on_press(Message::SaveFile),
        button("Enregistrer sous").on_press(Message::SaveAs),
        button("Fermer").on_press(Message::CloseFile),
        button("Insérer").on_press(Message::InsertReference),
        Space::new().width(iced::Fill),
        button("⚙ Settings").on_press(Message::OpenSettings),
    ]
    .spacing(10)
    .padding(10)
    .align_y(iced::Alignment::Center)
    .into()
}
