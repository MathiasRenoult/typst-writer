use crate::app::state::PreviewPosition;
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
const COULEURS: [(&str, &str); 7] = [
    ("Rouge", "red"),
    ("Bleu", "blue"),
    ("Vert", "green"),
    ("Orange", "orange"),
    ("Violet", "purple"),
    ("Noir", "black"),
    ("Gris", "gray"),
];

/// Les niveaux de titre proposés, du plus haut au plus bas.
const TITRES: [(&str, u8); 3] = [("H1", 1), ("H2", 2), ("H3", 3)];

/// La barre d'outils : des icônes, pas des mots.
///
/// Chaque bouton porte une infobulle — sans elle une icône seule est une
/// devinette, et le gain de place serait payé en friction. Les couleurs
/// viennent de la palette du thème : la barre suit le clair et le sombre.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let mut bar = row![].spacing(SPACING).align_y(Alignment::Center);

    // Le nom du document, à gauche : repère discret, il ne prend pas la place
    // d'un bouton.
    bar = bar.push(
        palette
            .label(state.editor.filename())
            .size(13)
            .color(palette.text_muted),
    );

    if state.editor.is_intermediate() {
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
        bar = bar.push(format_picker(
            TITRES
                .iter()
                .map(|(libelle, _)| (*libelle).to_string())
                .collect(),
            "Titre",
            |choisi| {
                Message::Format(crate::app::message::FormatAction::Titre(
                    niveau_titre(&choisi).unwrap_or(1),
                ))
            },
            palette,
            64.0,
        ));
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
        bar = bar.push(format_picker(
            COULEURS
                .iter()
                .map(|(libelle, _)| (*libelle).to_string())
                .collect(),
            "Couleur",
            |choisi| {
                Message::Format(crate::app::message::FormatAction::Couleur(
                    couleur_typst(&choisi).unwrap_or("red").to_string(),
                ))
            },
            palette,
            96.0,
        ));
        bar = bar.push(format_button(
            "N",
            "Note de bas de page sur la sélection",
            crate::app::message::FormatAction::Note,
            palette,
        ));
        bar = bar.push(format_button(
            "@",
            "Insérer une citation bibliographique",
            crate::app::message::FormatAction::Citation,
            palette,
        ));
        bar = bar.push(format_button(
            "T",
            "Entourer la sélection d'un bloc Typst brut",
            crate::app::message::FormatAction::BlocTypst,
            palette,
        ));
    }

    bar = bar.push(spacer());

    // Panneaux : ces trois-là gouvernent la disposition, ils vont ensemble.
    // En mode focus ils disparaissent — les afficher donnerait à voir des
    // commandes sans effet, et le mode focus n'a de sens que s'il ne reste que
    // le texte.
    if !state.layout.focus_mode {
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_panel_left::<Theme>(),
            if state.layout.sidebar_visible {
                "Masquer le panneau de travail"
            } else {
                "Afficher le panneau de travail"
            },
            Message::ToggleSidebar,
            palette,
            state.layout.sidebar_visible,
        ));

        bar = bar.push(icon_button(
            lucide_icons::iced::icon_eye::<Theme>(),
            if state.layout.preview_visible {
                "Masquer l'aperçu"
            } else {
                "Afficher l'aperçu"
            },
            Message::TogglePreview,
            palette,
            state.layout.preview_visible,
        ));

        // Le suivi du chapitre évite de croire qu'un chapitre n'est pas rendu ;
        // il arrache aussi l'aperçu sous les yeux dès qu'on écrit ailleurs dans
        // l'assemblage. Les deux usages sont légitimes : c'est donc un
        // interrupteur, et l'icône dit lequel des deux états est en vigueur.
        bar = bar.push(icon_button(
            if state.layout.follow_preview {
                lucide_icons::iced::icon_link::<Theme>()
            } else {
                lucide_icons::iced::icon_unlink::<Theme>()
            },
            if state.layout.follow_preview {
                "L'aperçu suit le chapitre écrit — cliquer pour couper le suivi"
            } else {
                "L'aperçu reste où vous le laissez — cliquer pour suivre le chapitre"
            },
            Message::TogglePreviewSync,
            palette,
            state.layout.follow_preview,
        ));

        if state.layout.preview_visible {
            bar = bar.push(toolbar_text_button(
                "W",
                "Ajuster l'aperçu à la largeur",
                Message::PreviewZoom(crate::app::state::PreviewZoomMode::FitWidth),
                palette,
            ));
            bar = bar.push(toolbar_text_button(
                "H",
                "Ajuster l'aperçu à la hauteur",
                Message::PreviewZoom(crate::app::state::PreviewZoomMode::FitHeight),
                palette,
            ));
            bar = bar.push(toolbar_text_button(
                "−",
                "Réduire le zoom de 10 %",
                Message::PreviewZoomStep(-1),
                palette,
            ));
            bar = bar.push(
                palette
                    .label(
                        crate::ui::preview::zoom_percent(state).unwrap_or_else(|| "—".to_string()),
                    )
                    .size(11)
                    .color(palette.text_muted),
            );
            bar = bar.push(toolbar_text_button(
                "+",
                "Agrandir le zoom de 10 %",
                Message::PreviewZoomStep(1),
                palette,
            ));
        }

        // Les normes du document : papier, marges, titres, couleurs.
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_file_cog::<Theme>(),
            "Les normes du document : papier, marges, titres, couleurs",
            Message::MiseEnPageOuvrir,
            palette,
            state.mise_en_page.is_some(),
        ));

        // Renommer le symbole sous le curseur : le serveur montrera ce que
        // cela changerait, fichier par fichier, avant que rien ne s'écrive.
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_pen_line::<Theme>(),
            "Renommer le symbole sous le curseur",
            Message::SymbolRenameStart,
            palette,
            state.renommage.is_some(),
        ));

        bar = bar.push(icon_button(
            preview_position_icon(state.layout.preview_position),
            state.layout.preview_position.label(),
            Message::SetPreviewPosition(state.layout.preview_position.next()),
            palette,
            false,
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

    bar = bar.push(spacer());

    // Fichier.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file::<Theme>(),
        "Ouvrir le document d'écriture du projet",
        Message::NewFile,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_folder_open::<Theme>(),
        // Le libellé dit ce que fait l'action : le fichier est ajouté au projet,
        // il n'est pas seulement affiché.
        "Charger un document dans le projet",
        Message::OpenFile,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_save::<Theme>(),
        "Enregistrer",
        Message::SaveFile,
        palette,
        state.editor.is_modified(),
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_save_all::<Theme>(),
        "Enregistrer sous…",
        Message::SaveAs,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_download::<Theme>(),
        "Exporter en PDF",
        Message::ExportPdf,
        palette,
        false,
    ));

    bar = bar.push(spacer());

    // Thèmes : la liste, puis le mode d'apparence, puis le chargement et la
    // suppression. Les quatre vont ensemble, ils sont donc côte à côte.
    let names: Vec<String> = state
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect();

    let current = state.current_theme().map(|theme| theme.name.clone());

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

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_refresh_cw::<Theme>(),
        "Relire le dossier des thèmes",
        Message::RefreshThemes,
        palette,
        false,
    ));

    // Un thème s'écrit à la main : sans ce bouton, le seul moyen d'atteindre le
    // dossier était de connaître son chemin par cœur.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_folder_cog::<Theme>(),
        "Ouvrir le dossier des thèmes",
        Message::OpenThemesFolder,
        palette,
        false,
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

