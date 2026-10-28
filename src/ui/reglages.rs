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
const COULEURS: &[&str] = &[
    "",
    "black",
    "white",
    "gray",
    "red",
    "green",
    "blue",
    "orange",
    "purple",
    "#1e1e2e",
    "#f2eee5",
];
const CASSES: &[&str] = &[
    "normale",
    "majuscules",
    "minuscules",
    "petites-majuscules",
];
const LANGUES: &[&str] = &["fr", "en", "de", "es", "it", "pt", "nl", "la"];
const DEFAUTS: &[&str] = &["aucune", "française", "anglaise"];
const ESPACES: &[&str] = &["aucune", "normale", "insécable", "fine-insécable"];
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
        palette.label("Normes du document").size(16),
        Space::new().width(Length::Fill),
        onglet(palette, "Mise en page", 0, dialogue.panneau),
        onglet(palette, "Typographie", 1, dialogue.panneau),
        onglet(palette, "Décor", 2, dialogue.panneau),
        Space::new().width(8),
        button(text("Fermer").size(12))
            .on_press(Message::MiseEnPageFermer)
            .padding([6, 10]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let corps = match dialogue.panneau {
        1 => corps_typographie(palette, reglages),
        2 => corps_decor(palette, reglages),
        _ => corps_mise_en_page(palette, reglages),
    };
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

    // La fenêtre tient dans la fenêtre : une taille fixe dépassait d'une
    // application ramenée à son minimum, et le pied — « Appliquer » —
    // atterrissait hors écran.
    let largeur = 760.0_f32.min(state.window_size.width - 32.0).max(320.0);
    let hauteur = 640.0_f32.min(state.window_size.height - 32.0).max(320.0);

    let fenetre =
        container(column![entete, scrollable(corps).height(Length::Fill), pied].spacing(16))
            .padding(18)
            .style(bubble_style(palette))
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

/// Une section de la fenêtre : son titre et ses réglages.
/// Le panneau de mise en page : la page, le texte, les titres, les couleurs.
fn corps_mise_en_page(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'static, Message> {
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
                    saisie(
                        palette,
                        &reglages.titres.taille_4,
                        Champ::TitresTaille4,
                        "11pt"
                    ),
                    saisie(
                        palette,
                        &reglages.titres.taille_5,
                        Champ::TitresTaille5,
                        "10pt"
                    ),
                    saisie(
                        palette,
                        &reglages.titres.taille_6,
                        Champ::TitresTaille6,
                        "10pt"
                    ),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Casse",
                "Normale, MAJUSCULES, minuscules ou petites majuscules.",
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

    corps
}

/// Le panneau de typographie : un défaut de langue, et ses règles.
///
/// Le défaut charge les règles de la langue choisie ; chaque règle se
/// reprend ensuite. Les guillemets se composent par niveau d'imbrication :
/// deux caractères par paire, ouvrant puis fermant.
/// Le décor du document : les mesures que la source ne porte jamais.
///
/// Un bloc habillé dit un nom — `{encadré}` — et trouve ici sa couleur,
/// son épaisseur, son rayon, sa marge et son remplissage. Changer les
/// normes change le document entier ; la source reste du markdown.
fn corps_decor(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'static, Message> {
    let decor = &reglages.decor;

    column![section(
        palette,
        "Les défauts du décor",
        column![
            ligne(
                palette,
                "Couleur de bordure",
                "Une valeur Typst : `gray`, `#555`, `luma(160)`.",
                saisie_couleur(palette, &decor.couleur, Champ::DecorCouleur, "gray")
            ),
            ligne(
                palette,
                "Épaisseur",
                "La graisse du trait : `1pt`, `0.5pt`.",
                saisie(palette, &decor.epaisseur, Champ::DecorEpaisseur, "1pt")
            ),
            ligne(
                palette,
                "Rayon des coins",
                "L'arrondi : `4pt`, `0pt` pour un angle droit.",
                saisie(palette, &decor.rayon, Champ::DecorRayon, "4pt")
            ),
            ligne(
                palette,
                "Marge intérieure",
                "L'air entre la bordure et le contenu : `8pt`.",
                saisie(palette, &decor.marge, Champ::DecorMarge, "8pt")
            ),
            ligne(
                palette,
                "Remplissage",
                "La teinte de fond : `aucun`, une couleur — ou un dégradé, écrit en valeur Typst (gradient.linear(#a, #b)).",
                saisie_couleur(
                    palette,
                    &decor.remplissage,
                    Champ::DecorRemplissage,
                    "aucun, ou une couleur — un dégradé s'écrit en valeur"
                )
            ),
        ]
        .spacing(theme::jetons::PAS),
    )]
    .spacing(theme::jetons::PAS)
}

fn corps_typographie(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'static, Message> {
    let typo = &reglages.typographie;

    column![
        section(
            palette,
            "Défaut",
            column![
                ligne(
                    palette,
                    "Langue",
                    "Charge les règles de la langue choisie — qu'on reprend ensuite.",
                    choix_defaut(typo.base.as_str())
                ),
                row![
                    button(
                        palette
                            .label("Réinitialiser aux valeurs de la langue")
                            .size(theme::jetons::DETAIL)
                    )
                    .on_press(Message::MiseEnPageTypographieDefaut(typo.base.clone()))
                    .padding([5, 12]),
                    Space::new().width(Length::Fill),
                ],
            ]
            .spacing(theme::jetons::PAS),
        ),
        section(
            palette,
            "Espacements",
            column![
                ligne(
                    palette,
                    "Devant ; : ! ?",
                    "En français, une espace insécable — `~` en Typst.",
                    choix(ESPACES, typo.avant_signes.as_str(), Champ::TypoAvantSignes)
                ),
                ligne(
                    palette,
                    "Après ; : ! ?",
                    "Une espace normale en français ; la ponctuation colle en anglais.",
                    choix(ESPACES, typo.apres_signes.as_str(), Champ::TypoApresSignes)
                ),
                ligne(
                    palette,
                    "Après , .",
                    "Hors nombres : `3.14` et `1,2` ne sont jamais touchés.",
                    choix(
                        ESPACES,
                        typo.apres_virgule_point.as_str(),
                        Champ::TypoApresVirgulePoint
                    )
                ),
                ligne(
                    palette,
                    "Dans les guillemets",
                    "L'espace entre le guillemet et le texte qu'il cite.",
                    choix(
                        ESPACES,
                        typo.espace_guillemets.as_str(),
                        Champ::TypoEspaceGuillemets
                    )
                ),
            ]
            .spacing(theme::jetons::PAS),
        ),
        section(
            palette,
            "Guillemets",
            column![
                ligne(
                    palette,
                    "Niveau 1",
                    "Deux caractères : l'ouvrant puis le fermant — « » en français.",
                    saisie(
                        palette,
                        &paire_lue(&typo.guillemets, 0),
                        Champ::TypoGuillemets1,
                        "«»"
                    )
                ),
                ligne(
                    palette,
                    "Niveau 2",
                    "Ce que la citation de niveau 1 contient — \" \" en français.",
                    saisie(
                        palette,
                        &paire_lue(&typo.guillemets, 1),
                        Champ::TypoGuillemets2,
                        "\"\""
                    )
                ),
                ligne(
                    palette,
                    "Niveau 3",
                    "Et le suivant — ' ' en français.",
                    saisie(
                        palette,
                        &paire_lue(&typo.guillemets, 2),
                        Champ::TypoGuillemets3,
                        "''"
                    )
                ),
            ]
            .spacing(theme::jetons::PAS),
        ),
    ]
    .spacing(20)
}

/// La paire de guillemets d'un niveau, pour le champ.
fn paire_lue(paires: &[String], niveau: usize) -> String {
    paires.get(niveau).cloned().unwrap_or_default()
}

/// La liste des défauts de langue : le choix charge leurs règles.
fn choix_defaut(valeur: &str) -> Element<'static, Message> {
    let options: Vec<String> = DEFAUTS.iter().map(|option| option.to_string()).collect();
    let valeur = valeur.to_string();

    pick_list(options, Some(valeur), |choisi: String| {
        Message::MiseEnPageTypographieDefaut(choisi)
    })
    .width(Length::Fixed(170.0))
    .into()
}

