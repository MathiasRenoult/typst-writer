//! La bibliothèque encadre une grille de panneaux internes, jamais de fenêtres OS.
use crate::app::espace::{Panneau, VueId};
use crate::app::state::{ModeVue, Splitter};
use crate::app::{Message, State};
use iced::widget::{container, mouse_area, pane_grid, responsive, row, rule};
use iced::{Element, Length};

pub fn view(state: &State) -> Element<'_, Message> {
    if state.layout.focus_mode { return super::editor::view(state); }
    let grille = pane_grid::PaneGrid::new(&state.espace.grille, |pane, panneau, _| {
        pane_grid::Content::new(panneau_vue(state, pane, panneau))
    })
    .spacing(6)
    .min_size(120)
    .on_click(Message::PanneauChoisir)
    .on_resize(8, Message::PanneauRedimensionner);
    let mut ligne = row![].spacing(0);
    if state.layout.sidebar_visible {
        ligne = ligne.push(container(super::sidebar::view(state)).width(state.layout.sidebar_width).height(Length::Fill));
        ligne = ligne.push(mouse_area(container(rule::vertical(1)).width(crate::app::state::LayoutState::SPLITTER_WIDTH).height(Length::Fill))
            .on_press(Message::SplitterPressed(Splitter::Sidebar))
            .interaction(iced::mouse::Interaction::ResizingHorizontally));
    }
    ligne.push(grille).width(Length::Fill).height(Length::Fill).into()
}

fn panneau_vue<'a>(state: &'a State, pane: pane_grid::Pane, panneau: &'a Panneau) -> Element<'a, Message> {
    let id = panneau.onglets.get(panneau.actif).copied();
    let palette = state.resolved.palette;
    // Les coordonnées locales viennent de la vraie géométrie iced. Le bord
    // cible a une épaisseur constante, quelle que soit la profondeur d'arbre.
    responsive(move |taille| {
        let bordure = state.espace.glisse.is_some();
        let cible_bord = state.espace.cible
            .filter(|(p, _)| *p == pane)
            .and_then(|(_, bord)| bord);
        let corps: iced::Element<'a, Message> = contenu_vue_panneau(state, panneau, id);
        let corps: iced::Element<'a, Message> = match cible_bord {
            Some(bord) => {
                let bande = container(iced::widget::Space::new()).style(move |_| container::Style {
                    background: Some(palette.accent.into()), ..Default::default() });
                match bord {
                    pane_grid::Edge::Left => iced::widget::row![bande.width(Length::Fixed(4.0)).height(Length::Fill), corps].into(),
                    pane_grid::Edge::Right => iced::widget::row![corps, bande.width(Length::Fixed(4.0)).height(Length::Fill)].into(),
                    pane_grid::Edge::Top => iced::widget::column![bande.width(Length::Fill).height(Length::Fixed(4.0)), corps].into(),
                    pane_grid::Edge::Bottom => iced::widget::column![corps, bande.width(Length::Fill).height(Length::Fixed(4.0))].into(),
                }
            },
            None => corps,
        };
        mouse_area(container(corps)
            .width(Length::Fill).height(Length::Fill)
            .style(move |_| {
                let mut s = container::Style::default();
                s.border.width = if pane == state.espace.focus || bordure { 1.0 } else { 0.0 };
                s.border.color = if pane == state.espace.focus { palette.accent } else { palette.border };
                s
            }))
            .on_move(move |position| Message::OngletCible(pane, bord_du_panneau(position, taille)))
            .on_enter(Message::OngletCible(pane, None))
            .into()
    }).into()
}

fn contenu_vue_panneau<'a>(state: &'a State, panneau: &'a Panneau, id: Option<VueId>) -> Element<'a, Message> {
    let corps = match id {
        Some(id) => contenu_vue(state, id).map(move |m| message_vue(id, m)),
        None => container(state.resolved.palette.label("Ouvrez un document ou déposez un onglet ici.")).into(),
    };
    iced::widget::column![super::onglets::panneau(state, panneau), corps]
        .width(Length::Fill).height(Length::Fill).into()
}

pub(crate) fn bord_du_panneau(position: iced::Point, taille: iced::Size) -> Option<pane_grid::Edge> {
    let bord = 32.0_f32.min(taille.width / 4.0).min(taille.height / 4.0);
    if position.x < bord { Some(pane_grid::Edge::Left) }
    else if position.x > taille.width - bord { Some(pane_grid::Edge::Right) }
    else if position.y < bord { Some(pane_grid::Edge::Top) }
    else if position.y > taille.height - bord { Some(pane_grid::Edge::Bottom) }
    else { None }
}

fn message_vue(id: VueId, message: Message) -> Message {
    let fond = match &message {
        Message::ViewportChanged(..) | Message::RetourVue(..) | Message::RetourDocument(..) => true,
        Message::Editor(message) => !matches!(message,
            iced_code_editor::Message::MouseClick(..)
            | iced_code_editor::Message::DoubleClick(..)
            | iced_code_editor::Message::TripleClick(..)
            | iced_code_editor::Message::ContextMenuRequested(..)
            | iced_code_editor::Message::JumpClick(..)),
        _ => false,
    };
    if fond { Message::RetourVue(id, Box::new(message)) }
    else { Message::VersVue(id, Box::new(message)) }
}

fn contenu_vue(state: &State, id: VueId) -> Element<'_, Message> {
    let active = id == state.espace.active;
    let (editor, render, lsp, mode, genere) = if active {
        (&state.editor, &state.render, &state.lsp, state.mode, state.typst_genere.as_str())
    } else if let Some(vue) = state.espace.vues.get(&id) {
        (&vue.editor, &vue.render, &vue.lsp, vue.mode, vue.typst.as_str())
    } else { return iced::widget::Space::new().into(); };
    match mode {
        ModeVue::Edition => super::editor::vue(state, editor, active),
        ModeVue::Rendu => {
            let (first, last) = render.visible.unwrap_or((0, 2));
            iced::widget::sensor::Sensor::new(super::preview::vue(state, render, editor, lsp, Some(id)))
                .key(id)
                .on_show(move |size| Message::ViewportChanged(first, last, size))
                .on_resize(move |size| Message::ViewportChanged(first, last, size))
                .into()
        },
        ModeVue::Typst => super::onglets::source_typst(state.resolved.palette, genere),
    }
}
