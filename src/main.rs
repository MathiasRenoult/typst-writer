mod app;
mod editor;
mod lsp;
mod project;
mod settings;
mod theming;
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
    // Le thème est déjà résolu au changement, jamais à chaque image : le
    // reconstruire ici ferait, soixante fois par seconde, un travail qui ne
    // change qu'au clic.
    state.resolved.iced.clone()
}
