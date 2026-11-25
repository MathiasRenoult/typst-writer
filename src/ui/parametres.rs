//! La page des paramètres propres à l'application.
//!
//! Elle ne confond pas deux choses : les **normes** décrivent le document
//! (papier, marges, titres) ; ces réglages-ci décrivent la façon dont
//! l'application se présente et se conduit. Les premières voyagent avec le
//! projet, les seconds avec la session de travail.

use crate::app::Message;
use crate::app::State;
use crate::app::state::ParametresPanneau;
use crate::base::theme::ThemeMode;
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, checkbox, column, container, pick_list, row, scrollable};
use iced::{Alignment, Element, Length, Theme};

/// Largeur du rail vertical : les intitulés respirent, le contenu garde la
/// place.
const RAIL: f32 = 210.0;

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.parametres_vue {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    let palette = state.resolved.palette;

    let entete = row![
        column![
            palette.label("Paramètres").size(28),
            palette
                .label("L'application, pas le document.")
                .size(theme::jetons::CORPS)
                .color(palette.text_muted),
        ]
        .spacing(4),
        Space::new().width(Length::Fill),
        button(
            row![
                theme::icon(
                    lucide_icons::iced::icon_arrow_left::<Theme>(),
                    15.0,
                    palette.text_muted
                ),
                palette
                    .label("Retour à l'écriture")
                    .size(theme::jetons::DETAIL),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ParametresFermer)
        .padding([9, 14])
        .style(move |_, statut| bouton_secondaire(statut, palette)),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let rail = rail(state, palette);
    let contenu = match state.parametres_panneau {
        ParametresPanneau::Barre => panneau_barre(state, palette),
        ParametresPanneau::Editeur => panneau_editeur(state, palette),
        ParametresPanneau::Apparence => panneau_apparence(state, palette),
        ParametresPanneau::Donnees => panneau_donnees(palette),
    };

    container(
        column![
            entete,
            iced::widget::rule::horizontal(1).style(move |_| {
                iced::widget::rule::Style {
                    color: palette.border,
                    radius: 0.0.into(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                }
            }),
            row![
                rail,
                iced::widget::rule::vertical(1).style(move |_| {
                    iced::widget::rule::Style {
                        color: palette.border,
                        radius: 0.0.into(),
                        fill_mode: iced::widget::rule::FillMode::Full,
                        snap: true,
                    }
                }),
                scrollable(
                    container(contenu)
                        .padding([24, 32])
                        .width(Length::Fill)
                        .max_width(760.0)
                )
                .height(Length::Fill)
                .width(Length::Fill),
            ]
            .height(Length::Fill)
            .width(Length::Fill),
        ]
        .spacing(0)
        .height(Length::Fill),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(palette.background.into()),
        ..Default::default()
    })
    .into()
}

fn rail(state: &State, palette: Palette) -> Element<'_, Message> {
    let mut colonne = column![].spacing(4).width(Length::Fixed(RAIL));

    let entrees = [
        (
            ParametresPanneau::Barre,
            lucide_icons::iced::icon_sliders_horizontal::<Theme>(),
        ),
        (
            ParametresPanneau::Editeur,
            lucide_icons::iced::icon_type::<Theme>(),
        ),
        (
            ParametresPanneau::Apparence,
            lucide_icons::iced::icon_contrast::<Theme>(),
        ),
        (
            ParametresPanneau::Donnees,
            lucide_icons::iced::icon_folder::<Theme>(),
        ),
    ];

    for (panneau, icone) in entrees {
        let actif = state.parametres_panneau == panneau;
        let couleur = if actif {
            palette.accent
        } else {
            palette.text_muted
        };
        colonne = colonne.push(
            button(
                row![
                    theme::icon(icone, 16.0, couleur),
                    palette
                        .label(panneau.libelle())
                        .size(theme::jetons::DETAIL)
                        .color(couleur),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::ParametresPanneau(panneau))
            .padding([10, 12])
            .width(Length::Fill)
            .style(move |_, statut| bouton_rail(statut, palette, actif)),
        );
    }

    colonne.into()
}

fn panneau_barre<'a>(state: &'a State, palette: Palette) -> Element<'a, Message> {
    column![
        titre(
            palette,
            "La barre supérieure",
            "Choisissez ce que la barre garde sous les yeux. Les raccourcis et la palette de commandes restent actifs quand un bouton est masqué.",
        ),
        section(
            palette,
            "Boutons de mise en page",
            "Gras, italique, titres, listes, citations de bloc et couleurs.",
            preference(
                palette,
                state.outils_mise_en_page,
                Message::ParametresBasculeOutilsMiseEnPage,
                "Afficher les boutons de mise en page",
                "Les masquer convient à qui écrit déjà le markdown et préfère le texte nu.",
            ),
        ),
        section(
            palette,
            "Menu d'insertion",
            "Sommaire, image, tableau, graphique, note, citation, sections et extensions.",
            preference(
                palette,
                state.outils_insertion,
                Message::ParametresBasculeOutilsInsertion,
                "Afficher le menu Insérer",
                "Il rassemble les gestes de contenu, sans encombrer la barre d'une icône par outil.",
            ),
        ),
        apercu_barre(palette, state),
    ]
    .spacing(22)
    .into()
}

