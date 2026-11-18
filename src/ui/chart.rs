//! Le panneau d'un graphique primaviz.
//!
//! Le type, les données CSV, la taille, le placement et les options de
//! composition : titre, sous-titre, axes, grille, courbe, anneau. Les données
//! restent dans le document, en CSV lisible ; le panneau n'a pas d'aperçu,
//! le document est la prévisualisation.

use crate::app::Message;
use crate::langage::chart;
use crate::ui::theme::{self, Palette};
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text_editor, text_input,
};
use iced::{Element, Length};

/// Les libellés des types, dans l'ordre de [`chart::TYPES`].
const TYPES: [&str; 7] = [
    "Secteurs (pie)",
    "Barres (bar)",
    "Lignes (line)",
    "Aires (area)",
    "Nuage de points (scatter)",
    "Boîtes à moustaches (box)",
    "Sucettes (lollipop)",
];

/// Les libellés des placements.
const PLACEMENTS: [&str; 4] = ["Centré", "À gauche", "À droite", "Pleine largeur"];

/// Les libellés des courbes.
const COURBES: [&str; 3] = ["Linéaire", "Lissée", "Catmull-Rom"];

/// Le libellé d'un type.
fn libelle_type(type_: &str) -> &'static str {
    match chart::normaliser_type(type_) {
        "pie" => TYPES[0],
        "line" => TYPES[2],
        "area" => TYPES[3],
        "scatter" => TYPES[4],
        "box" => TYPES[5],
        "lollipop" => TYPES[6],
        _ => TYPES[1],
    }
}

/// Le code d'un libellé de type.
fn code_type(libelle: &str) -> String {
    TYPES
        .iter()
        .position(|l| *l == libelle)
        .map(|rang| chart::TYPES[rang].to_string())
        .unwrap_or_else(|| "bar".to_string())
}

/// Le libellé d'un placement.
fn libelle_placement(placement: &str) -> &'static str {
    match placement {
        "gauche" => PLACEMENTS[1],
        "droite" => PLACEMENTS[2],
        "pleine-largeur" => PLACEMENTS[3],
        _ => PLACEMENTS[0],
    }
}

/// Le code d'un libellé de placement.
fn code_placement(libelle: &str) -> String {
    match libelle {
        "À gauche" => "gauche",
        "À droite" => "droite",
        "Pleine largeur" => "pleine-largeur",
        _ => "centre",
    }
    .to_string()
}

/// Le libellé d'une courbe.
fn libelle_courbe(courbe: &str) -> &'static str {
    match courbe {
        "smooth" => COURBES[1],
        "catmull-rom" => COURBES[2],
        _ => COURBES[0],
    }
}

/// Le code d'une courbe.
fn code_courbe(libelle: &str) -> String {
    match libelle {
        "Lissée" => "smooth",
        "Catmull-Rom" => "catmull-rom",
        _ => "linear",
    }
    .to_string()
}

