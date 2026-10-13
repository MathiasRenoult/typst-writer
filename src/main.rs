mod app;
mod editor;
mod lsp;
mod project;
mod settings;
mod typst;
mod ui;

fn main() -> iced::Result {
    iced::application(app::State::new, app::update, ui::view)
        .title("Typst Writer")
        .subscription(app::subscription)
        .run()
}
