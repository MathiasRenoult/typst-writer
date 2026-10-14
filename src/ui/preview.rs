use crate::app::{Message, State};
use crate::typst::render::Diagnostic;
use crate::ui::theme::{self, Palette};
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, image, row, scrollable, space, text};
use iced::{ContentFit, Element, Fill, Length, color};
use std::path::Path;

/// Taille du texte courant.
const BODY_SIZE: f32 = 14.0;

/// Espacement entre les pages affichées.
const PAGE_GAP: f32 = 12.0;

/// Marge intérieure du panneau.
const PANEL_PADDING: f32 = 16.0;

/// Densité de rendu des pages (pixels par point).
///
/// Doit correspondre à `crate::typst::render::PIXEL_PER_PT` : c'est le rapport
/// entre les pixels de l'image et sa taille logique à l'écran.
const PIXEL_PER_PT: f64 = 2.0;

/// Pages rendues au-delà du viewport, de part et d'autre.
///
/// Une page de marge garantit qu'un défilement rapide ne révèle jamais un
/// cadre vide, sans pour autant rasteriser tout le document.
const VIEWPORT_MARGIN: usize = 1;

/// Identifiant du défilement de l'aperçu.
///
/// C'est par lui que l'application peut amener l'aperçu sur une page précise :
/// un widget ne se pilote pas de l'extérieur sans être nommé.
pub(crate) const SCROLL_ID: &str = "apercu";

/// Dimensions logiques (pixels écran) des pages du document.
///
/// La géométrie est connue avant tout rendu : elle sert à calculer la hauteur
/// totale du panneau et à déterminer quelles pages sont visibles, ce qui rend
/// la liste de pages virtualisable.
fn page_sizes(state: &State) -> Vec<(f32, f32)> {
    match state.render.document.as_ref() {
        Some(document) => sizes_of(document),
        None => Vec::new(),
    }
}

/// Dimensions logiques des pages d'un document.
///
/// Distinct de [`page_sizes`] pour être appelable depuis le gestionnaire de
/// messages, qui tient le document sans passer par la vue.
pub(crate) fn sizes_of(document: &typst_layout::PagedDocument) -> Vec<(f32, f32)> {
    let document = document;

    (0..crate::typst::render::page_count(document))
        .map(|index| crate::typst::render::page_size(document, index).unwrap_or((595.0, 842.0)))
        // Les images sont rendues à 2 px/pt : on les affiche à leur taille
        // logique, et la géométrie suit cette même échelle.
        .map(|(width, height)| {
            (
                (width / PIXEL_PER_PT) as f32,
                (height / PIXEL_PER_PT) as f32,
            )
        })
        .collect()
}

/// Décalage de défilement qui amène la page `index` en tête du panneau.
///
/// La marge intérieure du panneau n'appartient pas au repère du défilement :
/// le décalage absolu du viewport vaut donc `page_offset` moins cette marge.
pub(crate) fn scroll_offset_for_page(sizes: &[(f32, f32)], index: usize) -> f32 {
    (page_offset(sizes, index) - PANEL_PADDING).max(0.0)
}

/// Décalage vertical du sommet de la page `index` dans le contenu défilant.
pub(crate) fn page_offset(sizes: &[(f32, f32)], index: usize) -> f32 {
    sizes
        .iter()
        .take(index)
        .fold(PANEL_PADDING, |offset, (_, height)| {
            offset + height + PAGE_GAP
        })
}

/// Première et dernière page visibles dans la fenêtre `[top, bottom]`, marge
/// comprise. Les deux bornes sont exprimées dans le contenu défilant.
///
/// Le calcul utilise les hauteurs réelles des pages : il reste juste même sur
/// un document dont les pages n'ont pas toutes la même taille.
fn visible_pages(sizes: &[(f32, f32)], top: f32, bottom: f32) -> (usize, usize) {
    let Some(last_index) = sizes.len().checked_sub(1) else {
        return (0, 0);
    };

    // Première page dont le bas atteint le haut de la fenêtre.
    let mut first = 0;
    let mut offset = 0.0;

    for (index, (_, height)) in sizes.iter().enumerate() {
        if offset + height >= top {
            first = index;
            break;
        }

        offset += height + PAGE_GAP;
        first = index + 1;
    }

    // Dernière page dont le haut reste au-dessus du bas de la fenêtre.
    let mut last = first.min(last_index);

    for index in first..=last_index {
        if page_offset(sizes, index) > bottom {
            break;
        }

        last = index;
    }

    (
        first.saturating_sub(VIEWPORT_MARGIN),
        (last + VIEWPORT_MARGIN).min(last_index),
    )
}

