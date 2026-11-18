//! Les gestes du lecteur PDF : sélection, copie, recherche et liens.
//!
//! Le texte vient de `PagedDocument`, jamais du PDF : la couche est extraite
//! à la compilation et n'est installée que lorsque le PDF qu'elle décrit est
//! affiché. Les pages restent des images ; seuls les rectangles de sélection
//! et de recherche sont dessinés par-dessus.

use super::super::{Message, State};
use crate::typst::texte::PositionTexte;
use iced::Task;
use typst::layout::{Abs, Point};

/// Traite les messages de l'aperçu. La porte d'activation et la garde du
/// verrou sont appliquées par `update_interne` avant l'appel.
pub fn traiter(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::TexteApercuPret(empreinte, texte) => {
            // Une extraction terminée après une compilation plus récente
            // appartient à un document périmé : elle ne remplace jamais la
            // couche du texte qui est en train d'être composé.
            if state.render.texte_en_cours.as_deref() == Some(empreinte.as_str()) {
                state.render.texte_en_cours = None;
            }
            if empreinte == state.render.content {
                state.render.preparer_texte(empreinte, texte);
            }
            Task::none()
        }

        Message::SelectionApercuAppuyer {
            page,
            point,
            etendue,
            clics,
        } => {
            let Some(texte) = state.render.texte_affiche() else {
                return Task::none();
            };
            let Some(glyphe) = texte.position_au_point(
                page,
                Point::new(Abs::pt(point.0 as f64), Abs::pt(point.1 as f64)),
            ) else {
                return Task::none();
            };
            let page_texte = texte.page(page).expect("page de la position");

            let selection = if clics >= 3 {
                let (debut, fin) = page_texte.ligne_autour(glyphe.glyphe);
                crate::typst::texte::SelectionApercu {
                    ancre: PositionTexte {
                        page,
                        glyphe: debut,
                    },
                    focus: PositionTexte { page, glyphe: fin },
                }
            } else if clics == 2 {
                let (debut, fin) = page_texte.mot_autour(glyphe.glyphe);
                crate::typst::texte::SelectionApercu {
                    ancre: PositionTexte {
                        page,
                        glyphe: debut,
                    },
                    focus: PositionTexte { page, glyphe: fin },
                }
            } else if etendue {
                let ancre = state
                    .render
                    .selection
                    .map_or(glyphe, |selection| selection.ancre);
                crate::typst::texte::SelectionApercu {
                    ancre,
                    focus: glyphe,
                }
            } else {
                crate::typst::texte::SelectionApercu {
                    ancre: glyphe,
                    focus: glyphe,
                }
            };

            state.render.selection = Some(selection);
            Task::none()
        }

        Message::SelectionApercuGlisser { page, point } => {
            let Some(texte) = state.render.texte_affiche() else {
                return Task::none();
            };
            let Some(focus) = texte.position_au_point(
                page,
                Point::new(Abs::pt(point.0 as f64), Abs::pt(point.1 as f64)),
            ) else {
                return Task::none();
            };

            if let Some(selection) = state.render.selection.as_mut() {
                selection.focus = focus;
            }

            Task::none()
        }

        Message::SelectionApercuRelacher => Task::none(),

        Message::SelectionApercuCopier => {
            let Some(selection) = state.render.selection else {
                return Task::none();
            };
            let Some(texte) = state.render.texte_affiche() else {
                return Task::none();
            };

            iced::clipboard::write(texte.texte_selection(selection.ancre, selection.focus))
        }

        Message::RechercheApercuOuvrir => {
            state.render.recherche_apercu.ouvrir();
            iced::widget::operation::focus(crate::ui::preview::id_recherche_apercu())
        }

        Message::RechercheApercuFermer => {
            state.render.recherche_apercu.fermer();
            Task::none()
        }

        Message::RechercheApercuMotif(motif) => {
            state.render.recherche_apercu.motif = motif;
            state.render.recherche_apercu.courante = 0;

            if let Some(texte) = state.render.texte_affiche() {
                state.render.recherche_apercu.resultats = crate::app::recherche_apercu::chercher(
                    texte,
                    &state.render.recherche_apercu.motif,
                );
            } else {
                state.render.recherche_apercu.resultats.clear();
            }

            Task::none()
        }

        Message::RechercheApercuSuivant => {
            state.render.recherche_apercu.avancer(1);
            aller_au_resultat(state)
        }

        Message::RechercheApercuPrecedent => {
            state.render.recherche_apercu.avancer(-1);
            aller_au_resultat(state)
        }

        Message::LienApercuActiver(page, rang) => activer_lien(state, page, rang),

        _ => Task::none(),
    }
}

