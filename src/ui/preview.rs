use crate::app::{Message, State};
use crate::typst::render::Diagnostic;
use crate::ui::theme::Palette;
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, image, scrollable, space};
use iced::{ContentFit, Element, Fill, Length};
use std::path::Path;

/// Taille du texte courant.
const BODY_SIZE: f32 = 14.0;

/// Espacement entre les pages affichées.
const PAGE_GAP: f32 = 12.0;

/// Marge intérieure du panneau.
const PANEL_PADDING: f32 = 16.0;

/// Place minimale laissée à une page dans le panneau, en pixels logiques.
///
/// Elle n'existe que pour qu'un panneau réduit à presque rien ne donne pas un
/// rapport d'affichage absurde.
const MIN_PAGE_BOX: f32 = 80.0;

/// Agrandissement maximal d'une page.
///
/// Un aperçu reste un aperçu : au-delà du double, la page tiendrait plus de
/// place que le texte qu'on écrit, et coûterait quatre fois plus de mémoire.
/// Plancher du rapport d'affichage : en deçà, la page ne serait plus lisible.
///
/// Ce n'est pas une cible, seulement un garde-fou : un panneau très étroit ne
/// doit pas réduire la page à rien.
const MIN_DISPLAY_RATIO: f64 = 0.05;

/// La limite de zoom : 500 %. Suffisant pour lire et vérifier, sans
/// provoquer de débordements ni rendre l'interface inutilisable.
const MAX_DISPLAY_RATIO: f64 = 5.0;

/// Place supposée d'une page dans le panneau, avant que la vue ne l'ait mesurée.
///
/// Un panneau de 420 px de large sur 700 de haut, soit la disposition par
/// défaut. Ce n'est qu'un point de départ : la première mesure de la vue la
/// remplace, et la géométrie, puis le rendu, suivent.
pub(crate) fn default_viewport() -> iced::Size {
    iced::Size::new(420.0, 700.0)
}

/// Rapport d'affichage d'une page : pixels logiques par point.
///
/// Le mode d'ajustement par défaut utilise la largeur disponible ; les modes
/// hauteur et zoom libre sont explicites dans [`display_ratio_for_zoom`]. Le
/// rendu suit toujours le rapport affiché au lieu de rasteriser des pixels que
/// le panneau ne peut pas montrer.
///
/// Le calcul est par page dans les modes d'ajustement : une page en paysage
/// n'a pas le même rapport qu'une page en portrait.
pub(crate) fn display_ratio(box_size: iced::Size, page: (f64, f64)) -> f64 {
    display_ratio_for_zoom(box_size, page, crate::app::state::PreviewZoomMode::FitWidth)
}

/// Rapport d'affichage selon le mode de zoom actif.
pub(crate) fn display_ratio_for_zoom(
    box_size: iced::Size,
    page: (f64, f64),
    zoom: crate::app::state::PreviewZoomMode,
) -> f64 {
    let width = f64::from((box_size.width - 2.0 * PANEL_PADDING).max(MIN_PAGE_BOX));
    let height = f64::from((box_size.height - 2.0 * PANEL_PADDING).max(MIN_PAGE_BOX));

    let ratio = match zoom {
        crate::app::state::PreviewZoomMode::FitWidth => width / page.0,
        crate::app::state::PreviewZoomMode::FitHeight => height / page.1,
        crate::app::state::PreviewZoomMode::Free(ratio) => ratio,
    };

    if ratio.is_finite() {
        ratio.clamp(MIN_DISPLAY_RATIO, MAX_DISPLAY_RATIO)
    } else {
        1.0
    }
}

/// Passe d'un mode d'ajustement au zoom libre avec un incrément de 10 %.
pub(crate) fn stepped_zoom(
    mode: crate::app::state::PreviewZoomMode,
    step: i8,
    box_size: iced::Size,
    page: (f64, f64),
) -> crate::app::state::PreviewZoomMode {
    let ratio = display_ratio_for_zoom(box_size, page, mode);
    let stepped = ratio * 1.1_f64.powi(i32::from(step));

    crate::app::state::PreviewZoomMode::Free(stepped.clamp(MIN_DISPLAY_RATIO, MAX_DISPLAY_RATIO))
}

