use crate::app::{Message, State};
use crate::typst::render::Diagnostic;
use crate::ui::apercu::CoucheApercu;
use crate::ui::theme::Palette;
use iced::widget::text::Wrapping;
use iced::widget::{
    button, canvas, column, container, image, row, scrollable, space, stack, text_input,
};
use iced::{ContentFit, Element, Fill, Length};
use std::path::Path;

/// Identifiant du champ de recherche de l'aperçu.
pub(crate) fn id_recherche_apercu() -> iced::widget::Id {
    iced::widget::Id::new("apercu-recherche")
}

/// Espacement entre les pages affichées.
const PAGE_GAP: f32 = 12.0;

/// Marge intérieure du panneau.
pub(crate) const PANEL_PADDING: f32 = 16.0;

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

/// Pages matérialisées au-delà du viewport, de part et d'autre.
///
/// Cette marge ne garde que des widgets et des images déjà en cache : elle
/// ne déborde jamais sur la plage exacte qui a le droit de rasteriser. Une
/// page de marge peut donc apparaître comme papier jusqu'à son entrée réelle
/// dans la fenêtre.
const VIEWPORT_MARGIN: usize = 1;

/// Identifiant du défilement de l'aperçu.
///
/// C'est par lui que l'application peut amener l'aperçu sur une page précise :
/// un widget ne se pilote pas de l'extérieur sans être nommé.
pub(crate) const SCROLL_ID: &str = "apercu";

/// Largeur de la colonne qui porte les pages dans le défilement.
///
/// Elle vaut la largeur du panneau quand la page y tient, et celle de la page
/// la plus large — marges comprises — au-delà. C'est cette largeur explicite
/// qui permet à `align_x(Center)` de centrer les pages : dans un défilement
/// horizontal, `Length::Fill` se résout sur le contenu, pas sur la fenêtre.
pub(crate) fn largeur_colonne(sizes: &[(f32, f32)], largeur_fenetre: f32) -> f32 {
    let largeur_page_max = sizes
        .iter()
        .map(|(largeur, _)| *largeur)
        .fold(0.0_f32, f32::max);

    (largeur_page_max + 2.0 * PANEL_PADDING).max(largeur_fenetre)
}

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

/// Les pages que la vue montre réellement : toutes, ou la tranche d'un
/// chapitre. Les index sont ceux du document compilé, jamais des positions
/// dans la tranche — le cache de pages et la couche texte restent ainsi
/// keyés sur les vraies pages.
pub(crate) fn pages_de_la_vue(render: &crate::app::state::RenderState) -> Vec<usize> {
    let total = render
        .document
        .as_deref()
        .map_or(0, crate::typst::render::page_count);
    crate::app::chapitres::plage_pages(render.chapitre.as_ref(), total).collect()
}

/// Les dimensions des pages de la tranche affichée, dans l'ordre du document.
pub(crate) fn sizes_de_la_vue(render: &crate::app::state::RenderState) -> Vec<(f32, f32)> {
    let toutes = page_sizes(render);
    pages_de_la_vue(render)
        .into_iter()
        .filter_map(|index| toutes.get(index).copied())
        .collect()
}

