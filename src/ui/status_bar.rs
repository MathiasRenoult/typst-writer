use crate::app::{Message, State};
use iced::widget::{Space, row, text};

pub fn view(state: &State) -> iced::Element<'_, Message> {
    let save_status = if state.editor.is_modified() {
        "● Modified"
    } else {
        "● Saved"
    };

    // Le statut affiché est celui du vrai compilateur typst ; tant que son
    // premier résultat n'est pas revenu, on se rabat sur l'analyse
    // syntaxique instantanée.
    let check_status = if state.check.pending {
        "● Vérification…".to_string()
    } else if !state.check.done {
        if state.doc.errors.is_empty() {
            "● Typst".to_string()
        } else {
            format!("⚠ {} erreur(s) de syntaxe", state.doc.errors.len())
        }
    } else if !state.check.errors.is_empty() {
        format!("⚠ {} erreur(s) typst", state.check.errors.len())
    } else if !state.check.warnings.is_empty() {
        format!("● {} avertissement(s)", state.check.warnings.len())
    } else {
        "● Typst OK".to_string()
    };

    row![
        text(state.editor.filename()),
        Space::new().width(iced::Fill),
        text("UTF-8"),
        text("Typst"),
        text(check_status),
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