pub fn view(state: &State) -> Element<'_, Message> {
    let render = &state.render;
    let sizes = page_sizes(state);
    let palette = Palette::from_theme(&theme::resolve(state.theme, state.system_theme));

    // Le fichier ouvert : c'est lui seul dont les diagnostics peuvent être
    // localisés, puisque c'est le seul que l'éditeur affiche. Emprunté à
    // l'état, et non cloné, pour que les éléments retournés vivent aussi
    // longtemps que lui.
    let open = state.editor.path.as_deref();

    let mut content = column![]
        .spacing(PAGE_GAP)
        .padding(PANEL_PADDING)
        .width(Fill);

    if let Some(status) = &state.export_status {
        content = content.push(text(status).size(13).color(color!(0x8a8a8a)));
    }

    // Diagnostics du compilateur, en tête du panneau.
    if !render.errors.is_empty() {
        content = content.push(diagnostics_panel(&render.errors, true, open, palette));
    }

    if !render.warnings.is_empty() {
        content = content.push(diagnostics_panel(&render.warnings, false, open, palette));
    }

    if sizes.is_empty() {
        // État d'attente avant la première compilation.
        let message = if render.pending {
            "Compilation…"
        } else if render.done {
            "Aucune page à afficher."
        } else {
            "Écrivez du contenu : le document est compilé automatiquement."
        };

        content = content.push(text(message).size(BODY_SIZE));

        return scrollable(content)
            .id(SCROLL_ID)
            .width(Fill)
            .height(Fill)
            // Aucune page : le viewport ne sert à rien, mais le message garde
            // la forme attendue par l'état de rendu.
            .on_scroll(|_| Message::ViewportChanged(0, 0))
            .into();
    }

    // Seules les pages proches du viewport sont matérialisées ; les autres
    // sont remplacées par un unique espaceur de la hauteur exacte qu'elles
    // occuperaient. Un document de 500 pages coûte donc le même nombre de
    // widgets qu'un document d'une page, et la géométrie du défilement reste
    // exacte — les hauteurs de pages sont connues avant tout rendu.
    let last_index = sizes.len() - 1;
    let (first, last) = render.visible.unwrap_or((0, last_index));

    let first = first.min(last_index);
    let last = last.clamp(first, last_index);

    let head = page_offset(&sizes, first);

    if head > PANEL_PADDING {
        content = content.push(
            space::vertical()
                .height(Length::Fixed(head - PANEL_PADDING))
                .width(Fill),
        );
    }

    for index in first..=last {
        content = content.push(page_view(state, index, sizes[index]));
    }

    if last < last_index {
        let tail = page_offset(&sizes, last_index + 1) - page_offset(&sizes, last + 1);

        if tail > 0.0 {
            content = content.push(space::vertical().height(Length::Fixed(tail)).width(Fill));
        }
    }

    scrollable(content)
        .id(SCROLL_ID)
        .width(Fill)
        .height(Fill)
        .on_scroll(move |viewport| {
            // La marge intérieure du panneau n'est pas dans le repère du
            // viewport : on la retire pour raisonner sur le contenu défilant.
            let top = viewport.absolute_offset().y - PANEL_PADDING;
            let bottom = top + viewport.bounds().height;

            let (first, last) = visible_pages(&sizes, top, bottom);

            Message::ViewportChanged(first, last)
        })
        .into()
}

