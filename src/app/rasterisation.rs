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
    // pages suffisent à couvrir ce cas comme le précédent.
    let (first, last) = state
        .render
        .visible_exact
        .or(state.render.visible)
        .unwrap_or((0, 2));
    let total = crate::typst::render::page_count(&document);
    let revision = state.render.revision;
    let generation = state.render.generation.clone();
    let box_size = state
        .render
        .viewport
        .unwrap_or_else(crate::ui::preview::default_viewport);
    let scale_factor = state.scale_factor;
    let ecran = f64::from(scale_factor).max(crate::typst::render::MIN_SCREEN_SCALE);

    let sizes = crate::ui::preview::sizes_of_with_zoom(&document, box_size, state.render.zoom);
    let scroll = state.render.scroll;
    let fenetre = (box_size.width, box_size.height);

    let mut candidates: Vec<usize> = (first..=last.min(total.saturating_sub(1)))
        .filter(|index| state.render.stale.contains(index))
        .collect();

    // La page au centre de la fenêtre d'abord, puis ses voisines : les
    // emplacements de rasterisation sont rares, et une page à peine visible
    // ne doit pas prendre celui de la page qu'on lit.
    let centre = if let Some((premiere, derniere)) = state.render.visible {
        (premiere + derniere) / 2
    } else {
        0
    };
    candidates.sort_by_key(|index| index.abs_diff(centre));

    // Réserver avant de construire les tâches : deux changements de viewport
    // qui arrivent avant la réponse ne doivent pas rasteriser deux fois la
    // même page. Chaque tâche conserve ainsi un seul tampon RGBA en mémoire.
    let wanted: Vec<usize> = candidates
        .into_iter()
        .filter(|index| state.render.begin_page_render(revision, *index))
        .collect();

    state.render.requested += wanted.len() as u64;

    let tasks: Vec<Task<Message>> = wanted
        .into_iter()
        .map(|index| {
            let pdf = pdf.clone();
            let generation = generation.clone();

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

            // La **région** à rendre : la fenêtre visible, élargie de la
            // marge — jamais la page entière (bug-report-9, fluidité).
            let taille = sizes.get(index).copied().unwrap_or((595.3, 841.9));
            let haut = crate::ui::preview::scroll_offset_for_page(&sizes, index);
            let boite = crate::ui::preview::region_pour_page(
                taille,
                haut,
                scroll,
                fenetre,
                super::MARGE_RENDU,
            );
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
                    if generation.load(Ordering::Relaxed) != revision {
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
