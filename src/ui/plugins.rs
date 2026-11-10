//! Les paramètres des plugins, séparés des normes du document.
//!
//! Les normes **activent** un plugin ; ce panneau le **configure**. La
//! bibliographie y trouve son style, sa forme de citation et son placement,
//! le plugin biblique sa forme de livre et ses chiffres.

use iced::widget::{
    Space, button, checkbox, column, container, row, scrollable, text, text_input, tooltip,
};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::ui::theme::{self, Palette};

/// Les onglets des plugins.
const ONGLETS: [&str; 4] = ["Bibliographie", "Biblique", "Glossaire", "Graphiques"];

/// La fenêtre des paramètres d'un plugin.
///
/// Même fenêtre que les normes : même largeur, même hauteur, même rail,
/// même pied. Seul le contenu change — c'est la règle de l'harmonisation.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(dialogue) = state.mise_en_page.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };
    let palette = state.resolved.palette;
    let onglet = dialogue.panneau.min(ONGLETS.len() - 1);

    let mut rail = column![].spacing(2);
    for (indice, nom) in ONGLETS.iter().enumerate() {
        let pictogramme = theme::icon(
            match indice {
                0 => lucide_icons::iced::icon_book_open::<iced::Theme>(),
                1 => lucide_icons::iced::icon_book_marked::<iced::Theme>(),
                2 => lucide_icons::iced::icon_book_a::<iced::Theme>(),
                _ => lucide_icons::iced::icon_chart_line::<iced::Theme>(),
            },
            15.0,
            palette.text,
        );
        rail = rail.push(crate::ui::reglages::onglet_vertical(
            palette,
            pictogramme,
            nom,
            indice,
            onglet,
        ));
    }
    let rail = container(rail).padding([4, 6]);

    let corps = match onglet {
        0 => crate::ui::reglages::corps_bibliographie(palette, dialogue),
        1 => corps_biblique(palette, dialogue),
        2 => explication(
            palette,
            "Le glossaire se gère dans son panneau : ses termes, leurs formes et leurs citations.",
        ),
        _ => explication(
            palette,
            "Les graphiques se règlent dans la fenêtre d'insertion, avec leurs données.",
        ),
    };

    let entete = row![
        palette.label("Paramètres des plugins").size(16),
        Space::new().width(Length::Fill),
        palette
            .label("Les normes activent ; ce panneau configure.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let pied = row![
        button(text("Annuler").size(12))
            .on_press(Message::MiseEnPageFermer)
            .padding([8, 12]),
        Space::new().width(Length::Fill),
        button(text("Appliquer").size(12))
            .on_press(Message::MiseEnPageAppliquer)
            .padding([8, 12]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let largeur = 860.0_f32.min(state.window_size.width - 32.0).max(340.0);
    let hauteur = 660.0_f32.min(state.window_size.height - 32.0).max(340.0);

    let vu = row![
        rail.width(Length::FillPortion(1)),
        scrollable(container(corps).padding([2, 10]))
            .height(Length::Fill)
            .width(Length::FillPortion(5)),
    ]
    .spacing(4)
    .height(Length::Fill);

    let fenetre = container(column![entete, vu, pied].spacing(12))
        .padding(18)
        .style(crate::ui::editor::bubble_style(palette))
        .width(Length::Fixed(largeur))
        .height(Length::Fixed(hauteur));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

/// L'onglet biblique : la forme d'écriture et les chiffres.
fn corps_biblique<'a>(
    palette: Palette,
    dialogue: &'a crate::app::state::DialogueMiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let reglages = &dialogue.reglages;
    let forme = match reglages.references.livre {
        crate::mise_en_page::modele::SystemeAbreviation::FrancaiseCourte => "Abréviation",
        crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue => "Nom complet",
        crate::mise_en_page::modele::SystemeAbreviation::Osis => "OSIS",
        crate::mise_en_page::modele::SystemeAbreviation::SansPoint => "Sans point",
    };
    let chiffres = match reglages.references.chiffres {
        crate::mise_en_page::modele::Chiffres::Arabes => "Arabes",
        crate::mise_en_page::modele::Chiffres::Romains => "Romains",
    };

    let mut corps = column![
        palette
            .label("Les références bibliques s'écrivent selon ces choix par défaut.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        iced::widget::pick_list(
            vec!["Abréviation", "Nom complet", "OSIS", "Sans point"],
            Some(forme),
            |choisi| Message::MiseEnPageChamp(
                crate::mise_en_page::modele::Champ::ReferencesLivre,
                match choisi {
                    "Nom complet" => "longue",
                    "OSIS" => "osis",
                    "Sans point" => "sans_point",
                    _ => "courte",
                }
                .to_string(),
            )
        )
        .width(Length::Fixed(190.0)),
        palette
            .label("Une balise peut choisir sa forme : {{Jn 3.16|long}}.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        iced::widget::pick_list(vec!["Arabes", "Romains"], Some(chiffres), |choisi| {
            Message::MiseEnPageChamp(
                crate::mise_en_page::modele::Champ::ReferencesChiffres,
                if choisi == "Romains" {
                    "romains"
                } else {
                    "arabes"
                }
                .to_string(),
            )
        })
        .width(Length::Fixed(190.0)),
        checkbox(reglages.references.ordre_canonique)
            .label("Ranger les références dans l'ordre canonique")
            .on_toggle(|actif| {
                Message::MiseEnPageChamp(
                    crate::mise_en_page::modele::Champ::ReferencesOrdreCanonique,
                    actif.to_string(),
                )
            })
            .size(theme::jetons::CORPS),
        palette
            .label("Le même livre n'est nommé qu'une fois : ses occurrences sont séparées par une virgule.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(10);

    corps = corps.push(
        palette
            .label("Séparateurs — les espaces comptent.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    corps = corps.push(champ_separateur(
        palette,
        "Chapitre / verset",
        &reglages.references.separateur_chapitre_verset,
        crate::mise_en_page::modele::Champ::ReferencesSeparateurChapitreVerset,
    ));
    corps = corps.push(champ_separateur(
        palette,
        "Plage de versets",
        &reglages.references.separateur_plage,
        crate::mise_en_page::modele::Champ::ReferencesSeparateurPlage,
    ));
    corps = corps.push(champ_separateur(
        palette,
        "Entre deux livres",
        &reglages.references.separateur_livre,
        crate::mise_en_page::modele::Champ::ReferencesSeparateurLivre,
    ));
    corps = corps.push(champ_separateur(
        palette,
        "Entre deux occurrences",
        &reglages.references.separateur_occurrence,
        crate::mise_en_page::modele::Champ::ReferencesSeparateurOccurrence,
    ));
    corps = corps.push(champ_separateur(
        palette,
        "Entre deux chapitres entiers",
        &reglages.references.separateur_chapitres,
        crate::mise_en_page::modele::Champ::ReferencesSeparateurChapitres,
    ));
    corps = corps.push(
        palette
            .label("Gabarit d'une référence")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    corps = corps.push(
        text_input("{livre} {nombres}", &reglages.references.gabarit)
            .on_input(|gabarit| {
                Message::MiseEnPageChamp(
                    crate::mise_en_page::modele::Champ::ReferencesGabarit,
                    gabarit,
                )
            })
            .padding([4, 6])
            .width(Length::Fill),
    );
    corps = corps.push(
        palette
            .label("Survolez un jeton pour connaître ce qu'il produit :")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    corps = corps.push(jetons_gabarit(palette));
    corps = corps.push(
        palette
            .label(
                "Une traduction se précise dans la citation : {{Jn 3.16|lsg}} ou \
                 {{Jn 3.16|darby}}. Sans précision, aucune mention n'est ajoutée.",
            )
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    corps = corps.push(
        palette
            .label("Index des citations")
            .size(theme::jetons::SAILLANT),
    );
    corps = corps.push(
        checkbox(reglages.index.afficher_pages)
            .label("Montrer les pages où chaque référence apparaît")
            .on_toggle(|actif| {
                Message::MiseEnPageChamp(
                    crate::mise_en_page::modele::Champ::IndexAfficherPages,
                    actif.to_string(),
                )
            })
            .size(theme::jetons::CORPS),
    );
    corps = corps.push(
        checkbox(reglages.index.regrouper_par_livre)
            .label("Regrouper les références sous leur livre")
            .on_toggle(|actif| {
                Message::MiseEnPageChamp(
                    crate::mise_en_page::modele::Champ::IndexRegrouperParLivre,
                    actif.to_string(),
                )
            })
            .size(theme::jetons::CORPS),
    );
    let forme_index = match reglages.index.forme {
        None => "Héritée des références",
        Some(crate::mise_en_page::modele::SystemeAbreviation::FrancaiseCourte) => "Abrégée",
        Some(crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue) => "Longue",
        Some(crate::mise_en_page::modele::SystemeAbreviation::Osis) => "OSIS",
        Some(crate::mise_en_page::modele::SystemeAbreviation::SansPoint) => "Sans point",
    };
    corps = corps.push(
        row![
            palette
                .label("Forme de l'index")
                .size(theme::jetons::DETAIL)
                .width(Length::Fill),
            iced::widget::pick_list(
                vec![
                    "Héritée des références",
                    "Abrégée",
                    "Longue",
                    "OSIS",
                    "Sans point",
                ],
                Some(forme_index),
                |choisi| {
                    Message::MiseEnPageChamp(
                        crate::mise_en_page::modele::Champ::IndexForme,
                        match choisi {
                            "Abrégée" => "courte",
                            "Longue" => "longue",
                            "OSIS" => "osis",
                            "Sans point" => "sans_point",
                            _ => "herite",
                        }
                        .to_string(),
                    )
                },
            )
            .width(Length::Fixed(220.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );
    let colonnes = match reglages.index.colonnes.clamp(1, 4) {
        4 => "4 colonnes",
        3 => "3 colonnes",
        2 => "2 colonnes",
        _ => "1 colonne",
    };
    corps = corps.push(
        row![
            palette
                .label("Colonnes de l'index")
                .size(theme::jetons::DETAIL)
                .width(Length::Fill),
            iced::widget::pick_list(
                vec!["1 colonne", "2 colonnes", "3 colonnes", "4 colonnes"],
                Some(colonnes),
                |choisi| {
                    let nombre = choisi.split_whitespace().next().unwrap_or("1");
                    Message::MiseEnPageChamp(
                        crate::mise_en_page::modele::Champ::IndexColonnes,
                        nombre.to_string(),
                    )
                },
            )
            .width(Length::Fixed(160.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );
    corps
}

/// Les neuf jetons du gabarit, chacun avec son explication au survol.
fn jetons_gabarit(palette: Palette) -> Element<'static, Message> {
    const JETONS: [(&str, &str); 9] = [
        (
            "{livre}",
            "Le nom du livre, dans la forme choisie plus haut.",
        ),
        (
            "{chapitres}",
            "Le chapitre, ou la suite de chapitres, sans le livre.",
        ),
        (
            "{versets}",
            "Le verset, ou la plage de versets, sans le chapitre.",
        ),
        (
            "{nombres}",
            "Le chapitre et les versets ensemble, avec les séparateurs.",
        ),
        ("{chapitre}", "Le premier numéro de chapitre."),
        ("{chapitre_fin}", "Le dernier chapitre d'une suite."),
        ("{verset}", "Le premier verset de la plage."),
        ("{verset_fin}", "Le dernier verset de la plage."),
        (
            "{reference}",
            "Le livre et les nombres, dans la forme standard.",
        ),
    ];

    let mut grille = column![].spacing(4);
    for ligne in JETONS.chunks(4) {
        let mut rang = row![].spacing(14);
        for (jeton, aide) in ligne {
            rang = rang.push(jeton_aide(palette, jeton, aide));
        }
        grille = grille.push(rang);
    }
    grille.into()
}

/// Un jeton du gabarit, avec son explication dans une bulle.
fn jeton_aide<'a>(palette: Palette, jeton: &'a str, aide: &'a str) -> Element<'a, Message> {
    let bulle = container(
        palette
            .label(aide)
            .size(theme::jetons::DETAIL)
            .color(palette.text),
    )
    .padding([4, 8])
    .style(move |_| container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    tooltip(
        palette
            .label(jeton)
            .size(theme::jetons::DETAIL)
            .color(palette.text),
        bulle,
        tooltip::Position::Top,
    )
    .gap(6)
    .into()
}

/// Une ligne de séparateur : son nom à gauche, sa valeur modifiable à droite.
fn champ_separateur<'a>(
    palette: Palette,
    libelle: &'a str,
    valeur: &'a str,
    champ: crate::mise_en_page::modele::Champ,
) -> Element<'a, Message> {
    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fill),
        text_input("", valeur)
            .on_input(move |valeur| Message::MiseEnPageChamp(champ, valeur))
            .padding([4, 6])
            .width(Length::Fixed(110.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Un onglet sans réglage : il le dit, plutôt que de rester muet.
fn explication<'a>(palette: Palette, texte: &'a str) -> iced::widget::Column<'a, Message> {
    column![
        palette
            .label(texte)
            .size(theme::jetons::CORPS)
            .color(palette.text_muted)
    ]
    .spacing(8)
}
