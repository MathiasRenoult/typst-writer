//! Chaque onglet nomme une vue d'un document, jamais un projet déguisé.
//!
//! L'onglet actif se lit d'un coup d'œil : fond teinté, texte en pleine
//! couleur, liseré sous la barre. Les inactifs restent discrets — le nom
//! du document reste lisible, mais sans disputer la vedette.
use crate::app::espace::Panneau;
use crate::app::state::ModeVue;
use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{button, container, mouse_area, row, scrollable, text};
use iced::{Alignment, Element, Length};

/// La barre d'onglets d'un panneau.
pub fn panneau<'a>(state: &'a State, panneau: &'a Panneau, focus: bool) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let mut barre = row![].spacing(2).align_y(Alignment::Center);
    for (index, &id) in panneau.onglets.iter().enumerate() {
        let (projet, chemin, mode) = if id == state.espace.active {
            (&state.project, state.editor.path.as_deref(), state.mode)
        } else if let Some(vue) = state.espace.vues.get(&id) {
            (&vue.project, vue.editor.path.as_deref(), vue.mode)
        } else {
            continue;
        };
        let fichier = chemin
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned());
        // Le nom du projet est le vrai nom de l'onglet : tous les projets
        // ont un `document.tw`, ce serait illisible. Le nom du fichier ne
        // s'ajoute que s'il dit quelque chose de plus.
        let nom = match fichier.as_deref() {
            Some("document.tw") | Some("main.typ") | None => projet.name.clone(),
            Some(fichier) => format!("{} · {fichier}", projet.name),
        };
        let courant = index == panneau.actif;
        let actif = id == state.espace.active;

        // Le nom du document, en pleine couleur pour l'onglet actif,
        // atténué pour les autres — mais toujours lisible.
        let couleur_texte = if actif { palette.text } else { palette.text_muted };
        let etiquette = text(nom).size(theme::jetons::CORPS).color(couleur_texte).font(palette.interface);

        // Un petit repère de mode : un point coloré, pas un mot de plus.
        let point_couleur = match mode {
            ModeVue::Edition => palette.accent,
            ModeVue::Rendu => palette.success,
            ModeVue::Typst => palette.warning,
        };
        let point = container(text("●").size(7).color(point_couleur))
            .padding(iced::Padding::new(0.0).left(4.0));

        // La fermeture ne se montre qu'au survol : un × permanent sur chaque
        // onglet est du bruit visuel — et une cible de clic dangereuse.
        let survole = state.survol_onglet == Some(id);
        let fermer: Element<'a, Message> = if survole {
            button(text("×").size(13).color(palette.text_muted))
                .on_press(Message::VueFermer(id))
                .padding(iced::Padding::new(2.0).right(4.0).bottom(2.0).left(4.0))
                .style(move |_, _| button::Style {
                    background: None,
                    text_color: palette.text_muted,
                    ..Default::default()
                })
                .into()
        } else {
            // L'espace est réservé même quand le × est caché, pour que
            // l'onglet ne change pas de largeur au survol.
            container(text(" ").size(13)).padding(iced::Padding::new(2.0).right(4.0).bottom(2.0).left(4.0)).into()
        };

        let contenu = row![etiquette, point, fermer]
            .spacing(0)
            .align_y(Alignment::Center);

        let fond = if actif {
            // L'onglet actif : un fond teinté à l'accent, avec un liseré
            // en dessous qui le rattache au contenu du panneau.
            container(contenu)
                .padding(iced::Padding::new(7.0).right(12.0).bottom(6.0).left(12.0))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(melange(palette.background, palette.accent, 0.12).into()),
                    border: iced::Border {
                        color: palette.accent,
                        width: 1.0,
                        radius: iced::border::Radius {
                            top_left: 8.0,
                            top_right: 8.0,
                            bottom_right: 0.0,
                            bottom_left: 0.0,
                        },
                    },
                    ..Default::default()
                })
        } else if courant {
            // L'onglet actif du panneau mais pas la vue active (un autre
            // panneau a le focus) : une bordure discrète, pas de fond.
            container(contenu)
                .padding(iced::Padding::new(7.0).right(12.0).bottom(6.0).left(12.0))
                .style(move |_: &iced::Theme| container::Style {
                    border: iced::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
        } else {
            container(contenu)
                .padding(iced::Padding::new(7.0).right(12.0).bottom(6.0).left(12.0))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(if survole {
                        palette.surface_hover.into()
                    } else {
                        palette.surface.into()
                    }),
                    border: iced::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
        };

        let onglet = mouse_area(fond)
            .on_press(Message::OngletPrendre(id))
            .on_enter(Message::SurvolOnglet(Some(id)))
            .on_exit(Message::SurvolOnglet(None))
            .interaction(iced::mouse::Interaction::Grab);

        barre = barre.push(onglet);
    }
    let bare = scrollable(barre)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .width(Length::Fill);
    // Le liseré du panneau actif vit sous sa barre d'onglets — pas autour
    // de tout le panneau. Un trait fin, au-dessus du contenu.
    let liseré: Element<'a, Message> = if focus {
        container(iced::widget::Space::new().height(2.0))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(palette.accent.into()),
                ..Default::default()
            })
            .into()
    } else {
        container(iced::widget::Space::new().height(2.0))
            .width(Length::Fill)
            .into()
    };
    iced::widget::column![bare, liseré].into()
}

/// Mélange deux couleurs — pour teinter un fond sans l'opacifier.
fn melange(a: iced::Color, b: iced::Color, poids: f32) -> iced::Color {
    iced::Color {
        r: a.r + (b.r - a.r) * poids,
        g: a.g + (b.g - a.g) * poids,
        b: a.b + (b.b - a.b) * poids,
        a: 1.0,
    }
}

pub fn source_typst(palette: Palette, genere: &str) -> Element<'_, Message> {
    scrollable(
        container(iced::widget::column![
            palette
                .label("Typst généré — lecture seule")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            iced::widget::text(genere).size(12),
        ]
        .spacing(8))
        .padding(12)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
