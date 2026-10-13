use crate::app::{Message, State};
use iced::widget::{Space, row, text};

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let save_status = if state.editor.is_modified() {
        "● Modified"
    } else {
        "● Saved"
    };

    let syntax_status = if state.doc.errors.is_empty() {
        "● Typst OK".to_string()
    } else {
        format!("⚠ {} erreur(s) de syntaxe", state.doc.errors.len())
    };

    row![
        text(state.editor.filename()),
        Space::new().width(iced::Fill),
        text("UTF-8"),
        text("Typst"),
        text(syntax_status),
        text(save_status),
        text(format!("Words: {}", word_count(state.editor.content()))),
        text(format!(
            "Characters: {}",
            state.editor.content().chars().count()
        )),
    ]
    .spacing(18)
    .padding([6, 12])
    .align_y(iced::Alignment::Center)
    .into()
}

fn word_count(content: String) -> usize {
    content.split_whitespace().count()
}
