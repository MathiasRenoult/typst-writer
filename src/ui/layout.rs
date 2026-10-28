use crate::app::state::{LayoutState, PreviewPosition, Splitter};
use crate::app::{Message, State};
use iced::widget::{container, mouse_area, rule};
use iced::{Alignment, Element, Length, Theme};

use super::{editor, preview, sidebar};

/// Mise en page de l'application : bibliothèque, éditeur, aperçu.
///
/// Les panneaux s'ouvrent et se ferment, se redimensionnent à la souris, et
/// l'aperçu se déplace à droite, à gauche ou en dessous. La largeur au début
/// du glissement est retenue et le mouvement applique un delta : inutile de
/// connaître la taille de la fenêtre, et le calcul reste juste même si elle
/// change en cours de route.
pub fn view(state: &State) -> Element<'_, Message> {
    // Mode focus : le texte source seul. Les préférences de panneaux ne sont
    // pas touchées — c'est l'affichage qui les ignore — donc en sortir rend la
    // disposition exactement telle qu'elle était.
    if state.layout.focus_mode {
        return editor::view(state);
    }

    if state.layout.preview_visible && state.layout.preview_position == PreviewPosition::Bottom {
        return column_layout(state).into();
    }

    let layout = &state.layout;
    let mut bar = iced::widget::row![].spacing(0).align_y(Alignment::Center);

    if layout.sidebar_visible {
        bar = bar.push(
            container(sidebar::view(state))
                .width(Length::Fixed(layout.sidebar_width))
                .height(Length::Fill),
        );
        bar = bar.push(splitter(Splitter::Sidebar, state));
    }

    // L'éditeur prend la place restante : c'est lui la raison d'être de
    // l'application, les panneaux ne font que l'entourer.
    bar = bar.push(crate::ui::onglets::view(state));
    bar = bar.push(editor::view(state));

    if layout.preview_visible {
        bar = bar.push(splitter(Splitter::Preview, state));
        bar = bar.push(
            container(preview::view(state))
                .width(Length::Fixed(layout.preview_width))
                .height(Length::Fill),
        );
    }

    bar.width(Length::Fill).height(Length::Fill).into()
}

/// Disposition en colonnes : l'aperçu sous l'éditeur, de pleine largeur.
fn column_layout(state: &State) -> iced::widget::Column<'_, Message> {
    let layout = &state.layout;

    let mut top = iced::widget::row![].align_y(Alignment::Center);

    if layout.sidebar_visible {
        top = top.push(
            container(sidebar::view(state))
                .width(Length::Fixed(layout.sidebar_width))
                .height(Length::Fill),
        );
        top = top.push(splitter(Splitter::Sidebar, state));
    }

    top = top.push(crate::ui::onglets::view(state));
    top = top.push(editor::view(state));

    iced::widget::column![
        top.height(Length::FillPortion(1)),
        splitter(Splitter::Preview, state),
        container(preview::view(state))
            .width(Length::Fill)
            .height(Length::FillPortion(1)),
    ]
    .spacing(0)
    .width(Length::Fill)
}

/// Séparateur entre deux panneaux.
///
/// La zone préhensible est plus large que le filet visible : une cible d'un
/// pixel serait frustrante à viser.
fn splitter<'a>(which: Splitter, state: &'a State) -> Element<'a, Message> {
    let dragging = state
        .layout
        .dragging
        .is_some_and(|drag| drag.which == which);

    // En colonnes, le séparateur de l'aperçu est horizontal : il règle une
    // hauteur, pas une largeur.
    let horizontal =
        which == Splitter::Preview && state.layout.preview_position == PreviewPosition::Bottom;

    // Les deux filets ont le même type : une règle iced, dont seule
    // l'orientation change.
    let line: iced::widget::Rule<Theme> = if horizontal {
        rule::horizontal(1)
    } else {
        rule::vertical(1)
    };

    let grabber = container(line)
        .style(move |t: &Theme| splitter_style(t, dragging))
        .width(if horizontal {
            Length::Fill
        } else {
            Length::Fixed(LayoutState::SPLITTER_WIDTH)
        })
        .height(if horizontal {
            Length::Fixed(LayoutState::SPLITTER_WIDTH)
        } else {
            Length::Fill
        });

    mouse_area(grabber)
        .on_press(Message::SplitterPressed(which))
        .on_release(Message::SplitterReleased)
        // Le curseur change sur la bande : c'est la seule indication que la
        // zone est préhensible, le filet étant volontairement discret.
        .interaction(iced::mouse::Interaction::ResizingHorizontally)
        .into()
}

/// Fond d'un séparateur : discret au repos, accentué quand on le tient.
fn splitter_style(theme: &Theme, dragging: bool) -> container::Style {
    let palette = crate::ui::theme::Palette::from_theme(theme);

    container::Style {
        background: Some(
            if dragging {
                palette.accent
            } else {
                palette.border
            }
            .into(),
        ),
        ..Default::default()
    }
}
