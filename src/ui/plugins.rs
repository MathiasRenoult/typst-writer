//! Les paramètres des plugins, séparés des normes du document.
//!
//! Les normes **activent** un plugin ; ce panneau le **configure**. La
//! bibliographie y trouve son style, sa forme de citation et son placement,
//! le plugin biblique sa forme de livre et ses chiffres.

use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
    tooltip,
};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::ui::theme::{self, Palette};

/// Les onglets des plugins.
const ONGLETS: [&str; 5] = [
    "Bibliographie",
    "Biblique",
    "Glossaire",
    "Graphiques",
    "Callouts",
];

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
                3 => lucide_icons::iced::icon_chart_line::<iced::Theme>(),
                _ => lucide_icons::iced::icon_text_quote::<iced::Theme>(),
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
        3 => corps_graphiques(palette),
        _ => corps_callouts(palette, dialogue),
    };

    let entete = row![
        palette.label("Paramètres des plugins").size(16),
        Space::new().width(Length::Fill),
        palette
            .label("Le fichier de normes règle aussi ces paramètres.")
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
        crate::mise_en_page::modele::SystemeAbreviation::Tob => "TOB",
    };
    let chiffres = match reglages.references.chiffres {
        crate::mise_en_page::modele::Chiffres::Arabes => "Arabes",
        crate::mise_en_page::modele::Chiffres::Romains => "Romains",
    };
    let traduction = match reglages.parametres_plugins.biblique.traduction.as_str() {
        "darby" => "J.N. Darby",
        _ => "Louis Segond 1910",
    };
    let tri = match reglages.parametres_plugins.biblique.tri.as_str() {
        "canonique" => "Ordre canonique",
        _ => "Ordre d'apparition",
    };

    let mut corps = column![
        palette
            .label("Les références bibliques s'écrivent selon ces choix par défaut.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row![
            palette
                .label("Traduction consultée")
                .size(theme::jetons::DETAIL)
                .width(Length::Fill),
            iced::widget::pick_list(
                vec!["Louis Segond 1910", "J.N. Darby"],
                Some(traduction),
                |choisi| {
                    Message::MiseEnPageChamp(
                        crate::mise_en_page::modele::Champ::BibliqueTraduction,
                        if choisi == "J.N. Darby" {
                            "darby"
                        } else {
                            "lsg"
                        }
                        .to_string(),
                    )
                },
            )
            .width(Length::Fixed(190.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            palette
                .label("Ordre de la liste")
                .size(theme::jetons::DETAIL)
                .width(Length::Fill),
            iced::widget::pick_list(
                vec!["Ordre d'apparition", "Ordre canonique"],
                Some(tri),
                |choisi| {
                    Message::MiseEnPageChamp(
                        crate::mise_en_page::modele::Champ::BibliqueTri,
                        if choisi == "Ordre canonique" {
                            "canonique"
                        } else {
                            "apparition"
                        }
                        .to_string(),
                    )
                },
            )
            .width(Length::Fixed(190.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        palette
            .label("Ces paramètres voyagent dans le fichier de normes du document.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        iced::widget::pick_list(
            vec!["Abréviation", "Nom complet", "OSIS", "Sans point", "TOB"],
            Some(forme),
            |choisi| Message::MiseEnPageChamp(
                crate::mise_en_page::modele::Champ::ReferencesLivre,
                match choisi {
                    "Nom complet" => "longue",
                    "OSIS" => "osis",
                    "Sans point" => "sans_point",
                    "TOB" => "tob",
                    _ => "courte",
                }
                .to_string(),
            )
        )
        .width(Length::Fixed(190.0)),
        palette
            .label("Une balise peut choisir sa forme : {{Jn 3.16|long}} ou {{1 S 1.1|tob}}.")
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
        Some(crate::mise_en_page::modele::SystemeAbreviation::Tob) => "TOB",
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
                    "TOB",
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
                            "TOB" => "tob",
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

/// L'onglet des graphiques : ce que le panneau d'insertion expose.
fn corps_graphiques(palette: Palette) -> iced::widget::Column<'static, Message> {
    column![
        palette
            .label("Le panneau d'insertion choisit le type et écrit la balise `{chart …}`.")
            .size(theme::jetons::CORPS),
        palette
            .label(
                "Types : secteurs, barres, lignes, aires, nuage de points, boîtes à \
                 moustaches et sucettes. Options : titre, sous-titre, axes, grille, \
                 graduations, points, aire remplie, courbe, anneau et barres horizontales."
            )
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        palette
            .label(
                "Les données restent dans le document, en CSV ; les options sont des \
                 propriétés explicites de la balise. Aucun aperçu n'est nécessaire : \
                 le document composé est la prévisualisation."
            )
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(10)
}

/// L'onglet des callouts : les préréglages vivent dans la norme, le panneau
/// d'insertion les choisit et peut en écarter ponctuellement le style.
fn corps_callouts<'a>(
    palette: Palette,
    dialogue: &'a crate::app::state::DialogueMiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let presets = &dialogue.reglages.parametres_plugins.callouts.presets;
    if presets.is_empty() {
        return column![
            palette
                .label("Aucun callout prédéfini dans cette norme.")
                .size(theme::jetons::CORPS),
            button(
                palette
                    .label("Créer un préréglage")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::CalloutPresetAjouter)
            .padding([6, 12]),
        ]
        .spacing(10);
    }

    let index = dialogue.callout_preset.min(presets.len() - 1);
    let preset = &presets[index];
    let libelles: Vec<String> = presets.iter().map(|p| p.nom.clone()).collect();
    let mut corps = column![
        palette
            .label("Callouts prédéfinis")
            .size(theme::jetons::SAILLANT),
        palette
            .label(
                "La source nomme un préréglage : `{callout attention \"Titre\"}`. \
                 Le panneau d'insertion peut ensuite changer son apparence pour une \
                 occurrence ; ces écarts restent dans la source."
            )
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        row![
            pick_list(libelles, Some(preset.nom.clone()), move |choisi: String| {
                Message::CalloutPresetChoisir(
                    presets.iter().position(|p| p.nom == choisi).unwrap_or(0),
                )
            })
            .padding(4)
            .width(Length::Fill),
            button(palette.label("Nouveau").size(theme::jetons::DETAIL))
                .on_press(Message::CalloutPresetAjouter)
                .padding([4, 8]),
            button(palette.label("Supprimer").size(theme::jetons::DETAIL))
                .on_press(Message::CalloutPresetSupprimer(index))
                .padding([4, 8]),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        palette
            .label(format!("Clé dans la source : {}", preset.cle))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        champ_preset(
            palette,
            "Nom affiché",
            index,
            crate::langage::callout::CalloutChamp::Nom,
            &preset.nom
        ),
        champ_preset(
            palette,
            "Titre proposé",
            index,
            crate::langage::callout::CalloutChamp::Titre,
            &preset.titre
        ),
        palette.label("Couleurs").size(theme::jetons::SAILLANT),
        couleur_preset(
            palette,
            "Bordure",
            index,
            crate::langage::callout::CalloutChamp::Bordure,
            &preset.bordure
        ),
        couleur_preset(
            palette,
            "Fond du corps",
            index,
            crate::langage::callout::CalloutChamp::Fond,
            &preset.fond
        ),
        couleur_preset(
            palette,
            "Fond du titre",
            index,
            crate::langage::callout::CalloutChamp::TitreFond,
            &preset.titre_fond
        ),
        couleur_preset(
            palette,
            "Texte du titre",
            index,
            crate::langage::callout::CalloutChamp::TitreTexte,
            &preset.titre_texte
        ),
        couleur_preset(
            palette,
            "Texte du corps",
            index,
            crate::langage::callout::CalloutChamp::CorpsTexte,
            &preset.corps_texte
        ),
        palette.label("Mesures").size(theme::jetons::SAILLANT),
        row![
            champ_preset(
                palette,
                "Épaisseur",
                index,
                crate::langage::callout::CalloutChamp::Epaisseur,
                &preset.epaisseur
            ),
            champ_preset(
                palette,
                "Rayon",
                index,
                crate::langage::callout::CalloutChamp::Rayon,
                &preset.rayon
            ),
        ]
        .spacing(8),
        row![
            champ_preset(
                palette,
                "Marge x",
                index,
                crate::langage::callout::CalloutChamp::MargeX,
                &preset.marge_x
            ),
            champ_preset(
                palette,
                "Marge y",
                index,
                crate::langage::callout::CalloutChamp::MargeY,
                &preset.marge_y
            ),
            champ_preset(
                palette,
                "Largeur",
                index,
                crate::langage::callout::CalloutChamp::Largeur,
                &preset.largeur
            ),
        ]
        .spacing(8),
        row![
            liste_preset(palette, "Bordure", preset, index),
            liste_alignement_preset(palette, preset, index),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            checkbox(preset.gras)
                .label("Titre en gras")
                .on_toggle(move |actif| {
                    Message::CalloutPresetBascule(
                        index,
                        crate::langage::callout::CalloutChamp::Gras,
                        actif,
                    )
                })
                .size(theme::jetons::CORPS),
            checkbox(preset.rupture)
                .label("Coupure entre pages")
                .on_toggle(move |actif| {
                    Message::CalloutPresetBascule(
                        index,
                        crate::langage::callout::CalloutChamp::Rupture,
                        actif,
                    )
                })
                .size(theme::jetons::CORPS),
        ]
        .spacing(16),
        palette
            .label("Ces réglages sont enregistrés avec la norme, pas dans l'application.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(10);

    corps = corps.push(
        palette
            .label("Le panneau d'insertion part de ce préréglage et n'écrit que les écarts.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    corps
}

/// Un champ texte d'un préréglage.
fn champ_preset<'a>(
    palette: Palette,
    libelle: &'a str,
    index: usize,
    champ: crate::langage::callout::CalloutChamp,
    valeur: &'a str,
) -> Element<'a, Message> {
    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fixed(110.0)),
        text_input("", valeur)
            .on_input(move |valeur| Message::CalloutPresetChamp(index, champ, valeur))
            .padding([4, 6])
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Une couleur d'un préréglage, avec la roue.
fn couleur_preset<'a>(
    palette: Palette,
    libelle: &'a str,
    index: usize,
    champ: crate::langage::callout::CalloutChamp,
    valeur: &'a str,
) -> Element<'a, Message> {
    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fixed(110.0)),
        text_input("#000000", valeur)
            .on_input(move |valeur| Message::CalloutPresetChamp(index, champ, valeur))
            .padding([4, 6])
            .width(Length::Fixed(110.0)),
        button(palette.label("Roue…").size(theme::jetons::DETAIL))
            .on_press(Message::PickerCouleurPourCallout(
                crate::app::state::CibleCouleur::Preset(index, champ)
            ))
            .padding([3, 7]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// Le style de bordure d'un préréglage.
fn liste_preset<'a>(
    palette: Palette,
    libelle: &'a str,
    preset: &crate::langage::callout::CalloutPreset,
    index: usize,
) -> Element<'a, Message> {
    let courant = match preset.bordure_style.as_str() {
        "dashed" => "Tirets",
        "dotted" => "Pointillés",
        _ => "Trait plein",
    };
    row![
        palette.label(libelle).size(theme::jetons::DETAIL),
        pick_list(
            vec!["Trait plein", "Tirets", "Pointillés"],
            Some(courant),
            move |choisi: &'static str| Message::CalloutPresetChamp(
                index,
                crate::langage::callout::CalloutChamp::BordureStyle,
                match choisi {
                    "Tirets" => "dashed",
                    "Pointillés" => "dotted",
                    _ => "solid",
                }
                .to_string(),
            )
        )
        .padding(4)
        .width(Length::Fixed(130.0)),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// L'alignement du titre d'un préréglage.
fn liste_alignement_preset<'a>(
    palette: Palette,
    preset: &crate::langage::callout::CalloutPreset,
    index: usize,
) -> Element<'a, Message> {
    let courant = match preset.alignement.as_str() {
        "center" => "Centre",
        "right" => "Droite",
        _ => "Gauche",
    };
    row![
        palette.label("Alignement").size(theme::jetons::DETAIL),
        pick_list(
            vec!["Gauche", "Centre", "Droite"],
            Some(courant),
            move |choisi: &'static str| Message::CalloutPresetChamp(
                index,
                crate::langage::callout::CalloutChamp::Alignement,
                match choisi {
                    "Centre" => "center",
                    "Droite" => "right",
                    _ => "left",
                }
                .to_string(),
            )
        )
        .padding(4)
        .width(Length::Fixed(130.0)),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
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
