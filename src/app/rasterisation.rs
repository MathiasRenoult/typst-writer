//! La rasterisation : les pages visibles, leurs textures, leur
//! invalidation.
//!
//! Ce traitement est le sien (feuille de route, phase 3, point 3.1) :
//! ni la compilation Typst, ni les événements Iced ne le conduisent.
//! Ses entrées : le document chargé et la fenêtre visible ; sa sortie :
//! les tâches de pages ; ses erreurs : les pages qui ne rendent pas.

use super::Message;
use super::State;
use iced::Task;
use std::sync::atomic::Ordering;
pub(super) fn visible(state: &mut State) -> Task<Message> {
    let Some(document) = state.render.document.clone() else {
        return Task::none();
    };

    // Le PDF est la source du rendu : sans lui, rien à rasteriser — la
    // bascule vers la vue rendu le génère systématiquement.
    let Some(pdf) = state.render.lecteur.clone() else {
        return Task::none();
    };

    // Tant qu'aucun viewport n'a été signalé, les pages du haut sont celles
    // qu'on voit. `iced` n'en signale d'ailleurs aucun tant que le contenu
    // tient dans la fenêtre — auquel cas tout est visible d'un coup, et trois
    // pages suffisent à couvrir ce cas comme le précédent. En vue de chapitre,
    // ces pages sont celles de la tranche, pas celles du document entier.
    let pages_vue = crate::ui::preview::pages_de_la_vue(&state.render);
    let defaut = (
        pages_vue.first().copied().unwrap_or(0),
        pages_vue
            .get(2)
            .copied()
            .or_else(|| pages_vue.first().copied())
            .unwrap_or(0),
    );
    let exact = state
        .render
        .visible_exact
        .or(state.render.visible)
        .unwrap_or(defaut);
    // On matérialise une page de marge de chaque côté. Elle est préchargée
    // par une bande de la hauteur de la fenêtre, pour qu'un défilement rapide
    // n'ouvre pas sur une page encore vide.
    let (first, last) = state
        .render
        .visible
        .or(state.render.visible_exact)
        .unwrap_or(defaut);
    let total = crate::typst::render::page_count(&document);
    let revision = state.render.revision;
    let generation = state.render.generation.clone();
    let vue = state.render.vue_generation.clone();
    let vue_courante = vue.load(Ordering::Relaxed);
    let sens = state.render.scroll_direction;
    let box_size = state
        .render
        .viewport
        .unwrap_or_else(crate::ui::preview::default_viewport);
    let scale_factor = state.scale_factor;
    let ecran = f64::from(scale_factor).max(crate::typst::render::MIN_SCREEN_SCALE);

    let sizes = crate::ui::preview::sizes_of_with_zoom(&document, box_size, state.render.zoom);
    let sizes_vue = crate::ui::preview::sizes_de_la_vue(&state.render);
    let pages_vue = crate::ui::preview::pages_de_la_vue(&state.render);
    let scroll = state.render.scroll;
    let fenetre = (box_size.width, box_size.height);

    let mut candidates: Vec<(usize, bool)> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .map(|index| {
            let prefetch = index < exact.0 || index > exact.1;
            (index, prefetch)
        })
        .collect();

    // Les pages réellement visibles passent avant tout préchargement. Un
    // lecteur rapide ne laisse jamais une page de marge prendre la place de
    // la page qu'on lit : le préchargement attend qu'elles soient toutes
    // réservées ou terminées.
    if candidates.iter().any(|(_, prefetch)| !prefetch) {
        candidates.retain(|(_, prefetch)| !prefetch);
    }

    // La page au centre de la fenêtre d'abord, puis ses voisines : les
    // emplacements de rasterisation sont rares, et une page à peine visible
    // ne doit pas prendre celui de la page qu'on lit. Pour le préchargement,
    // la page vers laquelle on va passe avant celle qu'on quitte.
    let centre = if let Some((premiere, derniere)) = state.render.visible {
        (premiere + derniere) / 2
    } else {
        0
    };
    candidates
        .sort_by_key(|(index, prefetch)| priorite_de_rendu(*index, *prefetch, exact, sens, centre));

    // Réserver avant de construire les tâches : deux changements de viewport
    // qui arrivent avant la réponse ne doivent pas rasteriser deux fois la
    // même page. Chaque tâche conserve ainsi un seul tampon RGBA en mémoire.
    let wanted: Vec<(usize, bool)> = candidates
        .into_iter()
        .filter(|(index, _)| state.render.begin_page_render(revision, *index))
        .collect();

    state.render.requested += wanted.len() as u64;

    let tasks: Vec<Task<Message>> = wanted
        .into_iter()
        .map(|(index, prefetch)| {
            let pdf = pdf.clone();
            let generation = generation.clone();
            let vue = vue.clone();

            // La densité est celle de **l'affichage** : la page est montrée
            // réduite — le panneau est plus étroit qu'une page — et le rendu
            // n'a besoin que des pixels que cette réduction consomme. Elle se
            // calcule page par page, parce qu'une page en paysage n'a pas le
            // même rapport qu'une page en portrait.
            let page = crate::typst::render::page_size(&document, index).unwrap_or((595.3, 841.9));
            let density = crate::typst::render::pixel_per_pt(
                crate::ui::preview::display_ratio_for_zoom(box_size, page, state.render.zoom),
                scale_factor,
            );
            let rapide = state.render.qualite_rapide.contains(&index);

            let taille = sizes.get(index).copied().unwrap_or((595.3, 841.9));
            let boite = if prefetch {
                // Une bande de la hauteur de la fenêtre : la page qui entre
                // par le haut montre son début, celle qui entre par le bas sa
                // fin. La bande coûte le même prix qu'une région visible,
                // même à fort zoom, et évite la page blanche au premier cran
                // de molette.
                crate::ui::preview::region_de_prechargement(taille, fenetre, index < exact.0)
            } else {
                let position = pages_vue
                    .iter()
                    .position(|page| *page == index)
                    .unwrap_or(0);
                let haut = crate::ui::preview::scroll_offset_for_page(&sizes_vue, position);
                crate::ui::preview::region_pour_page(
                    taille,
                    haut,
                    scroll,
                    fenetre,
                    super::MARGE_RENDU,
                )
            };
            // Fractions de la page -> pixels de sortie, à la densité du rendu.
            let (x0, y0) = (
                (boite.0 * taille.0 * ecran as f32) as u32,
                (boite.1 * taille.1 * ecran as f32) as u32,
            );
            let (largeur, hauteur) = (
                (boite.2 * taille.0 * ecran as f32).ceil() as u32,
                (boite.3 * taille.1 * ecran as f32).ceil() as u32,
            );

            Task::perform(
                async move {
                    if generation.load(Ordering::Relaxed) != revision
                        || vue.load(Ordering::Relaxed) != vue_courante
                    {
                        return None;
                    }

                    // Une seule passe pour l'aperçu. Le mode rapide de
                    // `pdf-render` est un pipeline u8 dont l'écart avec le
                    // pipeline f32 est sous-perceptuel ; le second passage
                    // « fin » doublait le coût de chaque page pendant un
                    // défilement pour un gain invisible.
                    let qualite = pdf_render::RasterQuality::Speed;

                    crate::typst::render::render_region_avec_qualite(
                        &pdf, index, density, x0, y0, largeur, hauteur, qualite,
                    )
                },
                move |image| Message::PageRendered(revision, index, image, boite, rapide),
            )
        })
        .collect();

    Task::batch(tasks)
}

