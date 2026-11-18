//! La bibliothèque est une page de l'application, pas une boîte de dialogue.
//! Les couvertures ouvrent les projets ; chaque étagère porte sa création.

use crate::app::Message;
use crate::app::State;
use crate::app::message::FormatExport;
use crate::project::Project;
use crate::ui::theme::{self, Palette};
use iced::widget::{
    Space, button, column, container, image, mouse_area, row, scrollable, text_input,
};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Les poignées d'image restent stables entre deux vues. Recréer une poignée
// à chaque image oblige le moteur graphique à réimporter la couverture.
thread_local! {
    static COUVERTURES: std::cell::RefCell<HashMap<String, (u64, image::Handle)>> =
        std::cell::RefCell::new(HashMap::new());
}

fn couverture(state: &State, nom: &str) -> Option<image::Handle> {
    let (_, pixels, largeur, hauteur) = state.vignettes.iter().find(|(cle, ..)| cle == nom)?;
    // Empreinte de cache, jamais écrite sur disque : elle sert seulement à
    // savoir si la couverture affichée est encore la bonne.
    let mut empreinte = std::collections::hash_map::DefaultHasher::new();
    pixels.hash(&mut empreinte);
    largeur.hash(&mut empreinte);
    hauteur.hash(&mut empreinte);
    let empreinte = empreinte.finish();

    COUVERTURES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.retain(|cle, _| state.projects.iter().any(|projet| projet.name == *cle));
        let entree = cache.entry(nom.to_string()).or_insert_with(|| {
            (
                empreinte,
                image::Handle::from_rgba(*largeur, *hauteur, pixels.clone()),
            )
        });
        if entree.0 != empreinte {
            *entree = (
                empreinte,
                image::Handle::from_rgba(*largeur, *hauteur, pixels.clone()),
            );
        }
        Some(entree.1.clone())
    })
}

// Toutes les vignettes partagent les proportions A4, même quand le
// document composé est en Letter, en A5 ou en paysage. 140 / 198 est
// exactement 210 / 297, le rapport du papier A4.
const LARGEUR_APERCU: f32 = 140.0;
const HAUTEUR_APERCU: f32 = 198.0;

/// Nombre de cartes par rangée : les couvertures gardent une largeur lisible.
fn nombre_de_colonnes(largeur: f32) -> usize {
    (((largeur - 64.0) / 218.0).floor() as usize).max(1)
}

