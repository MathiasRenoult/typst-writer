use crate::app::{Message, State};

use crate::app::state::Completion;
use crate::ui::theme::Palette;

use iced::advanced::layout::{self, Limits};
use iced::advanced::overlay;
use iced::advanced::widget::Tree;
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::widget::{button, column, container, row, space};
use iced::{Element, Event, Fill, Length, Padding, Rectangle, Size, Vector, alignment, mouse};

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

fn empty_completion_message(trigger: char, items_empty: bool, awaiting: bool) -> &'static str {
    if awaiting {
        return "Recherche des suggestions…";
    }

    if trigger == crate::lsp::completion::CITATION {
        if items_empty {
            "Aucune référence dans le projet"
        } else {
            "Aucune référence correspondante"
        }
    } else if items_empty {
        "Aucune suggestion disponible"
    } else {
        "Aucune suggestion correspondante"
    }
}

/// Marge en bas : de quoi voir venir la suite sans que le texte touche le bord.
const BOTTOM_MARGIN: f32 = 64.0;

/// Combien de lignes un survol montre au plus.
///
/// Un survol est une réponse courte, pas une page de documentation : au-delà, il
/// couvrirait le texte qu'on est en train d'écrire.
const HOVER_LINES: usize = 12;

/// Ce qui flotte au-dessus de l'éditeur : liste de complétion ou survol.
struct Bulle<'a> {
    contenu: Element<'a, Message>,
    /// Coin haut-gauche, en coordonnées de la vue de l'éditeur.
    position: iced::Point,
}

/// L'éditeur, et ce qui flotte au-dessus de lui.
///
/// Le recouvrement est un véritable `overlay` iced, pas un enfant de pile.
/// Deux conséquences, mesurées l'une contre l'autre :
///
/// - il ne participe pas à la mise en page : ouvrir la liste ne fait pas
///   repartir le défilement de l'éditeur de zéro, et l'arbre n'a plus à garder
///   une forme invariable pour préserver ce défilement ;
/// - iced lui donne les événements d'abord, mais ne les retire à la base que là
///   où la bulle est réellement interactive (ses boutons). Un clic hors de la
///   bulle tombe donc dans l'éditeur, et le placement du curseur reste, de bout
///   en bout, l'affaire de l'éditeur.
struct EditorOverlay<'a> {
    editor: Element<'a, Message>,
    bulle: Option<Bulle<'a>>,
}

/// Ce que la bulle doit montrer, et où.
///
/// Elle ne sert plus qu'au **survol** (tinymist) : la complétion s'affiche
/// désormais dans une fenêtre fixe au centre, pas près du curseur.
fn bulle(state: &State, palette: Palette) -> Option<Bulle<'_>> {
    if let Some(hover) = state.lsp.hover.as_ref() {
        return Some(Bulle {
            contenu: hover_popup(hover, palette),
            position: hover.anchor,
        });
    }

    // ÉPREUVE : une bulle de forme connue, sans rien à proposer.
    if crate::app::epreuves::menu_ouvert() {
        let essai = container(
            column![
                palette.label("premier essai").size(13),
                palette.label("deuxieme essai").size(13),
                palette.label("troisieme essai").size(13),
            ]
            .spacing(4.0),
        )
        .padding(8)
        .style(bubble_style(palette));

        return Some(Bulle {
            contenu: essai.into(),
            position: iced::Point::new(80.0, 80.0),
        });
    }

    None
}