/// Un onglet de la fenêtre des normes.
fn onglet(
    palette: Palette,
    libelle: &'static str,
    numero: usize,
    courant: usize,
) -> Element<'static, Message> {
    let actif = numero == courant;

    button(text(libelle).size(12).color(if actif {
        palette.accent
    } else {
        palette.text_muted
    }))
    .on_press(Message::MiseEnPagePanneau(numero))
    .padding([6, 12])
    .style(move |_, _| button::Style {
        background: None,
        text_color: if actif {
            palette.accent
        } else {
            palette.text_muted
        },
        border: iced::Border {
            color: if actif {
                palette.accent
            } else {
                iced::Color::TRANSPARENT
            },
            width: if actif { 1.5 } else { 1.0 },
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    })
    .into()
}

fn section<'a>(
    palette: Palette,
    titre: &'a str,
    corps: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![palette.label(titre.to_uppercase()).size(11), corps.into(),]
        .spacing(8)
        .into()
}

/// Une saisie de couleur, avec son carré de prévisualisation.
///
/// Le nom d'abord — les couleurs de la liste — puis la valeur libre,
/// hexadécimale ou expression Typst. Le carré dit ce qu'elle donne ; un
/// dégradé n'a pas de carré, mais il compose.
fn saisie_couleur(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
) -> Element<'static, Message> {
    // La teinte est calculée une fois : le style ne garde pas la valeur.
    let teinte = couleur_depuis(valeur).unwrap_or(palette.text_muted);

    let apercu = container(Space::new())
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .style(move |_| container::Style {
            background: Some(teinte.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    row![
        choix(COULEURS, valeur, champ),
        saisie(palette, valeur, champ, exemple),
        apercu,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
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

    // Le fond de l'infobulle : sans lui, du texte nu se perdait dans ce
    // qui est derrière. Une boîte, une lisière, et la lecture est tenue.
    let bulle = container(
        text(note)
            .size(theme::jetons::DETAIL)
            .color(palette.text),
    )
    .padding(8)
    .style(move |_| container::Style {
        background: Some(palette.surface.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    });

    let etiquette = tooltip(
        palette.label(libelle).size(12),
        bulle,
        tooltip::Position::Top,
    );

    row![etiquette, Space::new().width(Length::Fill), controle.into(),]
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
/// Une couleur, par son nom ou son hexadécimal — le nuancier la suit.
fn couleur_depuis(valeur: &str) -> Option<iced::Color> {
    let nom = valeur.trim().to_lowercase();

    let connue = match nom.as_str() {
        "black" | "noir" => Some(iced::Color::BLACK),
        "white" | "blanc" => Some(iced::Color::WHITE),
        "gray" | "gris" => Some(iced::Color::from_rgb8(128, 128, 128)),
        "red" | "rouge" => Some(iced::Color::from_rgb8(200, 60, 60)),
        "green" | "vert" => Some(iced::Color::from_rgb8(70, 140, 80)),
        "blue" | "bleu" => Some(iced::Color::from_rgb8(60, 90, 180)),
        "orange" => Some(iced::Color::from_rgb8(220, 140, 50)),
        "purple" | "violet" => Some(iced::Color::from_rgb8(130, 80, 160)),
        _ => None,
    };

    connue.or_else(|| couleur_depuis_hexa(valeur))
}

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