/// La fenêtre — vide quand aucun graphique n'est en cours.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(dialogue) = state.chart.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette: Palette = state.resolved.palette;
    let type_ = chart::normaliser_type(&dialogue.type_);
    let est_pie = type_ == "pie";
    let est_barre = type_ == "bar";
    let est_ligne = type_ == "line";
    let est_aire = type_ == "area";
    let est_scatter = type_ == "scatter";
    let est_boite = type_ == "box";
    let a_des_axes = !est_pie;
    let a_un_sous_titre = est_pie || est_barre || est_ligne || est_scatter;
    let colonnes = chart::colonnes(type_).join(", ");
    let largeur = 560.0_f32.min(state.window_size.width - 32.0).max(340.0);
    let hauteur = 680.0_f32.min(state.window_size.height - 32.0).max(380.0);

    let mut carte = column![
        palette
            .label("Insérer un graphique")
            .size(theme::jetons::SAILLANT),
        palette
            .label("Les données restent dans le document, en CSV lisible.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row![
            palette.label("Type").size(theme::jetons::DETAIL),
            pick_list(
                TYPES.to_vec(),
                Some(libelle_type(type_)),
                |libelle: &'static str| Message::ChartType(code_type(libelle)),
            )
            .padding(4)
            .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        row![
            palette.label("Titre").size(theme::jetons::DETAIL),
            text_input("Titre du graphique", &dialogue.titre)
                .on_input(Message::ChartTitre)
                .padding([4, 8])
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    if a_un_sous_titre {
        carte = carte.push(
            row![
                palette.label("Sous-titre").size(theme::jetons::DETAIL),
                text_input("Sous-titre (facultatif)", &dialogue.sous_titre)
                    .on_input(Message::ChartSousTitre)
                    .padding([4, 8])
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }

    if a_des_axes {
        carte = carte.push(
            row![
                palette.label("Axe x").size(theme::jetons::DETAIL),
                text_input("Axe horizontal", &dialogue.x_label)
                    .on_input(Message::ChartXLabel)
                    .padding([4, 8])
                    .width(Length::Fill),
                palette.label("Axe y").size(theme::jetons::DETAIL),
                text_input("Axe vertical", &dialogue.y_label)
                    .on_input(Message::ChartYLabel)
                    .padding([4, 8])
                    .width(Length::Fill),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        );
    }

    carte = carte.push(
        palette
            .label(format!("Données CSV — colonnes : {colonnes}"))
            .size(theme::jetons::DETAIL),
    );
    carte = carte.push(
        text_editor(&dialogue.csv)
            .id(crate::app::state::ChartDialogue::id_csv())
            .placeholder("étiquette,valeur\nNord,30\nSud,25")
            .on_action(Message::ChartCsv)
            .height(Length::Fixed(170.0))
            .padding(8),
    );
    carte = carte.push(
        column![
            button(
                palette
                    .label("Importer un CSV…")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ChartImporter)
            .padding([5, 12]),
            palette
                .label("Un en-tête est reconnu et sauté ; virgule ou point-virgule.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(6),
    );

    carte = carte.push(
        row![
            palette.label("Largeur").size(theme::jetons::DETAIL),
            text_input("10cm", &dialogue.largeur)
                .on_input(Message::ChartLargeur)
                .padding([4, 8])
                .width(Length::Fixed(76.0)),
            palette.label("Hauteur").size(theme::jetons::DETAIL),
            text_input("6cm", &dialogue.hauteur)
                .on_input(Message::ChartHauteur)
                .padding([4, 8])
                .width(Length::Fixed(76.0)),
            palette.label("Placement").size(theme::jetons::DETAIL),
            pick_list(
                PLACEMENTS.to_vec(),
                Some(libelle_placement(&dialogue.placement)),
                |libelle: &'static str| Message::ChartPlacement(code_placement(libelle)),
            )
            .padding(4),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    );

    let mut options = row![].spacing(16);
    if est_pie {
        options = options.push(
            checkbox(dialogue.legende)
                .label("Légende")
                .on_toggle(Message::ChartLegende),
        );
    }
    if est_pie || est_barre || est_ligne || est_scatter || est_boite || type_ == "lollipop" {
        options = options.push(
            checkbox(dialogue.valeurs)
                .label(if est_pie { "Pourcentages" } else { "Valeurs" })
                .on_toggle(Message::ChartValeurs),
        );
    }
    if est_barre || est_ligne || est_aire || est_scatter || est_boite {
        options = options.push(
            checkbox(dialogue.grille)
                .label("Grille")
                .on_toggle(Message::ChartGrille),
        );
    }
    if est_barre || est_ligne || est_aire || est_scatter {
        options = options.push(
            checkbox(dialogue.graduations)
                .label("Graduations")
                .on_toggle(Message::ChartGraduations),
        );
    }
    if est_barre {
        options = options.push(
            checkbox(dialogue.horizontal)
                .label("Barres horizontales")
                .on_toggle(Message::ChartHorizontal),
        );
    }
    if est_ligne || est_aire {
        options = options.push(
            checkbox(dialogue.points)
                .label("Points")
                .on_toggle(Message::ChartPoints),
        );
    }
    if est_ligne {
        options = options.push(
            checkbox(dialogue.aire)
                .label("Aire remplie")
                .on_toggle(Message::ChartAire),
        );
        options = options.push(
            pick_list(
                COURBES.to_vec(),
                Some(libelle_courbe(&dialogue.courbe)),
                |libelle: &'static str| Message::ChartCourbe(code_courbe(libelle)),
            )
            .padding(4)
            .width(Length::Fixed(130.0)),
        );
    }
    if est_pie {
        options = options.push(
            checkbox(dialogue.donut)
                .label("Anneau")
                .on_toggle(Message::ChartDonut),
        );
        if dialogue.donut {
            options = options.push(
                row![
                    palette.label("Trou").size(theme::jetons::DETAIL),
                    text_input("40%", &dialogue.anneau)
                        .on_input(Message::ChartAnneau)
                        .padding([4, 8])
                        .width(Length::Fixed(72.0)),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            );
        }
    }
    carte = carte.push(options);
    carte = carte.push(
        palette
            .label("Le panneau écrit les options dans la balise ; le CSV reste tel que collé.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    carte = carte.push(
        row![
            button(
                palette
                    .label("Insérer le graphique")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ChartInserer)
            .padding([6, 14]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::ChartFermer)
                .padding([6, 14]),
        ]
        .spacing(8),
    );

    let corps = scrollable(carte).height(Length::Fixed((hauteur - 72.0).max(280.0)));
    let carte = container(corps)
        .padding(18)
        .style(theme::panneau(palette))
        .width(Length::Fixed(largeur))
        .height(Length::Fixed(hauteur));

    crate::ui::voile(
        container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
