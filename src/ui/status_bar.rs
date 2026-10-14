use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{container, row, space, text};
use iced::{Element, Length};

/// Hauteur de la barre.
const BAR_HEIGHT: f32 = 26.0;

/// La barre d'état : l'essentiel, une fois, sans ponctuation décorative.
///
/// Les pastilles « ● » et les libellés en anglais ont disparu : un statut se
/// lit par sa couleur et son texte, pas par un symbole répété quatre fois.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = Palette::from_theme(&theme::resolve(state.theme, state.system_theme));

    let save = if state.editor.is_modified() {
        ("Non enregistré", palette.warning)
    } else {
        ("Enregistré", palette.success)
    };

    let (render, render_color): (String, iced::Color) = if state.render.pending {
        ("Compilation…".to_string(), palette.text_muted)
    } else if !state.render.errors.is_empty() {
        (
            format!("{} erreur(s)", state.render.errors.len()),
            palette.danger,
        )
    } else if !state.render.warnings.is_empty() {
        (
            format!("{} avertissement(s)", state.render.warnings.len()),
            palette.warning,
        )
    } else if state.render.done {
        ("Document prêt".to_string(), palette.success)
    } else {
        (String::new(), palette.text_muted)
    };

    let mut bar = row![].spacing(16).align_y(iced::Alignment::Center);

    bar = bar.push(
        text(state.editor.filename())
            .size(12)
            .color(palette.text_muted),
    );

    bar = bar.push(space::horizontal().width(Length::Fill));

    // Le compte de mots est le seul renseignement de fond utile pendant la
    // rédaction ; encodage et langage sont des constantes, on les omet.
    bar = bar.push(
        text(format!("{} mots", words(state.editor.content())))
            .size(12)
            .color(palette.text_muted),
    );

    if !render.is_empty() {
        bar = bar.push(text(render).size(12).color(render_color));
    }

    bar = bar.push(text(save.0).size(12).color(save.1));

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(BAR_HEIGHT))
        .padding([0, 12])
        .center_y(BAR_HEIGHT)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Nombre de mots : ce qui intéresse une rédaction, pas le nombre de
/// caractères.
fn words(content: String) -> usize {
    content.split_whitespace().count()
}
