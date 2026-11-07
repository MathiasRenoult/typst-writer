use crate::app::{Message, State};

use iced::widget::{container, row, space};
use iced::{Element, Length};

/// Hauteur de la barre.
pub(crate) const BAR_HEIGHT: f32 = 26.0;

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

    // Le dernier mot de l'application : succès ou erreur. La barre d'état
    // est toujours là — c'est le seul endroit où un retour ne peut pas
    // disparaître avec le panneau qui l'a produit.
    if let Some(note) = state.note.as_ref() {
        bar = bar.push(
            palette
                .label(note.texte.clone())
                .size(12)
                .color(if note.erreur {
                    palette.danger
                } else {
                    palette.success
                }),
        );
    }

    // Ce que l'application exporte ou enregistre vit dans le panneau
    // d'aperçu — masquable. La barre d'état le relaie quand il n'y a pas
    // de note plus fraîche : un retour ne peut pas disparaître avec le
    // panneau qui l'a produit.
    if state.note.is_none()
        && let Some(etat) = state.export_status.as_ref()
        && !etat.is_empty()
    {
        bar = bar.push(
            palette
                .label(etat.clone())
                .size(12)
                .color(palette.text_muted),
        );
    }

    bar = bar.push(space::horizontal().width(Length::Fill));

    // Le compte de mots est le seul renseignement de fond utile pendant la
    // rédaction ; encodage et langage sont des constantes, on les omet.
    bar = bar.push(
        palette
            // Le compte est tenu à jour par les tics de rendu : le calculer
            // ici revenait à cloner le document entier à chaque image.
            .label(format!("{} mots", state.word_count()))
            .size(12)
            .color(palette.text_muted),
    );

    // L'assistance à la frappe. Son absence se dit **sans alarme** : une
    // application qui fonctionne sans elle ne doit pas présenter son manque
    // comme une panne, mais rien dire du tout laisserait croire qu'elle est là.

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