fn panneau_editeur<'a>(state: &'a State, palette: Palette) -> Element<'a, Message> {
    column![
        titre(
            palette,
            "L'éditeur",
            "Des préférences de lecture, communes à tous vos projets.",
        ),
        section(
            palette,
            "Repères de lecture",
            "Les numéros de ligne restent invisibles par défaut en prose.",
            preference(
                palette,
                state.numeros_de_ligne,
                Message::NumerosDeLigneBascule,
                "Afficher les numéros de ligne",
                "Utile pour se repérer dans une longue note ou retrouver une ligne signalée.",
            ),
        ),
    ]
    .spacing(22)
    .into()
}

fn panneau_apparence<'a>(state: &'a State, palette: Palette) -> Element<'a, Message> {
    let noms: Vec<String> = state
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect();
    let courant = state.current_theme().map(|theme| theme.name.clone());

    let modes = [
        (ThemeMode::System, "Système", "Suit l'apparence du bureau."),
        (ThemeMode::Light, "Clair", "Le jour, ou un écran lumineux."),
        (ThemeMode::Dark, "Sombre", "Le soir, ou un écran discret."),
    ];

    let mut boutons_modes = row![].spacing(8);
    for (mode, libelle, detail) in modes {
        let actif = state.theme == mode;
        boutons_modes = boutons_modes.push(
            button(
                column![
                    palette.label(libelle).size(theme::jetons::DETAIL),
                    palette.label(detail).size(11).color(if actif {
                        palette.accent
                    } else {
                        palette.text_muted
                    }),
                ]
                .spacing(2),
            )
            .on_press(Message::ParametresModeTheme(mode))
            .padding([8, 12])
            .style(move |_, statut| bouton_mode(statut, palette, actif)),
        );
    }

    let mut actions = row![
        button(
            palette
                .label("Éditer le thème…")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::ThemeEditeurOuvrir)
        .padding([7, 11])
        .style(move |_, statut| bouton_secondaire(statut, palette)),
        button(
            palette
                .label("Charger un thème…")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::LoadTheme)
        .padding([7, 11])
        .style(move |_, statut| bouton_secondaire(statut, palette)),
        button(
            palette
                .label("Dossier des thèmes…")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::OpenThemesFolder)
        .padding([7, 11])
        .style(move |_, statut| bouton_secondaire(statut, palette)),
    ]
    .spacing(8);

    if let Some(courant) = state.current_theme()
        && !courant.is_native()
    {
        actions = actions.push(
            button(
                palette
                    .label("Supprimer ce thème…")
                    .size(theme::jetons::DETAIL)
                    .color(palette.danger),
            )
            .on_press(Message::DeleteTheme(courant.name.clone()))
            .padding([7, 11])
            .style(move |_, statut| bouton_secondaire(statut, palette)),
        );
    }

    column![
        titre(
            palette,
            "L'apparence",
            "Les couleurs de l'interface et la police qu'elle emploie. Les normes du document restent indépendantes.",
        ),
        section(
            palette,
            "Thème",
            "Le thème choisi pour l'apparence en cours.",
            column![
                pick_list(noms, courant, Message::SelectTheme)
                    .text_size(12)
                    .font(palette.interface)
                    .padding([7, 9])
                    .width(Length::Fixed(260.0)),
                Space::new().height(4),
                boutons_modes,
                Space::new().height(4),
                actions,
            ]
            .spacing(8),
        ),
    ]
    .spacing(22)
    .into()
}

