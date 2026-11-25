//! La couche interactive posée sur l'image d'une page.
//!
//! Elle ne recopie jamais le texte : elle dessine les surlignages et publie
//! les gestes de sélection. Le texte lui-même reste dans `DocumentTexte`.

use crate::app::Message;
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

/// Ce que la couche doit dessiner et savoir de la page.
pub struct CoucheApercu {
    pub page: usize,
    pub taille_page: (f32, f32),
    pub selection: Vec<Rectangle>,
    pub resultats: Vec<Rectangle>,
    pub courant: Option<Rectangle>,
    pub liens: Vec<Rectangle>,
    pub couleur_selection: Color,
    pub couleur_resultat: Color,
    pub couleur_courante: Color,
    pub maj_tenu: bool,
}

/// L'état de souris propre à une page.
#[derive(Debug, Default)]
pub struct EtatCouche {
    tenu: bool,
    /// Le point d'appui d'un clic simple, tant qu'aucun mouvement n'a
    /// déclenché la sélection.
    depart: Option<Point>,
    derniere_pression: Option<(std::time::Instant, Point, u8)>,
}

/// En deçà de ce déplacement, un appui reste un clic simple : il ne doit
/// sélectionner aucun texte.
const SEUIL_DE_GLISSER: f32 = 3.0;

impl canvas::Program<Message> for CoucheApercu {
    type State = EtatCouche;

    fn update(
        &self,
        etat: &mut Self::State,
        evenement: &canvas::Event,
        limites: Rectangle,
        curseur: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Le relâchement peut arriver hors de la page : il doit tout de même
        // terminer le glisser, sinon la sélection reprendrait au prochain
        // passage de la souris.
        if matches!(
            evenement,
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left
            ))
        ) {
            etat.tenu = false;
            let clic = etat.depart.take().is_some();
            return Some(
                canvas::Action::publish(if clic {
                    Message::SelectionApercuClic
                } else {
                    Message::SelectionApercuRelacher
                })
                .and_capture(),
            );
        }

        let position = curseur.position_in(limites)?;

        match evenement {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                // Un clic sur un lien ne commence pas une sélection : il
                // ouvre le lien. Le curseur le montre déjà.
                if let Some(rang) = self.lien_sous(position) {
                    return Some(
                        canvas::Action::publish(Message::LienApercuActiver(self.page, rang))
                            .and_capture(),
                    );
                }

                let maintenant = std::time::Instant::now();
                let clics = match etat.derniere_pression {
                    Some((instant, point, compte))
                        if maintenant.duration_since(instant)
                            < std::time::Duration::from_millis(400)
                            && (point.x - position.x).abs() < 5.0
                            && (point.y - position.y).abs() < 5.0 =>
                    {
                        compte.saturating_add(1).min(3)
                    }
                    _ => 1,
                };

                etat.derniere_pression = Some((maintenant, position, clics));
                etat.tenu = true;

                if clics >= 2 {
                    // Double et triple clic : le mot ou la ligne se
                    // sélectionne immédiatement, comme dans tout lecteur.
                    etat.depart = None;
                    return Some(
                        canvas::Action::publish(Message::SelectionApercuAppuyer {
                            page: self.page,
                            point: point_de_page(position, limites, self.taille_page),
                            etendue: self.maj_tenu,
                            clics,
                        })
                        .and_capture(),
                    );
                }

                // Clic simple : on retient le point, sans rien sélectionner.
                // La sélection ne naîtra qu'au premier mouvement réel ; la
                // sélection précédente s'efface dès l'appui.
                etat.depart = Some(position);
                Some(canvas::Action::publish(Message::SelectionApercuClic).and_capture())
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if etat.tenu => {
                let point = point_de_page(position, limites, self.taille_page);

                if let Some(depart) = etat.depart {
                    let dx = depart.x - position.x;
                    let dy = depart.y - position.y;
                    if dx * dx + dy * dy < SEUIL_DE_GLISSER * SEUIL_DE_GLISSER {
                        return Some(canvas::Action::capture());
                    }

                    etat.depart = None;
                    // Un glisser n'est pas un clic : la pression suivante ne
                    // doit pas être comptée comme un double-clic.
                    etat.derniere_pression = None;
                    return Some(
                        canvas::Action::publish(Message::SelectionApercuGlisserDepuis {
                            page: self.page,
                            ancre: point_de_page(depart, limites, self.taille_page),
                            point,
                            etendue: self.maj_tenu,
                        })
                        .and_capture(),
                    );
                }

                Some(
                    canvas::Action::publish(Message::SelectionApercuGlisser {
                        page: self.page,
                        point,
                    })
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _etat: &Self::State,
        limites: Rectangle,
        curseur: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if curseur
            .position_in(limites)
            .is_some_and(|position| self.liens.iter().any(|lien| lien.contains(position)))
        {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::Text
        }
    }

    fn draw(
        &self,
        _etat: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        limites: Rectangle,
        _curseur: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut geometry = canvas::Frame::new(renderer, limites.size());

        for rect in &self.selection {
            geometry.fill_rectangle(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
                self.couleur_selection,
            );
        }

        for rect in &self.resultats {
            geometry.fill_rectangle(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
                self.couleur_resultat,
            );
        }

        if let Some(rect) = self.courant {
            geometry.fill_rectangle(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
                self.couleur_courante,
            );
            geometry.stroke(
                &canvas::Path::rectangle(
                    Point::new(rect.x, rect.y),
                    Size::new(rect.width, rect.height),
                ),
                canvas::Stroke::default()
                    .with_width(1.5)
                    .with_color(self.couleur_courante),
            );
        }

        vec![geometry.into_geometry()]
    }
}

impl CoucheApercu {
    fn lien_sous(&self, point: Point) -> Option<usize> {
        self.liens.iter().position(|lien| lien.contains(point))
    }
}

/// Convertit un point de l'écran en point de la page Typst.
///
/// `position` vient de `Cursor::position_in` : il est déjà relatif au canevas.
/// Il ne faut donc pas retirer `limites.x` ni `limites.y` une seconde fois.
fn point_de_page(position: Point, limites: Rectangle, taille_page: (f32, f32)) -> (f32, f32) {
    let x = if limites.width > 0.0 {
        position.x / limites.width * taille_page.0
    } else {
        0.0
    };
    let y = if limites.height > 0.0 {
        position.y / limites.height * taille_page.1
    } else {
        0.0
    };

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_point_de_page_est_relatif_au_canevas() {
        let limites = Rectangle {
            x: 200.0,
            y: 300.0,
            width: 100.0,
            height: 200.0,
        };

        assert_eq!(
            point_de_page(Point::new(50.0, 50.0), limites, (200.0, 400.0)),
            (100.0, 100.0)
        );
    }
}
