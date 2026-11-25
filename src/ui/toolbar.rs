//! La barre supérieure : les domaines de l'application, séparés.
//!
//! Trois groupes, et un seul ordre : à gauche le projet et le document, au
//! centre les gestes d'écriture, à droite les outils du document et de
//! l'application. Les commandes qui ne se destinent pas au même domaine ne
//! partagent jamais un bouton — ni une icône.

use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, container, row};
use iced::{Alignment, Element, Length, Theme};

/// Taille des icônes de la barre.
const ICON_SIZE: f32 = 17.0;

/// Espacement entre les boutons.
const SPACING: f32 = 3.0;

/// Côté d'un bouton-icône : carré, pour que la cible de clic soit généreuse
/// sans que l'icône elle-même prenne de la place.
const BUTTON: f32 = 30.0;

/// Largeur des menus déroulants : assez pour les libellés longs, assez
/// étroite pour ne pas devenir une bande à travers la fenêtre.
const MENU_WIDTH: f32 = 252.0;

/// Largeur du menu des niveaux : ses libellés sont courts (« Titre 1 »).
const TITRE_MENU_WIDTH: f32 = 156.0;

/// Hauteur de la barre.
pub(crate) const BAR_HEIGHT: f32 = 42.0;

/// Les couleurs proposées pour teinter du texte, avec leur nom Typst.
///
/// Ce sont des noms reconnus par Typst : le générateur les passe tels quels
/// dans `#text(fill: …)`, donc aucun choix ici ne peut produire une couleur
/// inconnue du compilateur.
const COULEURS: [(&str, &str, &str); 7] = [
    ("Rouge", "red", "#e74c3c"),
    ("Bleu", "blue", "#3498db"),
    ("Vert", "green", "#27ae60"),
    ("Orange", "orange", "#e67e22"),
    ("Violet", "purple", "#9b59b6"),
    ("Noir", "black", "#1a1a2e"),
    ("Gris", "gray", "#7f8c8d"),
];

