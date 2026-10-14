use crate::app::{Message, State};

use crate::app::state::Completion;
use crate::ui::theme::Palette;

use iced::widget::{button, column, container, row, space};
use iced::{Element, Fill, Length, Padding, alignment};

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

/// Largeur de la liste de complétion.
const COMPLETION_WIDTH: f32 = 320.0;

/// Hauteur approximative d'une ligne de la liste, en pixels.
///
/// C'est une **estimation**, et elle suffit : la place calculée est bornée au
/// panneau, donc une imprécision ne peut pas faire sortir la liste de l'écran.
const COMPLETION_ROW: f32 = 28.0;

/// Combien d'éléments la liste montre d'un coup.
const COMPLETION_VISIBLE: usize = 8;

/// Marge en bas : de quoi voir venir la suite sans que le texte touche le bord.
const BOTTOM_MARGIN: f32 = 64.0;

/// Combien de lignes un survol montre au plus.
///
/// Un survol est une réponse courte, pas une page de documentation : au-delà, il
/// couvrirait le texte qu'on est en train d'écrire.
const HOVER_LINES: usize = 12;

/// Hauteur approximative d'une ligne de survol. Estimation, comme celle de la
/// liste : la place calculée est bornée au panneau.
const HOVER_ROW: f32 = 18.0;

/// La zone de rédaction.
///
/// Le texte est borné par la mesure et centré dans le panneau : c'est ce qui
/// donne à l'éditeur des marges latérales sur une fenêtre large, et ce qui
/// borne le retour à la ligne à une longueur lisible.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    // La largeur demandée est fixe, mais iced la borne à l'espace disponible :
    // sur une fenêtre étroite le texte se réduit au lieu de déborder, et les
    // marges latérales prennent le relais.
    let editor = state.editor.editor.view().map(Message::Editor);

    // Ce qui se dessine **par-dessus** le texte, à l'endroit du curseur : c'est
    // là que le regard est déjà, et l'on n'a pas à quitter la ligne qu'on écrit.
    // Les deux ne s'ouvrent jamais ensemble — demander l'un ferme l'autre — donc
    // il n'y a jamais qu'un seul cas à traiter.
    let overlay = match state.lsp.completion.as_ref() {
        Some(open) if !open.shown.is_empty() => Some((
            completion_popup(open, palette),
            open.shown.len().min(COMPLETION_VISIBLE) as f32 * COMPLETION_ROW + 2.0,
            open.anchor,
        )),
        _ => state.lsp.hover.as_ref().map(|hover| {
            let height =
                clamp_lines(&hover.text, HOVER_LINES).lines().count() as f32 * HOVER_ROW + 2.0;

            (hover_popup(hover, palette), height, hover.anchor)
        }),
    };

    // La forme de l'arbre change : conteneur simple quand rien ne se dessine,
    // pile quand une bulle s'ajoute. C'est la structure où le menu a été **vu**.
    // Une variante « arbre invariable » — un enfant vide en second, pour que le
    // chemin des widgets ne bouge jamais — a été essayée : le menu ne s'y
    // dessinait plus du tout. Elle est partie avec la raison qu'on lui donnait.
    let page = match overlay {
        Some((content, height, anchor)) => {
            let top = crate::lsp::completion::placement(anchor.y, panel_height(state), height);

            let popup = container(content)
                .width(Fill)
                .height(Fill)
                .align_x(alignment::Horizontal::Left)
                .align_y(alignment::Vertical::Top)
                // Le décalage se donne par une marge intérieure : c'est ainsi
                // qu'un élément de pile se pose ailleurs qu'au coin.
                .padding(Padding {
                    top,
                    left: anchor.x,
                    ..Padding::default()
                });

            container(iced::widget::stack![editor, popup])
        }
        None => container(editor),
    }
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

/// La place où le curseur peut se trouver : la fenêtre, moins les deux barres et
/// les marges du panneau. C'est le seul cadre qu'une bulle doive respecter.
fn panel_height(state: &State) -> f32 {
    // La hauteur **réellement vue** par l'éditeur quand il la connaît : c'est
    // elle qui donne le seul cadre où le curseur peut être. Le calcul par la
    // fenêtre reste en secours, pour la première image.
    let visible = state.editor.visible_height();

    if visible > 0.0 {
        return visible;
    }

    (state.window_size.height
        - crate::ui::toolbar::BAR_HEIGHT
        - crate::ui::status_bar::BAR_HEIGHT
        - TOP_MARGIN
        - BOTTOM_MARGIN)
        .max(0.0)
}