fn style_carte(palette: Palette, survol: bool, selectionne: bool) -> container::Style {
    container::Style {
        background: Some(if selectionne {
            palette.surface_hover.into()
        } else {
            palette.background.into()
        }),
        border: iced::Border {
            color: if selectionne || survol {
                palette.accent
            } else {
                palette.border
            },
            width: if selectionne { 2.0 } else { 1.0 },
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

fn carte<'a>(state: &'a State, projet: &'a Project) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let selectionne = state.selection_projets.contains(&projet.dir);
    let apercu: Element<'a, Message> = match couverture(state, &projet.name) {
        Some(poignee) => image(poignee)
            .width(Length::Fixed(LARGEUR_APERCU))
            .height(Length::Fixed(HAUTEUR_APERCU))
            .into(),
        None => container(
            column![
                theme::icon(
                    lucide_icons::iced::icon_file_text::<iced::Theme>(),
                    30.0,
                    palette.text_muted
                ),
                palette
                    .label("Aperçu indisponible")
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
            ]
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fixed(HAUTEUR_APERCU))
        .into(),
    };
    let _nom = projet.name.clone();
    let etiquette = if selectionne {
        "Sélectionné · Ctrl+clic pour retirer"
    } else if projet.dir == state.project.dir {
        "Ouvert · cliquer pour reprendre"
    } else {
        "Cliquer pour ouvrir · Ctrl pour sélectionner"
    };
    let ouvrir = button(
        column![
            container(apercu).center_x(Length::Fill),
            palette
                .label(projet.name.clone())
                .size(theme::jetons::SAILLANT),
            palette
                .label(etiquette)
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(10),
    )
    .on_press(Message::ProjetCliquer(projet.dir.clone()))
    .width(Length::Fill)
    .padding(12)
    .style(move |_, statut| iced::widget::button::Style {
        background: Some(
            if matches!(statut, iced::widget::button::Status::Hovered) {
                palette.surface_hover
            } else {
                palette.background
            }
            .into(),
        ),
        text_color: palette.text,
        ..Default::default()
    });

    // Changer la miniature est une action de la carte, pas l'ouverture du
    // projet : le bouton vit hors de la zone de glisser-déposer.
    let changer = button(
        row![
            theme::icon(
                lucide_icons::iced::icon_image::<iced::Theme>(),
                12.0,
                palette.text_muted
            ),
            palette
                .label("Changer la miniature")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .on_press(Message::MiniatureChoisir(projet.dir.clone()))
    .padding([3, 6])
    .style(move |_, statut| iced::widget::button::Style {
        background: matches!(statut, iced::widget::button::Status::Hovered)
            .then(|| palette.surface_hover.into()),
        text_color: palette.text_muted,
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    });

    let zone = mouse_area(ouvrir).on_press(Message::DragCommencer(projet.name.clone()));
    let contenu = column![zone, changer, Space::new().height(4)].spacing(8);
    container(contenu)
        .width(Length::Fixed(202.0))
        .style(move |_| {
            style_carte(
                palette,
                state.drag_projet.as_deref() == Some(projet.name.as_str()),
                selectionne,
            )
        })
        .into()
}

/// La bibliothèque occupe toute la fenêtre. Les dialogues de création sont
/// composés par le shell au-dessus de cette page, et non derrière elle.
pub fn view(state: &State) -> Element<'_, Message> {
    if !state.projets_vue {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }
    let palette = state.resolved.palette;

    // L'ordre vient du fichier d'ordre, y compris pour les étagères vides ;
    // un ancien classement sans ordre est ajouté à la suite, sans le perdre.
    let mut noms = state.etageres_ordre.clone();
    for etagere in state.etageres.values() {
        let etagere = etagere.trim();
        if !etagere.is_empty() && !noms.iter().any(|nom| nom == etagere) {
            noms.push(etagere.to_string());
        }
    }

    let mut groupes: Vec<(String, Vec<&Project>)> = noms
        .iter()
        .filter(|nom| !nom.trim().is_empty())
        .map(|nom| (nom.clone(), Vec::new()))
        .collect();
    let mut sans_etagere = Vec::new();
    for projet in &state.projects {
        let etagere = state
            .etageres
            .get(&projet.name)
            .map(|nom| nom.trim())
            .unwrap_or("");
        if etagere.is_empty() {
            sans_etagere.push(projet);
        } else if let Some((_, projets)) =
            groupes.iter_mut().find(|(nom, _)| nom.as_str() == etagere)
        {
            projets.push(projet);
        } else {
            groupes.push((etagere.to_string(), vec![projet]));
        }
    }
    if !sans_etagere.is_empty() || groupes.is_empty() {
        groupes.push((String::new(), sans_etagere));
    }

    let colonnes = nombre_de_colonnes(state.window_size.width);
    let mut rayons = column![].spacing(32);
    for (etagere, projets) in groupes {
        let titre = if etagere.is_empty() {
            "Sans étagère"
        } else {
            etagere.as_str()
        };
        let compte = projets.len();
        let position = noms.iter().position(|nom| nom == &etagere);
        let peut_monter = position.is_some_and(|position| position > 0);
        let peut_descendre = position.is_some_and(|position| position + 1 < noms.len());

        let mut bandeau = row![
            palette.label(titre.to_string()).size(theme::jetons::TITRE),
            palette
                .label(format!("{compte} projet(s)"))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        if !etagere.is_empty() {
            bandeau = bandeau.push(
                button(palette.label("↑").size(theme::jetons::DETAIL))
                    .on_press_maybe(peut_monter.then(|| Message::EtagereMonter(etagere.clone())))
                    .padding([2, 6]),
            );
            bandeau = bandeau.push(
                button(palette.label("↓").size(theme::jetons::DETAIL))
                    .on_press_maybe(
                        peut_descendre.then(|| Message::EtagereDescendre(etagere.clone())),
                    )
                    .padding([2, 6]),
            );
        }

        let mut rayon = column![bandeau].spacing(14);
        if projets.is_empty() {
            rayon = rayon.push(
                palette
                    .label("Cette étagère est vide. Créez son premier projet.")
                    .color(palette.text_muted),
            );
        }

        // La carte de création : la taille d'une vignette, à la suite.
        let creer = || {
            button(
                container(
                    iced::widget::column![
                        crate::ui::theme::icon(
                            lucide_icons::iced::icon_plus::<iced::Theme>(),
                            28.0,
                            palette.accent,
                        ),
                        palette
                            .label("Nouveau projet")
                            .size(theme::jetons::DETAIL)
                            .color(palette.text_muted),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .center_x(Length::Fixed(202.0))
                .center_y(Length::Fixed(HAUTEUR_APERCU)),
            )
            .on_press(Message::ProjetDepuisEtagere(etagere.clone()))
            .padding(12)
            .style(move |_, statut| button::Style {
                background: Some(
                    if matches!(statut, button::Status::Hovered) {
                        palette.surface_hover
                    } else {
                        palette.background
                    }
                    .into(),
                ),
                text_color: palette.text,
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: theme::jetons::RAYON.into(),
                },
                ..Default::default()
            })
        };

        let mut dernier_rangee = true;
        for (i, rangee) in projets.chunks(colonnes).enumerate() {
            let mut cartes = row![].spacing(16);
            for projet in rangee {
                cartes = cartes.push(carte(state, projet));
            }
            // La carte de création va à la suite de la dernière rangée.
            if i == projets.chunks(colonnes).count() - 1 {
                cartes = cartes.push(creer());
                dernier_rangee = false;
            }
            rayon = rayon.push(cartes);
        }
        if dernier_rangee {
            rayon = rayon.push(row![creer()].spacing(16));
        }

        let survolee =
            state.drag_projet.is_some() && state.drag_etagere.as_deref() == Some(etagere.as_str());
        rayons = rayons.push(
            mouse_area(
                container(rayon)
                    .padding(16)
                    .width(Length::Fill)
                    .style(move |_| style_carte(palette, survolee, false)),
            )
            .on_enter(Message::DragSurvol(etagere.clone()))
            .on_release(Message::DragPoser(etagere)),
        );
    }

    let entete = row![
        column![
            palette.label("Bibliothèque").size(28),
            palette
                .label("Vos projets, rangés sur leurs étagères.")
                .color(palette.text_muted),
        ]
        .spacing(4),
        Space::new().width(Length::Fill),
        button(palette.label("+ Nouveau projet"))
            .on_press(Message::ProjetCreer)
            .padding([10, 16]),
        button(palette.label("Retour à l’écriture"))
            .on_press(Message::ProjetsVue)
            .padding([10, 16]),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let brouillon = state.etagere_brouillon.trim();
    let nouvelle = row![
        palette
            .label("Nouvelle étagère")
            .size(theme::jetons::DETAIL),
        text_input("Ex. : Thèse, articles, cours…", &state.etagere_brouillon)
            .on_input(Message::EtagereBrouillon)
            .on_submit(Message::EtagereCreer)
            .padding(10)
            .width(Length::Fixed(280.0)),
        button(palette.label("Créer l'étagère").size(theme::jetons::DETAIL))
            .on_press_maybe((!brouillon.is_empty()).then_some(Message::EtagereCreer))
            .padding([10, 12]),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let selection: Element<'_, Message> = if state.selection_projets.is_empty() {
        Space::new().width(Length::Fill).height(0.0).into()
    } else {
        let nombre = state.selection_projets.len();
        column![
            row![
                palette
                    .label(format!("{nombre} document(s) sélectionné(s)"))
                    .size(theme::jetons::DETAIL),
                Space::new().width(Length::Fill),
                button(
                    palette
                        .label("Tout désélectionner")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::SelectionVider)
                .padding([5, 10]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                button(palette.label("Exporter PDF").size(theme::jetons::DETAIL))
                    .on_press(Message::SelectionExporter(FormatExport::Pdf))
                    .padding([5, 10]),
                button(
                    palette
                        .label("Exporter Typst Writer")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::SelectionExporter(FormatExport::Tw))
                .padding([5, 10]),
                button(
                    palette
                        .label("Changer les normes")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::SelectionNormes)
                .padding([5, 10]),
                button(
                    palette
                        .label("Exporter Markdown")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::SelectionExporter(FormatExport::Markdown))
                .padding([5, 10]),
                button(palette.label("Exporter DOCX").size(theme::jetons::DETAIL))
                    .on_press(Message::SelectionExporter(FormatExport::Docx))
                    .padding([5, 10]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(6)
        .into()
    };

    container(
        column![
            entete,
            nouvelle,
            selection,
            palette
                .label("Cliquez pour ouvrir. Ctrl+clic sélectionne, Maj+clic sélectionne une plage ; les actions portent sur les documents sélectionnés.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            scrollable(rayons).height(Length::Fill),
        ]
        .spacing(22),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(palette.background.into()),
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_cartes_se_repartissent_sans_deborder_aux_largeurs_utiles() {
        for largeur in [820.0, 1180.0, 1600.0, 2400.0] {
            let colonnes = nombre_de_colonnes(largeur);
            assert!(colonnes >= 1);
            assert!(colonnes as f32 * 218.0 <= largeur - 64.0);
        }
    }
}
