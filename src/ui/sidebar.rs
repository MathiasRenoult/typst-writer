use crate::app::{Message, State};
use iced::widget::scrollable;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Fill};

/// Largeur de la barre latérale.
const WIDTH: f32 = 240.0;

/// La bibliothèque personnelle : les documents du dossier de travail, le
/// document actif mis en évidence, et les références bibliographiques
/// citables.
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![section_title("BIBLIOTHÈQUE")]
        .spacing(4)
        .padding(12);

    // Bouton « nouveau devoir », toujours en tête.
    content = content.push(
        button(text("+ Nouveau devoir").size(13))
            .on_press(Message::NewFile)
            .width(Fill)
            .padding([6, 10])
            .style(button::secondary),
    );

    content = content.push(Space::new().height(4));

    if state.library.is_empty() {
        content = content.push(text("Aucun document.").size(13));
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

        let label = if is_active {
            format!("▸ {}", document.title())
        } else {
            format!("   {}", document.title())
        };

        let open = Message::OpenLibraryDocument(document.path.clone());

        content = content.push(
            row![
                button(text(label).size(13))
                    .on_press(open)
                    .width(Fill)
                    .padding([4, 6])
                    .style(if is_active {
                        button::secondary
                    } else {
                        button::text
                    }),
                button("🗑")
                    .on_press(Message::DeleteDocument(document.path.clone()))
                    .width(28)
                    .padding([2, 4])
                    .style(button::danger),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
        );

        // Taille du fichier, en dessous du nom : repère discret pour trier
        // visuellement les gros devoirs des ébauches.
        content = content.push(
            text(format!("   {}", format_size(document.size)))
                .size(11)
                .color(iced::color!(0x666666)),
        );
    }

    // Bibliographie : les clés citables du `.bib` à côté du document.
    if !state.bibliography.is_empty() {
        content = content.push(Space::new().height(12));
        content = content.push(section_title("RÉFÉRENCES"));

        for entry in &state.bibliography {
            // Le type BibTeX est rappelé à droite : `@clé` suffit à citer,
            // le type aide à reconnaître l'entrée.
            let label = format!("@{}  ({})", entry.key, entry.kind);

            content = content.push(
                button(text(label).size(13))
                    .on_press(Message::InsertCitation(entry.key.clone()))
                    .width(Fill)
                    .padding([4, 6])
                    .style(button::text),
            );
        }
    }

    container(scrollable(content).width(Fill).height(Fill))
        .width(iced::Length::Fixed(WIDTH))
        .height(Fill)
        .style(panel_style())
        .into()
}

/// Titre de section, en petites capitales discrètes.
fn section_title(title: &str) -> text::Text<'_> {
    text(title).size(11)
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

/// Fond de la barre latérale, légèrement distinct du reste de l'interface.
fn panel_style() -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(iced::color!(0x16181d).into()),
        ..Default::default()
    }
}