/// Les premières lignes d'un texte, empruntées et non copiées.
///
/// Le survol d'un symbole de tinymist peut être long ; ce qui s'affiche ne l'est
/// pas. Découper la chaîne pour n'en garder qu'un morceau coûterait une
/// allocation par image — et l'application en dessine soixante par seconde.
fn clamp_lines(text: &str, lines: usize) -> &str {
    let mut end = text.len();

    for (count, (index, _)) in text.match_indices('\n').enumerate() {
        if count + 1 == lines {
            end = index;

            break;
        }
    }

    &text[..end]
}

/// Ce que le serveur sait du symbole, posé près du curseur.
///
/// Le texte vient de tinymist, en markdown : il est montré tel quel, borné à
/// quelques lignes.
fn hover_popup(hover: &crate::app::state::Hover, palette: Palette) -> Element<'_, Message> {
    container(
        palette
            .label(clamp_lines(&hover.text, HOVER_LINES))
            .size(13.0)
            .width(Length::Fixed(COMPLETION_WIDTH)),
    )
    .padding(8)
    .style(bubble_style(palette))
    .into()
}

/// La liste de complétion, posée à l'endroit du curseur.
fn completion_popup<'a>(open: &'a Completion, palette: Palette) -> Element<'a, Message> {
    let total = open.shown.len();

    // La fenêtre suit la sélection : on doit pouvoir choisir au clavier plus bas
    // que ce qui tient à l'écran.
    let first = open
        .selected
        .saturating_sub(COMPLETION_VISIBLE.saturating_sub(1))
        .min(total.saturating_sub(COMPLETION_VISIBLE));

    let mut list = column![]
        .spacing(0.0)
        .width(Length::Fixed(COMPLETION_WIDTH));

    for (position, index) in open
        .shown
        .iter()
        .enumerate()
        .skip(first)
        .take(COMPLETION_VISIBLE)
    {
        let Some(item) = open.items.get(*index) else {
            continue;
        };

        let selected = position == open.selected;

        let mut line = row![]
            .width(Fill)
            .padding([5, 8])
            .spacing(8)
            .align_y(alignment::Vertical::Center);

        line = line.push(palette.label(&item.label).size(13).color(if selected {
            palette.background
        } else {
            palette.text
        }));

        if let Some(kind) = item.kind
            && !kind.label().is_empty()
        {
            line = line.push(space::horizontal().width(Length::Fill));
            line = line.push(palette.label(kind.label()).size(11).color(if selected {
                palette.background
            } else {
                palette.text_muted
            }));
        }

        list = list.push(
            button(line)
                .width(Fill)
                .padding(0)
                .style(move |_, _| button::Style {
                    background: selected.then(|| palette.accent.into()),
                    text_color: if selected {
                        palette.background
                    } else {
                        palette.text
                    },
                    ..Default::default()
                })
                .on_press(Message::CompletionPick(position)),
        );
    }

    container(list)
        .width(Length::Fixed(COMPLETION_WIDTH))
        .style(bubble_style(palette))
        .into()
}

/// Le fond d'une bulle : **détaché de ce qu'elle recouvre**.
///
/// `surface` est la couleur de la zone de rédaction elle-même : un menu peint de
/// cette teinte se confond avec le texte qu'il couvre, et l'on croit qu'il ne
/// s'est jamais ouvert. C'est très exactement ce qui est arrivé. La bulle prend
/// donc la teinte « active » — celle d'un élément choisi, mesurée ici à 50
/// niveaux au-dessus du fond de l'éditeur, donc visible — et un trait qui la
/// borde.
fn bubble_style(palette: Palette) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(palette.surface_active.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
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

    /// Un survol long se borne, sans rien copier et sans couper au milieu d'un
    /// caractère.
    #[test]
    fn a_long_hover_is_clamped() {
        assert_eq!(clamp_lines("une\ndeux\ntrois", 2), "une\ndeux");
        assert_eq!(clamp_lines("une\ndeux", 5), "une\ndeux");
        assert_eq!(clamp_lines("sans fin", 3), "sans fin");
        assert_eq!(clamp_lines("", 3), "");

        // Une ligne accentuée se coupe à sa frontière, jamais avant.
        assert_eq!(clamp_lines("éé\nà", 1), "éé");
    }

    /// Les marges doivent laisser de la place au texte, et le texte ne doit
    /// toucher aucun bord.
    ///
    /// Vérifié à la **compilation** : un invariant de mise en page qui se casse
    /// doit arrêter la construction, et non une épreuve qu'on peut ignorer.
    const _: () = assert!(MEASURE > SIDE_MARGIN * 2.0);
    const _: () = assert!(TOP_MARGIN > 0.0);
    const _: () = assert!(BOTTOM_MARGIN > 0.0);
}