/// La barre d'outils : des icônes, pas des mots. Les trois menus nomment
/// leur domaine et restent des pastilles compactes ; leur contenu porte
/// toute la largeur réglée, pas la largeur de la fenêtre.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let mut bar = row![]
        .spacing(SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // -- Projet et document, à gauche -------------------------------------
    // La bibliothèque des projets précède le projet ouvert : c'est le chemin
    // naturel de qui arrive dans l'application.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_library::<Theme>(),
        "Les projets et leurs étagères",
        Message::ProjetsVue,
        palette,
        state.projets_vue,
    ));

    bar = bar.push(projet_menu(palette));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file_text::<Theme>(),
        "Ouvrir le document d'écriture du projet (Ctrl+N)",
        Message::NewFile,
        palette,
        false,
    ));

    // Une flèche de fichier, jamais une flèche de téléchargement : celle-ci
    // appartient aux thèmes, dans les paramètres.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file_down::<Theme>(),
        "Exporter en PDF (Ctrl+E)",
        Message::ExportPdf,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_x::<Theme>(),
        "Fermer l'onglet actif",
        Message::VueFermer(state.espace.active),
        palette,
        false,
    ));

    bar = bar.push(separation(palette));
    bar = bar.push(spacer());

    // -- Écriture, au centre ----------------------------------------------
    let sur_rendu = matches!(state.mode, crate::app::state::ModeVue::Rendu);

    if sur_rendu {
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_zoom_out::<Theme>(),
            "Réduire le zoom",
            Message::PreviewZoomStep(-1),
            palette,
            false,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_arrow_left_right::<Theme>(),
            "Ajuster à la largeur",
            Message::PreviewZoom(crate::app::state::PreviewZoomMode::FitWidth),
            palette,
            false,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_arrow_up_down::<Theme>(),
            "Ajuster à la hauteur",
            Message::PreviewZoom(crate::app::state::PreviewZoomMode::FitHeight),
            palette,
            false,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_zoom_in::<Theme>(),
            "Augmenter le zoom",
            Message::PreviewZoomStep(1),
            palette,
            false,
        ));
        bar = bar.push(
            palette
                .label(crate::ui::preview::zoom_percent(state).unwrap_or_else(|| "—".to_string()))
                .size(11)
                .color(palette.text_muted),
        );
    } else if state.editor.is_intermediate() {
        // La mise en forme s'arrête à ce qui agit sur le texte déjà écrit :
        // styles, titres, listes et citations. Insérer un objet est un autre
        // geste, et vit dans le menu qui suit.
        if state.outils_mise_en_page {
            bar = bar.push(format_button(
                "B",
                "Gras : formater la sélection",
                crate::app::message::FormatAction::Gras,
                palette,
            ));
            bar = bar.push(format_button(
                "I",
                "Italique : formater la sélection",
                crate::app::message::FormatAction::Italique,
                palette,
            ));
            bar = bar.push(titre_menu(palette));
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_list::<Theme>(),
                "Basculer la ligne ou les lignes sélectionnées en liste à puces",
                Message::Format(crate::app::message::FormatAction::ListePuce),
                palette,
                false,
            ));
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_list_ordered::<Theme>(),
                "Basculer la ligne ou les lignes sélectionnées en liste numérotée",
                Message::Format(crate::app::message::FormatAction::ListeNumero),
                palette,
                false,
            ));
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_quote::<Theme>(),
                "Basculer la ligne en citation de bloc",
                Message::Format(crate::app::message::FormatAction::CitationBloc),
                palette,
                false,
            ));
            bar = bar.push(bouton_couleur(state, palette));
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_palette::<Theme>(),
                "Choisir une couleur pour la sélection",
                Message::PickerCouleurOuvrir,
                palette,
                false,
            ));
        }

        if state.outils_insertion {
            bar = bar.push(inserer_menu(state, palette));
        }
    }

    bar = bar.push(spacer());

    // -- Document et application, à droite --------------------------------
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_command::<Theme>(),
        "Commandes (Ctrl+K) — les balises s'écrivent avec ..",
        Message::PaletteOuvrir(crate::app::palette::ModePalette::Systeme),
        palette,
        state.palette.is_some(),
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file_cog::<Theme>(),
        "Les normes du document : papier, marges, titres, citations (Ctrl+P)",
        Message::MiseEnPageOuvrir,
        palette,
        state
            .mise_en_page
            .as_ref()
            .is_some_and(|dialogue| !dialogue.plugins),
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_puzzle::<Theme>(),
        "Les paramètres des plugins : bibliographie, biblique, glossaire, graphiques, callouts",
        Message::PluginsOuvrir,
        palette,
        state
            .mise_en_page
            .as_ref()
            .is_some_and(|dialogue| dialogue.plugins),
    ));

    bar = bar.push(separation(palette));

    if !state.layout.focus_mode {
        bar = bar.push(icon_button(
            if state.layout.sidebar_visible {
                lucide_icons::iced::icon_panel_left_close::<Theme>()
            } else {
                lucide_icons::iced::icon_panel_left_open::<Theme>()
            },
            if state.layout.sidebar_visible {
                "Masquer le panneau de travail"
            } else {
                "Afficher le panneau de travail"
            },
            Message::ToggleSidebar,
            palette,
            state.layout.sidebar_visible,
        ));
    }

    bar = bar.push(icon_button(
        if state.layout.focus_mode {
            lucide_icons::iced::icon_minimize_2::<Theme>()
        } else {
            lucide_icons::iced::icon_maximize_2::<Theme>()
        },
        if state.layout.focus_mode {
            "Quitter le mode focus (F11)"
        } else {
            "Mode focus — le texte seul (F11)"
        },
        Message::ToggleFocusMode,
        palette,
        state.layout.focus_mode,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_settings::<Theme>(),
        "Paramètres de l'application (Ctrl+,)",
        Message::ParametresOuvrir,
        palette,
        state.parametres_vue,
    ));

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(BAR_HEIGHT))
        .padding([0, 8])
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

