//! La bibliothèque est une page de l'application, pas une boîte de dialogue.
//! Les couvertures ouvrent les projets ; chaque étagère porte sa création.

use crate::app::{Message, State};
use crate::project::Project;
use crate::ui::theme::{self, Palette};
use iced::widget::{
    Space, button, column, container, image, mouse_area, pick_list, row, scrollable, text_input,
};
use iced::{Alignment, Element, Length};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

// Les poignées d'image restent stables entre deux vues. Recréer une poignée
// à chaque image oblige le moteur graphique à réimporter la couverture.
thread_local! {
    static COUVERTURES: std::cell::RefCell<HashMap<String, (u64, image::Handle)>> =
        std::cell::RefCell::new(HashMap::new());
}

fn couverture(state: &State, nom: &str) -> Option<image::Handle> {
    let (_, pixels, largeur, hauteur) = state.vignettes.iter().find(|(cle, ..)| cle == nom)?;
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

/// Nombre de cartes par rangée : les couvertures gardent une largeur lisible.
fn nombre_de_colonnes(largeur: f32) -> usize {
    (((largeur - 64.0) / 218.0).floor() as usize).max(1)
}

fn style_carte(palette: Palette, survol: bool) -> container::Style {
    container::Style {
        background: Some(palette.background.into()),
        border: iced::Border {
            color: if survol {
                palette.accent
            } else {
                palette.border
            },
            width: 1.0,
            radius: theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

fn carte<'a>(
    state: &'a State,
    projet: &'a Project,
    etagere: &str,
    choix: &[String],
) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let apercu: Element<'a, Message> = match couverture(state, &projet.name) {
        Some(poignee) => image(poignee)
            .width(Length::Fixed(142.0))
            .height(Length::Fixed(196.0))
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
        .center_y(Length::Fixed(196.0))
        .into(),
    };
    let nom = projet.name.clone();
    let etiquette = if projet.dir == state.project.dir {
        "Ouvert · cliquer pour reprendre"
    } else {
        "Cliquer pour ouvrir"
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
    .on_press(Message::OpenProject(projet.dir.clone()))
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

    let destination = if etagere.is_empty() {
        "Sans étagère".to_string()
    } else {
        etagere.to_string()
    };
    let choix = choix.to_vec();
    let classement = pick_list(choix, Some(destination), move |destination: String| {
        Message::EtagereDefinir(
            nom.clone(),
            if destination == "Sans étagère" {
                String::new()
            } else {
                destination
            },
        )
    })
    .placeholder("Déplacer vers…")
    .width(Length::Fill)
    .text_size(theme::jetons::DETAIL);

    let contenu = column![
        ouvrir,
        container(
            column![
                palette
                    .label("Étagère")
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
                classement,
            ]
            .spacing(4)
        )
        .padding([0, 12]),
        Space::new().height(12),
    ];
    mouse_area(
        container(contenu)
            .width(Length::Fixed(202.0))
            .style(move |_| {
                style_carte(
                    palette,
                    state.drag_projet.as_deref() == Some(projet.name.as_str()),
                )
            }),
    )
    .on_press(Message::DragCommencer(projet.name.clone()))
    .into()
}

/// La bibliothèque occupe toute la fenêtre. Les dialogues de création sont
/// composés par le shell au-dessus de cette page, et non derrière elle.
pub fn view(state: &State) -> Element<'_, Message> {
    if !state.projets_vue {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }
    let palette = state.resolved.palette;
    let classement = crate::project::etageres();
    let mut groupes: BTreeMap<String, Vec<&Project>> = BTreeMap::new();
    for projet in &state.projects {
        groupes
            .entry(classement.get(&projet.name).cloned().unwrap_or_default())
            .or_default()
            .push(projet);
    }
    // Même vide, la bibliothèque permet de commencer un premier projet.
    groupes.entry(String::new()).or_default();
    let mut choix = vec!["Sans étagère".to_string()];
    choix.extend(groupes.keys().filter(|nom| !nom.is_empty()).cloned());
    let brouillon = state.etagere_brouillon.trim();
    if !brouillon.is_empty() && !choix.iter().any(|nom| nom == brouillon) {
        choix.push(brouillon.to_string());
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
        let bandeau = row![
            palette.label(titre.to_string()).size(theme::jetons::TITRE),
            palette
                .label(format!("{compte} projet(s)"))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            Space::new().width(Length::Fill),
            button(
                palette
                    .label("+ Nouveau projet ici")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ProjetDepuisEtagere(etagere.clone()))
            .padding([8, 12]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        let mut rayon = column![bandeau].spacing(14);
        if projets.is_empty() {
            rayon = rayon.push(
                palette
                    .label("Votre prochain projet commence ici.")
                    .color(palette.text_muted),
            );
        }
        for rangee in projets.chunks(colonnes) {
            let mut cartes = row![].spacing(16);
            for projet in rangee {
                cartes = cartes.push(carte(state, projet, &etagere, &choix));
            }
            rayon = rayon.push(cartes);
        }
        let survolee =
            state.drag_projet.is_some() && state.drag_etagere.as_deref() == Some(etagere.as_str());
        rayons = rayons.push(
            mouse_area(
                container(rayon)
                    .padding(16)
                    .width(Length::Fill)
                    .style(move |_| style_carte(palette, survolee)),
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
    let nouvelle = row![
        palette
            .label("Nouvelle étagère")
            .size(theme::jetons::DETAIL),
        text_input("Ex. : Thèse, articles, cours…", &state.etagere_brouillon)
            .on_input(Message::EtagereBrouillon)
            .padding(10)
            .width(Length::Fixed(280.0)),
        button(
            palette
                .label("Créer un projet sur cette étagère")
                .size(theme::jetons::DETAIL)
        )
        .on_press_maybe(
            (!brouillon.is_empty()).then(|| Message::ProjetDepuisEtagere(brouillon.to_string()))
        )
        .padding([10, 12]),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    container(column![
        entete,
        nouvelle,
        palette.label("Ouvrez un projet par sa couverture. Pour le ranger, utilisez son menu Étagère ou déposez-le sur une autre étagère.")
            .size(theme::jetons::DETAIL).color(palette.text_muted),
        scrollable(rayons).height(Length::Fill),
    ].spacing(22)).padding(24).width(Length::Fill).height(Length::Fill)
        .style(move |_| container::Style { background: Some(palette.background.into()), ..Default::default() }).into()
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