/// L'ordre d'un travail de rasterisation : visible avant préchargement, puis
/// la page vers laquelle on va avant celle qu'on quitte, puis la proximité
/// du centre de la fenêtre.
pub(crate) fn priorite_de_rendu(
    index: usize,
    prefetch: bool,
    exact: (usize, usize),
    sens: i8,
    centre: usize,
) -> (u8, u8, usize) {
    let direction = if prefetch && ((sens >= 0 && index > exact.1) || (sens < 0 && index < exact.0))
    {
        0
    } else {
        1
    };

    (
        if prefetch { 1 } else { 0 },
        direction,
        index.abs_diff(centre),
    )
}

#[cfg(test)]
mod tests_de_priorite {
    use super::priorite_de_rendu;

    #[test]
    fn le_visible_passe_avant_le_prechargement() {
        let exact = (4, 5);
        assert!(
            priorite_de_rendu(4, false, exact, 1, 4) < priorite_de_rendu(6, true, exact, 1, 4),
            "la page visible passe avant la page de marge"
        );
    }

    #[test]
    fn le_prechargement_suit_le_sens_du_defilement() {
        let exact = (4, 5);
        let vers_le_bas = priorite_de_rendu(6, true, exact, 1, 4);
        let vers_le_haut = priorite_de_rendu(3, true, exact, 1, 4);
        assert!(
            vers_le_bas < vers_le_haut,
            "vers le bas : la page 6 d'abord"
        );

        let vers_le_haut = priorite_de_rendu(3, true, exact, -1, 4);
        let vers_le_bas = priorite_de_rendu(6, true, exact, -1, 4);
        assert!(
            vers_le_haut < vers_le_bas,
            "vers le haut : la page 3 d'abord"
        );
    }
}
