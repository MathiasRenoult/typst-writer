use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Theme};

/// Hauteur d'une entrée de document.
const ENTRY_HEIGHT: f32 = 30.0;

/// La bibliothèque personnelle : les documents du dossier de travail, le
/// document actif mis en évidence, et les références bibliographiques
/// citables.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = Palette::from_theme(&theme::resolve(state.theme, state.system_theme));

    let mut content = column![section_title("Bibliothèque", palette)]
        .spacing(2)
        .padding([14, 12]);

    content = content.push(
        button(
            row![
                theme::icon(lucide_icons::iced::icon_plus::<Theme>(), 15.0, palette.text),
                text("Nouveau").size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::NewFile)
        .width(Fill)
        .padding([7, 10])
        .style(move |t, status| new_button_style(t, status, palette)),
    );

    if state.library.is_empty() {
        content = content.push(text("Aucun document.").size(13).color(palette.text_muted));
    }

    // Les documents, du plus récent au plus ancien. Le document actif est
    // marqué ; un clic l'ouvre, la corbeille le supprime.
    for document in &state.library {
        let is_active = state
            .editor
            .path
            .as_ref()
            .map(|path| path == &document.path)
            .unwrap_or(false);

        let open = Message::OpenLibraryDocument(document.path.clone());

        let mut label = row![text(document.title()).size(13)]
            .spacing(8)
            .align_y(Alignment::Center);

        if is_active {
            // Le repère d'activité est un point, pas une flèche en texte :
            // même information, aucun bruit typographique.
            label = label.push(theme::icon(
                lucide_icons::iced::icon_dot::<Theme>(),
                9.0,
                palette.accent,
            ));
        }

        let entry = button(label)
            .on_press(open)
            .width(Fill)
            .height(Length::Fixed(ENTRY_HEIGHT))
            .padding([4, 8])
            .style(move |t, status| entry_style(t, status, palette, is_active));

        // La taille du fichier est un repère utile pour distinguer une
        // ébauche d'un gros devoir — mais elle n'a rien à faire sur chaque
        // ligne. Elle vit dans l'infobulle : disponible au survol, muette au
        // repos.
        let details = container(
            column![
                text(document.title()).size(12),
                text(format!(
                    "{} — {}",
                    format_size(document.size),
                    document.path.display()
                ))
                .size(11)
                .color(palette.text_muted),
            ]
            .spacing(2),
        )
        .padding([6, 10])
        .style(move |_| container::Style {
            background: Some(palette.surface_hover.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        content = content.push(
            row![
                iced::widget::tooltip(entry, details, iced::widget::tooltip::Position::Right,)
                    .gap(6),
                button(theme::icon(
                    lucide_icons::iced::icon_trash_2::<Theme>(),
                    14.0,
                    palette.text_muted,
                ))
                .on_press(Message::DeleteDocument(document.path.clone()))
                .width(Length::Fixed(26.0))
                .height(Length::Fixed(ENTRY_HEIGHT))
                .padding([4, 5])
                .style(move |t, status| delete_style(t, status, palette)),
            ]
            .spacing(2)
            .align_y(Alignment::Center),
        );
    }

    // Bibliographie : les clés citables du `.bib` à côté du document.
    if !state.bibliography.is_empty() {
        content = content.push(text("").size(4));
        content = content.push(section_title("Références", palette));

        for entry in &state.bibliography {
            // Le type BibTeX va dans l'infobulle : `@clé` suffit à citer, et
            // le type n'est qu'une aide à la reconnaissance.
            let kind = container(text(entry.kind.clone()).size(11).color(palette.text_muted))
                .padding([4, 8])
                .style(move |_| container::Style {
                    background: Some(palette.surface_hover.into()),
                    border: iced::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                });

            content = content.push(
                iced::widget::tooltip(
                    button(text(format!("@{}", entry.key)).size(13))
                        .on_press(Message::InsertCitation(entry.key.clone()))
                        .width(Fill)
                        .height(Length::Fixed(ENTRY_HEIGHT))
                        .padding([4, 8])
                        .style(move |t, status| entry_style(t, status, palette, false)),
                    kind,
                    iced::widget::tooltip::Position::Right,
                )
                .gap(6),
            );
        }
    }

    container(scrollable(content).width(Fill).height(Fill))
        .width(Length::Fill)
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            ..Default::default()
        })
        .into()
}

/// Titre de section, en petites capitales discrètes.
fn section_title(title: &str, palette: Palette) -> text::Text<'_> {
    text(title.to_uppercase())
        .size(10)
        .color(palette.text_muted)
}

/// Taille lisible : `1,2 Ko`, `340 o`.
fn format_size(bytes: u64) -> String {
    const KILO: f64 = 1024.0;

    if bytes < 1024 {
        format!("{bytes} o")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} Ko", bytes as f64 / KILO)
    } else {
        format!("{:.1} Mo", bytes as f64 / (KILO * KILO))
    }
}

/// Bouton « Nouveau » : plein, pour qu'il se détache de la liste.
fn new_button_style(t: &Theme, status: button::Status, palette: Palette) -> button::Style {
    let background = match status {
        button::Status::Active => Some(palette.accent),
        button::Status::Hovered => Some(palette.accent),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => Some(palette.border),
    };

    // Sur un fond accentué, le texte doit rester lisible : iced fournit la
    // couleur de contraste du thème plutôt que d'en choisir une à la main.
    let on_accent = t.extended_palette().primary.base.text;

    button::Style {
        background: background.map(Into::into),
        text_color: on_accent,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Entrée de la liste : fond transparent au repos, survol au passage, fond
/// accentué pour le document actif.
fn entry_style(
    _t: &Theme,
    status: button::Status,
    palette: Palette,
    active: bool,
) -> button::Style {
    let background = if active {
        Some(palette.surface_active)
    } else {
        match status {
            button::Status::Active => None,
            button::Status::Hovered => Some(palette.surface_hover),
            button::Status::Pressed => Some(palette.surface_active),
            button::Status::Disabled => None,
        }
    };

    button::Style {
        background: background.map(Into::into),
        text_color: palette.text,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Corbeille : discrète jusqu'au survol, où elle passe en rouge.
fn delete_style(_t: &Theme, status: button::Status, palette: Palette) -> button::Style {
    let (background, text) = match status {
        button::Status::Active => (None, palette.text_muted),
        button::Status::Hovered => (Some(palette.danger), palette.text),
        button::Status::Pressed => (Some(palette.danger), palette.text),
        button::Status::Disabled => (None, palette.text_muted),
    };

    button::Style {
        background: background.map(Into::into),
        text_color: text,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