/// Le menu du projet : les gestes de fichier et d'archive, sans les gestes
/// d'écriture qui ont leur propre menu.
fn projet_menu<'a>(palette: Palette) -> Element<'a, Message> {
    let items = vec![
        menu_item(palette, "Nouveau projet…", Message::ProjetCreer),
        menu_item(
            palette,
            "Définir comme modèle, ou l'en retirer",
            Message::ProjetModeleBascule,
        ),
        menu_item(palette, "Exporter en .tw unique…", Message::ProjetExportTw),
        menu_item(palette, "Importer un .tw unique…", Message::ProjetImportTw),
        menu_item(palette, "Importer un document .typ…", Message::OpenFile),
        menu_item(
            palette,
            "Exporter le projet (archive)…",
            Message::ProjetExportPortable,
        ),
        menu_item(
            palette,
            "Importer une sauvegarde…",
            Message::ProjetImportSauvegarde,
        ),
        menu_item(palette, "Exporter en Markdown…", Message::ExportMarkdown),
        menu_item(palette, "Exporter en DOCX…", Message::ExportDocx),
    ];

    iced_aw::menu::MenuBar::new(vec![iced_aw::menu::Item::with_menu(
        menu_declencheur(
            palette,
            lucide_icons::iced::icon_folder::<Theme>(),
            "Projet",
        ),
        iced_aw::menu::Menu::new(items).width(Length::Fixed(MENU_WIDTH)),
    )])
    .style(menu_bar_style(palette))
    .into()
}

/// Un déclencheur de menu de la barre : une petite surface, une icône et un
/// libellé court, plutôt qu'un simple mot posé à plat.
fn menu_declencheur<'a>(
    palette: Palette,
    icone: iced::widget::Text<'a, Theme>,
    titre: &'a str,
) -> Element<'a, Message> {
    container(
        row![
            theme::icon(icone, 14.0, palette.text_muted),
            palette.label(titre).size(theme::jetons::DETAIL),
            theme::icon(
                lucide_icons::iced::icon_chevron_down::<Theme>(),
                10.0,
                palette.text_muted
            ),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([5, 8])
    .style(move |_| container::Style {
        background: Some(palette.surface_hover.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 7.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Le style des menus de la barre.
///
/// Le style par défaut d'`iced_aw` peint la racine active avec la couleur
/// primaire du thème : elle devenait une grande plaque d'accent sous le
/// bouton. La barre reste transparente et seule la carte du menu porte une
/// surface, avec la bordure des panneaux de l'application.
fn menu_bar_style(
    palette: Palette,
) -> impl Fn(&Theme, iced_aw::style::Status) -> iced_aw::style::menu_bar::Style {
    move |_, _| iced_aw::style::menu_bar::Style {
        bar_background: iced::Color::TRANSPARENT.into(),
        bar_border: iced::Border::default(),
        bar_shadow: iced::Shadow::default(),
        menu_background: palette.surface.into(),
        menu_border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        menu_shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        path: palette.surface_hover.into(),
        path_border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 6.0.into(),
        },
    }
}

/// Le menu des gestes de contenu.
///
/// Il est le seul endroit où les insertions se rangent : une icône par outil
/// faisait de la barre une planche de pictogrammes, et la même icône servait
/// à deux gestes différents.
fn inserer_menu<'a>(state: &State, palette: Palette) -> Element<'a, Message> {
    let mut items = Vec::new();

    // Les insertions de texte : elles restent accessibles même quand les
    // boutons de mise en page sont masqués.
    items.push(menu_item(
        palette,
        "Note de bas de page…",
        Message::NoteInserer,
    ));
    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
    {
        items.push(menu_item(
            palette,
            "Citation bibliographique…",
            Message::CitationOuvrir,
        ));
    }
    items.push(menu_item(
        palette,
        "Bloc Typst brut",
        Message::Format(crate::app::message::FormatAction::BlocTypst),
    ));

    items.push(menu_item(palette, "Sommaire", Message::SommaireInserer));
    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
    {
        items.push(menu_item(
            palette,
            "Section bibliographie",
            Message::BibliographieSectionInserer,
        ));
    }
    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_GLOSSAIRE)
    {
        items.push(menu_item(
            palette,
            "Section glossaire",
            Message::GlossaireSectionInserer,
        ));
    }
    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
    {
        items.push(menu_item(
            palette,
            "Référence biblique…",
            Message::ReferenceBibliqueInserer,
        ));
        items.push(menu_item(
            palette,
            "Index des citations bibliques",
            Message::BibleIndexInserer,
        ));
    }
    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_GLOSSAIRE)
    {
        items.push(menu_item(
            palette,
            "Consulter le glossaire",
            Message::GlossaireFenetreOuvrir,
        ));
    }

    items.push(menu_item(palette, "Image…", Message::ImageInserer));
    items.push(menu_item(palette, "Tableau…", Message::TableauOuvrir));
    items.push(menu_item(
        palette,
        "Aligner les tableaux",
        Message::BeautifierTableaux,
    ));
    if state.reglages.fonctionnalites.graphiques {
        items.push(menu_item(palette, "Graphique…", Message::ChartOuvrir));
    }
    if state.reglages.fonctionnalites.callouts {
        items.push(menu_item(palette, "Callout…", Message::CalloutOuvrir));
    }
    items.push(menu_item(palette, "Extension…", Message::ExtensionOuvrir));

    iced_aw::menu::MenuBar::new(vec![iced_aw::menu::Item::with_menu(
        menu_declencheur(palette, lucide_icons::iced::icon_plus::<Theme>(), "Insérer"),
        iced_aw::menu::Menu::new(items).width(Length::Fixed(MENU_WIDTH)),
    )])
    .style(menu_bar_style(palette))
    .into()
}

fn menu_item<'a>(
    palette: Palette,
    label: impl Into<String>,
    message: Message,
) -> iced_aw::menu::Item<'a, Message, Theme, iced::Renderer> {
    iced_aw::menu::Item::new(
        button(palette.label(label.into()).size(theme::jetons::DETAIL))
            .on_press(message)
            .padding([5, 10])
            .width(Length::Fill)
            .style(move |_, status| menu_item_style(status, palette)),
    )
}

