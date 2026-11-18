//! La palette de commandes : un seul panneau, deux modes.
//!
//! `..` y cherche les balises et snippets ; `/` ou `Ctrl+K` y cherche toutes
//! les commandes. Le panneau n'est pas une fenêtre modale : il flotte au-dessus
//! du texte, et Échap le referme sans toucher au déclencheur déjà écrit.

use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::palette::{Icone, Item, Palette};
use crate::app::state::State;
use crate::ui::editor::bubble_style;
use crate::ui::theme::{self, Palette as ThemePalette};

const LARGEUR: f32 = 620.0;

/// La palette, ou sa place vide.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(palette) = state.palette.as_ref() else {
        return Space::new().into();
    };

    let theme_palette = state.resolved.palette;
    let resultats = palette.resultats(state);

    let invitation = match palette.mode {
        crate::app::palette::ModePalette::Balises => {
            "Que veux-tu insérer ?  (balise, snippet, métadonnée)"
        }
        crate::app::palette::ModePalette::Systeme => "Quelle commande ?",
    };

    let champ = text_input(invitation, &palette.requete)
        .id(Palette::id_champ())
        .on_input(Message::PaletteRecherche)
        .padding(8)
        .width(Length::Fill);

    let entete = row![
        theme_palette
            .label(
                if matches!(palette.mode, crate::app::palette::ModePalette::Balises) {
                    "Insérer"
                } else {
                    "Commandes"
                }
            )
            .size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        theme_palette
            .label("Échap pour fermer")
            .size(theme::jetons::DETAIL)
            .color(theme_palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let lignes: Vec<Element<'_, Message>> = resultats
        .iter()
        .enumerate()
        .map(|(rang, item)| ligne(item, rang, rang == palette.selection, theme_palette))
        .collect();

    let liste: Element<'_, Message> = if lignes.is_empty() {
        container(
            theme_palette
                .label("Aucun résultat. Essaie un autre mot, ou tape un alias.")
                .size(theme::jetons::CORPS)
                .color(theme_palette.text_muted),
        )
        .padding([10, 8])
        .into()
    } else {
        column(lignes).spacing(2).into()
    };

    let pied = theme_palette
        .label("↑↓ choisir · Entrée insérer · Tab configurer · Échap fermer")
        .size(theme::jetons::DETAIL)
        .color(theme_palette.text_muted);

    let carte = container(column![entete, champ, liste, pied].spacing(10))
        .width(Length::Fixed(LARGEUR))
        .padding(16)
        .style(bubble_style(theme_palette));

    // Le panneau flotte au-dessus du texte, sans voile : on peut le fermer
    // et reprendre l'écriture au même endroit.
    container(carte)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn ligne(
    item: &Item,
    rang: usize,
    selection: bool,
    palette: ThemePalette,
) -> Element<'static, Message> {
    let icone: Element<'_, Message> = icone(item.icone);

    let titre = palette
        .label(item.titre.clone())
        .size(theme::jetons::CORPS)
        .color(if selection {
            palette.accent
        } else {
            palette.text
        });

    let categorie = libelle_categorie(item.categorie);
    let description = palette
        .label(format!("{} · {}", item.description, categorie))
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    let mut contenu = row![
        icone,
        column![titre, description].spacing(1).width(Length::Fill),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if let Some(raccourci) = item.raccourci {
        contenu = contenu.push(
            palette
                .label(raccourci)
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    button(contenu)
        .on_press(Message::PaletteChoisir(rang))
        .padding([7, 9])
        .width(Length::Fill)
        .style(move |_, status| style_ligne(palette, selection, status))
        .into()
}

fn libelle_categorie(categorie: crate::app::palette::Categorie) -> &'static str {
    use crate::app::palette::Categorie;

    match categorie {
        Categorie::Fichier => "fichier",
        Categorie::Projet => "projet",
        Categorie::Insertion => "insertion",
        Categorie::Affichage => "affichage",
        Categorie::Recherche => "recherche",
        Categorie::Aide => "aide",
        Categorie::Reglage => "réglage",
    }
}

fn style_ligne(palette: ThemePalette, selection: bool, status: button::Status) -> button::Style {
    let survole = matches!(status, button::Status::Hovered | button::Status::Pressed);

    let fond = if selection {
        iced::Color {
            a: 0.22,
            ..palette.accent
        }
    } else if survole {
        palette.surface_hover
    } else {
        Color::TRANSPARENT
    };

    button::Style {
        background: Some(fond.into()),
        text_color: if selection {
            palette.accent
        } else {
            palette.text
        },
        border: Border {
            color: if selection {
                palette.accent
            } else {
                Color::TRANSPARENT
            },
            width: if selection { 1.0 } else { 0.0 },
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

fn icone(icone: Icone) -> Element<'static, Message> {
    match icone {
        Icone::Fichier => lucide_icons::iced::icon_file::<Theme>().into(),
        Icone::Dossier => lucide_icons::iced::icon_folder_open::<Theme>().into(),
        Icone::Bibliotheque => lucide_icons::iced::icon_library::<Theme>().into(),
        Icone::Disquette => lucide_icons::iced::icon_save::<Theme>().into(),
        Icone::Telecharger => lucide_icons::iced::icon_download::<Theme>().into(),
        Icone::Mode => lucide_icons::iced::icon_scan::<Theme>().into(),
        Icone::Focus => lucide_icons::iced::icon_maximize::<Theme>().into(),
        Icone::Recherche => lucide_icons::iced::icon_search::<Theme>().into(),
        Icone::Aide => lucide_icons::iced::icon_book_open::<Theme>().into(),
        Icone::Note => lucide_icons::iced::icon_text_quote::<Theme>().into(),
        Icone::Image => lucide_icons::iced::icon_image::<Theme>().into(),
        Icone::Citation => lucide_icons::iced::icon_book_marked::<Theme>().into(),
        Icone::Tableau => lucide_icons::iced::icon_table::<Theme>().into(),
        Icone::Graphique => lucide_icons::iced::icon_chart_column::<Theme>().into(),
        Icone::Encadre => lucide_icons::iced::icon_text_quote::<Theme>().into(),
        Icone::Glossaire => lucide_icons::iced::icon_book_a::<Theme>().into(),
        Icone::Bible => lucide_icons::iced::icon_book_open::<Theme>().into(),
        Icone::Etiquette => lucide_icons::iced::icon_tag::<Theme>().into(),
        Icone::Lien => lucide_icons::iced::icon_link::<Theme>().into(),
        Icone::Calendrier => lucide_icons::iced::icon_calendar::<Theme>().into(),
        Icone::Palette => lucide_icons::iced::icon_palette::<Theme>().into(),
        Icone::EtiquetteMeta => lucide_icons::iced::icon_tag::<Theme>().into(),
        Icone::Fragment => lucide_icons::iced::icon_file_text::<Theme>().into(),
        Icone::Reglages => lucide_icons::iced::icon_file_cog::<Theme>().into(),
        Icone::Puzzle => lucide_icons::iced::icon_puzzle::<Theme>().into(),
        Icone::Plus => lucide_icons::iced::icon_plus::<Theme>().into(),
        Icone::Panneau => lucide_icons::iced::icon_panel_left_open::<Theme>().into(),
    }
}
