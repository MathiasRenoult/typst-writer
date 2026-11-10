//! Le panneau de construction d'une citation biblique.
//!
//! Un livre, un chapitre, des versets : la référence s'ajoute à la liste,
//! puis les références accumulées s'insèrent ensemble dans **une seule**
//! balise `{{…}}`. La forme choisie ici ne touche pas aux normes du projet.

use crate::app::Message;
use crate::app::state::State;
use crate::plugins::bible::{
    BiblePanneau, FormeLivre, LivreConnu, Traduction, afficher, afficher_groupe_forme, livres,
};
use crate::ui::editor::bubble_style;
use crate::ui::theme;
use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length};

/// L'identifiant du premier champ du panneau.
pub fn id_champ() -> iced::widget::Id {
    iced::widget::Id::new("bible-panneau-champ")
}

/// L'étiquette d'un livre dans la liste : son nom et son abréviation française.
fn libelle_livre(livre: &LivreConnu) -> String {
    let abreviation =
        crate::plugins::bible::abreviation_francaise(&livre.osis).unwrap_or(&livre.abreviation);
    format!("{} — {}", livre.nom, abreviation)
}

/// La fenêtre du panneau d'insertion.
pub fn view<'a>(state: &'a State, panneau: &'a BiblePanneau) -> Element<'a, Message> {
    let palette = state.resolved.palette;
    let tous_les_livres = livres(Traduction::Lsg);
    let filtre = panneau.recherche.to_lowercase();
    let options: Vec<String> = tous_les_livres
        .iter()
        .filter(|livre| {
            filtre.is_empty()
                || livre.nom.to_lowercase().contains(&filtre)
                || livre.abreviation.to_lowercase().contains(&filtre)
                || crate::plugins::bible::abreviation_francaise(&livre.osis)
                    .is_some_and(|abreviation| abreviation.to_lowercase().contains(&filtre))
        })
        .map(libelle_livre)
        .collect();
    let courant = panneau
        .livre
        .and_then(|index| tous_les_livres.get(index))
        .map(libelle_livre);

    let entete = row![
        palette
            .label("Insérer une référence biblique")
            .size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label("Échap pour fermer")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let recherche = text_input("Chercher un livre", &panneau.recherche)
        .id(id_champ())
        .on_input(Message::BiblePanneauRecherche)
        .padding(6);

    let choix_livre = pick_list(options, courant, move |choix| {
        let index = tous_les_livres
            .iter()
            .position(|livre| libelle_livre(livre) == choix)
            .unwrap_or(0);
        Message::BiblePanneauLivre(index)
    })
    .width(Length::Fill)
    .placeholder("Choisir un livre");

    let champs = row![
        text_input("Chapitre (1-11)", &panneau.chapitre)
            .on_input(Message::BiblePanneauChapitre)
            .on_submit(Message::BiblePanneauAjouter)
            .padding(6)
            .width(Length::Fixed(110.0)),
        text_input("Versets : 16, 17-18", &panneau.versets)
            .on_input(Message::BiblePanneauVersets)
            .on_submit(Message::BiblePanneauAjouter)
            .padding(6)
            .width(Length::Fill),
        button(palette.label("Ajouter").size(theme::jetons::CORPS))
            .on_press(Message::BiblePanneauAjouter)
            .padding([6, 12]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut references = column![
        palette
            .label("Références de cette citation")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted)
    ]
    .spacing(4);

    if panneau.references.is_empty() {
        references = references.push(
            palette
                .label("Aucune référence ajoutée.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    } else {
        for (index, reference) in panneau.references.iter().enumerate() {
            references = references.push(
                row![
                    palette
                        .label(afficher(reference, &state.reglages.references))
                        .size(theme::jetons::CORPS)
                        .width(Length::Fill),
                    button(
                        palette
                            .label("Retirer")
                            .size(theme::jetons::DETAIL)
                            .color(palette.text_muted)
                    )
                    .on_press(Message::BiblePanneauRetirer(index))
                    .padding([2, 6]),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            );
        }
    }

    let references = scrollable(references).height(Length::Fixed(140.0));
    let texte_libre = text_input(
        "Texte libre (les guillemets sont ajoutés)",
        &panneau.texte_libre,
    )
    .on_input(Message::BiblePanneauTexte)
    .padding(6);

    let options_forme = [
        "Hérité du document",
        "Long",
        "Abrégé avec points",
        "Abrégé sans points",
        "OSIS",
    ];
    let forme_courante = match panneau.forme {
        None => "Hérité du document",
        Some(FormeLivre::FrancaiseLongue) => "Long",
        Some(FormeLivre::FrancaiseCourte) => "Abrégé avec points",
        Some(FormeLivre::SansPoint) => "Abrégé sans points",
        Some(FormeLivre::Osis) => "OSIS",
    };
    let forme = pick_list(options_forme, Some(forme_courante), |choix| {
        Message::BiblePanneauForme(match choix {
            "Long" => Some(FormeLivre::FrancaiseLongue),
            "Abrégé avec points" => Some(FormeLivre::FrancaiseCourte),
            "Abrégé sans points" => Some(FormeLivre::SansPoint),
            "OSIS" => Some(FormeLivre::Osis),
            _ => None,
        })
    })
    .width(Length::Fixed(190.0));

    let apercu = if panneau.references.is_empty() {
        String::new()
    } else {
        let corps = match panneau.forme {
            Some(forme) => {
                afficher_groupe_forme(&panneau.references, &state.reglages.references, Some(forme))
            }
            None => crate::plugins::bible::afficher_groupe(
                &panneau.references,
                &state.reglages.references,
            ),
        };
        let mut corps = corps;
        if !panneau.texte_libre.trim().is_empty() {
            if !corps.is_empty() {
                corps.push_str(&state.reglages.references.separateur_livre);
            }
            corps.push_str(&crate::langage::references::texte_libre_source(
                &panneau.texte_libre,
            ));
        }
        format!("{{{{{corps}}}}}")
    };

    let pied = row![
        palette
            .label("Tab · Entrée pour ajouter · Échap")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().width(Length::Fill),
        button(palette.label("Fermer").size(theme::jetons::CORPS))
            .on_press(Message::BiblePanneauFermer)
            .padding([6, 12]),
        button(
            palette
                .label("Insérer la citation")
                .size(theme::jetons::CORPS)
        )
        .on_press(Message::BiblePanneauInserer)
        .padding([6, 12]),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let apercu_widget: Element<'_, Message> = if apercu.is_empty() {
        column![].into()
    } else {
        container(text(apercu).size(theme::jetons::DETAIL))
            .padding(6)
            .style(move |_| container::Style {
                background: Some(palette.surface_hover.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    };

    let fenetre = container(
        column![
            entete,
            recherche,
            choix_livre,
            champs,
            references,
            texte_libre,
            row![
                palette
                    .label("Forme d'affichage")
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
                Space::new().width(Length::Fill),
                forme,
            ]
            .align_y(Alignment::Center),
            apercu_widget,
            pied,
        ]
        .spacing(10),
    )
    .width(Length::Fixed(560.0))
    .padding(18)
    .style(bubble_style(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
