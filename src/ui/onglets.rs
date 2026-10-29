//! Chaque onglet nomme une vue d'un document, jamais un projet déguisé.
use crate::app::espace::Panneau;
use crate::app::state::ModeVue;
use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{button, container, mouse_area, row, scrollable, tooltip};
use iced::{Element, Length};

pub fn panneau<'a>(state: &'a State, panneau: &'a Panneau) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let mut barre = row![].spacing(4).align_y(iced::Alignment::Center);
    for (index, &id) in panneau.onglets.iter().enumerate() {
        let (projet, chemin, mode) = if id == state.espace.active {
            (&state.project, state.editor.path.as_deref(), state.mode)
        } else if let Some(vue) = state.espace.vues.get(&id) {
            (&vue.project, vue.editor.path.as_deref(), vue.mode)
        } else { continue; };
        let nom = chemin.and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or_default();
        let mode = match mode { ModeVue::Edition => "édition", ModeVue::Rendu => "rendu", ModeVue::Typst => "Typst" };
        let label = format!("{} · {nom} · {mode}", projet.name);
        let courant = index == panneau.actif;
        // Une étiquette, et non un bouton qui capturerait l'appui avant le
        // début du glissement. Un clic simple sélectionne aussi la vue.
        let onglet = mouse_area(container(palette.label(label).size(theme::jetons::DETAIL))
            .padding([6, 10])
            .style(move |_| container::Style { border: iced::Border { width: if courant { 1.0 } else { 0.0 }, color: palette.accent, radius: 5.0.into() }, ..Default::default() }))
            .on_press(Message::OngletPrendre(id))
            .interaction(iced::mouse::Interaction::Grab);
        let fermer = tooltip(button(palette.label("×")).on_press(Message::VueFermer(id)).padding(4), palette.label("Fermer cette vue"), tooltip::Position::Bottom);
        barre = barre.push(row![onglet, fermer].spacing(1));
    }
    let dupliquer = tooltip(button(palette.label("+")).on_press(Message::VueDupliquer).padding(5), palette.label("Dupliquer la vue du document actif"), tooltip::Position::Bottom);
    row![scrollable(barre).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())).width(Length::Fill), dupliquer]
        .width(Length::Fill).into()
}

pub fn source_typst(palette: Palette, genere: &str) -> Element<'_, Message> {
    scrollable(container(iced::widget::column![
        palette.label("Typst généré — lecture seule").size(theme::jetons::DETAIL).color(palette.text_muted),
        iced::widget::text(genere).size(12),
    ].spacing(8)).padding(12).width(Length::Fill)).width(Length::Fill).height(Length::Fill).into()
}
