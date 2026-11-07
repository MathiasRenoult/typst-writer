mod app;
mod editor;
// L'architecture des extensions (bug-report-7, point 4) : le registre
// attend sa GUI (jalon 3 — le composeur cetz).
#[allow(dead_code)]
mod extensions;
mod langage;
mod mise_en_page;
mod project;
mod theming;
mod typst;
mod ui;
mod zotero;

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
    // Le titre nomme ce qu'on écrit — le kit le reprend à chaque image,
    // et il suit le document ouvert sans qu'on ait à y penser.
    .title(|state: &app::State| {
        let nom = state
            .editor
            .path
            .as_ref()
            .and_then(|chemin| chemin.file_name())
            .and_then(|nom| nom.to_str())
            .unwrap_or("Sans titre");

        format!("{nom} — Typst Writer")
    })
    .window(iced::window::Settings {
        // Une taille de départ qui montre l'éditeur, le panneau de travail et
        // l'aperçu côte à côte : la mesure de lecture fait 760 px, les panneaux
        // prennent le reste.
        size: iced::Size::new(1180.0, 780.0),
        // Sans minimum, la fenêtre pouvait être réduite à rien, et la
        // disposition n'a plus de sens dès que la mesure ne tient plus.
        min_size: Some(iced::Size::new(820.0, 540.0)),
        // La fermeture attend que l'application ait sauvé ses documents.
        exit_on_close_request: false,
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
