//! La fenêtre de mise en page du document.
//!
//! Une fenêtre flottante, assez grande pour tout montrer d'un regard : le
//! papier, les marges, le texte, les titres, les numéros, les couleurs. Ce
//! qu'elle montre est une copie de travail — seul « Appliquer » touche le
//! projet, et les valeurs fautives sont dites, champ par champ, avant que
//! quoi que ce soit ne soit écrit.

use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::mise_en_page::modele::Champ;
use crate::ui::editor::bubble_style;
use crate::ui::theme::{self, Palette};

const FORMATS: &[&str] = &["a3", "a4", "a5", "a6", "b4", "b5", "letter", "legal"];
const ORIENTATIONS: &[&str] = &["portrait", "paysage"];
const FORMES: &[&str] = &["1", "I", "i", "A", "a"];
const POSITIONS: &[&str] = &["haut", "bas"];
const ALIGNEMENTS: &[&str] = &["gauche", "centre", "droite"];
const CASSES: &[&str] = &["normale", "majuscules", "minuscules"];
const LANGUES: &[&str] = &["fr", "en", "de", "es", "it", "pt", "nl", "la"];
const NUMEROTATIONS: &[&str] = &["1.1", "1.1.1", "I.i", "A.a", "1)", "(1)", "A.", "a."];

