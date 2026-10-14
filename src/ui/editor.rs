use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::container;
use iced::{Element, Fill, Padding, alignment};

/// Mesure de lecture : la largeur maximale du texte, en pixels.
///
/// Environ 75 caractères à la police monospace de l'éditeur (un caractère
/// occupe ≈ 0,6 fois la taille du texte). Au-delà, retrouver le début de la
/// ligne suivante demande à l'œil un repérage qui coûte à la lecture ; en
/// deçà, la ligne se coupe trop souvent.
const MEASURE: f32 = 760.0;

/// Marge latérale minimale.
///
/// Elle n'a d'effet que si le panneau est plus étroit que la mesure ; sinon
/// c'est la mesure elle-même qui laisse du blanc de chaque côté.
const SIDE_MARGIN: f32 = 40.0;

/// Marge en haut du texte : respirer avant la première ligne.
const TOP_MARGIN: f32 = 28.0;

/// Marge en bas : de quoi voir venir la suite sans que le texte touche le bord.
const BOTTOM_MARGIN: f32 = 64.0;

/// La zone de rédaction.
///
/// Le texte est borné par la mesure et centré dans le panneau : c'est ce qui
/// donne à l'éditeur des marges latérales sur une fenêtre large, et ce qui
/// borne le retour à la ligne à une longueur lisible.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = Palette::from_theme(&theme::resolve(state.theme, state.system_theme));

    // La largeur demandée est fixe, mais iced la borne à l'espace disponible :
    // sur une fenêtre étroite le texte se réduit au lieu de déborder, et les
    // marges latérales prennent le relais.
    let page = container(state.editor.editor.view().map(Message::Editor))
        .width(Fill)
        .height(Fill)
        .max_width(MEASURE);

    // La zone de rédaction se détache sur le fond de l'application : sans
    // cette différence de teinte, la marge existe en géométrie mais reste
    // invisible — on ne voit pas de marge sur un fond uniforme.
    container(page)
        .width(Fill)
        .height(Fill)
        .align_x(alignment::Horizontal::Center)
        .padding(Padding {
            top: TOP_MARGIN,
            right: SIDE_MARGIN,
            bottom: BOTTOM_MARGIN,
            left: SIDE_MARGIN,
        })
        // Le tour est au fond de l'application, la page au ton du panneau : la
        // différence est ce qui rend la marge visible.
        .style(move |_| container::Style {
            background: Some(palette.background.into()),
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La mesure doit rester dans la fourchette de lisibilité : entre 45 et
    /// 90 caractères. À la police de l'éditeur, un caractère vaut environ 0,6
    /// fois la taille du texte.
    #[test]
    fn measure_is_within_readable_bounds() {
        let character_width = crate::editor::BODY_SIZE * 0.6;
        let characters = MEASURE / character_width;

        assert!(
            (45.0..=90.0).contains(&characters),
            "{characters:.0} caractères par ligne : hors de la fourchette lisible"
        );
    }

    /// Les marges doivent laisser de la place au texte : une mesure plus étroite
    /// que les deux marges réunies ne laisserait rien.
    #[test]
    fn margins_leave_room_for_the_text() {
        assert!(MEASURE > SIDE_MARGIN * 2.0);
        assert!(TOP_MARGIN > 0.0, "le texte ne doit pas toucher le haut");
        assert!(BOTTOM_MARGIN > 0.0, "le texte ne doit pas toucher le bas");
    }
}
