//! L'éditeur de thème : les couleurs de l'application, en place.
//!
//! Le thème est un fichier ; nous avons un éditeur ; il sert. La copie de
//! travail ne touche le fichier qu'à l'enregistrement — la règle de tout
//! dialogue.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, pick_list, row, text_input};
use iced::{Element, Length};

/// Les couleurs à portée de clic : le nuancier de l'éditeur (bug 11 du
/// rapport 2). La valeur libre reste possible à côté.
const NUANCIER: &[&str] = &[
    "#1a1a1a", "#f5f5f0", "#8c7853", "#4f5b66", "#7a9e7e", "#c47c6c", "#7c6f9e", "#d4af37",
];

/// Les champs de couleur, et le libellé qu'ils portent.
const CHAMPS: &[(&str, &str)] = &[
    ("background", "Fond de l'application"),
    ("surface", "Fond des panneaux"),
    ("text", "Texte"),
    ("accent", "Accent"),
    ("text_muted", "Texte discret"),
    ("border", "Traits"),
    ("danger", "Erreur"),
    ("warning", "Avertissement"),
    ("success", "Succès"),
];

/// La fenêtre, quand l'éditeur est ouvert.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let Some(fichier) = state.editeur_theme.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    // Les thèmes à éditer — les natifs n'ont pas de fichier.
    let noms: Vec<String> = state
        .themes
        .iter()
        .filter(|theme| theme.file.is_some())
        .map(|theme| theme.name.clone())
        .collect();

    let mut corps = column![].spacing(10);

    corps = corps.push(
        row![
            pick_list(noms, Some(fichier.meta.name.clone()), Message::ThemeEditeurChoisir)
                .width(Length::Fixed(200.0)),
            palette.label("le thème à éditer").size(theme::jetons::DETAIL),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    );

    let couleurs = &fichier.colors;

    for &(champ, libelle) in CHAMPS {
        let hexa = |couleur: crate::theming::HexColor| -> String {
            let c = couleur.0;
            format!(
                "#{:02x}{:02x}{:02x}",
                (c.r * 255.0).round() as u8,
                (c.g * 255.0).round() as u8,
                (c.b * 255.0).round() as u8
            )
        };

        // La saisie brute de l'utilisateur passe avant la valeur formatée :
        // une frappe incomplète (`#1a`) ne doit pas être éjectée.
        let valeur = state
            .editeur_theme_textes
            .get(champ)
            .cloned()
            .unwrap_or_else(|| match champ {
                "background" => hexa(couleurs.background),
                "surface" => hexa(couleurs.surface),
                "text" => hexa(couleurs.text),
                "accent" => hexa(couleurs.accent),
                "text_muted" => couleurs.text_muted.map(hexa).unwrap_or_default(),
                "border" => couleurs.border.map(hexa).unwrap_or_default(),
                "danger" => couleurs.danger.map(hexa).unwrap_or_default(),
                "warning" => couleurs.warning.map(hexa).unwrap_or_default(),
                _ => couleurs.success.map(hexa).unwrap_or_default(),
            });

        // La pastille montre la couleur telle qu'elle sera : les mots ne
        // suffisent pas à choisir une teinte (bug 10 du rapport 2).
        let teinte = crate::theming::HexColor::parse(valeur.trim()).ok().map(|c| c.0);

        let pastille = container(Space::new().width(Length::Fixed(18.0)).height(Length::Fixed(18.0)))
            .style(move |_theme| container::Style {
                background: teinte.map(iced::Background::from),
                border: iced::Border {
                    width: 1.0,
                    color: palette.border,
                    ..Default::default()
                },
                ..Default::default()
            });

        let mut nuancier = row![].spacing(3);

        for teinte in NUANCIER {
            let teinte = *teinte;
            let voulue = crate::theming::HexColor::parse(teinte).ok().map(|c| c.0);

            nuancier = nuancier.push(
                button(Space::new().width(Length::Fixed(12.0)).height(Length::Fixed(12.0)))
                    .on_press(Message::ThemeEditeurChamp(champ, teinte.to_string()))
                    .padding(2)
                    .style(move |_t, _s| iced::widget::button::Style {
                        background: voulue.map(iced::Background::from),
                        border: iced::Border {
                            width: 1.0,
                            color: palette.border,
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        }

        corps = corps.push(
            row![
                palette
                    .label(libelle)
                    .size(theme::jetons::DETAIL)
                    .width(Length::Fixed(170.0)),
                pastille,
                text_input("#1a1a1a", &valeur)
                    .on_input(move |v| Message::ThemeEditeurChamp(champ, v))
                    .padding(5)
                    .width(Length::Fixed(110.0)),
                nuancier,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }

    corps = corps.push(Space::new().height(6));
    corps = corps.push(
        row![
            button(palette.label("Réinitialiser").size(theme::jetons::DETAIL))
                .on_press(Message::ThemeEditeurReset)
                .padding([5, 12]),
            button(palette.label("Enregistrer").size(theme::jetons::DETAIL))
                .on_press(Message::ThemeEditeurEnregistrer)
                .padding([5, 12]),
            button(palette.label("Fermer").size(theme::jetons::DETAIL))
                .on_press(Message::ThemeEditeurFermer)
                .padding([5, 12]),
        ]
        .spacing(theme::jetons::PAS),
    );

    let fenetre = container(column![
        palette.label("L'éditeur de thème").size(theme::jetons::SAILLANT),
        Space::new().height(6),
        corps,
    ])
    .padding(18)
    .width(Length::Fixed(430.0))
    .style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
