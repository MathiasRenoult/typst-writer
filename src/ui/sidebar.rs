use crate::app::state::ChapterRename;
use crate::app::{Message, State};
use crate::project::{BibliographyEntry, Chapter};
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input, tooltip};
use iced::{Alignment, Element, Fill, Length, Theme};

/// Hauteur d'une ligne.
const ENTRY_HEIGHT: f32 = 32.0;

/// Largeur de la colonne des comptes de mots.
///
/// Fixe, et alignée à droite : c'est ce qui fait qu'on parcourt la colonne des
/// chiffres d'un seul regard, sans que les nombres dansent d'une ligne à
/// l'autre.
const COUNT_WIDTH: f32 = 58.0;

/// Le panneau de travail : où en est le projet.
///
/// Il répond à une seule question — « où j'en suis ? » — et l'ordonne du plus
/// général au plus précis : le projet et son volume, ses chapitres, sa
/// bibliographie, les autres projets. Chaque chapitre porte son compte de mots,
/// qui est le repère de progression.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let project = &state.project;

    let mut content = column![].spacing(2).padding([14, 12]);

    // ---- Le projet et son volume ----
    content = content.push(
        palette
            .label(project.name.to_uppercase())
            .size(10)
            .color(palette.text_muted),
    );

    content = content.push(
        row![
            palette
                .label(format!("{} mots", format_count(live_words(state))))
                .size(13)
                .color(palette.text),
            Space::new().width(Length::Fill),
            palette
                .label(format!("{} chapitre(s)", project.chapters.len()))
                .size(11)
                .color(palette.text_muted),
        ]
        .align_y(Alignment::Center),
    );

    content = content.push(Space::new().height(14));

    // ---- Les chapitres ----
    content = content.push(section_title("Chapitres", palette));

    if project.chapters.is_empty() {
        content = content.push(
            palette
                .label("Aucun chapitre.")
                .size(12)
                .color(palette.text_muted),
        );
    }

    let active = state.editor.path.clone();

    for chapter in &project.chapters {
        content = content.push(chapter_row(
            chapter,
            state.renaming.as_ref(),
            active.as_deref() == Some(chapter.path.as_path()),
            palette,
        ));
    }

    // Un chapitre absent de l'assemblage ne finira pas dans le PDF. C'est le
    // seul avertissement que ce panneau se permette, parce que l'oubli serait
    // invisible autrement.
    let orphans = project.orphan_chapters().count();

    if orphans > 0 {
        content = content.push(Space::new().height(8));
        content = content.push(
            palette
                .label(format!(
                    "{orphans} chapitre(s) hors assemblage — absent(s) du PDF"
                ))
                .size(11)
                .color(palette.warning),
        );
    }

    content = content.push(Space::new().height(8));

    content = content.push(
        button(
            row![
                theme::icon(lucide_icons::iced::icon_plus::<Theme>(), 15.0, palette.text),
                palette.label("Nouveau chapitre").size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::NewChapter)
        .width(Fill)
        .padding([7, 10])
        .style(move |t, status| primary_style(t, status, palette)),
    );

    // ---- La bibliographie ----
    content = content.push(Space::new().height(18));
    content = content.push(section_title("Bibliographie", palette));

    let missing = project.bibliography.is_none();

    content = content.push(info_row(
        theme::icon(
            lucide_icons::iced::icon_book::<Theme>(),
            14.0,
            palette.text_muted,
        ),
        "references.bib".to_string(),
        if missing {
            "absent".to_string()
        } else {
            format!("{} entrée(s)", project.bibliography_entries)
        },
        missing,
        palette,
    ));

    // Les clés citables, quand la bibliographie en contient : un clic insère
    // `@clé` au curseur. Tant que le fichier est vide, il n'y a rien à citer.
    for entry in &state.bibliography {
        content = content.push(citation_row(entry, palette));
    }

    // ---- Les projets ----
    content = content.push(Space::new().height(18));
    content = content.push(section_title("Projets", palette));

    // Le projet courant en tête : la liste sert à en changer, pas à se repérer.
    for other in &state.projects {
        let is_current = other.dir == project.dir;

        content = content.push(
            button(palette.label(other.name.clone()).size(12))
                .on_press(Message::OpenProject(other.dir.clone()))
                .width(Fill)
                .height(Length::Fixed(ENTRY_HEIGHT))
                .padding([4, 8])
                .style(move |t, status| entry_style(t, status, palette, is_current)),
        );
    }

    content = content.push(
        button(
            row![
                theme::icon(
                    lucide_icons::iced::icon_plus::<Theme>(),
                    13.0,
                    palette.text_muted
                ),
                palette.label("Nouveau projet").size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::NewProject)
        .width(Fill)
        .height(Length::Fixed(ENTRY_HEIGHT))
        .padding([4, 8])
        .style(move |t, status| entry_style(t, status, palette, false)),
    );

    container(scrollable(content).width(Fill).height(Fill))
        .width(Length::Fill)
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            ..Default::default()
        })
        .into()
}