/// Une page : l'image si elle est rasterisée, sinon un cadre de la même
/// taille (la géométrie du document est connue avant tout rendu).
fn page_view(state: &State, index: usize, size: (f32, f32)) -> Element<'_, Message> {
    let render = &state.render;
    let (width, height) = size;

    // Double tampon : l'image de la page est affichée dès qu'elle existe,
    // même si elle date de la compilation précédente. Une page modifiée se
    // met à jour en place quand son nouveau rendu arrive, sans jamais
    // disparaître de l'écran.
    //
    // Le `Handle` mis en cache est cloné tel quel : il porte le même
    // identifiant, donc le backend graphique retrouve la texture déjà
    // téléversée au lieu d'en créer une neuve à chaque image.
    match render.page_cache.get(&index) {
        Some(handle) => row![
            image(handle.clone())
                .width(Length::Fixed(width))
                .height(Length::Fixed(height))
                .content_fit(ContentFit::Contain)
        ]
        .width(Fill)
        .into(),

        None => container(
            text(if render.rendering.contains(&index) {
                "Rendu…"
            } else {
                ""
            })
            .size(13)
            .color(color!(0x666666)),
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .center_x(Fill)
        .center_y(Fill)
        .style(container::rounded_box)
        .into(),
    }
}

/// Bandeau listant les erreurs (ou avertissements) du compilateur.
///
/// Le document compilé est l'assemblage, qui inclut les chapitres : un
/// diagnostic peut donc tomber dans un fichier qui n'est pas celui qu'on est en
/// train d'écrire. Ceux-là nomment leur fichier et ne proposent pas de saut —
/// un saut calculé pour un autre fichier mènerait au mauvais endroit, ce qui est
/// pire que pas de saut du tout.
fn diagnostics_panel<'a>(
    diagnostics: &'a [Diagnostic],
    is_error: bool,
    open: Option<&'a Path>,
    palette: Palette,
) -> Element<'a, Message> {
    let title = if is_error {
        format!(
            "{} erreur(s) — le document ne compilera pas en PDF",
            diagnostics.len()
        )
    } else {
        format!("{} avertissement(s)", diagnostics.len())
    };

    let accent = if is_error {
        palette.danger
    } else {
        palette.warning
    };

    let mut list = column![text(title).size(13).color(accent)].spacing(4);

    for diagnostic in diagnostics {
        let here = diagnostic.file.as_deref() == open;

        let location = match (diagnostic.line, diagnostic.column) {
            (Some(line), Some(column)) => format!("ligne {line}, colonne {column}"),
            (Some(line), None) => format!("ligne {line}"),
            _ => String::new(),
        };

        // Le fichier n'est nommé que s'il diffère de celui qui est ouvert :
        // répéter « le fichier que vous éditez » sur chaque ligne serait du
        // bruit.
        let where_ = match (&diagnostic.file, here) {
            (Some(path), false) => format!("{} — ", file_name(path)),
            _ => String::new(),
        };

        let label = if location.is_empty() {
            format!("{where_}{}", diagnostic.message)
        } else {
            format!("{where_}{location} — {}", diagnostic.message)
        };

        let jump = if here {
            diagnostic
                .line
                .zip(diagnostic.column)
                .map(|(line, column)| Message::GotoError(line, column))
        } else {
            None
        };

        list = list.push(
            button(text(label).size(13).wrapping(Wrapping::Word).width(Fill))
                .on_press_maybe(jump)
                .padding([6, 10])
                .width(Fill)
                .style(button::text),
        );
    }

    container(list)
        .padding(10)
        .width(Fill)
        .style(move |_| panel_style(is_error, palette))
        .into()
}

/// Nom d'un fichier, pour nommer un diagnostic venu d'ailleurs.
///
/// Le nom seul suffit : le chemin complet est le même préfixe pour tout le
/// monde, et l'infobulle d'une ligne de chapitre le donne déjà.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

