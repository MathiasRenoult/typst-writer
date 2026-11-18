use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, container, pick_list, row};
use iced::{Alignment, Element, Length, Theme};

/// Taille des icônes de la barre.
const ICON_SIZE: f32 = 17.0;

/// Espacement entre les boutons.
const SPACING: f32 = 2.0;

/// Côté d'un bouton-icône : carré, pour que la cible de clic soit généreuse
/// sans que l'icône elle-même prenne de la place.
const BUTTON: f32 = 30.0;

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

/// La barre d'outils : des icônes, pas des mots.
///
/// Chaque bouton porte une infobulle — sans elle une icône seule est une
/// devinette, et le gain de place serait payé en friction. Les couleurs
/// viennent de la palette du thème : la barre suit le clair et le sombre.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let mut bar = row![]
        .spacing(SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // TOPBAR-06-01 : le bouton de fermeture de l'onglet actif, à gauche.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_x::<Theme>(),
        "Fermer l'onglet actif",
        Message::VueFermer(state.espace.active),
        palette,
        false,
    ));

    // TOPBAR-04-10 : les normes à gauche — la mise en page est le
    // cadre du travail, pas une commande parmi d'autres.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file_cog::<Theme>(),
        "Les normes du document : papier, marges, titres, couleurs (Ctrl+P)",
        Message::MiseEnPageOuvrir,
        palette,
        state
            .mise_en_page
            .as_ref()
            .is_some_and(|dialogue| !dialogue.plugins),
    ));

    // Les paramètres des plugins sont un panneau distinct des normes :
    // activer un plugin se fait dans les normes, le configurer se fait ici.
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

    // Le graphique et le callout vivent avec les autres gestes d'insertion,
    // dans le groupe central de la barre — pas à côté des commandes de
    // projet. Un add-on débranché n'expose pas un bouton mort.

    bar = bar.push(separation(palette));

    // Fichier : les commandes essentielles. L'ouverture, l'import et la
    // sauvegarde se font par le clavier ou la bibliothèque — TOPBAR-04-02.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file::<Theme>(),
        "Ouvrir le document d'écriture du projet (Ctrl+N)",
        Message::NewFile,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_library::<Theme>(),
        "Les projets et leurs étagères",
        Message::ProjetsVue,
        palette,
        state.projets_vue,
    ));

    // TOPBAR-07 : le menu « Projet » — les gestes du projet réunis dans
    // un menu déroulant de la barre supérieure (bug-report-7, point 14).
    bar = bar.push(iced_aw::menu::MenuBar::new(vec![
        iced_aw::menu::Item::with_menu(
            palette.label("Projet").size(theme::jetons::DETAIL),
            iced_aw::menu::Menu::new(vec![
                iced_aw::menu::Item::new(
                    button(palette.label("Nouveau projet…").size(theme::jetons::DETAIL))
                        .on_press(Message::ProjetCreer)
                        .padding([4, 10])
                        .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Définir comme modèle, ou l'en retirer")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ProjetModeleBascule)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Insérer une note de bas de page…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::NoteInserer)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Insérer un graphique…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ChartOuvrir)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Insérer une extension…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ExtensionOuvrir)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Exporter en .tw unique…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ProjetExportTw)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Importer un .tw unique…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ProjetImportTw)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Importer un document .typ…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::OpenFile)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Exporter le projet (archive)…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ProjetExportPortable)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Importer une sauvegarde…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ProjetImportSauvegarde)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Exporter en Markdown…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ExportMarkdown)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
                iced_aw::menu::Item::new(
                    button(
                        palette
                            .label("Exporter en DOCX…")
                            .size(theme::jetons::DETAIL),
                    )
                    .on_press(Message::ExportDocx)
                    .padding([4, 10])
                    .width(Length::Fill),
                ),
            ]),
        ),
    ]));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_download::<Theme>(),
        "Exporter en PDF (Ctrl+E)",
        Message::ExportPdf,
        palette,
        false,
    ));

    // La palette : le point d'entrée unique des commandes. Le menu `..` du
    // texte reste le chemin d'insertion ; `Ctrl+K` ouvre celui-ci partout.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_command::<Theme>(),
        "Commandes (Ctrl+K) — balises dans le texte avec ..",
        Message::PaletteOuvrir(crate::app::palette::ModePalette::Systeme),
        palette,
        state.palette.is_some(),
    ));

    // TOPBAR-08 : la barre suit le mode (bug-report-7, point 1). Le zoom
    // n'a de sens que sur un rendu, les outils d'éditent que sur le
    // texte — les communs (normes, recherche, projet) restent partout.
    let sur_rendu = matches!(state.mode, crate::app::state::ModeVue::Rendu);

    // TOPBAR-06-02 : les commandes de panneau et focus vont dans le
    // groupe général à gauche — elles ne relèvent ni de l'édition ni du thème.
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

    bar = bar.push(separation(palette));

    // TOPBAR-06-02 : le format est centré entre deux espacements flexibles.
    bar = bar.push(spacer());

    // Le zoom se centre avec le format (bug-report-9, T2.1) : les
    // quatre commandes et leur pourcentage, dans le segment central —
    // la fenêtre de rendu ne porte plus les siennes (T2.2).
    if sur_rendu {
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_zoom_out::<Theme>(),
            "Réduire le zoom",
            Message::PreviewZoomStep(-1),
            palette,
            false,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_scan::<Theme>(),
            "Ajuster à la largeur",
            Message::PreviewZoom(crate::app::state::PreviewZoomMode::FitWidth),
            palette,
            false,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_maximize::<Theme>(),
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
    }

    // Les outils d'édition : sur le texte seulement — jamais sur un rendu
    // (bug-report-7, point 1).
    if state.editor.is_intermediate() && !sur_rendu {
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

        bar = bar.push(format_button(
            "•",
            "Basculer la ligne en liste à puces",
            crate::app::message::FormatAction::ListePuce,
            palette,
        ));
        bar = bar.push(format_button(
            "1.",
            "Basculer la ligne en liste numérotée",
            crate::app::message::FormatAction::ListeNumero,
            palette,
        ));
        bar = bar.push(format_button(
            ">",
            "Basculer la ligne en citation de bloc",
            crate::app::message::FormatAction::CitationBloc,
            palette,
        ));

        // TOPBAR-09 : les sections de fin s'insèrent au curseur — la
        // bibliographie, le glossaire, et un mot du glossaire déjà
        // référencé (bug-report-9).
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_list::<Theme>(),
            "Insérer une table des matières",
            Message::SommaireInserer,
            palette,
            false,
        ));
        if state
            .reglages
            .fonctionnalites
            .actif(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
        {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_library::<Theme>(),
                "Insérer une section bibliographie",
                Message::BibliographieSectionInserer,
                palette,
                false,
            ));
        }
        if state
            .reglages
            .fonctionnalites
            .actif(crate::base::valeurs::CLE_GLOSSAIRE)
        {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_book_a::<Theme>(),
                "Insérer une section glossaire",
                Message::GlossaireSectionInserer,
                palette,
                false,
            ));
        }
        if state
            .reglages
            .fonctionnalites
            .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
        {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_book_open::<Theme>(),
                "Insérer une référence biblique",
                Message::ReferenceBibliqueInserer,
                palette,
                false,
            ));
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_list_tree::<Theme>(),
                "Insérer l'index des citations bibliques",
                Message::BibleIndexInserer,
                palette,
                false,
            ));
        }

        // TOPBAR-10 : une icône ouvre la fenêtre du glossaire — la
        // liste déroulante ne permettait que d'insérer (bug-report-10, 5.3).
        if state
            .reglages
            .fonctionnalites
            .actif(crate::base::valeurs::CLE_GLOSSAIRE)
        {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_book_marked::<Theme>(),
                "Glossaire : consulter et gérer les termes",
                Message::GlossaireFenetreOuvrir,
                palette,
                state.plugins.glossaire.fenetre,
            ));
        }

        // TOPBAR-06 : la pastille applique la dernière couleur choisie,
        // le picker ouvre le choix. La pastille mémorise la couleur courante.
        let derniere = state
            .derniere_couleur
            .clone()
            .unwrap_or_else(|| "red".to_string());
        // TOPBAR-04-06 : la pastille montre la vraie couleur, pas un nom.
        let couleur_actuelle = COULEURS
            .iter()
            .find(|(_, nom, _)| *nom == derniere)
            .and_then(|(_, _, hex)| crate::theming::HexColor::parse(hex).ok().map(|c| c.0))
            .or_else(|| crate::theming::HexColor::parse(&derniere).ok().map(|c| c.0));

        let pastille = iced::widget::button(
            iced::widget::container(
                iced::widget::Space::new()
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0)),
            )
            .style(move |_t| iced::widget::container::Style {
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
            derniere.clone(),
        )))
        .padding(4)
        .style(move |_, s| button_style(s, palette));

        bar = bar.push(
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
            .gap(6),
        );

        // Le sélecteur de couleur : une roue, centrée dans la fenêtre —
        // le choix se fait en regardant, pas en cherchant un nom de couleur.
        bar = bar.push(
            button(palette.label("Couleur…").size(theme::jetons::DETAIL))
                .on_press(Message::PickerCouleurOuvrir)
                .padding([4, 8])
                .style(move |_, s| button_style(s, palette)),
        );
        bar = bar.push(format_button(
            "N",
            "Note de bas de page sur la sélection",
            crate::app::message::FormatAction::Note,
            palette,
        ));
        if state
            .reglages
            .fonctionnalites
            .actif(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
        {
            bar = bar.push(toolbar_text_button(
                "@",
                "Insérer une citation bibliographique",
                Message::CitationOuvrir,
                palette,
            ));
        }
        bar = bar.push(format_button(
            "T",
            "Entourer la sélection d'un bloc Typst brut",
            crate::app::message::FormatAction::BlocTypst,
            palette,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_image::<Theme>(),
            "Insérer une image",
            Message::ImageInserer,
            palette,
            false,
        ));

        if state.reglages.fonctionnalites.graphiques {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_chart_column::<Theme>(),
                "Insérer un graphique : secteurs, barres, lignes, aires, points, boîtes",
                Message::ChartOuvrir,
                palette,
                state.chart.is_some(),
            ));
        }

        if state.reglages.fonctionnalites.callouts {
            bar = bar.push(icon_button(
                lucide_icons::iced::icon_text_quote::<Theme>(),
                "Insérer un callout : encadrer une citation",
                Message::CalloutOuvrir,
                palette,
                state.callout.is_some(),
            ));
        }

        bar = bar.push(icon_button(
            lucide_icons::iced::icon_table::<Theme>(),
            "Insérer un tableau",
            Message::TableauOuvrir,
            palette,
            false,
        ));

        bar = bar.push(separation(palette));
    }

    // TOPBAR-06-02 : le groupe de thème est à droite, séparé par un
    // espacement flexible. Le format est déjà centré par le spacer précédent.
    bar = bar.push(Space::new().width(Length::Fill));

    // Thèmes : la liste, puis le mode d'apparence, puis le chargement et la
    // suppression. Les quatre vont ensemble, ils sont donc côte à côte.
    let names: Vec<String> = state
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect();

    let current = state.current_theme().map(|theme| theme.name.clone());

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_palette::<Theme>(),
        "L'éditeur de thème",
        Message::ThemeEditeurOuvrir,
        palette,
        state.editeur_theme.is_some(),
    ));

    bar = bar.push(
        pick_list(names, current, Message::SelectTheme)
            .text_size(12)
            // La liste suit la police du thème comme le reste de l'interface :
            // sans cela, choisir une famille laissait ce seul contrôle dans
            // celle du moteur.
            .font(palette.interface)
            .padding([4, 8])
            .width(Length::Fixed(168.0)),
    );

    bar = bar.push(icon_button(
        theme_icon(state.theme),
        state.theme.label(),
        Message::CycleTheme,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_download::<Theme>(),
        "Charger un thème (fichier .toml)",
        Message::LoadTheme,
        palette,
        false,
    ));

    // Supprimer n'a de sens que pour un thème chargé : les thèmes natifs n'ont
    // pas de fichier, et un bouton qui ne ferait rien serait un mensonge.
    if let Some(theme) = state.current_theme()
        && !theme.is_native()
    {
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_trash_2::<Theme>(),
            "Supprimer ce thème",
            Message::DeleteTheme(theme.name.clone()),
            palette,
            false,
        ));
    }

    // TOPBAR-06-02 : pas de scrollable — il empêche Space::Fill de
    // s'étendre et les icônes restent collées à gauche. Le container
    // donne toute la largeur au row, les trois groupes se répartissent.
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

    // L'infobulle est ce qui rend une icône seule acceptable : sans elle, un
    // bouton de 30 px est une devinette, et le gain de place serait payé en
    // friction.
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

