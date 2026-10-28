//! La carte des raccourcis (`F1`).
//!
//! Le clavier de l'application tient en une fenêtre : ouverte par `F1`,
//! fermée par `Échap` comme toutes les autres. C'est elle qui rend le
//! système lisible — un raccourci qu'on ne trouve pas n'existe pas.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row};
use iced::{Element, Length};

/// La carte, quand elle est demandée.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    if !state.aide {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    let palette = state.resolved.palette;

    // Les gestes, du plus quotidien au plus rare — chacun à sa place.
    let groupes: [(&str, &[(&str, &str)]); 4] = [
        (
            "Écrire",
            &[
                ("Ctrl+S", "Enregistrer"),
                ("Ctrl+Z / Ctrl+Y", "Annuler / refaire"),
                ("Ctrl+F", "Chercher — expressions régulières"),
                ("F3 / Maj+F3", "Suivant / précédent"),
                ("Ctrl+T / Ctrl+Maj+T", "Titre suivant / précédent"),
                ("Ctrl+Maj+Haut/Bas", "Le bloc monte ou descend"),
                ("Ctrl+Espace", "Accepter la complétion"),
            ],
        ),
        (
            "Le document",
            &[
                ("Ctrl+N", "Le document d'écriture"),
                ("Ctrl+O", "Charger un .typ dans le projet"),
                ("Ctrl+E", "Exporter en PDF"),
                ("Ctrl+I", "L'assistance à la frappe"),
                ("F2", "Renommer le symbole sous le curseur"),
                ("F12", "Aller à la définition"),
            ],
        ),
        (
            "L'affichage",
            &[
                ("F11", "Le mode focus — le texte seul"),
                ("Ctrl+= / Ctrl+-", "Agrandir / réduire l'aperçu"),
                ("Ctrl+0", "Ajuster à la largeur"),
            ],
        ),
        (
            "Les fenêtres",
            &[
                ("Tab / Maj+Tab", "Circuler dans les champs"),
                ("Entrée", "Valider — ou le champ suivant"),
                ("Échap", "Fermer celle du dessus"),
                ("F1", "Cette carte"),
            ],
        ),
    ];

    let mut colonnes = row![].spacing(theme::jetons::PAS * 2.0);

    for (titre, raccourcis) in groupes {
        let mut groupe = column![palette.label(titre).size(theme::jetons::CORPS)].spacing(6);

        for &(touches, geste) in raccourcis.iter() {
            groupe = groupe.push(
                row![
                    palette
                        .label(touches)
                        .size(theme::jetons::DETAIL)
                        .color(palette.accent),
                    palette
                        .label(geste)
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .spacing(8),
            );
        }

        colonnes = colonnes.push(groupe);
    }

    let corps = column![
        palette.label("Les raccourcis").size(theme::jetons::TITRE),
        colonnes,
        row![
            button(palette.label("Fermer").size(theme::jetons::DETAIL))
                .on_press(Message::AideBasculer)
                .padding([4, 10]),
        ],
    ]
    .spacing(theme::jetons::PAS);

    let fenetre = container(corps).padding(16).style(theme::panneau(palette));

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