/// Refuse un rapport libre invalide ou hors des bornes de rendu.
pub(crate) fn normalize_zoom(
    zoom: crate::app::state::PreviewZoomMode,
) -> crate::app::state::PreviewZoomMode {
    match zoom {
        crate::app::state::PreviewZoomMode::Free(ratio) if ratio.is_finite() => {
            crate::app::state::PreviewZoomMode::Free(
                ratio.clamp(MIN_DISPLAY_RATIO, MAX_DISPLAY_RATIO),
            )
        }
        crate::app::state::PreviewZoomMode::Free(_) => crate::app::state::PreviewZoomMode::FitWidth,
        other => other,
    }
}

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
fn page_sizes(render: &crate::app::state::RenderState) -> Vec<(f32, f32)> {
    match render.document.as_ref() {
        Some(document) => sizes_of_with_zoom(
            document,
            render.viewport.unwrap_or_else(default_viewport),
            render.zoom,
        ),
        None => Vec::new(),
    }
}

/// Pourcentage lisible du rapport appliqué à la première page visible.
pub(crate) fn zoom_percent(state: &State) -> Option<String> {
    let document = state.render.document.as_deref()?;
    let index = state.render.visible.map_or(0, |(first, _)| first);
    let page = crate::typst::render::page_size(document, index)?;
    let box_size = state.render.viewport.unwrap_or_else(default_viewport);
    let ratio = display_ratio_for_zoom(box_size, page, state.render.zoom);

    Some(format!("{:.0}%", ratio * 100.0))
}

/// Dimensions logiques des pages d'un document.
///
/// Distinct de [`page_sizes`] pour être appelable depuis le gestionnaire de
/// messages, qui tient le document sans passer par la vue.
#[cfg(test)]
pub(crate) fn sizes_of(
    document: &typst_layout::PagedDocument,
    box_size: iced::Size,
) -> Vec<(f32, f32)> {
    sizes_of_with_zoom(
        document,
        box_size,
        crate::app::state::PreviewZoomMode::FitWidth,
    )
}

/// Dimensions logiques des pages selon le mode de zoom du panneau.
pub(crate) fn sizes_of_with_zoom(
    document: &typst_layout::PagedDocument,
    box_size: iced::Size,
    zoom: crate::app::state::PreviewZoomMode,
) -> Vec<(f32, f32)> {
    (0..crate::typst::render::page_count(document))
        .map(|index| crate::typst::render::page_size(document, index).unwrap_or((595.3, 841.9)))
        // La géométrie est en points, et l'affichage applique le rapport que le
        // panneau impose : la place d'une page dépend donc du panneau, et non
        // de la densité du rendu. C'est ce qui permet de rasteriser plus ou
        // moins fin — donc de consommer plus ou moins de mémoire — sans que la
        // mise en page du panneau bouge d'un pixel.
        .map(|(width, height)| {
            let ratio = display_ratio_for_zoom(box_size, (width, height), zoom);

            ((width * ratio) as f32, (height * ratio) as f32)
        })
        .collect()
}

/// Hauteur du blanc qui précède la première page dessinée.
pub(crate) fn head_height(sizes: &[(f32, f32)], first: usize) -> f32 {
    // `page_offset` inclut la marge du panneau, que la colonne pose déjà.
    page_offset(sizes, first) - PANEL_PADDING
}

/// Hauteur du blanc qui suit la dernière page dessinée.
pub(crate) fn tail_height(sizes: &[(f32, f32)], last: usize) -> f32 {
    page_offset(sizes, sizes.len()) - page_offset(sizes, last + 1)
}