/// Fond et bordure du bandeau de diagnostics.
fn panel_style(is_error: bool, palette: Palette) -> container::Style {
    let accent = if is_error {
        palette.danger
    } else {
        palette.warning
    };

    container::Style {
        background: Some(palette.surface_hover.into()),
        border: iced::Border {
            color: accent,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le défilement amène la page demandée en tête du panneau.
    ///
    /// Le décalage est exprimé dans le repère du défilement, qui ne comprend
    /// pas la marge intérieure du panneau : d'où la différence avec
    /// `page_offset`.
    #[test]
    fn scrolling_puts_the_wanted_page_at_the_top() {
        let sizes = vec![(100.0, 100.0); 3];

        assert_eq!(scroll_offset_for_page(&sizes, 0), 0.0);
        assert_eq!(scroll_offset_for_page(&sizes, 1), 100.0 + PAGE_GAP);
        assert_eq!(scroll_offset_for_page(&sizes, 2), 2.0 * (100.0 + PAGE_GAP));

        // Le décalage suit toujours la géométrie de la page visée, quelle que
        // soit sa hauteur.
        let mixed = vec![(100.0, 50.0), (100.0, 200.0), (100.0, 100.0)];

        assert_eq!(
            scroll_offset_for_page(&mixed, 2),
            50.0 + PAGE_GAP + 200.0 + PAGE_GAP
        );
    }

    /// Cinq pages de 100 px, séparées par `PAGE_GAP`.
    fn sizes() -> Vec<(f32, f32)> {
        vec![(600.0, 100.0); 5]
    }

    /// Le fenêtre visible, telle que `view()` la calcule depuis le viewport.
    fn window(offset: f32, height: f32) -> (f32, f32) {
        let top = offset - PANEL_PADDING;

        (top, top + height)
    }

    #[test]
    fn offsets_are_cumulative_and_include_padding() {
        let sizes = sizes();

        assert_eq!(page_offset(&sizes, 0), PANEL_PADDING);
        assert_eq!(page_offset(&sizes, 1), PANEL_PADDING + 100.0 + PAGE_GAP);
        assert_eq!(
            page_offset(&sizes, 2),
            PANEL_PADDING + 2.0 * (100.0 + PAGE_GAP)
        );
        // Au-delà de la dernière page : la hauteur totale du contenu.
        assert_eq!(
            page_offset(&sizes, sizes.len()),
            PANEL_PADDING + 5.0 * (100.0 + PAGE_GAP)
        );
    }

    /// La page sous le curseur est toujours dans l'intervalle rendu : c'est ce
    /// qui garantit qu'un défilement ne révèle jamais un cadre vide.
    #[test]
    fn visible_pages_cover_the_scrolled_position() {
        let sizes = sizes();
        let stride = 100.0 + PAGE_GAP;

        for scroll in 0..400 {
            let (top, bottom) = window(scroll as f32, 300.0);
            let (first, last) = visible_pages(&sizes, top, bottom);

            assert!(first <= last, "intervalle inversé à {scroll}");
            assert!(last < sizes.len(), "intervalle hors bornes à {scroll}");

            // La page dont le sommet est dans la fenêtre est bien incluse.
            let pointed = ((scroll as f32 + 150.0 - PANEL_PADDING) / stride) as usize;

            assert!(
                pointed >= first && pointed <= last,
                "page pointue {pointed} hors de [{first}, {last}] au scroll {scroll}"
            );
        }
    }

    /// La marge d'une page est bien ajoutée des deux côtés.
    #[test]
    fn visible_pages_add_a_page_of_margin() {
        let sizes = sizes();
        let (top, bottom) = window(0.0, 100.0);

        let (first, last) = visible_pages(&sizes, top, bottom);

        assert_eq!(first, 0, "pas de marge négative en haut du document");
        assert!(last >= 1, "une page de marge en dessous de la fenêtre");
    }

    /// Aucune page : intervalle dégénéré, pas de débordement.
    #[test]
    fn visible_pages_on_empty_document() {
        assert_eq!(visible_pages(&[], 0.0, 300.0), (0, 0));
    }

    /// Des pages de hauteurs différentes ne doivent pas fausser le calcul :
    /// c'est le cas des documents mixtes (titres, figures, sauts de page).
    #[test]
    fn visible_pages_with_variable_page_heights() {
        let sizes = vec![
            (600.0, 100.0),
            (600.0, 800.0),
            (600.0, 100.0),
            (600.0, 100.0),
        ];

        // La fenêtre commence au sommet de la troisième page.
        let top = page_offset(&sizes, 2);
        let (first, last) = visible_pages(&sizes, top, top + 120.0);

        assert!(first <= 2, "la page visée est dans l'intervalle : {first}");
        assert!(last >= 2, "la page visée est dans l'intervalle : {last}");
    }

    /// En fin de document, la marge ne doit pas faire déborder l'intervalle.
    #[test]
    fn visible_pages_clamp_at_the_end_of_the_document() {
        let sizes = sizes();
        let (top, bottom) = window(10_000.0, 300.0);

        let (first, last) = visible_pages(&sizes, top, bottom);

        assert!(first < sizes.len(), "début dans le document : {first}");
        assert_eq!(last, sizes.len() - 1, "fin bornée à la dernière page");
    }
}