/// Amène l'aperçu sur l'occurrence courante.
fn aller_au_resultat(state: &State) -> Task<Message> {
    let Some(resultat) = state.render.recherche_apercu.courante() else {
        return Task::none();
    };
    let y = state
        .render
        .texte_affiche()
        .and_then(|texte| texte.page(resultat.page))
        .and_then(|page| page.glyphes.get(resultat.debut))
        .map_or(Abs::zero(), |glyphe| glyphe.rect.min.y);

    aller_a_position(state, resultat.page, Point::new(Abs::zero(), y))
}

/// Active un lien de la page.
fn activer_lien(state: &mut State, page: usize, rang: usize) -> Task<Message> {
    let Some(lien) = state
        .render
        .texte_affiche()
        .and_then(|texte| texte.page(page))
        .and_then(|page| page.liens.get(rang))
        .cloned()
    else {
        return Task::none();
    };

    match lien.destination {
        typst::model::Destination::Url(url) => {
            ouvrir_url(&url.into_inner());
            Task::none()
        }
        typst::model::Destination::Location(location) => {
            let Some(document) = state.render.document.as_ref() else {
                return Task::none();
            };
            let Some(position) = document.introspector().position(location) else {
                return Task::none();
            };
            aller_a_position(state, position.page.get() - 1, position.point)
        }
        typst::model::Destination::Position(position) => {
            aller_a_position(state, position.page.get() - 1, position.point)
        }
    }
}

/// Va à une page, à une hauteur donnée dans cette page.
fn aller_a_position(state: &State, page: usize, point: typst::layout::Point) -> Task<Message> {
    let Some(document) = state.render.document.as_ref() else {
        return Task::none();
    };
    let Some(position) = crate::ui::preview::position_dans_la_vue(&state.render, page) else {
        return Task::none();
    };
    let sizes = crate::ui::preview::sizes_de_la_vue(&state.render);
    let Some((_, hauteur)) = sizes.get(position).copied() else {
        return Task::none();
    };
    let Some((_, hauteur_page)) = crate::typst::render::page_size(document, page) else {
        return Task::none();
    };

    let ratio = if hauteur_page > 0.0 {
        hauteur as f64 / hauteur_page
    } else {
        1.0
    };
    let dans_la_page = (point.y.to_pt() * ratio) as f32;
    let offset = crate::ui::preview::scroll_offset_for_page(&sizes, position) + dans_la_page;

    iced::advanced::widget::operate(iced::advanced::widget::operation::scrollable::scroll_to::<
        Message,
    >(
        super::super::id_du_scrollable(state),
        iced::advanced::widget::operation::scrollable::AbsoluteOffset {
            x: None,
            y: Some(offset),
        },
    ))
}

/// Ouvre une adresse avec l'outil du système.
fn ouvrir_url(url: &str) {
    #[cfg(target_os = "macos")]
    let mut commande = std::process::Command::new("open");
    #[cfg(windows)]
    let mut commande = std::process::Command::new("explorer");
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let mut commande = std::process::Command::new("xdg-open");

    let _ = commande.arg(url).spawn();
}
