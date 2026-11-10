//! Les références bibliques du document : la mémoire des versets cités.

use crate::app::Message;
use crate::app::state::State;
use crate::plugins::bible::TriReferences;
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, row, text};
use iced::{Element, Length};

/// Le panneau : combien de références, lesquelles, et le geste d'index.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;
    let stats = &state.plugins.bible.stats;
    let filtre = state.plugins.bible.recherche.to_lowercase();
    let references = ordonnees(stats, state.plugins.bible.tri)
        .into_iter()
        .filter(|(reference, _)| filtre.is_empty() || reference.to_lowercase().contains(&filtre))
        .collect::<Vec<_>>();

    let entete = row![
        text(format!("{} références", references.len())).size(15),
        Space::new().width(Length::Fill),
        text(match state.plugins.bible.tri {
            TriReferences::Apparition => "ordre du document",
            TriReferences::Canonique => "ordre canonique",
        })
        .size(11)
        .color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut lignes: Vec<Element<'_, Message>> = Vec::new();
    if references.is_empty() {
        lignes.push(
            text(if stats.par_reference.is_empty() {
                "Aucune référence biblique citée dans ce document."
            } else {
                "Aucune référence ne correspond à la recherche."
            })
            .size(11)
            .color(palette.text_muted)
            .into(),
        );
    } else {
        for (reference, nombre) in references {
            let cible = reference.clone();
            lignes.push(
                button(
                    row![
                        text(reference).width(Length::Fill),
                        text(format!("×{nombre}")).size(11),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .padding([4, 6])
                .style(move |_, statut| bouton_style(statut, palette))
                .on_press(Message::BibleAllerReference(cible))
                .into(),
            );
        }
    }

    let mut corps = column![entete].spacing(5);
    for ligne in lignes {
        corps = corps.push(ligne);
    }

    corps.padding(10).into()
}

/// Les références dans l'ordre demandé, avec leur nombre d'occurrences.
fn ordonnees(
    stats: &crate::plugins::bible::Statistiques,
    tri: TriReferences,
) -> Vec<(String, usize)> {
    match tri {
        TriReferences::Apparition => stats
            .ordre_apparition
            .iter()
            .filter_map(|reference| {
                stats
                    .par_reference
                    .get(reference)
                    .map(|nombre| (reference.clone(), *nombre))
            })
            .collect(),
        TriReferences::Canonique => {
            let mut references: Vec<(String, usize)> = stats
                .par_reference
                .iter()
                .map(|(reference, nombre)| (reference.clone(), *nombre))
                .collect();
            references.sort_by_key(|(reference, _)| {
                crate::langage::references::depuis_texte(reference)
                    .map(|r| {
                        (
                            crate::langage::references::rang_du_livre(&r.livre)
                                .unwrap_or(usize::MAX),
                            r.chapitre,
                            r.verset_debut,
                        )
                    })
                    .unwrap_or((usize::MAX, u32::MAX, u32::MAX))
            });
            references
        }
    }
}

fn bouton_style(statut: button::Status, palette: Palette) -> button::Style {
    button::Style {
        background: match statut {
            button::Status::Hovered | button::Status::Pressed => Some(palette.surface_hover.into()),
            _ => None,
        },
        text_color: palette.text,
        border: iced::Border {
            radius: theme::jetons::RAYON.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::bible::Statistiques;

    /// Le tri d'apparition suit le document ; le tri canonique remet les
    /// livres dans l'ordre biblique, quel que soit l'ordre d'écriture.
    #[test]
    fn les_deux_tris_des_references_sont_distincts() {
        let mut stats = Statistiques::default();
        stats.ajouter("{{Jn 3:16 ; Gn 1:1}}");

        let apparition = ordonnees(&stats, TriReferences::Apparition);
        assert_eq!(
            apparition
                .iter()
                .map(|(reference, _)| reference.as_str())
                .collect::<Vec<_>>(),
            vec!["Jn 3.16", "Gn 1.1"]
        );

        let canonique = ordonnees(&stats, TriReferences::Canonique);
        assert_eq!(
            canonique
                .iter()
                .map(|(reference, _)| reference.as_str())
                .collect::<Vec<_>>(),
            vec!["Gn 1.1", "Jn 3.16"]
        );
    }
}