/// La fenêtre, ou sa place vide.
///
/// Sa place est **toujours** dans la page : la forme de l'arbre de widgets ne
/// change jamais, et rien ne s'y déplace quand la fenêtre s'ouvre.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(dialogue) = state.mise_en_page.as_ref() else {
        return container(Space::new()).into();
    };

    let palette = state.resolved.palette;
    let reglages = &dialogue.reglages;

    let entete = row![
        palette.label("Mise en page du document").size(16),
        Space::new().width(Length::Fill),
        button(text("Fermer").size(12))
            .on_press(Message::MiseEnPageFermer)
            .padding([6, 10]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut corps = column![].spacing(20);

    // ---- La page ----
    corps = corps.push(section(
        palette,
        "Page",
        column![
            ligne(
                palette,
                "Format",
                "Le papier de composition : a4, letter, a5…",
                choix(
                    FORMATS,
                    reglages.papier.format.as_str(),
                    Champ::PapierFormat
                )
            ),
            ligne(
                palette,
                "Orientation",
                "Portrait, ou paysage.",
                choix(
                    ORIENTATIONS,
                    reglages.papier.orientation.as_str(),
                    Champ::PapierOrientation
                )
            ),
            ligne(
                palette,
                "Marges (haut, bas)",
                "La marge verticale. Toute mesure Typst : 2cm, 1in, 24pt.",
                row![
                    saisie(palette, &reglages.marges.haut, Champ::MargeHaut, "2cm"),
                    saisie(palette, &reglages.marges.bas, Champ::MargeBas, "2cm"),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Marges (gauche, droite)",
                "La marge horizontale. Toute mesure Typst : 2cm, 1in, 24pt.",
                row![
                    saisie(palette, &reglages.marges.gauche, Champ::MargeGauche, "2cm"),
                    saisie(palette, &reglages.marges.droite, Champ::MargeDroite, "2cm"),
                ]
                .spacing(8)
            ),
        ]
        .spacing(10),
    ));

    // ---- Le texte ----
    corps = corps.push(section(
        palette,
        "Texte",
        column![
            ligne(
                palette,
                "Police",
                "La police de composition, par son nom système.",
                saisie(
                    palette,
                    &reglages.texte.police,
                    Champ::TextePolice,
                    "Nom de la police"
                )
            ),
            ligne(
                palette,
                "Corps, langue",
                "Le corps du texte (11pt) et la langue de césure (fr).",
                row![
                    saisie(palette, &reglages.texte.taille, Champ::TexteTaille, "11pt"),
                    choix(LANGUES, reglages.texte.langue.as_str(), Champ::TexteLangue),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Interligne, espace",
                "L'interligne (0.65em) et l'espace entre les paragraphes (1.2em).",
                row![
                    saisie(
                        palette,
                        &reglages.texte.interligne,
                        Champ::TexteInterligne,
                        "0.65em"
                    ),
                    saisie(
                        palette,
                        &reglages.texte.espacement,
                        Champ::TexteEspacement,
                        "1.2em"
                    ),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Retrait de la 1re ligne",
                "Le retrait du premier paragraphe. 0cm pour aucun retrait.",
                saisie(palette, &reglages.texte.retrait, Champ::TexteRetrait, "0cm")
            ),
            row![
                case(
                    "Paragraphes justifiés",
                    reglages.texte.justification,
                    Champ::TexteJustification
                ),
                case(
                    "Couper les mots",
                    reglages.texte.coupure,
                    Champ::TexteCoupure
                ),
            ]
            .spacing(18),
        ]
        .spacing(10),
    ));

    // ---- Les titres ----
    corps = corps.push(section(
        palette,
        "Titres de chapitres",
        column![
            ligne(
                palette,
                "Police (vide : celle du texte)",
                "La police des titres. Vide : celle du texte.",
                saisie(
                    palette,
                    &reglages.titres.police,
                    Champ::TitresPolice,
                    "La même"
                )
            ),
            ligne(
                palette,
                "Numérotation des chapitres",
                "Le motif des numéros : 1.1, I.i, A.a…",
                choix(
                    NUMEROTATIONS,
                    reglages.titres.numerotation.as_str(),
                    Champ::TitresNumerotation
                )
            ),
            ligne(
                palette,
                "Corps des titres 1, 2, 3",
                "Le corps de chaque niveau de titre.",
                row![
                    saisie(
                        palette,
                        &reglages.titres.taille_1,
                        Champ::TitresTaille1,
                        "16pt"
                    ),
                    saisie(
                        palette,
                        &reglages.titres.taille_2,
                        Champ::TitresTaille2,
                        "14pt"
                    ),
                    saisie(
                        palette,
                        &reglages.titres.taille_3,
                        Champ::TitresTaille3,
                        "12pt"
                    ),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Casse",
                "Normale, MAJUSCULES ou minuscules.",
                choix(CASSES, reglages.titres.casse.as_str(), Champ::TitresCasse)
            ),
            ligne(
                palette,
                "Espace autour d'un titre",
                "L'espace au-dessus et au-dessous d'un titre.",
                saisie(
                    palette,
                    &reglages.titres.espacement,
                    Champ::TitresEspacement,
                    "1.2em"
                )
            ),
            case("Titres en gras", reglages.titres.gras, Champ::TitresGras),
        ]
        .spacing(10),
    ));

    // ---- Les numéros de page ----
    corps = corps.push(section(
        palette,
        "Numéros de page",
        column![
            row![case(
                "Numéroter les pages",
                reglages.numerotation.actif,
                Champ::NumerotationActive
            ),],
            ligne(
                palette,
                "Forme",
                "Le chiffre des numéros de page : 1, I, i, A, a.",
                choix(
                    FORMES,
                    reglages.numerotation.forme.as_str(),
                    Champ::NumerotationForme
                )
            ),
            ligne(
                palette,
                "Position",
                "En haut ou en bas de la page.",
                choix(
                    POSITIONS,
                    reglages.numerotation.position.as_str(),
                    Champ::NumerotationPosition
                )
            ),
            ligne(
                palette,
                "Alignement",
                "L'alignement du numéro dans sa marge.",
                choix(
                    ALIGNEMENTS,
                    reglages.numerotation.alignement.as_str(),
                    Champ::NumerotationAlignement
                )
            ),
        ]
        .spacing(10),
    ));

    // ---- Les couleurs ----
    corps = corps.push(section(
        palette,
        "Couleurs",
        column![
            ligne(
                palette,
                "Texte",
                "La couleur du texte composé, en hexadécimal.",
                row![
                    pastille(&reglages.couleurs.texte),
                    saisie(
                        palette,
                        &reglages.couleurs.texte,
                        Champ::CouleurTexte,
                        "#000000"
                    )
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Titres",
                "La couleur des titres, en hexadécimal.",
                row![
                    pastille(&reglages.couleurs.titres),
                    saisie(
                        palette,
                        &reglages.couleurs.titres,
                        Champ::CouleurTitres,
                        "#000000"
                    )
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Liens",
                "La couleur des liens et citations, en hexadécimal.",
                row![
                    pastille(&reglages.couleurs.liens),
                    saisie(
                        palette,
                        &reglages.couleurs.liens,
                        Champ::CouleurLiens,
                        "#0000ff"
                    )
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Papier",
                "La couleur du papier, en hexadécimal.",
                row![
                    pastille(&reglages.couleurs.fond),
                    saisie(
                        palette,
                        &reglages.couleurs.fond,
                        Champ::CouleurFond,
                        "#ffffff"
                    )
                ]
                .spacing(8)
            ),
        ]
        .spacing(10),
    ));

    // ---- Le pied : ce qui ne va pas, et les gestes ----
    let mut pied = column![].spacing(10);

    let problemes = reglages.problemes();

    if !problemes.is_empty() {
        let mut liste = column![].spacing(4);

        for (champ, probleme) in &problemes {
            liste = liste.push(palette.label(format!("{champ} : {probleme}")).size(11));
        }

        pied = pied.push(liste);
    }

    if dialogue.regles_main > 0 {
        pied = pied.push(
            palette
                .label(format!(
                    "main.typ porte déjà {} règles #set écrites à la main : elles priment sur cette fenêtre.",
                    dialogue.regles_main
                ))
                .size(11),
        );
    }

    if dialogue.accroche_manquante {
        pied = pied.push(
            checkbox(dialogue.accrocher)
                .label("Accrocher la mise en page à main.typ (deux lignes)")
                .on_toggle(Message::MiseEnPageAccrocher),
        );
    }

    pied = pied.push(
        row![
            button(text("Importer").size(12))
                .on_press(Message::MiseEnPageImporter)
                .padding([8, 12]),
            button(text("Exporter").size(12))
                .on_press(Message::MiseEnPageExporter)
                .padding([8, 12]),
            Space::new().width(Length::Fill),
            button(text("Annuler").size(12))
                .on_press(Message::MiseEnPageFermer)
                .padding([8, 12]),
            button(text("Appliquer").size(12))
                .on_press(Message::MiseEnPageAppliquer)
                .padding([8, 12]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    );

    let fenetre =
        container(column![entete, scrollable(corps).height(Length::Fill), pied].spacing(16))
            .padding(18)
            .style(bubble_style(palette))
            .width(Length::Fixed(760.0))
            .height(Length::Fixed(640.0));

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Une section de la fenêtre : son titre et ses réglages.
fn section<'a>(
    palette: Palette,
    titre: &'a str,
    corps: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![palette.label(titre.to_uppercase()).size(11), corps.into(),]
        .spacing(8)
        .into()
}

/// Une ligne : son libellé, son infobulle, et son contrôle.
///
/// L'infobulle vient au survol : la fenêtre reste dépouillée, mais rien
/// n'est muet — laisser la souris sur un paramètre dit ce qu'il règle et
/// sous quelle forme l'écrire.
fn ligne<'a>(
    palette: Palette,
    libelle: &'a str,
    note: &'a str,
    controle: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    use iced::widget::tooltip;

    let etiquette = tooltip(
        palette.label(libelle).size(12),
        text(note).size(theme::jetons::DETAIL),
        tooltip::Position::Top,
    );

    row![
        etiquette,
        Space::new().width(Length::Fill),
        controle.into(),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Une case à cocher.
fn case<'a>(libelle: &'a str, valeur: bool, champ: Champ) -> Element<'a, Message> {
    checkbox(valeur)
        .label(libelle)
        .on_toggle(move |_| Message::MiseEnPageBascule(champ))
        .into()
}

/// Une saisie de texte.
fn saisie(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
) -> Element<'static, Message> {
    let _ = palette;

    text_input(exemple, valeur)
        .on_input(move |texte| Message::MiseEnPageChamp(champ, texte))
        .width(Length::Fixed(170.0))
        .padding(6)
        .size(12)
        .into()
}

/// Une liste déroulante.
fn choix(options: &[&'static str], valeur: &str, champ: Champ) -> Element<'static, Message> {
    let options: Vec<String> = options.iter().map(|option| option.to_string()).collect();
    let valeur = valeur.to_string();

    pick_list(options, Some(valeur), move |choisi: String| {
        Message::MiseEnPageChamp(champ, choisi)
    })
    .width(Length::Fixed(170.0))
    .into()
}

/// La pastille d'une couleur, à côté de sa saisie.
///
/// Elle ne se teinte que d'une couleur **valide** : une case vide est une
/// case vide, pas un carré noir qui mentirait sur ce qui sera composé.
fn pastille(valeur: &str) -> Element<'static, Message> {
    let couleur = couleur_depuis_hexa(valeur).unwrap_or(iced::Color::TRANSPARENT);

    container(Space::new())
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(couleur.into()),
            border: iced::Border {
                color: iced::Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Une couleur `#rrggbb` ou `#rgb`, en couleur d'écran.
fn couleur_depuis_hexa(valeur: &str) -> Option<iced::Color> {
    let hexa = valeur.trim().strip_prefix('#')?;

    let double = |c: char| -> Option<u8> { u8::from_str_radix(&format!("{c}{c}"), 16).ok() };
    let paire =
        |a: char, b: char| -> Option<u8> { u8::from_str_radix(&format!("{a}{b}"), 16).ok() };

    match hexa.chars().collect::<Vec<_>>().as_slice() {
        [r, v, b] => Some(iced::Color::from_rgb8(
            double(*r)?,
            double(*v)?,
            double(*b)?,
        )),
        [r1, r2, v1, v2, b1, b2] => Some(iced::Color::from_rgb8(
            paire(*r1, *r2)?,
            paire(*v1, *v2)?,
            paire(*b1, *b2)?,
        )),
        _ => None,
    }
}