/// Le menu des niveaux de titre : un `pick_list`, pas un `tooltip`.
///
/// Un `tooltip` se ferme dès que la souris quitte le bouton — impossible
/// de choisir un niveau. Un `pick_list` reste ouvert jusqu'au choix ou au
/// clic extérieur, et se conduit au clavier. TOPBAR-04-04.
fn titre_menu(palette: Palette) -> Element<'static, Message> {
    // TOPBAR-10 : une icône ouvre le menu des styles de titre — la
    // liste déroulante prenait la place d'un geste (bug-report-10, 5.2).
    let mut choix = Vec::new();

    for niveau in 1..=6u8 {
        choix.push(iced_aw::menu::Item::new(
            button(
                palette
                    .label(format!("Titre {niveau}"))
                    .size(theme::jetons::DETAIL),
            )
            .on_press(Message::Format(crate::app::message::FormatAction::Titre(
                niveau,
            )))
            .padding([4, 10])
            .width(Length::Fill),
        ));
    }

    iced_aw::menu::MenuBar::new(vec![iced_aw::menu::Item::with_menu(
        theme::icon(
            lucide_icons::iced::icon_heading::<Theme>(),
            15.0,
            palette.text_muted,
        ),
        iced_aw::menu::Menu::new(choix),
    )])
    .into()
}

/// Bouton compact avec infobulle, partagé par les commandes de texte et zoom.
fn toolbar_text_button<'a>(
    label: &'static str,
    help: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    // TOPBAR-04-03 : le libellé est centré dans le bouton.
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

/// Espacement élastique : pousse ce qui suit vers la droite.
/// Un filet entre deux clusters : les sépare sans les couper.
///
/// Un trait de la couleur des bordures, haut comme les boutons : la barre
/// se lit en groupes, et chaque groupe a sa raison d'être ensemble.
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

fn spacer() -> Space {
    Space::new().width(Length::Fill).height(Length::Shrink)
}

/// Icône du thème : on voit d'un coup d'œil ce qui est actif.
fn theme_icon(mode: theme::ThemeMode) -> iced::widget::Text<'static, Theme> {
    match mode {
        theme::ThemeMode::System => lucide_icons::iced::icon_sun_moon::<Theme>(),
        theme::ThemeMode::Light => lucide_icons::iced::icon_sun::<Theme>(),
        theme::ThemeMode::Dark => lucide_icons::iced::icon_moon::<Theme>(),
    }
}
