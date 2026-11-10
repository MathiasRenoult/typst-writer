//! Le sélecteur de couleur : une roue de teinte, un carré de saturation et
//! de valeur — pas une liste de noms.
//!
//! Il s'ouvre au centre de la fenêtre, jamais rejeté dans un coin : la
//! couleur se choisit en regardant, pas en cherchant. Les conversions
//! HSV ↔ RVB sont pures et testées — c'est le cœur du geste.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::widget::{button, column, container, row, text_input};
use iced::{Element, Length, Point, Rectangle, Theme};

// ---------- Le cœur : les conversions, pures et testées ----------

/// HSV (teinte en degrés, saturation et valeur en 0..=1) vers RVB 0..=255.
pub fn hsv_vers_rgb(teinte: f32, saturation: f32, valeur: f32) -> (u8, u8, u8) {
    let teinte = ((teinte % 360.0) + 360.0) % 360.0;
    let saturation = saturation.clamp(0.0, 1.0);
    let valeur = valeur.clamp(0.0, 1.0);

    let c = valeur * saturation;
    let x = c * (1.0 - ((teinte / 60.0) % 2.0 - 1.0).abs());
    let m = valeur - c;
    let (r, v, b) = match teinte {
        t if t < 60.0 => (c, x, 0.0),
        t if t < 120.0 => (x, c, 0.0),
        t if t < 180.0 => (0.0, c, x),
        t if t < 240.0 => (0.0, x, c),
        t if t < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((v + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

/// RVB 0..=255 vers `#rrggbb` — la forme que Typst avale.
pub fn rgb_vers_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// HSV vers `#rrggbb`, d'un seul geste.
pub fn hsv_vers_hex(teinte: f32, saturation: f32, valeur: f32) -> String {
    let (r, g, b) = hsv_vers_rgb(teinte, saturation, valeur);
    rgb_vers_hex(r, g, b)
}

/// `#rrggbb` (ou `#rgb`) vers HSV — pour ouvrir la roue sur la couleur
/// courante au lieu de la perdre.
pub fn hex_vers_hsv(hex: &str) -> Option<(f32, f32, f32)> {
    let hex = hex.trim().trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        ),
        3 => (
            u8::from_str_radix(&hex[0..1], 16).ok()? * 17,
            u8::from_str_radix(&hex[1..2], 16).ok()? * 17,
            u8::from_str_radix(&hex[2..3], 16).ok()? * 17,
        ),
        _ => return None,
    };

    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let teinte = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let teinte = (teinte + 360.0) % 360.0;
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    Some((teinte, saturation, max))
}

// ---------- La roue de teinte ----------

const RAYON_ROUE: f32 = 84.0;
const EPAISSEUR_ROUE: f32 = 18.0;
const TAILLE_ROUE: f32 = (RAYON_ROUE + EPAISSEUR_ROUE) * 2.0;

/// La couronne des teintes : cliquer-déposer y règle la teinte.
struct RoueTeinte;

impl canvas::Program<Message> for RoueTeinte {
    type State = bool; // la souris est-elle tenue ?

    fn update(
        &self,
        tenu: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let position = cursor.position_in(bounds)?;
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                *tenu = true;
                Some(canvas::Action::publish(teinte_depuis_point(
                    position, bounds,
                )))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if *tenu => Some(
                canvas::Action::publish(teinte_depuis_point(position, bounds)),
            ),
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                *tenu = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::Crosshair
    }

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let centre = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let mut geometries = Vec::new();
        let mut frame = canvas::Frame::new(_renderer, bounds.size());
        for degres in (0..360).step_by(3) {
            let angle = (degres as f32).to_radians();
            let interieur = Point::new(
                centre.x + (RAYON_ROUE - EPAISSEUR_ROUE / 2.0) * angle.cos(),
                centre.y + (RAYON_ROUE - EPAISSEUR_ROUE / 2.0) * angle.sin(),
            );
            let exterieur = Point::new(
                centre.x + (RAYON_ROUE + EPAISSEUR_ROUE / 2.0) * angle.cos(),
                centre.y + (RAYON_ROUE + EPAISSEUR_ROUE / 2.0) * angle.sin(),
            );
            let (r, g, b) = hsv_vers_rgb(degres as f32, 1.0, 1.0);
            frame.stroke(
                &Path::line(interieur, exterieur),
                Stroke::default()
                    .with_width(EPAISSEUR_ROUE / 1.5)
                    .with_color(iced::Color::from_rgb8(r, g, b)),
            );
        }
        geometries.push(frame.into_geometry());
        geometries
    }
}

/// L'angle du point cliqué, en degrés de teinte.
fn teinte_depuis_point(position: Point, bounds: Rectangle) -> Message {
    let centre = Point::new(bounds.width / 2.0, bounds.height / 2.0);
    let angle = (position.y - centre.y)
        .atan2(position.x - centre.x)
        .to_degrees();
    let teinte = (angle + 360.0) % 360.0;
    Message::PickerCouleurTeinte(teinte)
}

// ---------- Le carré saturation / valeur ----------

const COTE_CARRE: f32 = 160.0;

struct CarreSatVal {
    /// La teinte dont le carré montre le champ — pour le repeindre.
    teinte: f32,
}

impl canvas::Program<Message> for CarreSatVal {
    type State = bool;

    fn update(
        &self,
        tenu: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let position = cursor.position_in(bounds)?;
        let sv = |p: Point| {
            Message::PickerCouleurSatVal(
                (p.x / bounds.width).clamp(0.0, 1.0),
                1.0 - (p.y / bounds.height).clamp(0.0, 1.0),
            )
        };
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                *tenu = true;
                Some(canvas::Action::publish(sv(position)))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if *tenu => {
                Some(canvas::Action::publish(sv(position)))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                *tenu = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::Crosshair
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let (r, g, b) = hsv_vers_rgb(self.teinte, 1.0, 1.0);
        let teinte = iced::Color::from_rgb8(r, g, b);

        // Le champ : blanc en horizontal, la teinte au bout ; noir en
        // vertical. Des bandes fines suffisent — le geste est à la souris,
        // pas au pixel.
        let pas = 2.0;
        let mut x = 0.0;
        while x < bounds.width {
            let melange = x / bounds.width;
            let couleur = iced::Color {
                r: 1.0 - (1.0 - teinte.r) * melange,
                g: 1.0 - (1.0 - teinte.g) * melange,
                b: 1.0 - (1.0 - teinte.b) * melange,
                a: 1.0,
            };
            frame.fill_rectangle(
                Point::new(x, 0.0),
                iced::Size::new(pas + 0.5, bounds.height),
                couleur,
            );
            x += pas;
        }
        let mut y = 0.0;
        while y < bounds.height {
            let alpha = 1.0 - y / bounds.height;
            frame.fill_rectangle(
                Point::new(0.0, y),
                iced::Size::new(bounds.width, pas + 0.5),
                iced::Color::from_rgba(0.0, 0.0, 0.0, 1.0 - alpha),
            );
            y += pas;
        }
        vec![frame.into_geometry()]
    }
}

/// Les couleurs rapides : la palette de base, fixe — un clic applique, la
/// roue reste ouverte pour la suite.
const RAPIDES: [(&str, &str); 8] = [
    ("Noir", "#1a1a2e"),
    ("Gris", "#7f8c8d"),
    ("Blanc", "#ffffff"),
    ("Rouge", "#e74c3c"),
    ("Orange", "#e67e22"),
    ("Vert", "#27ae60"),
    ("Bleu", "#3498db"),
    ("Violet", "#9b59b6"),
];

// ---------- La carte ----------

/// Le sélecteur, centré dans la fenêtre : la roue, le champ, la couleur
/// obtenue, et les gestes rapides.
pub fn view(state: &crate::app::State, palette: crate::ui::theme::Palette) -> Element<'_, Message> {
    let Some(picker) = state.picker_couleur.as_ref() else {
        return iced::widget::Space::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    };
    let hex = hsv_vers_hex(picker.teinte, picker.saturation, picker.valeur);
    let (r, g, b) = hsv_vers_rgb(picker.teinte, picker.saturation, picker.valeur);

    let roue = Canvas::new(RoueTeinte)
        .width(Length::Fixed(TAILLE_ROUE))
        .height(Length::Fixed(TAILLE_ROUE));

    let carre = Canvas::new(CarreSatVal {
        teinte: picker.teinte,
    })
    .width(Length::Fixed(COTE_CARRE))
    .height(Length::Fixed(COTE_CARRE));

    // L'aperçu : la couleur obtenue, et son écriture pour Typst.
    let apercu = container(
        row![
            container(
                iced::widget::Space::new()
                    .width(Length::Fixed(64.0))
                    .height(Length::Fixed(64.0))
            )
            .style(move |_t| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(r, g, b))),
                border: iced::Border {
                    width: 1.0,
                    color: palette.border,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
            palette.label(hex.clone()).size(13).color(palette.accent),
        ]
        .spacing(10),
    )
    .padding(4);

    // Les valeurs, en direct : le hex se tape, se copie, se colle — c'est
    // lui qui pilote la roue en retour.
    let valeur_hex = picker.hex_saisi.clone().unwrap_or_else(|| hex.clone());
    let saisie_hex = text_input("#rrggbb", &valeur_hex)
        .on_input(Message::PickerCouleurHex)
        .on_submit(Message::PickerCouleurAppliquer)
        .padding(4)
        .width(Length::Fixed(110.0));
    let rgb = palette
        .label(format!("R {r}  V {g}  B {b}"))
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    // Les couleurs rapides : la palette de base, fixe.
    let mut grille_rapides = row![].spacing(6);
    for (nom, valeur) in RAPIDES {
        let pastille = crate::theming::HexColor::parse(valeur)
            .ok()
            .map(|couleur| couleur.0);
        grille_rapides = grille_rapides.push(
            iced::widget::tooltip(
                button(
                    container(
                        iced::widget::Space::new()
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0)),
                    )
                    .style(move |_t| iced::widget::container::Style {
                        background: pastille.map(iced::Background::from),
                        border: iced::Border {
                            width: 1.0,
                            color: palette.border,
                            radius: 3.0.into(),
                        },
                        ..Default::default()
                    }),
                )
                .on_press(Message::PickerCouleurRapide(valeur.to_string()))
                .padding(3),
                container(palette.label(nom).size(12)).padding([4, 8]),
                iced::widget::tooltip::Position::Top,
            )
            .gap(4),
        );
    }

    // Le choix et l'application sont deux gestes distincts
    // (bug-report-10, 5.4) : l'un enregistre la couleur pour la pastille,
    // l'autre l'emploie — jamais les deux d'un seul clic.
    let gestes = column![
        row![
            button(palette.label("Enregistrer").size(theme::jetons::DETAIL))
                .on_press(Message::PickerCouleurEnregistrer)
                .padding([5, 12]),
            button(palette.label("Appliquer").size(theme::jetons::DETAIL))
                .on_press(Message::PickerCouleurAppliquer)
                .padding([5, 12]),
            button(palette.label("Fermer").size(theme::jetons::DETAIL))
                .on_press(Message::PickerCouleurFermer)
                .padding([5, 12]),
        ]
        .spacing(8),
        palette
            .label(
                "« Enregistrer » garde la couleur pour la pastille de la barre ; \
                 « Appliquer » l'emploie ici."
            )
            .size(theme::jetons::DETAIL),
    ]
    .spacing(6);

    let carte = column![
        palette.label("Couleur du texte").size(theme::jetons::TITRE),
        row![roue, column![carre, apercu].spacing(8)].spacing(14),
        row![saisie_hex, rgb]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        grille_rapides,
        gestes,
    ]
    .spacing(12);

    // Centré dans la fenêtre — jamais rejeté dans un coin.
    crate::ui::voile(
        container(container(carte).padding(18).style(theme::panneau(palette)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les couleurs primaires, à la roue comme au pixel.
    #[test]
    fn les_primaires_sortent_justes() {
        assert_eq!(hsv_vers_rgb(0.0, 1.0, 1.0), (255, 0, 0), "rouge");
        assert_eq!(hsv_vers_rgb(120.0, 1.0, 1.0), (0, 255, 0), "vert");
        assert_eq!(hsv_vers_rgb(240.0, 1.0, 1.0), (0, 0, 255), "bleu");
        assert_eq!(hsv_vers_rgb(0.0, 0.0, 1.0), (255, 255, 255), "blanc");
        assert_eq!(hsv_vers_rgb(0.0, 0.0, 0.0), (0, 0, 0), "noir");
    }

    /// L'aller-retour tient : une couleur revient à elle-même.
    #[test]
    fn l_aller_retour_hsv_hex_tient() {
        for teinte in [0.0, 37.0, 120.0, 200.0, 289.0, 359.0] {
            for (sat, val) in [(1.0, 1.0), (0.5, 0.7), (0.0, 0.4)] {
                let hex = hsv_vers_hex(teinte, sat, val);
                let (t2, s2, v2) = hex_vers_hsv(&hex).expect("hex lisible");
                let hex2 = hsv_vers_hex(t2, s2, v2);
                assert_eq!(hex, hex2, "teinte {teinte} sat {sat} val {val}");
            }
        }
    }

    /// Un hex court (`#f00`) se lit aussi.
    #[test]
    fn le_hex_court_se_lit() {
        let (t, s, v) = hex_vers_hsv("#f00").expect("hex court");
        assert_eq!(hsv_vers_hex(t, s, v), "#ff0000");
    }
}