fn panneau_donnees(palette: Palette) -> Element<'static, Message> {
    column![
        titre(
            palette,
            "Vos données",
            "Tout vit dans un dossier local, lisible et sauvegardable — jamais dans un service distant.",
        ),
        section(
            palette,
            "Dossier des données",
            "Les projets, les sauvegardes, les thèmes et la session de travail.",
            column![
                palette
                    .label(crate::services::stockage::work_dir().display().to_string())
                    .size(theme::jetons::DETAIL)
                    .color(palette.accent),
                button(palette.label("Ouvrir le dossier des données").size(theme::jetons::DETAIL))
                    .on_press(Message::OuvrirDossierDonnees)
                    .padding([7, 11])
                    .style(move |_, statut| bouton_secondaire(statut, palette)),
            ]
            .spacing(8),
        ),
    ]
    .spacing(22)
    .into()
}

fn apercu_barre(palette: Palette, state: &State) -> Element<'_, Message> {
    let couleur = palette.text_muted;
    let mut barre = row![].spacing(2).align_y(Alignment::Center);

    barre = barre.push(theme::icon(
        lucide_icons::iced::icon_library::<Theme>(),
        15.0,
        couleur,
    ));
    barre = barre.push(palette.label("Projet").size(11).color(couleur));
    barre = barre.push(theme::icon(
        lucide_icons::iced::icon_file_text::<Theme>(),
        15.0,
        couleur,
    ));
    barre = barre.push(theme::icon(
        lucide_icons::iced::icon_file_down::<Theme>(),
        15.0,
        couleur,
    ));

    if state.outils_mise_en_page {
        barre = barre.push(palette.label("B I  H  •  1.  ❝").size(11).color(couleur));
    }
    if state.outils_insertion {
        barre = barre.push(palette.label("Insérer").size(11).color(couleur));
    }

    barre = barre.push(Space::new().width(Length::Fill));
    barre = barre.push(palette.label("Paramètres").size(11).color(couleur));

    container(
        column![
            palette
                .label("Aperçu")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            container(barre)
                .padding([8, 10])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(palette.surface.into()),
                    border: iced::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: theme::jetons::RAYON.into(),
                    },
                    ..Default::default()
                }),
        ]
        .spacing(6),
    )
    .into()
}

fn titre<'a>(palette: Palette, titre: &'a str, detail: &'a str) -> Element<'a, Message> {
    column![
        palette.label(titre).size(20),
        palette
            .label(detail)
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(4)
    .into()
}

fn section<'a>(
    palette: Palette,
    titre: &'a str,
    detail: &'a str,
    corps: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            palette.label(titre).size(theme::jetons::CORPS),
            palette
                .label(detail)
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            Space::new().height(2),
            corps.into(),
        ]
        .spacing(5),
    )
    .padding(14)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    })
    .into()
}

fn preference<'a>(
    palette: Palette,
    actif: bool,
    message: Message,
    titre: &'a str,
    detail: &'a str,
) -> Element<'a, Message> {
    row![
        checkbox(actif).on_toggle(move |_| message.clone()),
        column![
            iced::widget::text(titre).size(theme::jetons::DETAIL),
            iced::widget::text(detail)
                .size(11)
                .color(palette.text_muted),
        ]
        .spacing(2),
    ]
    .spacing(10)
    .align_y(Alignment::Start)
    .into()
}

fn bouton_rail(status: button::Status, palette: Palette, actif: bool) -> button::Style {
    button::Style {
        background: Some(
            if actif || matches!(status, button::Status::Hovered) {
                palette.surface_hover
            } else {
                palette.surface
            }
            .into(),
        ),
        text_color: if actif {
            palette.accent
        } else {
            palette.text_muted
        },
        border: iced::Border {
            color: if actif {
                palette.accent
            } else {
                palette.border
            },
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

fn bouton_secondaire(status: button::Status, palette: Palette) -> button::Style {
    button::Style {
        background: matches!(status, button::Status::Hovered).then(|| palette.surface_hover.into()),
        text_color: palette.text,
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

fn bouton_mode(status: button::Status, palette: Palette, actif: bool) -> button::Style {
    button::Style {
        background: Some(
            if actif || matches!(status, button::Status::Hovered) {
                palette.surface_hover
            } else {
                palette.surface
            }
            .into(),
        ),
        text_color: if actif { palette.accent } else { palette.text },
        border: iced::Border {
            color: if actif {
                palette.accent
            } else {
                palette.border
            },
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}