/// Une ligne de chapitre : son numéro, son titre, son compte de mots, et de
/// quoi le renommer ou le supprimer.
///
/// Quand ce chapitre est celui qu'on renomme, la ligne devient un champ de
/// saisie : l'édition se fait là où on lit, sans boîte de dialogue.
fn chapter_row<'a>(
    chapter: &'a Chapter,
    renaming: Option<&'a ChapterRename>,
    is_active: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let being_renamed = renaming
        .filter(|renaming| renaming.path == chapter.path)
        .map(|renaming| renaming.buffer.clone());

    if let Some(buffer) = being_renamed {
        return renaming_row(chapter, buffer, palette);
    }

    let mut label = row![].spacing(8).align_y(Alignment::Center);

    if is_active {
        // Le repère d'activité est un point, pas une flèche en texte : même
        // information, aucun bruit typographique.
        label = label.push(theme::icon(
            lucide_icons::iced::icon_dot::<Theme>(),
            9.0,
            palette.accent,
        ));
    }

    // Le numéro passe en avertissement quand le chapitre n'est pas assemblé :
    // la couleur suffit à repérer l'oubli dans la liste.
    label = label.push(
        palette
            .label(format!("{:02}", chapter.order))
            .size(11)
            .color(if chapter.included {
                palette.text_muted
            } else {
                palette.warning
            }),
    );

    label = label.push(
        palette
            .label(chapter.title.clone())
            .size(13)
            .color(palette.text),
    );

    let title = button(label)
        .on_press(Message::OpenChapter(chapter.path.clone()))
        .width(Fill)
        .height(Length::Fixed(ENTRY_HEIGHT))
        .padding([4, 8])
        .style(move |t, status| entry_style(t, status, palette, is_active));

    row![
        tooltip(
            title,
            chapter_details(chapter, palette),
            tooltip::Position::Right
        )
        .gap(6),
        container(
            palette
                .label(format_count(chapter.words))
                .size(12)
                .color(palette.text_muted)
        )
        .width(Length::Fixed(COUNT_WIDTH))
        .height(Length::Fixed(ENTRY_HEIGHT))
        .align_x(Alignment::End)
        .center_y(ENTRY_HEIGHT),
        icon_action(
            lucide_icons::iced::icon_pencil::<Theme>(),
            "Renommer le chapitre",
            Message::StartRename(chapter.path.clone()),
            palette,
        ),
        icon_action(
            lucide_icons::iced::icon_trash_2::<Theme>(),
            "Supprimer le chapitre",
            Message::DeleteChapter(chapter.path.clone()),
            palette,
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .into()
}

/// La ligne d'un chapitre en cours de renommage : un champ de saisie, et de
/// quoi valider ou abandonner.
fn renaming_row<'a>(
    chapter: &'a Chapter,
    buffer: String,
    palette: Palette,
) -> Element<'a, Message> {
    row![
        palette
            .label(format!("{:02}", chapter.order))
            .size(11)
            .color(palette.text_muted),
        text_input::TextInput::new("Titre du chapitre", &buffer)
            .on_input(Message::RenameChanged)
            .on_submit(Message::RenameCommit)
            .size(13)
            .padding([4, 8])
            .width(Fill),
        icon_action(
            lucide_icons::iced::icon_check::<Theme>(),
            "Valider",
            Message::RenameCommit,
            palette,
        ),
        icon_action(
            lucide_icons::iced::icon_x::<Theme>(),
            "Abandonner",
            Message::RenameCancel,
            palette,
        ),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .height(Length::Fixed(ENTRY_HEIGHT))
    .into()
}

/// Une action discrète, réduite à une icône.
fn icon_action<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    tooltip_text: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    tooltip(
        button(theme::icon(glyph, 13.0, palette.text_muted))
            .on_press(message)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(ENTRY_HEIGHT))
            .padding([4, 4])
            .style(move |t, status| delete_style(t, status, palette)),
        container(palette.label(tooltip_text).size(11).color(palette.text))
            .padding([4, 8])
            .style(move |_| tooltip_style(palette)),
        tooltip::Position::Right,
    )
    .gap(6)
    .into()
}

/// L'infobulle d'un chapitre : son état d'assemblage et son chemin.
fn chapter_details<'a>(chapter: &'a Chapter, palette: Palette) -> Element<'a, Message> {
    let (status, color) = if chapter.included {
        ("Inclus dans main.typ".to_string(), palette.text_muted)
    } else {
        (
            "Hors assemblage : n'apparaîtra pas dans le PDF".to_string(),
            palette.warning,
        )
    };

    container(
        column![
            palette.label(chapter.title.clone()).size(12),
            palette.label(status).size(11).color(color),
            palette
                .label(format_path(&chapter.path))
                .size(10)
                .color(palette.text_muted),
        ]
        .spacing(2),
    )
    .padding([6, 10])
    .style(move |_| tooltip_style(palette))
    .into()
}

