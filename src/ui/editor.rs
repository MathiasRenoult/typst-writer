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

    // ÉPREUVE (`TYPST_WRITER_EPREUVE_MENU=1`) : un recouvrement de forme connue,
    // dessiné alors même que rien n'est proposé.
    //
    // Un menu **vide** ne prouve rien : une bulle sans ligne n'a pas de hauteur,
    // et son absence à l'écran ne dit pas si c'est la structure qui l'empêche de
    // paraître. Ici il a trois lignes, une bulle colorée et une place connue.
    let overlay = match overlay {
        Some(overlay) => Some(overlay),
        None if std::env::var_os("TYPST_WRITER_EPREUVE_MENU").is_some()
            && MENU_EPREUVE.load(std::sync::atomic::Ordering::Relaxed) =>
        {
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

            Some((
                essai.into(),
                3.0 * COMPLETION_ROW + 2.0,
                iced::Point::new(80.0, 80.0),
            ))
        }
        None => None,
    };

    // **L'arbre a toujours la même forme, avec ou sans bulle.**
    //
    // Le défilement de l'éditeur vit dans l'état de son propre ascenseur, et cet
    // état est rattaché au *chemin des widgets* : faire d'un conteneur simple une
    // pile le recrée, et le défilement repart de zéro. Mesuré, la nuit du
    // rapport : défilement 7267, ouverture de la bulle, défilement **0** une
    // image plus tard — exactement « dès que je mets # ou @, l'éditeur scroll
    // tout en haut », et la fermeture fait de même. L'arbre invariable conserve
    // le défilement à travers les deux transitions (17086 mesuré).
    //
    // Un commentaire prétendait qu'avec une forme invariable le menu ne se
    // dessinait plus : refait, le constat était faux — les deux formes rendent
    // des images **pixel identiques**, bulle comprise.
    //
    // L'enfant de remplacement a une taille nulle : présent, donc le chemin des
    // widgets ne bouge pas ; invisible, donc la mise en page ne change pas d'un
    // pixel. À ne pas confondre avec un remplisseur, qui s'étendrait et donnerait
    // sa taille à la pile.
    // Le défilement **vu par la vue**, et non celui de l'état.
    //
    // Les deux ne disent pas la même chose : l'état est lu avant que l'arbre de
    // widgets ne soit reconstruit, si bien qu'un défilement perdu pendant la
    // reconstruction ne s'y voit jamais. C'est ici, et ici seulement, qu'on peut
    // constater qu'un menu ouvert fait remonter l'éditeur en tête de document.
    if std::env::var_os("TYPST_WRITER_VUE").is_some() {
        // Seulement au changement : l'image passe soixante fois par seconde, et
        // ce qu'on cherche est exactement la *discontinuité*.
        static DERNIERE: std::sync::Mutex<Option<(u32, bool)>> = std::sync::Mutex::new(None);

        let vue = (
            state.editor.editor.viewport_scroll().to_bits(),
            overlay.is_some(),
        );

        let mut derniere = DERNIERE.lock().unwrap();

        if *derniere != Some(vue) {
            println!(
                "[vue] défilement {:.0} | liste {}",
                f32::from_bits(vue.0),
                if vue.1 { "ouverte" } else { "fermée" }
            );

            *derniere = Some(vue);
        }
    }

    let bulle: Element<'_, Message> = match overlay {
        Some((content, height, anchor)) => {
            let panneau = panel_height(state);
            let top = crate::lsp::completion::placement(anchor.y, panneau, height);

            if std::env::var_os("TYPST_WRITER_VUE").is_some() {
                println!(
                    "[vue] bulle construite : ancre {:?} | top {:.0} | panneau {:.0} | hauteur {:.0}",
                    anchor, top, panneau, height
                );
            }

            container(content)
                .width(Fill)
                .height(Fill)
                .align_x(alignment::Horizontal::Left)
                .align_y(alignment::Vertical::Top)
                .padding(Padding {
                    top,
                    left: anchor.x,
                    ..Padding::default()
                })
                .into()
        }
        // **Toujours un conteneur**, même fermé : c'est le *contenu* qui
        // apparaît, jamais la nature de l'enfant de pile. Un enfant qui change
        // de widget est un nœud d'arbre qu'iced recrée — et on a mesuré que le
        // recouvrement ne se peint plus après une telle mutation, fût-ce avec un
        // aplat magenta criard.
        None => container(
            iced::widget::Space::new()
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0)),
        )
        .width(Fill)
        .height(Fill)
        .into(),
    };

    let page = container(iced::widget::stack![editor, bulle])
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

/// Bulle de l'épreuve d'arbre (`TYPST_WRITER_EPREUVE_MENU`).
///
/// L'épreuve a besoin de faire **ouvrir** la bulle au milieu d'un défilement :
/// c'est la transition — et non la présence — qui met en cause l'arbre. Ce
/// drapeau est ce que l'épreuve bascule ; le témoin de structure sert à
/// prouver que le chemin « arbre invariable » a bien été pris.
pub(crate) static MENU_EPREUVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