/// La position d'une page du document dans la tranche affichée.
pub(crate) fn position_dans_la_vue(
    render: &crate::app::state::RenderState,
    page: usize,
) -> Option<usize> {
    pages_de_la_vue(render)
        .into_iter()
        .position(|index| index == page)
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
/// Les pages visibles dans la fenêtre, aux tailles courantes — sans attendre
/// un défilement (feuille de route, phase 3, point 3.3) : un zoom ou un
/// redimensionnement change ce que montre la fenêtre, même à défilement
/// immobile.
pub fn pages_visibles(
    tailles: &[(f32, f32)],
    hauteur_fenetre: f32,
    defilement_y: f32,
) -> (usize, usize) {
    if tailles.is_empty() {
        return (0, 0);
    }
    let haut = defilement_y.max(0.0);
    let bas = haut + hauteur_fenetre.max(1.0);
    let mut premier = tailles.len() - 1;
    let mut dernier = 0;
    let mut y = 0.0f32;
    for (index, (_, hauteur)) in tailles.iter().enumerate() {
        let fin = y + hauteur;
        if fin > haut && y < bas {
            premier = premier.min(index);
            dernier = dernier.max(index);
        }
        y = fin;
    }
    if dernier < premier {
        // Rien n'est visible : la page la plus proche du bord du haut.
        let index = (haut / y.max(1.0) * tailles.len() as f32) as usize;
        let index = index.min(tailles.len() - 1);
        (index, index)
    } else {
        (premier, dernier)
    }
}

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

/// Le décalage absolu du scrollable qui correspond à un défilement de contenu.
///
/// `RenderState::scroll` est exprimé sans la marge du panneau (voir
/// [`scroll_offset_for_page`]) ; le scrollable, lui, part de zéro en haut du
/// contenu. Cette conversion est l'inverse exact de celle du `on_scroll`.
pub(crate) fn offset_absolu(scroll_contenu: f32) -> f32 {
    (scroll_contenu + PANEL_PADDING).max(0.0)
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
pub(crate) fn visible_pages(sizes: &[(f32, f32)], top: f32, bottom: f32) -> (usize, usize) {
    let (first, last) = visible_pages_exact(sizes, top, bottom);

    if sizes.is_empty() {
        return (first, last);
    }

    (
        first.saturating_sub(VIEWPORT_MARGIN),
        (last + VIEWPORT_MARGIN).min(sizes.len() - 1),
    )
}

/// Les pages qui touchent réellement la fenêtre, sans marge de préchargement.
///
/// La marge de [`visible_pages`] sert à garder des widgets autour du viewport ;
/// elle ne doit pas demander de rasterisation : prefetcher une page invisible
/// faisait payer un rendu complet à chaque défilement.
pub(crate) fn visible_pages_exact(sizes: &[(f32, f32)], top: f32, bottom: f32) -> (usize, usize) {
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

    (first, last)
}

pub fn vue<'a>(
    state: &'a State,
    render: &'a crate::app::state::RenderState,
    editor: &'a crate::editor::EditorState,
    id: Option<u64>,
) -> Element<'a, Message> {
    let pages = pages_de_la_vue(render);
    let sizes = sizes_de_la_vue(render);
    let palette = state.resolved.palette;

    // Le fichier ouvert : c'est lui seul dont les diagnostics peuvent être
    // localisés, puisque c'est le seul que l'éditeur affiche. Emprunté à
    // l'état, et non cloné, pour que les éléments retournés vivent aussi
    // longtemps que lui.
    let open = editor.path.as_deref();

    // Espacement nul, volontairement : les écarts entre pages sont posés avec
    // les pages elles-mêmes, pour que la hauteur du contenu ne dépende pas du
    // nombre de pages dessinées. Voir [`content_height`].
    // Les pages d'un document ne sont pas toutes larges comme la fenêtre :
    // l'alignement les centre dans le panneau au lieu de les coller à gauche.
    // La colonne reçoit une largeur explicite — celle du panneau quand les
    // pages y tiennent, celle de la page la plus large au-delà — parce que
    // `Length::Fill` dans un défilement horizontal se résout sur le contenu,
    // pas sur la fenêtre, et ne centrerait donc rien.
    let box_size = render.viewport.unwrap_or_else(default_viewport);
    let largeur_colonne = largeur_colonne(&sizes, box_size.width);
    let mut content = column![]
        .spacing(0.0)
        .padding(PANEL_PADDING)
        .width(Length::Fixed(largeur_colonne))
        .align_x(iced::Alignment::Center);

    // Les erreurs et avertissements ne défilent pas avec les pages : ils
    // gardent toute la largeur du panneau et restent visibles quand on lit
    // plus bas. Les loger dans le contenu les comprimait à la largeur d'une
    // page, et le moindre message long se coupait.
    let mut entete = column![].spacing(0).width(Fill);

    // Fichiers de thème illisibles. Ils sont **dits** : un thème absent de la
    // liste sans explication est un défaut de plus, pas une discrétion.
    for problem in &state.theme_problems {
        entete = entete.push(
            palette
                .label(format!("Thème illisible — {problem}"))
                .size(12)
                .color(palette.danger),
        );
    }

    if !render.errors.is_empty() {
        entete = entete.push(diagnostics_panel(&render.errors, true, open, palette));
    }

    if !render.warnings.is_empty() {
        entete = entete.push(diagnostics_panel(&render.warnings, false, open, palette));
    }

    // Diagnostics du serveur de langage : ils arrivent **au fil de la frappe**,
    // là où notre compilation attend le repos de la frappe. Le fichier ouvert
    // est le seul qu'on puisse rejoindre depuis l'éditeur.

    if sizes.is_empty() {
        // Progrès silencieux : aucun texte de statut ne vient s'ajouter
        // au-dessus du document. Les erreurs, elles, restent visibles.
        return column![entete, space::vertical().height(Fill)]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into();
    }

    // Seules les pages proches du viewport sont matérialisées ; les autres
    // sont remplacées par un unique espaceur de la hauteur exacte qu'elles
    // occuperaient. Un document de 500 pages coûte donc le même nombre de
    // widgets qu'un document d'une page, et la géométrie du défilement reste
    // exacte — les hauteurs de pages sont connues avant tout rendu.
    let last_index = sizes.len() - 1;
    let (visible_debut, visible_fin) = render.visible.unwrap_or((
        pages.first().copied().unwrap_or(0),
        pages.get(2).copied().unwrap_or(0),
    ));

    // `visible` désigne des pages du document ; la tranche n'en montre qu'une
    // partie. On convertit donc les bornes en positions dans la tranche avant
    // de dessiner, sans changer l'identité des pages.
    let first = pages
        .iter()
        .position(|index| *index >= visible_debut)
        .unwrap_or(0);
    let last = pages
        .iter()
        .rposition(|index| *index <= visible_fin)
        .unwrap_or(last_index)
        .clamp(first, last_index);

    let head = head_height(&sizes, first);

    if head > 0.0 {
        content = content.push(space::vertical().height(Length::Fixed(head)).width(Fill));
    }

    for position in first..=last {
        let Some(index) = pages.get(position).copied() else {
            continue;
        };
        let Some(size) = sizes.get(position).copied() else {
            continue;
        };

        content = content.push(page_view(render, palette, index, size, state.maj_tenu));

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

    let defilable = scrollable(content)
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
        .on_scroll({
            let sizes = sizes.clone();
            let pages = pages.clone();
            move |viewport| {
                // La marge intérieure du panneau n'est pas dans le repère du
                // viewport : on la retire pour raisonner sur le contenu défilant.
                let gauche = viewport.absolute_offset().x - PANEL_PADDING;
                let top = viewport.absolute_offset().y - PANEL_PADDING;
                let bottom = top + viewport.bounds().height;

                let (premiere, derniere) = visible_pages(&sizes, top, bottom);

                // Les positions dans la tranche redeviennent des pages du
                // document : le cache, la couche texte et la rasterisation
                // parlent toujours la même langue.
                let premiere = pages.get(premiere).copied().unwrap_or(0);
                let derniere = pages.get(derniere).copied().unwrap_or(premiere);

                // La place que le panneau laisse à une page vient d'ici : la vue est
                // seule à la connaître, et c'est elle qui décide de la taille
                // d'affichage — donc de la finesse dont le rendu a besoin. Les
                // bornes du viewport la portent, et ne changent qu'au
                // redimensionnement : un défilement ne les retransmet donc pas.
                Message::ViewportChanged(premiere, derniere, viewport.bounds(), Some((gauche, top)))
            }
        });

    // La molette est dirigée (bug-report-9) : `Ctrl` la destine au zoom
    // seul — jamais au défilement — `Maj` au défilement horizontal, et
    // la molette simple laisse défiler verticalement.
    let defilable: Element<'_, Message> = defilable.into();

    // Le pied de page dit où l'on est dans le document. Il ne défile pas :
    // sans lui, un lecteur de thèse ne sait pas sur quelle page il se trouve.
    let indicateur = indicateur_page(render, palette, &sizes, &pages);

    let mut colonne = column![].spacing(0).width(Fill).height(Fill);

    // La barre de recherche reste au-dessus du document : elle ne défile
    // pas avec les pages.
    if render.recherche_apercu.visible {
        colonne = colonne.push(barre_recherche(render, palette));
    }

    // Les diagnostics gardent toute la largeur du panneau, au-dessus du
    // document qui défile.
    colonne = colonne.push(entete);

    colonne = colonne.push(defilable).push(indicateur);

    MoletteDirigee::new(colonne.into(), state.controle_tenu).into()
}

/// Le numéro de page courant, à partir du défilement du panneau.
///
/// `visible` contient une marge de pages autour de la fenêtre : il ne dit donc
/// pas exactement la page qu'on lit. Le défilement, lui, désigne la page dont
/// la tranche occupe le haut de la fenêtre.
pub(crate) fn page_au_defilement(sizes: &[(f32, f32)], y: f32) -> Option<usize> {
    if sizes.is_empty() {
        return None;
    }

    let mut offset = 0.0;
    for (index, (_, hauteur)) in sizes.iter().enumerate() {
        let bas = offset + hauteur;
        if y < bas {
            return Some(index);
        }
        offset = bas + PAGE_GAP;
    }

    Some(sizes.len() - 1)
}

/// La petite barre « Page n / N » du panneau de rendu.
fn indicateur_page<'a>(
    render: &crate::app::state::RenderState,
    palette: Palette,
    sizes: &[(f32, f32)],
    pages: &[usize],
) -> Element<'a, Message> {
    let total = render
        .document
        .as_deref()
        .map_or(sizes.len(), crate::typst::render::page_count);
    let position = page_au_defilement(sizes, render.scroll.1).unwrap_or(0);
    let page = pages.get(position).copied().unwrap_or(position) + 1;

    container(
        palette
            .label(format!("Page {page} / {total}"))
            .size(11)
            .color(palette.text_muted),
    )
    .padding([4, 10])
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// La barre de recherche du document rendu.
fn barre_recherche<'a>(
    render: &'a crate::app::state::RenderState,
    palette: Palette,
) -> Element<'a, Message> {
    let recherche = &render.recherche_apercu;

    let champ = text_input("Rechercher dans le rendu…", &recherche.motif)
        .id(id_recherche_apercu())
        .on_input(Message::RechercheApercuMotif)
        .on_submit(Message::RechercheApercuSuivant)
        .padding([6, 8])
        .size(13);

    let compteur = palette
        .label(recherche.compteur())
        .size(12)
        .color(palette.text_muted);

    let precedent = button(palette.label("↑").size(13))
        .on_press(Message::RechercheApercuPrecedent)
        .padding([5, 8])
        .style(button::secondary);

    let suivant = button(palette.label("↓").size(13))
        .on_press(Message::RechercheApercuSuivant)
        .padding([5, 8])
        .style(button::secondary);

    let fermer = button(palette.label("×").size(14))
        .on_press(Message::RechercheApercuFermer)
        .padding([5, 8])
        .style(button::text);

    container(
        row![champ, compteur, precedent, suivant, fermer]
            .spacing(6)
            .align_y(iced::Alignment::Center),
    )
    .padding([5, 8])
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Une tranche de page dans le cadre complet de la page.
///
/// Le fond papier n'est mis que sur la couche du dessous : une image neuve
/// dont la texture n'est pas encore prête doit laisser voir l'ancienne, pas
/// un rectangle blanc.
fn couche_image<'a>(
    tranche: &crate::app::state::Tranche,
    width: f32,
    height: f32,
    paper: Option<iced::Color>,
) -> Element<'a, Message> {
    container(
        container(
            image(tranche.image.clone())
                .width(Length::Fixed(tranche.largeur * width))
                .height(Length::Fixed(tranche.hauteur * height))
                .content_fit(ContentFit::Fill),
        )
        .padding(iced::Padding {
            top: tranche.haut * height,
            right: 0.0,
            bottom: 0.0,
            left: tranche.gauche * width,
        }),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .style(move |_| container::Style {
        background: paper.map(Into::into),
        ..Default::default()
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
    maj_tenu: bool,
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

    let base: Element<'_, Message> = match render.page_cache.get(&index) {
        // La tranche rendue se place dans le cadre de la page : le papier
        // reste dessous, la page hors écran n'a jamais été rasterisée.
        Some(tranche) => {
            let mut pile = iced::widget::Stack::new();

            // L'image précédente reste sous la nouvelle : si le backend
            // n'a pas encore téléversé la texture neuve, on voit l'ancienne
            // plutôt qu'un blanc.
            if let Some(ancienne) = &tranche.precedente {
                pile = pile.push(couche_image(ancienne, width, height, Some(paper)));
                pile = pile.push(couche_image(tranche, width, height, None));
            } else {
                pile = pile.push(couche_image(tranche, width, height, Some(paper)));
            }
            pile.width(Length::Fixed(width))
                .height(Length::Fixed(height))
                .into()
        }

        // Aucune texture encore : le cadre est du papier, de la taille de la
        // page. Il n'apparaît donc jamais de trou dans le document, et aucun
        // texte de chargement ne vient se coller sur une page en cours.
        None => container(space::vertical())
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .style(paper_style(paper))
            .into(),
    };

    let Some(texte) = render.texte_affiche() else {
        return base;
    };

    // La couche texte et l'image doivent décrire le même PDF. Après une
    // compilation, l'ancienne image reste affichée le temps du nouveau rendu :
    // dessiner le texte neuf dessus ferait glisser les surlignages.
    let image_empreinte = render
        .page_cache
        .get(&index)
        .and_then(|tranche| tranche.empreinte.as_deref());
    if render.lecteur_empreinte.is_none() || image_empreinte != render.lecteur_empreinte.as_deref()
    {
        return base;
    }

    let Some(page) = texte.page(index) else {
        return base;
    };

    let selection = render
        .selection
        .map(|selection| {
            texte
                .surlignage_selection(index, selection.ancre, selection.focus)
                .into_iter()
                .map(|rect| rect_affichage(rect, page.taille, width, height))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut resultats = Vec::new();
    let mut courant = None;

    for (rang, resultat) in render.recherche_apercu.resultats.iter().enumerate() {
        if resultat.page != index {
            continue;
        }

        let rects = texte.surlignage_resultat(index, resultat.debut, resultat.fin);
        let rects: Vec<iced::Rectangle> = rects
            .into_iter()
            .map(|rect| rect_affichage(rect, page.taille, width, height))
            .collect();

        if rang == render.recherche_apercu.courante {
            courant = union_rectangles(&rects);
        }

        resultats.extend(rects);
    }

    let liens: Vec<iced::Rectangle> = page
        .liens
        .iter()
        .map(|lien| rect_affichage(lien.rect, page.taille, width, height))
        .collect();

    let overlay = canvas::Canvas::new(CoucheApercu {
        page: index,
        taille_page: (page.taille.x.to_pt() as f32, page.taille.y.to_pt() as f32),
        selection,
        resultats,
        courant,
        liens,
        couleur_selection: iced::Color {
            a: 0.32,
            ..palette.accent
        },
        couleur_resultat: iced::Color {
            a: 0.24,
            ..palette.warning
        },
        couleur_courante: iced::Color {
            a: 0.48,
            ..palette.accent
        },
        maj_tenu,
    })
    .width(Length::Fixed(width))
    .height(Length::Fixed(height));

    stack![base, overlay]
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

/// Convertit une boîte de la page en rectangle d'écran.
fn rect_affichage(
    rect: typst::layout::Rect,
    taille: typst::layout::Size,
    largeur: f32,
    hauteur: f32,
) -> iced::Rectangle {
    let echelle_x = if taille.x.to_pt() > 0.0 {
        largeur as f64 / taille.x.to_pt()
    } else {
        1.0
    };
    let echelle_y = if taille.y.to_pt() > 0.0 {
        hauteur as f64 / taille.y.to_pt()
    } else {
        1.0
    };

    let x = (rect.min.x.to_pt() * echelle_x) as f32;
    let y = (rect.min.y.to_pt() * echelle_y) as f32;
    let largeur = ((rect.max.x.to_pt() - rect.min.x.to_pt()) * echelle_x) as f32;
    let hauteur = ((rect.max.y.to_pt() - rect.min.y.to_pt()) * echelle_y) as f32;

    iced::Rectangle {
        x,
        y,
        width: largeur.max(1.0),
        height: hauteur.max(1.0),
    }
}

/// La boîte englobante d'un ensemble de rectangles.
fn union_rectangles(rects: &[iced::Rectangle]) -> Option<iced::Rectangle> {
    let premier = rects.first()?;
    let mut union = *premier;

    for rect in &rects[1..] {
        let x = union.x.min(rect.x);
        let y = union.y.min(rect.y);
        let max_x = (union.x + union.width).max(rect.x + rect.width);
        let max_y = (union.y + union.height).max(rect.y + rect.height);

        union = iced::Rectangle {
            x,
            y,
            width: max_x - x,
            height: max_y - y,
        };
    }

    Some(union)
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
        let here = diagnostic.concerne(open);

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
            format!(
                "{where_}[{}] {}",
                diagnostic.origine.libelle(),
                diagnostic.message
            )
        } else {
            format!(
                "{where_}{location} [{}] — {}",
                diagnostic.origine.libelle(),
                diagnostic.message
            )
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
pub(crate) fn file_name(path: &Path) -> String {
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
    fn le_numero_de_page_suit_le_defilement() {
        let sizes = [(100.0, 100.0), (100.0, 100.0)];

        assert_eq!(page_au_defilement(&sizes, 0.0), Some(0));
        assert_eq!(page_au_defilement(&sizes, 99.0), Some(0));
        assert_eq!(page_au_defilement(&sizes, 100.0), Some(1));
        assert_eq!(page_au_defilement(&sizes, 500.0), Some(1));
        assert_eq!(page_au_defilement(&[], 0.0), None);
    }

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

    /// La plage exacte ne précharge pas : seule la page réellement touchée
    /// par la fenêtre a le droit de coûter une rasterisation.
    #[test]
    fn la_plage_exacte_ne_prefetche_pas() {
        let sizes = sizes();
        let (top, bottom) = window(0.0, 100.0);

        assert_eq!(visible_pages_exact(&sizes, top, bottom), (0, 0));
        assert_eq!(visible_pages(&sizes, top, bottom), (0, 1));
        assert_eq!(visible_pages_exact(&[], 0.0, 300.0), (0, 0));
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

/// `Ctrl` consomme la molette pour le zoom seul ; tout le reste — `Maj`
/// y compris, dont le scrollable fait le défilement horizontal — lui
/// revient (bug-report-9).
fn la_molette_est_au_zoom(controle: bool) -> bool {
    controle
}

/// L'emballeur qui tient la molette : il **consomme** l'événement quand
/// `Ctrl` le destine au zoom — le défilement ne le reçoit donc jamais
/// (bug-report-9 : « désacréer le défilement lorsque je zoome avec
/// ctrl+molette »).
struct MoletteDirigee<'a> {
    contenu: Element<'a, Message>,
    controle: bool,
}

impl<'a> MoletteDirigee<'a> {
    fn new(contenu: Element<'a, Message>, controle: bool) -> Self {
        Self { contenu, controle }
    }
}

impl<'a> From<MoletteDirigee<'a>> for Element<'a, Message> {
    fn from(widget: MoletteDirigee<'a>) -> Self {
        Element::new(widget)
    }
}

impl<'a> iced::advanced::widget::Widget<Message, iced::Theme, iced::Renderer>
    for MoletteDirigee<'a>
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.contenu.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.contenu
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.contenu.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(self.contenu.as_widget())]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(&[self.contenu.as_widget()]);
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        if let iced::Event::Mouse(iced::mouse::Event::WheelScrolled { .. }) = event
            && la_molette_est_au_zoom(self.controle)
        {
            shell.capture_event();
            return;
        }

        self.contenu.as_widget_mut().update(
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

    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::layout::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.contenu
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.contenu.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

/// La région à rasteriser pour une page : ce que la fenêtre montre,
/// élargi d'une marge de chaque côté — défiler dans la marge ne coûte
/// aucun rendu (bug-report-9, fluidité). En fractions de la page.
pub(crate) fn region_pour_page(
    taille: (f32, f32),
    haut_page: f32,
    scroll: (f32, f32),
    fenetre: (f32, f32),
    marge: f32,
) -> (f32, f32, f32, f32) {
    let marge_x = fenetre.0 * marge;
    let marge_y = fenetre.1 * marge;

    let x_debut = (scroll.0 - marge_x).clamp(0.0, taille.0);
    let x_fin = (scroll.0 + fenetre.0 + marge_x).clamp(0.0, taille.0);
    let y_debut = (scroll.1 - haut_page - marge_y).clamp(0.0, taille.1);
    let y_fin = (scroll.1 - haut_page + fenetre.1 + marge_y).clamp(0.0, taille.1);

    // Fractions de la page : la tranche s'étire avec le zoom.
    (
        x_debut / taille.0,
        y_debut / taille.1,
        ((x_fin - x_debut) / taille.0).max(1.0 / taille.0),
        ((y_fin - y_debut) / taille.1).max(1.0 / taille.1),
    )
}

/// La bande d'une page voisine qui sera vue la première quand elle entrera
/// dans la fenêtre : son bas si elle vient du haut, son haut si elle vient
/// du bas. Un préchargement plein page coûterait le prix d'une page entière
/// à fort zoom ; cette bande coûte celui d'une fenêtre.
pub(crate) fn region_de_prechargement(
    taille: (f32, f32),
    fenetre: (f32, f32),
    vient_du_haut: bool,
) -> (f32, f32, f32, f32) {
    let hauteur_page = taille.1.max(1.0);
    let fraction = (fenetre.1 / hauteur_page).clamp(0.0, 1.0);
    let hauteur = fraction.max(1.0 / hauteur_page);

    (
        0.0,
        if vient_du_haut { 1.0 - hauteur } else { 0.0 },
        1.0,
        hauteur,
    )
}

/// Un rectangle en fractions de page couvre-t-il une région demandée ?
pub(crate) fn rectangle_couvre(
    rectangle: (f32, f32, f32, f32),
    region: (f32, f32, f32, f32),
) -> bool {
    let (x, y, l, h) = region;
    const EPSILON: f32 = 1.0e-3;

    rectangle.0 <= x + EPSILON
        && rectangle.1 <= y + EPSILON
        && rectangle.0 + rectangle.2 >= x + l - EPSILON
        && rectangle.1 + rectangle.3 >= y + h - EPSILON
}

/// Une tranche couvre-t-elle la région demandée ? Tant qu'elle la couvre,
/// défiler est gratuit.
pub(crate) fn tranche_couvre(
    tranche: &crate::app::state::Tranche,
    region: (f32, f32, f32, f32),
) -> bool {
    rectangle_couvre(
        (
            tranche.gauche,
            tranche.haut,
            tranche.largeur,
            tranche.hauteur,
        ),
        region,
    )
}

#[cfg(test)]
mod tests_des_regions {
    use super::*;

    #[test]
    fn la_region_est_la_fenetre_elargie_et_bornee_a_la_page() {
        // Une page de 500 × 800, la fenêtre au milieu, une marge de 50 %.
        let region = region_pour_page((500.0, 800.0), 0.0, (0.0, 300.0), (500.0, 200.0), 0.5);
        assert_eq!(region, (0.0, 0.25, 1.0, 0.5), "fenêtre 300-500 ±100");

        // En haut de page : la marge ne sort pas par le haut.
        let region = region_pour_page((500.0, 800.0), 0.0, (0.0, 0.0), (500.0, 200.0), 0.5);
        assert_eq!(region, (0.0, 0.0, 1.0, 0.375), "haut de page");

        // En bas : ni par le bas.
        let region = region_pour_page((500.0, 800.0), 0.0, (0.0, 750.0), (500.0, 200.0), 0.5);
        assert_eq!(region, (0.0, 0.8125, 1.0, 0.1875), "bas de page");
    }

    #[test]
    fn le_prechargement_prend_la_bande_par_laquelle_la_page_entre() {
        // Une page fait quatre fenêtres de haut.
        let page = (500.0, 800.0);
        let fenetre = (500.0, 200.0);

        // La page qui vient du haut montre son bas.
        let bande = region_de_prechargement(page, fenetre, true);
        assert_eq!(bande, (0.0, 0.75, 1.0, 0.25));

        // Celle qui vient du bas montre son haut.
        let bande = region_de_prechargement(page, fenetre, false);
        assert_eq!(bande, (0.0, 0.0, 1.0, 0.25));

        // Une page plus courte que la fenêtre est préchargée entière.
        let bande = region_de_prechargement((500.0, 100.0), fenetre, false);
        assert_eq!(bande, (0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn la_tranche_couvre_ce_qu_elle_contient() {
        let tranche = crate::app::state::Tranche {
            gauche: 0.0,
            haut: 0.125,
            largeur: 1.0,
            hauteur: 0.5,
            image: iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0]),
            empreinte: None,
            precedente: None,
        };

        assert!(
            tranche_couvre(&tranche, (0.0, 0.25, 1.0, 0.25)),
            "la fenêtre dedans est couverte"
        );
        assert!(
            !tranche_couvre(&tranche, (0.0, 0.5625, 1.0, 0.125)),
            "la fenêtre qui dépasse ne l'est pas"
        );
    }
}

#[cfg(test)]
mod tests_de_la_molette_dirigee {
    use super::*;

    #[test]
    fn seule_ctrl_consome_la_molette() {
        assert!(la_molette_est_au_zoom(true), "Ctrl : le zoom seul");
        assert!(
            !la_molette_est_au_zoom(false),
            "sinon la molette revient au scrollable — Maj y compris, qui en fait le défilement horizontal"
        );
    }
}

#[cfg(test)]
mod tests_pages_visibles {
    use super::{largeur_colonne, pages_visibles};

    /// La colonne fait au moins la fenêtre : les pages y sont centrées, et
    /// une page plus large que le panneau élargit la colonne au lieu d'être
    /// rognée.
    #[test]
    fn la_colonne_centre_les_pages_sans_les_rogner() {
        assert_eq!(largeur_colonne(&[(400.0, 600.0)], 600.0), 600.0);
        assert_eq!(largeur_colonne(&[(800.0, 600.0)], 600.0), 832.0);
        assert_eq!(largeur_colonne(&[], 600.0), 600.0);
    }

    /// Les pages visibles (feuille de route, phase 3, point 3.3) : le calcul
    /// suit la fenêtre et les tailles, sans défilement manuel.
    #[test]
    fn les_pages_visibles_suivent_la_fenetre() {
        let tailles = [(100.0f32, 100.0f32), (100.0, 100.0), (100.0, 100.0)];
        // Au tout début : la première page.
        assert_eq!(pages_visibles(&tailles, 50.0, 0.0), (0, 0));
        // Une fenêtre haute montre deux pages.
        assert_eq!(pages_visibles(&tailles, 150.0, 0.0), (0, 1));
        // Au milieu : la deuxième seule.
        assert_eq!(pages_visibles(&tailles, 50.0, 120.0), (1, 1));
        // Une fenêtre qui a grandi montre plus de pages — le cas du zoom.
        assert_eq!(pages_visibles(&tailles, 250.0, 0.0), (0, 2));
    }

    /// Rien n'est visible : on reste sur la page la plus proche.
    #[test]
    fn hors_du_document_on_reste_sur_une_page() {
        let tailles = [(100.0f32, 100.0f32), (100.0, 100.0)];
        let (premier, dernier) = pages_visibles(&tailles, 10.0, 500.0);
        assert_eq!(premier, dernier, "une seule page, la plus proche");
        assert!(premier < tailles.len());
    }
}