/// Une ligne d'information : une icône, un nom, un détail à droite.
fn info_row<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    name: String,
    detail: String,
    missing: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let color = if missing {
        palette.warning
    } else {
        palette.text
    };

    row![
        container(glyph).padding([0, 4]).center_y(ENTRY_HEIGHT),
        palette.label(name).size(13).color(color),
        Space::new().width(Length::Fill),
        palette.label(detail).size(11).color(if missing {
            palette.warning
        } else {
            palette.text_muted
        }),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .height(Length::Fixed(ENTRY_HEIGHT))
    .into()
}

/// Une clé citable : un clic insère `@clé` au curseur.
fn citation_row<'a>(entry: &'a BibliographyEntry, palette: Palette) -> Element<'a, Message> {
    // Le type BibTeX va dans l'infobulle : `@clé` suffit à citer, le type n'est
    // qu'une aide à la reconnaissance.
    let kind = container(
        palette
            .label(entry.kind.clone())
            .size(11)
            .color(palette.text_muted),
    )
    .padding([4, 8])
    .style(move |_| tooltip_style(palette));

    tooltip(
        button(palette.label(format!("@{}", entry.key)).size(12))
            .on_press(Message::InsertCitation(entry.key.clone()))
            .width(Fill)
            .height(Length::Fixed(ENTRY_HEIGHT))
            .padding([4, 8])
            .style(move |t, status| entry_style(t, status, palette, false)),
        kind,
        tooltip::Position::Right,
    )
    .gap(6)
    .into()
}

/// Titre de section, en petites capitales discrètes.
fn section_title(title: &str, palette: Palette) -> text::Text<'_> {
    palette
        .label(title.to_uppercase())
        .size(10)
        .color(palette.text_muted)
}

/// Compte de mots du projet, en tenant compte du chapitre en cours d'édition.
///
/// Les comptes relevés sur le disque datent du dernier scan ; celui du chapitre
/// ouvert est lu en direct dans l'éditeur, sinon le total resterait figé
/// pendant toute la séance d'écriture.
fn live_words(state: &State) -> usize {
    let total = state.project.words();

    let active = state.editor.path.as_deref();

    // Le compte du chapitre ouvert est remplacé par sa valeur en direct : la
    // somme reste juste pendant la frappe, au lieu de dater du dernier scan.
    let substituted = state
        .project
        .chapters
        .iter()
        .find(|chapter| Some(chapter.path.as_path()) == active);

    match substituted {
        Some(chapter) => {
            total - chapter.words + crate::project::count_words(&state.editor.content())
        }
        None => total,
    }
}

/// Nombre de mots, groupé par milliers pour rester lisible.
fn format_count(words: usize) -> String {
    let digits = words.to_string();

    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    let mut count = 0;

    for character in digits.chars().rev() {
        if count > 0 && count % 3 == 0 {
            // Une espace fine insécable : le groupe ne doit pas se couper en fin
            // de ligne.
            grouped.push('\u{202f}');
        }

        grouped.push(character);
        count += 1;
    }

    grouped.chars().rev().collect()
}

/// Chemin réduit à ses deux derniers éléments.
///
/// Le préfixe `~/.typst-writer/projets/<nom>/` est le même pour tout le monde :
/// l'afficher dans une infobulle n'apprend rien.
fn format_path(path: &std::path::Path) -> String {
    let parts: Vec<String> = path
        .components()
        .rev()
        .take(2)
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();

    parts.into_iter().rev().collect::<Vec<_>>().join("/")
}

/// Bouton principal : plein, pour qu'il se détache du reste.
fn primary_style(t: &Theme, status: button::Status, palette: Palette) -> button::Style {
    let background = match status {
        button::Status::Active | button::Status::Hovered => Some(palette.accent),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => Some(palette.border),
    };

    button::Style {
        background: background.map(Into::into),
        // Sur un fond accentué, le texte doit rester lisible : iced fournit la
        // couleur de contraste plutôt que d'en choisir une à la main.
        text_color: t.extended_palette().primary.base.text,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Entrée de liste : fond transparent au repos, survol au passage, fond
/// accentué pour l'élément courant.
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
    let (background, text_color) = match status {
        button::Status::Active => (None, palette.text_muted),
        button::Status::Hovered | button::Status::Pressed => (Some(palette.danger), palette.text),
        button::Status::Disabled => (None, palette.text_muted),
    };

    button::Style {
        background: background.map(Into::into),
        text_color,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Fond d'une infobulle.
fn tooltip_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(palette.surface_hover.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les comptes de mots sont groupés par milliers : un projet de 12 000 mots
    /// doit se lire d'un coup d'œil, pas se déchiffrer.
    #[test]
    fn word_counts_are_grouped() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_000), "1\u{202f}000");
        assert_eq!(format_count(12_345), "12\u{202f}345");
        assert_eq!(format_count(1_234_567), "1\u{202f}234\u{202f}567");
    }

    /// Un chemin affiché se réduit à ses deux derniers éléments.
    #[test]
    fn displayed_paths_keep_only_the_last_components() {
        let path = std::path::Path::new(
            "/home/mat/.typst-writer/projets/mon-travail/chapitres/01-intro.typ",
        );

        assert_eq!(format_path(path), "chapitres/01-intro.typ");
    }
}
