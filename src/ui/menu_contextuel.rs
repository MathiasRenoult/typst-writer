//! Le menu du clic droit ordinaire.
//!
//! La balise et l'en-tête de tableau ont leurs gestes propres ; le reste du
//! texte garde les actions familières d'un éditeur (couper, copier, coller,
//! annuler, rétablir), sans le menu du widget qui se superposerait aux
//! panneaux de l'application.
//!
//! Chaque action est une ligne pleine largeur : l'icône donne le geste d'un
//! coup d'œil, le libellé reste lisible, et le raccourci rappelle le clavier.
//! Le panneau est posé **au point du clic**, dans la vue de rédaction.

use crate::app::Message;
use crate::app::message::MenuContextuelAction;
use crate::app::state::State;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row};
use iced::{Alignment, Element, Length, Theme};

const LARGEUR: f32 = 232.0;
const LIGNE: f32 = 34.0;

/// La carte du menu, quand il est ouvert.
///
/// Elle est placée par le recouvrement de l'éditeur, qui connaît le point
/// exact du clic dans la vue de rédaction. Ce module ne fabrique que le
/// contenu : il ne devine aucune coordonnée de fenêtre.
pub fn carte<'a>(state: &'a State) -> Option<Element<'a, Message>> {
    let _menu = state.menu_contextuel?;
    let palette = state.resolved.palette;

    let actions = column![
        ligne(
            lucide_icons::iced::icon_scissors::<Theme>(),
            "Couper",
            "Ctrl+X",
            MenuContextuelAction::Couper,
            palette,
        ),
        ligne(
            lucide_icons::iced::icon_copy::<Theme>(),
            "Copier",
            "Ctrl+C",
            MenuContextuelAction::Copier,
            palette,
        ),
        ligne(
            lucide_icons::iced::icon_clipboard_paste::<Theme>(),
            "Coller",
            "Ctrl+V",
            MenuContextuelAction::Coller,
            palette,
        ),
        separateur(palette),
        ligne(
            lucide_icons::iced::icon_text_select::<Theme>(),
            "Tout sélectionner",
            "Ctrl+A",
            MenuContextuelAction::ToutSelectionner,
            palette,
        ),
        separateur(palette),
        ligne(
            lucide_icons::iced::icon_undo::<Theme>(),
            "Annuler",
            "Ctrl+Z",
            MenuContextuelAction::Annuler,
            palette,
        ),
        ligne(
            lucide_icons::iced::icon_redo::<Theme>(),
            "Rétablir",
            "Ctrl+Y",
            MenuContextuelAction::Retablir,
            palette,
        ),
    ]
    .spacing(1);

    Some(
        container(actions)
            .padding(6)
            .width(Length::Fixed(LARGEUR))
            .style(theme::panneau(palette))
            .into(),
    )
}

/// Une ligne d'action : icône, libellé, raccourci, toute la largeur cliquable.
fn ligne<'a>(
    glyphe: iced::widget::Text<'a, Theme>,
    libelle: &'static str,
    raccourci: &'static str,
    action: MenuContextuelAction,
    palette: theme::Palette,
) -> Element<'a, Message> {
    let contenu = row![
        theme::icon(glyphe, 17.0, palette.text_muted),
        palette
            .label(libelle)
            .size(theme::jetons::SAILLANT)
            .color(palette.text),
        Space::new().width(Length::Fill),
        palette
            .label(raccourci)
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    button(contenu)
        .on_press(Message::MenuContextuelExecuter(action))
        .padding([7, 9])
        .width(Length::Fill)
        .height(Length::Fixed(LIGNE))
        .style(move |_, statut| style_ligne(statut, palette))
        .into()
}

/// Un filet discret entre deux familles de gestes.
fn separateur(palette: theme::Palette) -> Element<'static, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .padding([4, 0])
        .style(move |_| container::Style {
            background: Some(palette.border.into()),
            ..Default::default()
        })
        .into()
}

fn style_ligne(statut: button::Status, palette: theme::Palette) -> button::Style {
    let fond = match statut {
        button::Status::Active => None,
        button::Status::Hovered => Some(palette.surface_hover),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => None,
    };

    button::Style {
        background: fond.map(Into::into),
        text_color: palette.text,
        border: iced::Border {
            radius: 7.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
