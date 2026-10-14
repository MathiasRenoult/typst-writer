use crate::app::{Message, State};

use iced::widget::{container, row, space};
use iced::{Element, Length};

/// Hauteur de la barre.
const BAR_HEIGHT: f32 = 26.0;

/// La barre d'état : l'essentiel, une fois, sans ponctuation décorative.
///
/// Les pastilles « ● » et les libellés en anglais ont disparu : un statut se
/// lit par sa couleur et son texte, pas par un symbole répété quatre fois.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

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
        palette
            .label(state.editor.filename())
            .size(12)
            .color(palette.text_muted),
    );

    bar = bar.push(space::horizontal().width(Length::Fill));

    // Le compte de mots est le seul renseignement de fond utile pendant la
    // rédaction ; encodage et langage sont des constantes, on les omet.
    bar = bar.push(
        palette
            // Le compte est tenu à jour par les tics de rendu : le calculer
            // ici revenait à cloner le document entier à chaque image.
            .label(format!("{} mots", state.render.words.unwrap_or(0)))
            .size(12)
            .color(palette.text_muted),
    );

    // L'assistance à la frappe. Son absence se dit **sans alarme** : une
    // application qui fonctionne sans elle ne doit pas présenter son manque
    // comme une panne, mais rien dire du tout laisserait croire qu'elle est là.
    bar = bar.push(
        palette
            .label(lsp_status(state).0)
            .size(12)
            .color(lsp_status(state).1),
    );

    if !render.is_empty() {
        bar = bar.push(palette.label(render).size(12).color(render_color));
    }

    bar = bar.push(palette.label(save.0).size(12).color(save.1));

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

/// Ce que dit l'assistance à la frappe, et de quelle couleur.
///
/// Son absence se dit **sans alarme** : l'application fonctionne sans elle, et
/// présenter ce manque comme une panne serait faux. Mais ne rien dire du tout
/// laisserait croire qu'elle est là.
fn lsp_status(state: &State) -> (String, iced::Color) {
    let palette = state.resolved.palette;

    if state.lsp.unavailable {
        return ("assistance absente".to_string(), palette.text_muted);
    }

    if state.lsp.session.is_none() {
        return (String::new(), palette.text_muted);
    }

    if !state.lsp.ready {
        return ("assistance au démarrage".to_string(), palette.text_muted);
    }

    let (errors, total) = crate::lsp::messages::counts(&state.lsp.diagnostics);

    if total == 0 {
        return ("assistance active".to_string(), palette.success);
    }

    if errors > 0 {
        return (format!("{errors} erreur(s) du serveur"), palette.danger);
    }

    (
        format!("{total} avertissement(s) du serveur"),
        palette.warning,
    )
}
