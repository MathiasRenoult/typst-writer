//! Le rendu : la compilation, l'export PDF, la rasterisation,
//! la surface de prévisualisation (zoom, séparateur, fenêtre).
//!
//! Ses bras vivaient dans le conducteur (feuille de route, phase 1,
//! cartographie, geste 3) ; ils sont réunis par famille.

use super::Message;
use super::State;
use crate::app::state::PreviewPosition;
use crate::app::state::Splitter;
use iced::Task;
use iced::widget::image::Handle;

pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::RenduInitialiser => {
            if state.render.pending || state.render.export_en_cours {
                return Task::none();
            }

            // Le partage d'abord : si ce texte a déjà son rendu, la
            // fenêtre le montre — le même PDF pour tout le monde.
            let empreinte = state.compilation_fingerprint();

            if let Some(partage) = state.rendus_partages.get(&empreinte) {
                if partage.document.is_some() {
                    state.render.document.clone_from(&partage.document);
                }

                if partage.lecteur.is_some() {
                    state.render.lecteur.clone_from(&partage.lecteur);
                    let total = state
                        .render
                        .document
                        .as_deref()
                        .map_or(0, crate::typst::render::page_count);
                    state.render.invalidate_rasterization(total);
                    return super::rasterisation::visible(state);
                }
            }

            if state.render.document.is_none() && !state.render.done {
                state.render.pending = true;
                return super::compile_task(state);
            }

            if state.render.document.is_some()
                && state.render.lecteur.is_none()
                && !state.render.export_tente
            {
                state.render.export_tente = true;
                return super::generer_le_pdf_du_rendu(state);
            }

            Task::none()
        }
        Message::RenderTick => {
            // Le geste de zoom est fini : le rendu net part, une seule
            // fois, et non un rendu par cran de molette.
            if state
                .render
                .zoom_depuis
                .is_some_and(|depuis| depuis.elapsed() >= super::DUREE_GESTE_ZOOM)
            {
                state.render.zoom_depuis = None;
                return super::rasterisation::visible(state);
            }

            if let Some(tache) = crate::app::epreuves::driver_zoom(state) {
                return tache;
            }

            // Les épreuves peuvent piloter l'éditeur elles-mêmes : un pas par
            // tic, et elles rendent la main aussitôt.
            if let Some(tache) = crate::app::epreuves::driver(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_renommage(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_reglages(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_composeur(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_glyphe(state) {
                return tache;
            }

            if let Some(tache) = crate::app::epreuves::driver_annulation(state) {
                return tache;
            }

            // Chaque fenêtre de rendu visible montre son document — celle
            // du démarrage aussi (bug-report-9 : l'initialisation laissait
            // l'aperçu vide tant qu'on ne basculait pas de mode).
            if let Some(tache) = super::rendre_les_fenetres_de_rendu(state) {
                return tache;
            }

            // Le compte de mots est une donnée de la vue, et la vue n'a pas le
            // droit de modifier l'état : il se calcule donc ici, au rythme des
            // tics, et seulement quand le texte a changé. Le calculer chez elle,
            // c'était cloner le document entier à chaque image — deux fois,
            // puisque la barre d'état et le panneau le demandent chacun.
            // Les glyphes suivent le texte, **quel que soit** le chemin qui
            // l'a changé : frappe, citation insérée, remplacement, bloc
            // déplacé. Leur repose est sensible au changement : un document
            // immobile ne coûte rien, et il n'y a plus de chemin oublié.
            state.editor.marques_a_jour();

            if state.render.words_pending {
                let content = state.editor.content();

                state.render.words = Some(state.content_word_count(&content));
                state.render.words_pending = false;
            }

            if state.render.pending {
                // Une seule compilation à la fois ; si le contenu a changé
                // pendant ce temps, le tick suivant relancera.
                return Task::none();
            }

            // L'empreinte couvre l'assemblage et le fichier ouvert : comparer le
            // seul chapitre rendrait l'aperçu sourd à toute frappe, puisque
            // l'assemblage ne change pas quand on écrit dans un chapitre.
            let fingerprint = state.compilation_fingerprint();
            let newlines = state.editor.content().matches('\n').count();

            if !state
                .render
                .compile_due(&fingerprint, newlines, std::time::Instant::now())
            {
                return Task::none();
            }

            state.render.pending = true;

            super::compile_task(state)
        }
        Message::RenderDone(outcome) => {
            state.render.pending = false;
            // Un retour périmé ne se fait pas passer pour l'état courant : le
            // texte a changé depuis la compilation, ou le projet portant ce
            // chemin a été remplacé sous le même nom. Comparer les deux
            // empreintes — celle que `compile` rapporte et celle que la vue
            // compilerait maintenant — est la seule façon de le savoir ; le
            // tick suivant recompile ce qui est réellement à l'écran.
            if outcome.content != state.compilation_fingerprint() {
                return Task::none();
            }
            state.render.export_tente = false;
            state.render.done = true;
            let chapter_page = outcome.chapter_page;

            state.render.content = outcome.content;
            // Les erreurs du fichier généré parlent du document source :
            // le générateur garde les numéros de ligne (feuille de route,
            // phase 2, point 2.5).
            let source = state.editor.path.clone().unwrap_or_default();
            state.render.errors = outcome
                .errors
                .into_iter()
                .map(|d| d.vers_la_source(&source))
                .collect();
            state.render.warnings = outcome.warnings;

            // Un échec ne vide pas l'aperçu : la dernière image bonne
            // reste affichée sous les erreurs, et ses textures gardent
            // leur place jusqu'à ce qu'une nouvelle compilation les
            // remplace. C'était la promesse du double tampon, tenue.
            // Le document compilé rejoint le partage : toute fenêtre qui
            // montre ce texte le montre lui, jamais une copie (bug-report-10).
            state
                .rendus_partages
                .entry(state.render.content.clone())
                .or_default()
                .completant(&crate::app::state::RenduPartage {
                    document: outcome.document.clone(),
                    lecteur: None,
                });

            if outcome.document.is_some() {
                state.render.document = outcome.document;
            }

            // Nouveau document : toute image en cache est dorénavant suspecte.
            // On ne les supprime pas pour autant — elles restent affichées
            // (double tampon) — mais elles sont marquées périmées, et seules
            // les pages visibles seront rasterisées à nouveau.
            state.render.revision += 1;

            // Les tâches de l'ancienne révision peuvent encore finir, mais
            // chacune porte sa propre clé et ne réserve plus de place pour la
            // nouvelle. Le cache et les pages périmées, eux, sont bornés au
            // nombre de pages actuel.
            state.render.rendering.clear();

            if let Some(document) = state.render.document.as_deref() {
                let total = crate::typst::render::page_count(document);

                state.render.page_cache.retain(|index, _| *index < total);
                state.render.stale.retain(|index| *index < total);
            }

            state.render.chapter_page = chapter_page;
            state.render.outline = super::source_outline(state, outcome.outline);

            // Toutes les pages sont périmées, et pas seulement les visibles :
            // l'image d'une page hors écran appartient elle aussi à la
            // révision précédente, et se retrouverait affichée telle quelle en
            // y revenant. Le coût reste borné, `rasterize_visible` ne rendant
            // que les pages visibles.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            for index in 0..total {
                state.render.stale.insert(index);
            }

            // Rendre les pages visibles, et amener l'aperçu sur le chapitre
            // qu'on écrit s'il vient de changer.
            Task::batch([
                super::rasterisation::visible(state),
                super::follow_chapter(state),
            ])
        }
        Message::PdfApercu(resultat) => {
            state.render.export_en_cours = false;

            let octets = match resultat {
                Ok(octets) => octets,
                Err(probleme) => {
                    state.render.errors.push(crate::typst::render::Diagnostic {
                        origine: crate::typst::render::Origine::ExportPdf,
                        message: probleme.clone(),
                        line: None,
                        column: None,
                        file: None,
                    });
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                    return Task::none();
                }
            };

            match crate::typst::render::ouvrir_pdf(octets) {
                Some(pdf) => {
                    let partage = std::sync::Arc::new(pdf);

                    if let Some(empreinte) = state.render.export_empreinte.take() {
                        state
                            .rendus_partages
                            .entry(empreinte)
                            .or_default()
                            .completant(&crate::app::state::RenduPartage {
                                document: None,
                                lecteur: Some(partage.clone()),
                            });
                    }

                    state.render.lecteur = Some(partage);
                    let total = state
                        .render
                        .document
                        .as_deref()
                        .map_or(0, crate::typst::render::page_count);
                    state.render.invalidate_rasterization(total);
                    super::rasterisation::visible(state)
                }
                None => {
                    state.render.errors.push(crate::typst::render::Diagnostic {
                        origine: crate::typst::render::Origine::RenduPdf,
                        message: "Le PDF généré'est pas lisible : le rendu reste en attente."
                            .to_string(),
                        line: None,
                        column: None,
                        file: None,
                    });
                    state.note = Some(crate::app::state::Note::erreur(
                        "Le PDF généré'est pas lisible : le rendu reste en attente.".to_string(),
                    ));
                    Task::none()
                }
            }
        }
        Message::PageRendered(revision, index, image, boite) => {
            state.render.finish_page_render(revision, index);

            // Une compilation est arrivée pendant le rendu : cette image
            // appartient à un document dépassé, on la jette plutôt que de
            // faire régresser l'affichage.
            if state.render.image_perimee(revision, index) {
                return super::rasterisation::visible(state);
            }

            state.render.stale.remove(&index);

            if let Some(image) = image {
                // Le papier se lit sur le premier pixel de la première page
                // rendue : c'est le coin supérieur gauche, donc le fond du
                // document. Une seule fois — il ne change pas d'une page à
                // l'autre — et il sert ensuite à remplir les cadres de page
                // avant que leur texture n'existe.
                if state.render.paper.is_none() {
                    state.render.paper = super::paper_colour(&image);
                }

                // Pixels identiques à ceux déjà en place : la page n'a pas
                // changé. On garde le **même** `Handle`, donc la même texture,
                // et le compositeur n'a rien à refaire. Remplacer l'image
                // faisait naître une texture par compilation et par page ; à
                // plusieurs compilations par seconde, c'est ce qui se voyait
                // comme un clignotement.
                if state.render.image_hashes.get(&index) == Some(&image.hash) {
                    return Task::none();
                }

                state.render.image_hashes.insert(index, image.hash);

                // Un seul `Handle` par page, créé ici et réutilisé tel quel
                // par `view()` : le backend retrouve sa texture et ne la
                // retéléverse pas. L'ancienne image reste en place jusqu'à
                // ce que celle-ci la remplace — l'écran ne passe jamais par
                // le vide.
                state.render.page_cache.insert(
                    index,
                    crate::app::state::Tranche {
                        gauche: boite.0,
                        haut: boite.1,
                        largeur: boite.2,
                        hauteur: boite.3,
                        image: Handle::from_rgba(image.width, image.height, image.pixels),
                    },
                );

                // Une image peut arriver après qu'on a défilé ailleurs :
                // la refuser tout de suite, plutôt que de la garder pour rien.
                state.render.bound_page_cache();
            }

            // Une place s'est libérée : ce qui attendait part.
            super::rasterisation::visible(state)
        }
        Message::ViewportChanged(first, last, vue, scroll) => {
            // Le rectangle de fenêtre ne vient que d'un vrai défilement :
            // les sondes de taille du panneau le rapportent à l'origine,
            // et une ancre de zoom sur cette origine serait fausse.
            if scroll.is_some() {
                state.render.fenetre = Some(vue);
            }
            let scroll = scroll.unwrap_or(state.render.scroll);
            let bounds = vue.size();
            // « Aucune page » envoie (0, 0) : ce n'est pas une plage,
            // c'est l'absence d'information. Un document tenu dans la
            // fenêtre n'en reçoit jamais d'autre, et ses pages 2+ ne
            // venaient qu'après un défilement manuel.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            let (first, last) = if (first, last) == (0, 0) && total > 1 {
                (0, 2.min(total - 1))
            } else {
                (first, last)
            };

            let unchanged = state.render.visible == Some((first, last))
                && state.render.viewport == Some(bounds)
                && state.render.scroll == scroll;

            state.render.visible = Some((first, last));
            state.render.scroll = scroll;

            // La place d'une page a changé : la taille d'affichage change, donc
            // la densité dont le rendu a besoin. Les images en cache
            // appartiennent à l'ancienne — leur empreinte ne veut plus rien
            // dire — mais elles restent affichées, simplement remises à
            // l'échelle, jusqu'à l'arrivée des nouvelles.
            if state.render.viewport != Some(bounds) {
                state.render.viewport = Some(bounds);

                let total = state
                    .render
                    .document
                    .as_deref()
                    .map_or(0, crate::typst::render::page_count);

                // Une rasterisation lancée à l'ancienne taille ne satisfait
                // pas le nouveau viewport, même si elle porte sur la même page.
                state.render.invalidate_rasterization(total);
            }

            // **Rien de neuf : ne rien faire.** C'est la garde qui empêche un
            // cycle. Un viewport identique — même place, mêmes pages — signifie
            // que rien n'a bougé ; le traiter relançait des rasterisations, qui
            // produisaient des images neuves, qui reconstruisaient la vue, qui
            // signalait le même viewport : un tour de plus, plusieurs mégaoctets
            // à chaque fois.
            if unchanged {
                return Task::none();
            }

            // Le cache d'images est borné aux pages visibles et à leur
            // voisinage : c'est ici, où l'on sait ce qu'on voit, qu'on retire
            // le reste. Les `Handle` relâchés libèrent leurs textures.
            state.render.bound_page_cache();

            // Une page qui entre dans le viewport sans image valide doit
            // être rasterisée ; une page déjà en cache et à jour ne l'est
            // plus (c'est ce qui rendait chaque défilement coûteux).
            // Une page sans tranche couvrant la fenêtre visible part au
            // rendu : défiler hors de la marge coûte un rendu, défiler
            // dedans rien du tout.
            if let Some(document) = state.render.document.clone() {
                let sizes =
                    crate::ui::preview::sizes_of_with_zoom(&document, bounds, state.render.zoom);

                for index in first..=last {
                    let Some(taille) = sizes.get(index).copied() else {
                        continue;
                    };
                    let haut = crate::ui::preview::scroll_offset_for_page(&sizes, index);
                    let region = crate::ui::preview::region_pour_page(
                        taille,
                        haut,
                        scroll,
                        (bounds.width, bounds.height),
                        0.0,
                    );

                    if !state
                        .render
                        .page_cache
                        .get(&index)
                        .is_some_and(|tranche| crate::ui::preview::tranche_couvre(tranche, region))
                    {
                        state.render.stale.insert(index);
                    }
                }
            }

            super::rasterisation::visible(state)
        }
        Message::PreviewZoom(zoom) => super::set_preview_zoom(state, zoom),

        Message::PreviewZoomStep(step) => {
            let box_size = state
                .render
                .viewport
                .unwrap_or_else(crate::ui::preview::default_viewport);
            let page = state
                .render
                .document
                .as_deref()
                .and_then(|document| {
                    let index = state.render.visible.map_or(0, |(first, _)| first);
                    crate::typst::render::page_size(document, index)
                })
                .unwrap_or((595.3, 841.9));
            let zoom = crate::ui::preview::stepped_zoom(state.render.zoom, step, box_size, page);

            super::set_preview_zoom(state, zoom)
        }
        Message::TogglePreview => {
            state.layout.preview_visible = !state.layout.preview_visible;

            // La bascule rafraîchit : ce qui reparaît est ce qu'on écrit,
            // pas la mémoire du dernier rendu. Jamais de purge — l'ancien
            // reste affiché jusqu'au neuf, sans clignotement.
            if state.layout.preview_visible {
                return super::compile_task(state);
            }

            Task::none()
        }
        Message::TogglePreviewSync => {
            state.layout.follow_preview = !state.layout.follow_preview;

            if crate::app::epreuves::actif("lsp") {
                println!(
                    "[vue] suivi de l'aperçu : {}",
                    if state.layout.follow_preview {
                        "actif"
                    } else {
                        "coupé"
                    }
                );
            }

            Task::none()
        }
        Message::SetPreviewPosition(position) => {
            state.layout.preview_position = position;

            Task::none()
        }
        Message::SplitterPressed(which) => {
            // L'origine est la dernière position connue du curseur : elle est
            // tenue à jour à chaque mouvement de souris, y compris hors
            // glissement. Le premier mouvement pendant le glissement sert donc
            // à déplacer, pas à repérer l'origine — un glissement court ne
            // reste plus sans effet.
            let vertical = state.layout.preview_position == PreviewPosition::Bottom
                && which == Splitter::Preview;

            let origin = if vertical {
                state.layout.cursor.y
            } else {
                state.layout.cursor.x
            };

            // L'étendue de départ est mémorisée ici, une fois pour toutes : les
            // mouvements suivants sont des deltas depuis cette origine, et les
            // ajouter à la valeur courante les cumulerait.
            let extent = if vertical {
                state.layout.preview_height
            } else {
                match which {
                    Splitter::Sidebar => state.layout.sidebar_width,
                    Splitter::Preview => state.layout.preview_width,
                }
            };

            state.layout.dragging = Some(crate::app::state::Drag {
                which,
                origin,
                extent,
            });

            Task::none()
        }
        Message::SplitterReleased => {
            state.layout.dragging = None;

            // **Le relâchement va aussi à l'éditeur.** Si le bouton a été
            // relâché ailleurs que sur lui — sur une ligne de la liste de
            // complétion, par exemple —, il ne le voit jamais, et reste en
            // glissement : ensuite, chaque mouvement de souris déplace le caret
            // et la vue se met à le suivre. Le message est sans effet s'il n'y
            // avait rien à relâcher.
            Task::done(Message::Editor(iced_code_editor::Message::MouseRelease))
        }
        Message::MouseMoved(at) => {
            let Some(drag) = state.layout.dragging else {
                // Aucun séparateur tenu : on retient seulement la position du
                // curseur, pour que la prochaine saisie parte du bon endroit.
                state.layout.cursor = at;

                return Task::none();
            };

            // L'axe dépend de la disposition : en colonnes, l'aperçu se règle
            // en hauteur.
            let vertical = state.layout.preview_position == PreviewPosition::Bottom
                && drag.which == Splitter::Preview;

            let delta = if vertical {
                at.y - drag.origin
            } else {
                at.x - drag.origin
            };

            state.layout.cursor = at;

            // Tout part de l'étendue de départ, jamais de la valeur courante :
            // le delta est mesuré depuis l'origine.
            let dragged = crate::app::state::LayoutState::dragged_extent(
                drag.which,
                vertical,
                drag.extent,
                delta,
            );

            if vertical {
                state.layout.preview_height = dragged;
            } else {
                match drag.which {
                    Splitter::Sidebar => state.layout.sidebar_width = dragged,
                    Splitter::Preview => state.layout.preview_width = dragged,
                }
            }

            Task::none()
        }
        Message::ScaleFactor(scale_factor) => {
            // La densité du rendu en découle, et elle décide de la mémoire d'une
            // page : une A4 pèse 8 Mo de pixels à 2 px/pt, dont le backend fait
            // une texture d'environ 14 Mio.
            if (scale_factor - state.scale_factor).abs() < 0.001 {
                return Task::none();
            }

            state.scale_factor = scale_factor;

            // Les images en cache appartiennent à l'ancienne densité : leur
            // empreinte ne veut plus rien dire, et toutes les pages sont à
            // refaire. Elles ne sont pas jetées pour autant — une image à
            // l'ancienne densité reste juste à l'écran, simplement remise à
            // l'échelle, jusqu'à l'arrivée de la nouvelle. Changer d'écran ne
            // doit pas faire disparaître le document.
            let total = state
                .render
                .document
                .as_deref()
                .map_or(0, crate::typst::render::page_count);

            state.render.invalidate_rasterization(total);

            super::rasterisation::visible(state)
        }
        _ => Task::none(),
    }
}
