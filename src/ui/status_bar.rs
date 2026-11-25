use crate::app::{Message, State};

use iced::widget::{container, row, space};
use iced::{Element, Length};

/// Hauteur de la barre.
pub(crate) const BAR_HEIGHT: f32 = 26.0;

/// Le nom du fichier de normes, et la norme de bibliothèque quand elle
/// correspond exactement aux réglages appliqués.
fn libelle_norme(fichier_present: bool, norme: &str) -> String {
    let fichier = crate::mise_en_page::REGLAGES;

    if !fichier_present {
        return format!("Norme : {fichier} (absente)");
    }

    if norme.trim().is_empty() {
        format!("Norme : {fichier}")
    } else {
        format!("Norme : {fichier} · {}", norme.trim())
    }
}

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
        let suffixe = if state.render.apercu_perime() {
            " — aperçu ancien"
        } else {
            ""
        };
        (
            format!("{} erreur(s){suffixe}", state.render.errors.len()),
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

    // Le fichier de normes est une propriété du document, pas un réglage
    // caché : son nom reste sous les yeux, avec la norme de la bibliothèque
    // quand les réglages appliqués lui correspondent exactement.
    let norme = libelle_norme(
        state
            .project
            .dir
            .join(crate::mise_en_page::REGLAGES)
            .is_file(),
        &state.norme_document,
    );
    bar = bar.push(palette.label(norme).size(12).color(palette.text_muted));

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
    // rédaction ; encodage et langage sont des constantes, on les omet. Une
    // sélection prend la main : c'est ainsi qu'on mesure un passage sans
    // quitter le document.
    let total = state.word_count();
    let libelle = match state.selection_word_count() {
        Some(selection) => format!("{selection} mots sélectionnés · {total} mots"),
        None => format!("{total} mots"),
    };
    bar = bar.push(
        palette
            // Le compte est tenu à jour par les tics de rendu : le calculer
            // ici revenait à cloner le document entier à chaque image.
            .label(libelle)
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

#[cfg(test)]
mod tests {
    use super::libelle_norme;

    #[test]
    fn le_fichier_de_normes_est_nomme_meme_sans_norme_de_bibliotheque() {
        assert_eq!(libelle_norme(true, ""), "Norme : mise-en-page.toml");
        assert_eq!(
            libelle_norme(true, "Thèse"),
            "Norme : mise-en-page.toml · Thèse"
        );
        assert_eq!(
            libelle_norme(false, ""),
            "Norme : mise-en-page.toml (absente)"
        );
    }
}
