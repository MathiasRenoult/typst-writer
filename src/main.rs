mod app;
mod editor;
mod lsp;
mod project;
mod theming;
mod typst;
mod ui;

/// Allocateur global.
///
/// `mimalloc` rend au système la mémoire qu'on lui rend, au lieu de la garder
/// dans des arènes par fil. L'application alloue par gros blocs et par à-coups —
/// chaque page rasterisée pèse plusieurs mégaoctets, et chaque image en alloue
/// pour la vue — et l'allocateur de la glibc conserve ces blocs libérés, un
/// arène par fil de travail : la mémoire résidente enfle alors qu'aucune
/// structure ne retient rien.
#[global_allocator]
static ALLOCATEUR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> iced::Result {
    iced::application(
        || (app::State::new(), app::startup()),
        app::update,
        ui::view,
    )
    .title("Typst Writer")
    .window(iced::window::Settings {
        // Une taille de départ qui montre l'éditeur, le panneau de travail et
        // l'aperçu côte à côte : la mesure de lecture fait 760 px, les panneaux
        // prennent le reste.
        size: iced::Size::new(1180.0, 780.0),
        // Sans minimum, la fenêtre pouvait être réduite à rien, et la
        // disposition n'a plus de sens dès que la mesure ne tient plus.
        min_size: Some(iced::Size::new(820.0, 540.0)),
        ..Default::default()
    })
    // Police Lucide embarquée : les icônes s'affichent sans dépendre des
    // polices installées sur le système.
    .font(lucide_icons::LUCIDE_FONT_BYTES)
    .subscription(app::subscription)
    .theme(theme)
    .centered()
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
