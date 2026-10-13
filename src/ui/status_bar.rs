use crate::app::{Message, State};
use iced::widget::{Space, row, text};

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let save_status = if state.editor.is_modified() {
        "● Modified"
    } else {
        "● Saved"
    };

    let render_status = if state.render.pending {
        "● Rendu…".to_string()
    } else if !state.render.errors.is_empty() {
        format!("⚠ {} erreur(s) typst", state.render.errors.len())
    } else if !state.render.warnings.is_empty() {
        format!("● {} avertissement(s)", state.render.warnings.len())
    } else if state.render.done {
        "● Typst OK".to_string()
    } else {
        "● …".to_string()
    };

    row![
        text(state.editor.filename()),
        Space::new().width(iced::Fill),
        text("UTF-8"),
        text("Typst"),
        text(render_status),
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