/// Hauteur totale du contenu défilant.
///
/// Employée par les tests seuls : c'est **l'invariant** que la construction de
/// la vue doit respecter, et les tests le mesurent pour toutes les plages de
/// pages possibles. Le nommer vaut mieux que de le laisser implicite dans une
/// suite de `push`.
///
/// **C'est un invariant, et non un détail de mise en page.** Si la hauteur du
/// contenu dépendait du nombre de pages dessinées, chaque défilement déplacerait
/// le contenu sous le curseur ; le défilement signalerait alors un nouveau
/// viewport, qui relancerait une rasterisation, qui changerait les pages
/// dessinées — un cycle sans fin, qui alloue à chaque tour. Les écarts entre
/// pages appartiennent donc aux pages, jamais à l'espacement de la colonne.
#[cfg(test)]
pub(crate) fn content_height(sizes: &[(f32, f32)]) -> f32 {
    2.0 * PANEL_PADDING
        + sizes
            .iter()
            .map(|(_, height)| height + PAGE_GAP)
            .sum::<f32>()
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

pub fn vue<'a>(
    state: &'a State,
    render: &'a crate::app::state::RenderState,
    editor: &'a crate::editor::EditorState,
    lsp: &'a crate::app::state::LspState,
    id: Option<u64>,
) -> Element<'a, Message> {
    let sizes = page_sizes(render);
    let palette = state.resolved.palette;

    // Le fichier ouvert : c'est lui seul dont les diagnostics peuvent être
    // localisés, puisque c'est le seul que l'éditeur affiche. Emprunté à
    // l'état, et non cloné, pour que les éléments retournés vivent aussi
    // longtemps que lui.
    let open = editor.path.as_deref();

    // Espacement nul, volontairement : les écarts entre pages sont posés avec
    // les pages elles-mêmes, pour que la hauteur du contenu ne dépende pas du
    // nombre de pages dessinées. Voir [`content_height`].
    let mut content = column![].spacing(0.0).padding(PANEL_PADDING).width(Fill);

    if let Some(status) = &state.export_status {
        // Le gris était codé en dur : il ne suivait donc aucun thème, et un
        // thème clair l'aurait rendu illisible. C'était la dernière couleur
        // fixe de l'interface.
        content = content.push(palette.label(status).size(13).color(palette.text_muted));
    }

    // Fichiers de thème illisibles. Ils sont **dits** : un thème absent de la
    // liste sans explication est un défaut de plus, pas une discrétion.
    for problem in &state.theme_problems {
        content = content.push(
            palette
                .label(format!("Thème illisible — {problem}"))
                .size(12)
                .color(palette.danger),
        );
    }

    // Les commandes de zoom, dans la fenêtre de rendu — pas dans la barre
    // supérieure, où elles n'ont rien à faire (TOPBAR-08).
    let zoom_bar = iced::widget::row![
        iced::widget::button(iced::widget::text("−").size(12))
            .on_press(Message::PreviewZoomStep(-1))
            .padding([3, 7]),
        palette
            .label(crate::ui::preview::zoom_percent(state).unwrap_or_else(|| "—".to_string()))
            .size(11)
            .color(palette.text_muted),
        iced::widget::button(iced::widget::text("+").size(12))
            .on_press(Message::PreviewZoomStep(1))
            .padding([3, 7]),
        iced::widget::button(iced::widget::text("W").size(11))
            .on_press(Message::PreviewZoom(
                crate::app::state::PreviewZoomMode::FitWidth
            ))
            .padding([3, 7]),
        iced::widget::button(iced::widget::text("H").size(11))
            .on_press(Message::PreviewZoom(
                crate::app::state::PreviewZoomMode::FitHeight
            ))
            .padding([3, 7]),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);
    content = content.push(zoom_bar);

    // Diagnostics du compilateur, en tête du panneau.
    if !render.errors.is_empty() {
        content = content.push(diagnostics_panel(&render.errors, true, open, palette));
    }

    // Un aperçu affiché sous des erreurs est un aperçu **ancien** : le
    // texte courant ne ressemble pas à ces pages, et il faut le dire.
    if render.apercu_perime() {
        content = content.push(
            container(
                palette
                    .label("Aperçu ancien — la compilation actuelle a échoué.")
                    .size(11),
            )
            .padding([4, 8])
            .width(iced::Length::Fill),
        );
    }

    if !render.warnings.is_empty() {
        content = content.push(diagnostics_panel(&render.warnings, false, open, palette));
    }

    // Diagnostics du serveur de langage : ils arrivent **au fil de la frappe**,
    // là où notre compilation attend deux secondes de silence. Le fichier ouvert
    // est le seul qu'on puisse rejoindre depuis l'éditeur.
    if !lsp.reported.is_empty() {
        content = content.push(server_diagnostics_panel(&lsp.reported, palette));
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

        content = content.push(palette.label(message).size(BODY_SIZE));

        return scrollable(content)
            .id(id.map_or_else(
                || iced::widget::Id::from(SCROLL_ID),
                |id| iced::widget::Id::from(format!("apercu-{id}")),
            ))
            .direction(iced::widget::scrollable::Direction::Both {
                vertical: iced::widget::scrollable::Scrollbar::default(),
                horizontal: iced::widget::scrollable::Scrollbar::default(),
            })
            .width(Fill)
            .height(Fill)
            // Aucune page : le viewport ne sert à rien, mais le message garde
            // la forme attendue par l'état de rendu.
            .on_scroll(|viewport| Message::ViewportChanged(0, 0, viewport.bounds().size()))
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

    let head = head_height(&sizes, first);

    if head > 0.0 {
        content = content.push(space::vertical().height(Length::Fixed(head)).width(Fill));
    }

    for (index, size) in sizes
        .iter()
        .enumerate()
        .skip(first)
        .take(last.saturating_sub(first) + 1)
    {
        content = content.push(page_view(render, palette, index, *size));

        // L'écart **après** la page : il l'accompagne, au lieu d'être posé par
        // la colonne entre deux widgets. C'est ce qui rend la hauteur du
        // contenu indépendante des pages dessinées.
        content = content.push(
            space::vertical()
                .height(Length::Fixed(PAGE_GAP))
                .width(Fill),
        );
    }

    let tail = tail_height(&sizes, last);

    if tail > 0.0 {
        content = content.push(space::vertical().height(Length::Fixed(tail)).width(Fill));
    }

    scrollable(content)
        .id(id.map_or_else(
            || iced::widget::Id::from(SCROLL_ID),
            |id| iced::widget::Id::from(format!("apercu-{id}")),
        ))
        .direction(iced::widget::scrollable::Direction::Both {
            vertical: iced::widget::scrollable::Scrollbar::default(),
            horizontal: iced::widget::scrollable::Scrollbar::default(),
        })
        .width(Fill)
        .height(Fill)
        .on_scroll(move |viewport| {
            // La marge intérieure du panneau n'est pas dans le repère du
            // viewport : on la retire pour raisonner sur le contenu défilant.
            let top = viewport.absolute_offset().y - PANEL_PADDING;
            let bottom = top + viewport.bounds().height;

            let (first, last) = visible_pages(&sizes, top, bottom);

            // La place que le panneau laisse à une page vient d'ici : la vue est
            // seule à la connaître, et c'est elle qui décide de la taille
            // d'affichage — donc de la finesse dont le rendu a besoin. Les
            // bornes du viewport la portent, et ne changent qu'au
            // redimensionnement : un défilement ne les retransmet donc pas.
            Message::ViewportChanged(first, last, viewport.bounds().size())
        })
        .into()
}

