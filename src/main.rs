mod app;
mod editor;
mod lsp;
mod project;
mod settings;
mod typst;
mod ui;

fn main() -> iced::Result {
    iced::application(
        || (app::State::new(), app::startup()),
        app::update,
        ui::view,
    )
    .title("Typst Writer")
    // Police Lucide embarquée : les icônes s'affichent sans dépendre des
    // polices installées sur le système.
    .font(lucide_icons::LUCIDE_FONT_BYTES)
    .subscription(app::subscription)
    .theme(theme)
    .run()
}

/// Thème à appliquer, recalculé à chaque rendu : il suit la préférence de
/// couleur du système tant que l'utilisateur n'a pas choisi clair ou sombre.
fn theme(state: &app::State) -> iced::Theme {
    ui::theme::resolve(state.theme, state.system_theme)
}