/// Une ligne de menu : neutre au repos, une surface douce au survol.
///
/// Le style de bouton par défaut d'iced prenait la couleur primaire du
/// thème au survol : les choix paraissaient tous « actifs » et formaient
/// une grande plaque d'accent dans le menu.
fn menu_item_style(status: button::Status, palette: Palette) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered => Some(palette.surface_hover.into()),
            button::Status::Pressed => Some(palette.surface_active.into()),
            _ => None,
        },
        text_color: palette.text,
        border: iced::Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// La dernière couleur appliquée : une pastille, pas un nom.
fn bouton_couleur<'a>(state: &State, palette: Palette) -> Element<'a, Message> {
    let derniere = state
        .derniere_couleur
        .clone()
        .unwrap_or_else(|| "red".to_string());
    let couleur_actuelle = COULEURS
        .iter()
        .find(|(_, nom, _)| *nom == derniere)
        .and_then(|(_, _, hex)| crate::theming::HexColor::parse(hex).ok().map(|c| c.0))
        .or_else(|| crate::theming::HexColor::parse(&derniere).ok().map(|c| c.0));

    let pastille = button(
        container(
            Space::new()
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0)),
        )
        .style(move |_| container::Style {
            background: couleur_actuelle.map(iced::Background::from),
            border: iced::Border {
                width: 1.0,
                color: palette.border,
                radius: 3.0.into(),
            },
            ..Default::default()
        }),
    )
    .on_press(Message::Format(crate::app::message::FormatAction::Couleur(
        derniere,
    )))
    .padding(4)
    .style(move |_, statut| button_style(statut, palette));

    iced::widget::tooltip(
        pastille,
        container(
            palette
                .label("Appliquer la dernière couleur choisie")
                .size(12)
                .color(palette.text),
        )
        .padding([4, 8])
        .style(move |_| container::Style {
            background: Some(palette.surface_hover.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Un bouton-icône, sans bordure et sans fond au repos.
///
/// `active` ne signale pas une action mais un état : panneau ouvert,
/// document modifié.
fn icon_button<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    tooltip: &'static str,
    message: Message,
    palette: Palette,
    active: bool,
) -> Element<'a, Message> {
    let color = if active {
        palette.accent
    } else {
        palette.text_muted
    };

    let button = button(theme::icon(glyph, ICON_SIZE, color))
        .on_press(message)
        .padding([6, 7])
        .width(Length::Fixed(BUTTON))
        .height(Length::Fixed(BUTTON))
        .style(move |_, status| button_style(status, palette));

    iced::widget::tooltip(
        button,
        container(palette.label(tooltip).size(12).color(palette.text))
            .padding([4, 8])
            .style(move |_| container::Style {
                background: Some(palette.surface_hover.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Bouton compact de mise en forme, affiché pour `document.tw`.
fn format_button<'a>(
    label: &'static str,
    help: &'static str,
    action: crate::app::message::FormatAction,
    palette: Palette,
) -> Element<'a, Message> {
    toolbar_text_button(label, help, Message::Format(action), palette)
}

/// Le menu des niveaux de titre : un menu qui reste ouvert jusqu'au choix
/// ou au clic extérieur, et se conduit au clavier. TOPBAR-04-04.
fn titre_menu(palette: Palette) -> Element<'static, Message> {
    let mut choix = Vec::new();

    for niveau in 1..=6u8 {
        choix.push(menu_item(
            palette,
            format!("Titre {niveau}"),
            Message::Format(crate::app::message::FormatAction::Titre(niveau)),
        ));
    }

    iced_aw::menu::MenuBar::new(vec![iced_aw::menu::Item::with_menu(
        theme::icon(
            lucide_icons::iced::icon_heading::<Theme>(),
            15.0,
            palette.text_muted,
        ),
        iced_aw::menu::Menu::new(choix).width(Length::Fixed(TITRE_MENU_WIDTH)),
    )])
    .width(Length::Fixed(BUTTON))
    .height(Length::Fixed(BUTTON))
    .style(menu_bar_style(palette))
    .into()
}

/// Bouton compact avec infobulle, partagé par les commandes de texte.
fn toolbar_text_button<'a>(
    label: &'static str,
    help: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let button = button(
        container(palette.label(label).size(12).color(palette.text))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(message)
    .padding([4, 6])
    .width(Length::Fixed(BUTTON))
    .height(Length::Fixed(BUTTON))
    .style(move |_, status| button_style(status, palette));

    iced::widget::tooltip(
        button,
        container(palette.label(help).size(12).color(palette.text))
            .padding([4, 8])
            .style(move |_| container::Style {
                background: Some(palette.surface_hover.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Bouton-icône : invisible au repos, un fond doux au survol.
fn button_style(status: button::Status, palette: Palette) -> button::Style {
    let background = match status {
        button::Status::Active => None,
        button::Status::Hovered => Some(palette.surface_hover),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => None,
    };

    button::Style {
        background: background.map(Into::into),
        text_color: palette.text_muted,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Un filet entre deux clusters : les sépare sans les couper.
fn separation(palette: Palette) -> Element<'static, Message> {
    iced::widget::container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(18.0))
        .style(move |_| container::Style {
            background: Some(palette.border.into()),
            ..Default::default()
        })
        .into()
}

/// Espacement élastique : pousse ce qui suit vers la droite.
fn spacer() -> Space {
    Space::new().width(Length::Fill).height(Length::Shrink)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le menu ne doit jamais prendre la couleur d'accent comme fond : elle
    /// transformait les déclencheurs en grandes plaques colorées.
    #[test]
    fn le_fond_du_menu_n_est_pas_l_accent() {
        let palette = Palette::from_theme(&iced::Theme::Dark);
        let style = menu_bar_style(palette)(&iced::Theme::Dark, iced_aw::style::Status::Active);

        assert_ne!(
            style.path,
            iced::Background::from(palette.accent),
            "le chemin actif reste une surface neutre"
        );
        assert_eq!(
            style.path,
            iced::Background::from(palette.surface_hover),
            "le chemin actif suit la surface de survol"
        );
        assert_eq!(
            style.bar_background,
            iced::Background::from(iced::Color::TRANSPARENT),
            "la barre elle-même ne peint rien"
        );
    }

    /// Les lignes du menu ne prennent pas l'accent au survol : elles restent
    /// une surface neutre, sinon le menu paraît entièrement sélectionné.
    #[test]
    fn une_ligne_de_menu_ne_prend_pas_l_accent() {
        let palette = Palette::from_theme(&iced::Theme::Dark);
        let style = menu_item_style(button::Status::Hovered, palette);

        assert_eq!(
            style.background,
            Some(palette.surface_hover.into()),
            "la ligne survolée reste une surface"
        );
        assert_ne!(
            style.background,
            Some(palette.accent.into()),
            "l'accent ne doit pas peindre une ligne de menu"
        );
    }
}