/// Une page : l'image si elle est rasterisée, sinon un cadre de la même
/// taille (la géométrie du document est connue avant tout rendu).
fn page_view(
    render: &crate::app::state::RenderState,
    palette: Palette,
    index: usize,
    size: (f32, f32),
) -> Element<'_, Message> {
    let (width, height) = size;

    // Double tampon : l'image de la page est affichée dès qu'elle existe,
    // même si elle date de la compilation précédente. Une page modifiée se
    // met à jour en place quand son nouveau rendu arrive, sans jamais
    // disparaître de l'écran.
    //
    // Le `Handle` mis en cache est cloné tel quel : il porte le même
    // identifiant, donc le backend graphique retrouve la texture déjà
    // téléversée au lieu d'en créer une neuve à chaque image.
    // Le **papier** du document est dessiné sous l'image, et non le fond de
    // l'application. Le backend graphique téléverse une texture neuve au moment
    // où elle change ; tant qu'elle n'est pas prête, ou si l'atlas n'a pas pu
    // l'accueillir dans cette image-là, le dessin de l'image est simplement
    // absent — et c'est alors le papier qu'on voit, à l'endroit exact de la
    // page, et non le fond de l'application.
    let paper = render.paper.unwrap_or(iced::Color::WHITE);

    match render.page_cache.get(&index) {
        Some(handle) => container(
            image(handle.clone())
                .width(Length::Fixed(width))
                .height(Length::Fixed(height))
                .content_fit(ContentFit::Contain),
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(paper_style(paper))
        .into(),

        // Aucune texture encore : le cadre est du papier, de la taille de la
        // page. Il n'apparaît donc jamais de trou dans le document.
        None => container(
            palette
                .label(if render.rendering.contains(&(render.revision, index)) {
                    "Rendu…"
                } else {
                    ""
                })
                .size(13)
                .color(palette.text_muted),
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .center_x(Fill)
        .center_y(Fill)
        .style(paper_style(paper))
        .into(),
    }
}

/// Fond d'un cadre de page : le papier du document.
///
/// Le backend graphique téléverse une texture neuve au moment où elle change ;
/// tant qu'elle n'est pas prête, ou si l'atlas n'a pas pu l'accueillir dans
/// cette image-là, le dessin de l'image est simplement absent. C'est alors le
/// papier qu'on voit, à l'endroit exact de la page — et jamais le fond de
/// l'application.
fn paper_style(paper: iced::Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(paper.into()),
        ..Default::default()
    }
}