/// Le nom Typst de la couleur désignée par son libellé français.
fn couleur_typst(label: &str) -> Option<&'static str> {
    COULEURS
        .iter()
        .find(|(libelle, _)| *libelle == label)
        .map(|(_, nom)| *nom)
}

/// Le niveau du titre désigné par son libellé.
fn niveau_titre(label: &str) -> Option<u8> {
    TITRES
        .iter()
        .find(|(libelle, _)| *libelle == label)
        .map(|(_, niveau)| *niveau)
}

/// Une liste déroulante qui applique une mise en forme au choix.
///
/// Contrairement à un bouton, elle **montre** les possibilités avant le clic :
/// c'est ce qui rend le choix d'un niveau de titre ou d'une couleur immédiat,
/// au lieu d'une suite de clics à l'aveugle. Comme c'est une action et non un
/// état, rien n'y est présélectionné : le libellé vide dit ce qu'elle propose.
fn format_picker<'a>(
    options: Vec<String>,
    placeholder: &'static str,
    map: impl Fn(String) -> Message + 'a,
    palette: Palette,
    width: f32,
) -> Element<'a, Message> {
    pick_list(options, None::<String>, map)
        .placeholder(placeholder)
        .text_size(12)
        .font(palette.interface)
        .padding([4, 8])
        .width(Length::Fixed(width))
        .into()
}

/// Bouton compact avec infobulle, partagé par les commandes de texte et zoom.
fn toolbar_text_button<'a>(
    label: &'static str,
    help: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let button = button(palette.label(label).size(12).color(palette.text))
        .on_press(message)
        .padding([4, 6])
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
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
fn spacer() -> Space {
    Space::new().width(Length::Fill).height(Length::Shrink)
}

/// Icône de la position de l'aperçu : elle montre où il ira au prochain clic.
fn preview_position_icon(position: PreviewPosition) -> iced::widget::Text<'static, Theme> {
    match position {
        PreviewPosition::Right => lucide_icons::iced::icon_panel_right::<Theme>(),
        PreviewPosition::Left => lucide_icons::iced::icon_panel_left::<Theme>(),
        PreviewPosition::Bottom => lucide_icons::iced::icon_panel_bottom::<Theme>(),
    }
}

/// Icône du thème : on voit d'un coup d'œil ce qui est actif.
fn theme_icon(mode: theme::ThemeMode) -> iced::widget::Text<'static, Theme> {
    match mode {
        theme::ThemeMode::System => lucide_icons::iced::icon_sun_moon::<Theme>(),
        theme::ThemeMode::Light => lucide_icons::iced::icon_sun::<Theme>(),
        theme::ThemeMode::Dark => lucide_icons::iced::icon_moon::<Theme>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque libellé de couleur donne son nom Typst, et rien d'autre.
    #[test]
    fn colour_labels_map_to_typst_names() {
        assert_eq!(couleur_typst("Rouge"), Some("red"));
        assert_eq!(couleur_typst("Violet"), Some("purple"));
        assert_eq!(couleur_typst("Inexistant"), None);
    }

    /// Chaque libellé de titre donne son niveau.
    #[test]
    fn heading_labels_map_to_levels() {
        assert_eq!(niveau_titre("H1"), Some(1));
        assert_eq!(niveau_titre("H3"), Some(3));
        assert_eq!(niveau_titre("H4"), None);
    }
}
