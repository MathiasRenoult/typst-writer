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
const BAR_HEIGHT: f32 = 42.0;

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
        "Nouveau document",
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