impl<'a> Widget<Message, iced::Theme, iced::Renderer> for EditorOverlay<'a> {
    fn size(&self) -> Size<Length> {
        self.editor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.editor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &Limits,
    ) -> layout::Node {
        self.editor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn children(&self) -> Vec<Tree> {
        let vide: Element<'_, Message> = iced::widget::Space::new()
            .width(Length::Fixed(0.0))
            .height(Length::Fixed(0.0))
            .into();

        let bulle = match &self.bulle {
            Some(bulle) => &bulle.contenu,
            None => &vide,
        };

        vec![Tree::new(&self.editor), Tree::new(bulle)]
    }

    fn diff(&self, tree: &mut Tree) {
        let vide: Element<'_, Message> = iced::widget::Space::new()
            .width(Length::Fixed(0.0))
            .height(Length::Fixed(0.0))
            .into();

        let bulle = match &self.bulle {
            Some(bulle) => &bulle.contenu,
            None => &vide,
        };

        tree.diff_children(&[self.editor.as_widget(), bulle.as_widget()]);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.editor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.editor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.editor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.editor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        // Les recouvrements de l'éditeur d'abord : les perdre, c'est perdre
        // ses propres panneaux. Puis la bulle, à sa place absolue — les
        // deux vivent dans un même groupe, comme le fait le `tooltip`
        // d'iced.
        let mut enfants = Vec::new();

        let (premier, reste) = tree.children.split_at_mut(1);

        let editeur = self.editor.as_widget_mut().overlay(
            &mut premier[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        enfants.extend(editeur);

        if let Some(bulle) = self.bulle.as_mut() {
            enfants.push(overlay::Element::new(Box::new(BulleRecouvrement {
                contenu: &mut bulle.contenu,
                arbre: &mut reste[0],
                position: layout.position()
                    + translation
                    + Vector::new(bulle.position.x, bulle.position.y),
            })));
        }

        if enfants.is_empty() {
            None
        } else {
            Some(overlay::Group::with_children(enfants).overlay())
        }
    }
}

impl<'a> From<EditorOverlay<'a>> for Element<'a, Message> {
    fn from(widget: EditorOverlay<'a>) -> Self {
        Element::new(widget)
    }
}

/// Le recouvrement lui-même : la bulle, dessinée à une place absolue.
struct BulleRecouvrement<'a, 'b> {
    contenu: &'b mut Element<'a, Message>,
    arbre: &'b mut Tree,
    /// Coin haut-gauche, en coordonnées absolues de la fenêtre.
    position: iced::Point,
}

impl overlay::Overlay<Message, iced::Theme, iced::Renderer> for BulleRecouvrement<'_, '_> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let contenu = self.contenu.as_widget_mut().layout(
            self.arbre,
            renderer,
            &Limits::new(Size::ZERO, bounds),
        );

        let size = contenu.bounds().size();

        // La bulle reste dans la fenêtre, quoi qu'il arrive.
        let x = self
            .position
            .x
            .clamp(0.0, (bounds.width - size.width).max(0.0));
        let y = self
            .position
            .y
            .clamp(0.0, (bounds.height - size.height).max(0.0));

        layout::Node::with_children(size, vec![contenu]).translate(Vector::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.contenu.as_widget().draw(
            self.arbre,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.contenu.as_widget_mut().update(
            self.arbre,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.contenu.as_widget().mouse_interaction(
            self.arbre,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        )
    }
}

/// La zone de rédaction.
///
/// Le texte est borné par la mesure et centré dans le panneau : c'est ce qui
/// donne à l'éditeur des marges latérales sur une fenêtre large, et ce qui
/// borne le retour à la ligne à une longueur lisible.
pub fn view(state: &State) -> Element<'_, Message> {
    vue(state, &state.editor, true)
}

pub fn vue<'a>(
    state: &'a State,
    source: &'a crate::editor::EditorState,
    active: bool,
) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let editor = source.editor.view().map(Message::Editor);
    let bulle = if active { bulle(state, palette) } else { None };
    crate::app::epreuves::note_vue(source.editor.viewport_scroll(), bulle.is_some());

    let page = container(EditorOverlay { editor, bulle })
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
        // **Le cadre de la rédaction est de la rédaction.** Lui donner le ton
        // des panneaux dessinait une bande sombre tout autour du texte, et
        // l'éditeur paraissait posé sur le fond de l'application au lieu d'y
        // être. La zone de rédaction et son cadre portent donc la même teinte —
        // celle du fond de l'application — et il n'y a plus de couture. Les
        // panneaux gardent la leur : ce sont eux qui se détachent, et non le
        // texte.
        .style(move |_| container::Style {
            background: Some(palette.background.into()),
            ..Default::default()
        })
        .into()
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

/// La liste de complétion, posée dans la fenêtre centrale.
pub(super) fn completion_popup<'a>(
    open: &'a Completion,
    palette: Palette,
    width: f32,
    awaiting: bool,
) -> Element<'a, Message> {
    let total = open.shown.len();

    // La fenêtre suit la sélection : on doit pouvoir choisir au clavier plus bas
    // que ce qui tient à l'écran.
    let first = open
        .selected
        .saturating_sub(COMPLETION_VISIBLE.saturating_sub(1))
        .min(total.saturating_sub(COMPLETION_VISIBLE));

    if open.shown.is_empty() {
        let message = empty_completion_message(open.trigger, open.items.is_empty(), awaiting);

        return container(
            container(palette.label(message).size(13).color(palette.text_muted))
                .width(Fill)
                .height(Length::Fixed(COMPLETION_ROW))
                .padding([5, 8]),
        )
        .width(Length::Fixed(width))
        .style(bubble_style(palette))
        .into();
    }

    let mut list = column![].spacing(0.0).width(Length::Fixed(width));

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
        .width(Length::Fixed(width))
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
pub(super) fn bubble_style(palette: Palette) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(palette.surface_active.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: crate::ui::theme::jetons::RAYON.into(),
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

    /// Même sans résultat, la fenêtre annonce ce qu'elle attend.
    #[test]
    fn empty_completion_message_matches_its_trigger_and_wait_state() {
        assert_eq!(
            empty_completion_message(crate::lsp::completion::CITATION, true, false),
            "Aucune référence dans le projet"
        );
        assert_eq!(
            empty_completion_message('#', true, true),
            "Recherche des suggestions…"
        );
        assert_eq!(
            empty_completion_message('#', true, false),
            "Aucune suggestion disponible"
        );
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
