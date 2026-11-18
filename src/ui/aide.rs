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

    // Le parcours de première utilisation : quatre gestes, du projet à
    // l'export — c'est toute la promesse du premier quart d'heure.
    let premiers_pas = column![
        palette.label("Premiers pas").size(theme::jetons::TITRE),
        palette
            .label("1. Créer un projet — les boutons du panneau Projets.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label("2. Écrire — le document s'enregistre sans y penser.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label("3. Ajouter une source — elle rejoint la bibliographie du projet.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label("4. Exporter — PDF, Markdown ou DOCX.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label("5. En cas d'accident, Aide → Restaurer une version.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(6);

    // Où vivent les documents : visible, et ouvrable d'un geste.
    let donnees = column![
        palette.label("Vos données").size(theme::jetons::TITRE),
        palette
            .label(crate::services::stockage::work_dir().display().to_string())
            .size(theme::jetons::DETAIL)
            .color(palette.accent),
        row![
            button(
                palette
                    .label("Ouvrir le dossier des données")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::OuvrirDossierDonnees)
            .padding([4, 10]),
        ],
    ]
    .spacing(6);

    // Le diagnostic : l'état du poste, dit avec ses remèdes.
    let diagnostic = column![
        palette.label("Diagnostic").size(theme::jetons::TITRE),
        palette
            .label(diagnostic_resume(state))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(6);

    // La récupération : ce qui a été retiré se retrouve, ce qui a été
    // remplacé se relit, et les temporaires perdus reviennent seuls.
    let recuperation = column![
        palette.label("Récupération").size(theme::jetons::TITRE),
        palette
            .label("Les temporaires perdus sont récupérés à l'ouverture du projet.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row![
            button(
                palette
                    .label("Corbeille et suppressions…")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::CorbeilleOuvrir)
            .padding([4, 10]),
            button(
                palette
                    .label("Ouvrir les sauvegardes du projet")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::OuvrirSauvegardes)
            .padding([4, 10]),
            button(
                palette
                    .label("Restaurer une version…")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::RestaurationOuvrir)
            .padding([4, 10]),
        ]
        .spacing(8),
    ]
    .spacing(6);

    let corps = column![
        palette.label("Les raccourcis").size(theme::jetons::TITRE),
        colonnes,
        premiers_pas,
        donnees,
        diagnostic,
        recuperation,
        row![
            button(palette.label("Fermer").size(theme::jetons::DETAIL))
                .on_press(Message::AideBasculer)
                .padding([4, 10]),
        ],
    ]
    .spacing(theme::jetons::PAS);

    let fenetre = container(corps).padding(16).style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

/// L'état du poste en une ligne — la même pour la vue et les épreuves.
///
/// Chaque état dit son remède : un diagnostic qui n'explique pas quoi
/// faire n'est qu'une inquiétude de plus.
pub fn diagnostic_resume(state: &crate::app::State) -> String {
    let bibliotheque = match state.zotero.source {
        crate::zotero::Source::Locale => "locale",
        crate::zotero::Source::Zotero => "Zotero (lecture seule)",
    };
    format!(
        "Version {} — rendu : {} diagnostic(s) — l'écriture reste entière — bibliothèque : {bibliotheque}",
        env!("CARGO_PKG_VERSION"),
        state.render.errors.len()
    )
}