/// Bandeau listant les erreurs (ou avertissements) du compilateur.
///
/// Le document compilé est l'assemblage, qui inclut les chapitres : un
/// diagnostic peut donc tomber dans un fichier qui n'est pas celui qu'on est en
/// train d'écrire. Ceux-là nomment leur fichier et ne proposent pas de saut —
/// un saut calculé pour un autre fichier mènerait au mauvais endroit, ce qui est
/// pire que pas de saut du tout.
/// Les diagnostics du serveur de langage, au fil de la frappe.
///
/// Le compilateur a les siens : plus tard, et pour tout l'assemblage. Le serveur
/// parle du document qu'on écrit, tout de suite. Les deux sources restent
/// **nommées** — un même problème peut apparaître chez les deux, et ce n'est pas
/// un doublon à taire mais deux regards à distinguer.
fn server_diagnostics_panel<'a>(
    diagnostics: &'a [crate::lsp::messages::Reported],
    palette: Palette,
) -> Element<'a, Message> {
    use crate::lsp::messages::Severity;

    let is_error = diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.severity, Some(Severity::Error) | None));

    let title = format!("{} diagnostic(s) du serveur", diagnostics.len());
    let mut list = column![palette.label(title).size(13).color(if is_error {
        palette.danger
    } else {
        palette.warning
    })]
    .spacing(4);

    for diagnostic in diagnostics {
        // La gravité se lit à la couleur du rang : un avertissement ne doit pas
        // prendre le rouge d'une erreur.
        let accent = match diagnostic.severity {
            Some(Severity::Error) | None => palette.danger,
            Some(Severity::Warning) => palette.warning,
            Some(Severity::Information) | Some(Severity::Hint) => palette.text_muted,
        };

        list = list.push(
            button(
                palette
                    .label(diagnostic.label.clone())
                    .size(13)
                    .wrapping(Wrapping::Word)
                    .width(Fill)
                    .color(accent),
            )
            .on_press_maybe(Some(Message::GotoError(diagnostic.line, diagnostic.column)))
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

    let mut list = column![palette.label(title).size(13).color(accent)].spacing(4);

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
            button(
                palette
                    .label(label)
                    .size(13)
                    .wrapping(Wrapping::Word)
                    .width(Fill),
            )
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

    /// DIAGNOSTIC : la géométrie du document réel, page par page.
    ///
    /// Ignoré par défaut. C'est l'outil qui dit si une page a une taille
    /// anormale, où le défilement situe les pages, et si la fin du document est
    /// couverte par les plages visibles.
    ///
    ///     PROJET=mon-travail cargo test --release -- --ignored diagnose_page_geometry --nocapture
    #[test]
    #[ignore]
    fn diagnose_page_geometry() {
        let home = std::env::var("HOME").unwrap_or_default();
        let name = std::env::var("PROJET").unwrap_or_else(|_| "mon-travail".to_string());
        let dir = std::path::PathBuf::from(home)
            .join(".typst-writer/projets")
            .join(&name);

        let entry = std::fs::read_to_string(dir.join("main.typ")).expect("main.typ lisible");
        let outcome = crate::typst::render::compile(&dir, &entry, &[]);

        println!("projet {name} : {} erreur(s)", outcome.errors.len());

        let Some(document) = outcome.document else {
            println!("pas de document");
            return;
        };

        let box_size = default_viewport();
        let sizes = sizes_of(&document, box_size);

        println!("pages : {}", sizes.len());

        for (index, (width, height)) in sizes.iter().enumerate() {
            let (point_width, point_height) =
                crate::typst::render::page_size(&document, index).unwrap_or_default();

            let ratio = display_ratio(box_size, (point_width, point_height));
            let rendered = crate::typst::render::pixel_per_pt(ratio, 1.0);

            println!(
                "  page {index:>3} : {width:>7.1} × {height:>7.1} px à l'écran \
                 | {point_width:>7.1} × {point_height:>7.1} pt | rapport {ratio:.3} \
                 | rendu {:.0} × {:.0} px à {rendered:.2} px/pt",
                point_width * rendered,
                point_height * rendered,
            );
        }

        let content = page_offset(&sizes, sizes.len());

        println!("hauteur du contenu : {content:.1} px");

        // Le panneau d'aperçu fait environ 390 px de haut, la fenêtre 780.
        let visible_height = 390.0;

        for top in [
            0.0,
            800.0,
            2000.0,
            4000.0,
            8000.0,
            12000.0,
            content - visible_height,
            content - visible_height - 40.0,
            content - visible_height + 40.0,
        ] {
            let top = top.max(0.0);
            let (first, last) = visible_pages(&sizes, top, top + visible_height);

            println!("  défilement {top:>9.1} → pages {first}..={last}");
        }
    }

    /// La hauteur du contenu ne dépend pas des pages dessinées.
    ///
    /// C'est l'invariant qui empêche le cycle : une hauteur qui bougerait avec
    /// le défilement déplacerait le contenu sous le curseur, ferait signaler un
    /// nouveau viewport, et relancerait une rasterisation à chaque tour.
    #[test]
    fn the_content_height_does_not_depend_on_the_pages_drawn() {
        let sizes = vec![
            (595.0, 842.0),
            (595.0, 842.0),
            (842.0, 595.0),
            (595.0, 300.0),
        ];

        let expected = content_height(&sizes);

        for first in 0..sizes.len() {
            for last in first..sizes.len() {
                let drawn: f32 = (first..=last).map(|index| sizes[index].1 + PAGE_GAP).sum();

                let total = head_height(&sizes, first) + drawn + tail_height(&sizes, last);

                assert!(
                    (total - (expected - 2.0 * PANEL_PADDING)).abs() < 0.01,
                    "pages {first}..={last} : {total} au lieu de {}",
                    expected - 2.0 * PANEL_PADDING
                );
            }
        }
    }

    /// Une page est montrée entière : elle tient dans la place que le panneau
    /// lui laisse, quelles que soient ses dimensions.
    #[test]
    fn every_page_fits_in_the_panel() {
        let box_size = iced::Size::new(420.0, 700.0);

        // Un panneau plus étroit qu'une page : elle est réduite, et tient.
        let portrait = display_ratio(box_size, (595.3, 841.9));

        assert!(
            portrait < 1.0,
            "une page A4 doit être réduite, eu {portrait}"
        );
        assert!(595.3 * portrait <= 420.0, "la page doit tenir en largeur");
        assert!(841.9 * portrait <= 700.0, "la page doit tenir en hauteur");

        // Une page en paysage a son propre rapport : elle est plus large, donc
        // réduite davantage — mais elle tient aussi.
        let landscape = display_ratio(box_size, (841.9, 595.3));

        assert!(
            landscape < portrait,
            "une page large doit être réduite plus"
        );
        assert!(841.9 * landscape <= 420.0);
        assert!(595.3 * landscape <= 700.0);

        // Un panneau large ne fait pas enfler une page indéfiniment.
        let huge = display_ratio(iced::Size::new(4000.0, 4000.0), (595.3, 841.9));

        assert!((huge - MAX_DISPLAY_RATIO).abs() < 1e-9);
    }

    /// Les modes largeur, hauteur et zoom libre ont des rapports distincts.
    #[test]
    fn zoom_modes_compute_the_expected_display_ratio() {
        let box_size = iced::Size::new(300.0, 220.0);
        let page = (600.0, 800.0);

        let width =
            display_ratio_for_zoom(box_size, page, crate::app::state::PreviewZoomMode::FitWidth);
        let height = display_ratio_for_zoom(
            box_size,
            page,
            crate::app::state::PreviewZoomMode::FitHeight,
        );
        let free = display_ratio_for_zoom(
            box_size,
            page,
            crate::app::state::PreviewZoomMode::Free(1.25),
        );

        assert!(
            width > height,
            "ajuster à la largeur utilise toute la largeur"
        );
        assert!((width - 268.0 / 600.0).abs() < 1e-9);
        assert!((height - 188.0 / 800.0).abs() < 1e-9);
        assert_eq!(free, 1.25);
    }

    /// Les incréments convertissent l'ajustement en zoom libre et respectent
    /// les bornes qui gardent l'allocation rasterisée raisonnable.
    #[test]
    fn zoom_steps_switch_to_free_and_stay_bounded() {
        let box_size = iced::Size::new(300.0, 220.0);
        let page = (600.0, 800.0);

        let stepped = stepped_zoom(
            crate::app::state::PreviewZoomMode::FitWidth,
            1,
            box_size,
            page,
        );
        let maximum = stepped_zoom(
            crate::app::state::PreviewZoomMode::Free(MAX_DISPLAY_RATIO),
            1,
            box_size,
            page,
        );
        let invalid = normalize_zoom(crate::app::state::PreviewZoomMode::Free(f64::NAN));

        assert_eq!(
            stepped,
            crate::app::state::PreviewZoomMode::Free(268.0 / 600.0 * 1.1)
        );
        assert_eq!(
            maximum,
            crate::app::state::PreviewZoomMode::Free(MAX_DISPLAY_RATIO)
        );
        assert_eq!(invalid, crate::app::state::PreviewZoomMode::FitWidth);
    }

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

    /// Un document long : cinq cents pages, comme une thèse.
    fn document_long() -> Vec<(f32, f32)> {
        (0..500)
            .map(|index| {
                // Un peu de variété : une page sur vingt en paysage, une sur
                // cinquante plus courte — les cumuls ne peuvent pas compter
                // sur des hauteurs uniformes.
                if index % 20 == 19 {
                    (842.0, 595.0)
                } else if index % 50 == 49 {
                    (595.0, 300.0)
                } else {
                    (595.0, 842.0)
                }
            })
            .collect()
    }

    /// Sur cinq cents pages, les cumuls de défilement ne dérivent pas : la
    /// dernière page tombe là où le calcul dit, pas à quelques pages près.
    #[test]
    fn les_cumuls_sur_cinq_cents_pages_ne_derivent_pas() {
        let sizes = document_long();

        for index in [0, 1, 100, 250, 499] {
            let attendu: f32 = PANEL_PADDING
                + sizes[..index]
                    .iter()
                    .map(|(_, hauteur)| hauteur + PAGE_GAP)
                    .sum::<f32>();
            let vu = page_offset(&sizes, index);

            assert!(
                (vu - attendu).abs() < 1.0,
                "page {index} : {vu} au lieu de {attendu}"
            );
        }

        // Et le défilement met la page voulue en haut — le geste du sommaire.
        for index in [0, 7, 249, 498] {
            let haut = scroll_offset_for_page(&sizes, index);
            let (premiere, _) = visible_pages(&sizes, haut, haut + 842.0);

            assert!(
                premiere <= index,
                "page {index} : la première visible est {premiere}"
            );
        }
    }

    /// Les trois modes d'affichage tiennent sur un document long : la
    /// largeur, la hauteur, et le zoom libre restent dans leurs bornes,
    /// page après page.
    #[test]
    fn les_modes_d_affichage_tiennent_sur_un_document_long() {
        let sizes = document_long();
        let box_size = default_viewport();

        for index in [0, 250, 499] {
            let page = (f64::from(sizes[index].0), f64::from(sizes[index].1));

            for zoom in [
                crate::app::state::PreviewZoomMode::FitWidth,
                crate::app::state::PreviewZoomMode::FitHeight,
                crate::app::state::PreviewZoomMode::Free(4.0),
            ] {
                let ratio = display_ratio_for_zoom(box_size, page, zoom);

                assert!(
                    (MIN_DISPLAY_RATIO..=MAX_DISPLAY_RATIO).contains(&ratio),
                    "page {index}, zoom {zoom:?} : ratio {ratio}"
                );
            }
        }
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
