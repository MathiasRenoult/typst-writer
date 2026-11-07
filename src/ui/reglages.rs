//! La fenêtre de mise en page du document.
//!
//! Une fenêtre flottante, assez grande pour tout montrer d'un regard : le
//! papier, les marges, le texte, les titres, les numéros, les couleurs. Ce
//! qu'elle montre est une copie de travail — seul « Appliquer » touche le
//! projet, et les valeurs fautives sont dites, champ par champ, avant que
//! quoi que ce soit ne soit écrit.

use iced::widget::{
    Space, button, checkbox, column, combo_box, container, pick_list, row, scrollable, text,
    text_input,
};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::mise_en_page::modele::Champ;
use crate::ui::editor::bubble_style;
use crate::ui::theme::{self, Palette};

const FORMATS: &[&str] = &["a3", "a4", "a5", "a6", "b4", "b5", "letter", "legal"];
const ORIENTATIONS: &[&str] = &["portrait", "paysage"];
const CASSES: &[&str] = &["normale", "majuscules", "minuscules", "petites-majuscules"];
const LANGUES: &[&str] = &["fr", "en", "de", "es", "it", "pt", "nl", "la"];
const DEFAUTS: &[&str] = &["aucune", "française", "anglaise"];
const ESPACES: &[&str] = &["aucune", "normale", "insécable", "fine-insécable"];
const NUMEROTATIONS: &[&str] = &["1.1", "1.1.1", "I.i", "A.a", "1)", "(1)", "A.", "a."];

/// Les formes de numérotation des figures, et les styles de légende —
/// en liste, comme il se doit (demande du 2026-11-04).
/// Les préréglages de guillemets, par niveau (bug-report-8, T5.1) — les
/// systèmes typographiques européens. Un choix ne fait que remplir le
/// champ : la saisie manuelle reste souveraine.
/// La langue des dates (bug-report-8, T3.2) : le libellé choisi écrit le
/// code que le moteur de noms comprend.
const LANGUES_DATES: [(&str, &str); 4] = [
    ("français", "fr"),
    ("anglais", "en"),
    ("allemand", "de"),
    ("italien", "it"),
];

const GUILLEMETS_1: &[&str] = &["«»", "“”", "„“", "‹›", "\"\""];
const GUILLEMETS_2: &[&str] = &["“”", "‘’", "‚‘", "‹›", "\"\"", "''"];
const GUILLEMETS_3: &[&str] = &["‘’", "''", "“”", "\"\"", "«»"];

const NUMERATIONS_FIGURES: &[&str] = &["1", "A", "a", "I", "i", "1.1"];
const STYLES_LEGENDE: &[&str] = &["italique", "gras", "normal"];

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

    // ---- Le titre, et la gestion des modèles (NORMS-04-03 : au-dessus
    // des réglages, jamais au milieu) ----
    let noms_normes = crate::project::noms_de_normes();
    let entete = row![
        palette.label("Normes du document").size(16),
        Space::new().width(Length::Fill),
        button(palette.label("Importer").size(theme::jetons::DETAIL))
            .on_press(Message::MiseEnPageImporter)
            .padding([5, 12]),
        button(palette.label("Exporter").size(theme::jetons::DETAIL))
            .on_press(Message::MiseEnPageExporter)
            .padding([5, 12]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ---- La norme en cours : celle du document, ou un brouillon ----
    // L'état dit ce qu'on modifie, et l'enregistrement propose d'écraser
    // la norme chargée ou d'en garder une copie nommée (bug-report-9).
    let chargee = !dialogue.norme_originale.is_empty();
    let etat_normes = match (chargee, dialogue.modifie) {
        (true, true) => format!("Modification de « {} »", dialogue.norme_originale),
        (true, false) => format!("« {} » est chargée", dialogue.norme_originale),
        (false, true) => "Normes personnalisées — modifiées".to_string(),
        (false, false) => "Normes personnalisées".to_string(),
    };
    let ecraser = format!("Écraser « {} »", dialogue.norme_originale);

    // Le choix s'applique en choisissant : le menu est le seul « Appliquer ».
    let modeles = row![
        palette
            .label("Modèle de normes :")
            .size(theme::jetons::DETAIL),
        pick_list(
            noms_normes,
            Some(dialogue.norme.to_string()),
            Message::NormesAppliquer
        )
        .width(Length::Fixed(160.0)),
        button(theme::icon(
            lucide_icons::iced::icon_trash_2::<iced::Theme>(),
            12.0,
            palette.text_muted,
        ))
        .on_press(Message::NormesSupprimer(dialogue.norme.to_string()))
        .padding([4, 6]),
        Space::new().width(Length::Fill),
        text_input("nom de la copie", &dialogue.norme)
            .on_input(Message::NormesNom)
            .padding(5)
            .width(Length::Fixed(180.0)),
        button(theme::icon(
            lucide_icons::iced::icon_save::<iced::Theme>(),
            12.0,
            palette.text_muted,
        ))
        .on_press(Message::NormesEnregistrer(dialogue.norme.to_string()))
        .padding([4, 6]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // La ligne du bas : ce qu'on modifie, et l'écrasement éventuel — à
    // part, pour ne jamais écraser les autres éléments (bug-report-9).
    let mut etat = row![palette.label(etat_normes).size(theme::jetons::DETAIL)]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    if chargee {
        etat = etat.push(
            button(palette.label(ecraser).size(theme::jetons::DETAIL))
                .on_press(Message::NormesEcraser)
                .padding([5, 12]),
        );
    }

    // ---- Le rail des onglets : à gauche, une icône et son nom ----
    macro_rules! picto {
        ($icone:ident) => {
            theme::icon(
                lucide_icons::iced::$icone::<iced::Theme>(),
                15.0,
                palette.text,
            )
        };
    }
    let onglets = [
        (picto!(icon_tag), "Métadonnées"),
        (picto!(icon_file), "Page"),
        (picto!(icon_type), "Texte"),
        (picto!(icon_heading), "Titres"),
        (picto!(icon_calendar), "Figures et dates"),
        (picto!(icon_layout), "En-tête et pied"),
        (picto!(icon_paintbrush), "Typographie"),
        (picto!(icon_palette), "Couleurs et décor"),
        (picto!(icon_book), "Bibliographie"),
    ];
    let mut rail = column![].spacing(2);
    for (indice, (picto, nom)) in onglets.into_iter().enumerate() {
        rail = rail.push(onglet_vertical(
            palette,
            picto,
            nom,
            indice,
            dialogue.panneau,
        ));
    }
    let rail = container(rail).padding([4, 6]);

    let corps = match dialogue.panneau {
        1 => corps_page(palette, reglages),
        2 => corps_texte(palette, reglages, &state.police_combo),
        3 => corps_titres(palette, reglages),
        4 => corps_figures_dates(palette, reglages),
        5 => corps_entete_pied(palette, reglages, &dialogue.metadonnees),
        6 => corps_typographie(palette, reglages),
        7 => column![
            corps_couleurs(palette, reglages),
            corps_decor(palette, reglages)
        ]
        .spacing(20),
        8 => corps_bibliographie(palette, dialogue),
        _ => column![corps_metadonnees(palette, dialogue)].spacing(20),
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
                .label(
                    "Écrire dans main.typ le rappel des normes : le document se \
                     compose alors seul, hors de l'application, avec sa mise en page",
                )
                .on_toggle(Message::MiseEnPageAccrocher),
        );
    }

    let pied = pied.push(
        row![
            button(text("Annuler").size(12))
                .on_press(Message::MiseEnPageFermer)
                .padding([8, 12]),
            Space::new().width(Length::Fill),
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
    let largeur = 860.0_f32.min(state.window_size.width - 32.0).max(340.0);
    let hauteur = 660.0_f32.min(state.window_size.height - 32.0).max(340.0);

    // Le corps : le rail des onglets à gauche — un sixième de la
    // largeur, pas davantage — et le panneau à droite.
    let vu = row![
        rail.width(Length::FillPortion(1)),
        scrollable(container(corps).padding([2, 10]))
            .height(Length::Fill)
            .width(Length::FillPortion(5)),
    ]
    .spacing(4)
    .height(Length::Fill);

    // Les fonctionnalités du projet : une débranchée, son interface
    // disparaît (demande de l'auteur).
    let fonctionnalites = row![
        palette
            .label("Fonctionnalités du projet :")
            .size(theme::jetons::DETAIL),
        iced::widget::checkbox(reglages.fonctionnalites.bibliographie)
            .label("Bibliographie")
            .on_toggle(|v| Message::ReglagesFonctionnalite("bibliographie".to_string(), v)),
        iced::widget::checkbox(reglages.fonctionnalites.glossaire)
            .label("Glossaire")
            .on_toggle(|v| Message::ReglagesFonctionnalite("glossaire".to_string(), v)),
        iced::widget::checkbox(reglages.fonctionnalites.graphiques)
            .label("Graphiques")
            .on_toggle(|v| Message::ReglagesFonctionnalite("graphiques".to_string(), v)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let fenetre = container(column![entete, modeles, fonctionnalites, etat, vu, pied].spacing(12))
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
/// L'onglet Métadonnées : une paire clé → valeur par ligne.
///
/// La clé est le nom de la quick part — `{titre}`, `{auteur}` — et
/// l'utilisateur choisit lesquelles il y a (bug-report-7, point 9).
fn corps_metadonnees<'a>(
    palette: Palette,
    dialogue: &'a crate::app::state::DialogueMiseEnPage,
) -> Element<'a, Message> {
    let mut contenu = column![].spacing(theme::jetons::PAS);

    contenu = contenu.push(
        palette
            .label("Les métadonnées du document")
            .size(theme::jetons::SAILLANT),
    );
    contenu = contenu.push(
        palette
            .label("Vos quick parts : ce que vous voulez, dans votre ordre.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );
    contenu = contenu.push(
        palette
            .label("La clé est le nom de la quick part : {directeur}, {université}…")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    // ---- Les profils (bug-report-10, 4.1–4.3) : le profil actif donne
    // ses valeurs aux balises ; « Enregistrer » range les valeurs
    // courantes sous le nom tapé.
    let noms_profil: Vec<String> = dialogue
        .metadonnees
        .profils
        .iter()
        .map(|profil| profil.nom.clone())
        .collect();
    let actif = Some(dialogue.metadonnees.profil_actif.clone()).filter(|nom| !nom.is_empty());

    contenu = contenu.push(
        row![
            palette.label("Profil :").size(theme::jetons::DETAIL),
            pick_list(noms_profil, actif, Message::MetaProfilActiver).width(Length::Fixed(160.0)),
            text_input("nom du profil", &dialogue.profil_nom)
                .on_input(Message::MetaProfilNom)
                .padding(5)
                .width(Length::Fixed(140.0)),
            button(theme::icon(
                lucide_icons::iced::icon_save::<iced::Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::MetaProfilEnregistrer(dialogue.profil_nom.clone()))
            .padding([4, 6]),
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<iced::Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::MetaProfilSupprimer(
                dialogue.metadonnees.profil_actif.clone()
            ))
            .padding([4, 6]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    );

    // Le remplacement des balises par leurs valeurs (4.4).
    contenu = contenu.push(
        button(
            palette
                .label("Remplacer les balises du texte par leurs valeurs")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::MetaRemplacer)
        .padding([5, 12]),
    );

    for (cle, valeur) in dialogue.metadonnees.visibles() {
        let ancienne = cle.clone();
        let pour_cle = ancienne.clone();
        let pour_valeur = ancienne.clone();
        contenu = contenu.push(
            row![
                champ_texte("clé", cle, Length::Fixed(150.0), move |v| {
                    Message::MetaChamp(pour_cle.clone(), v, valeur.clone())
                }),
                champ_texte("valeur", valeur, Length::Fill, move |v| {
                    Message::MetaChamp(pour_valeur.clone(), pour_valeur.clone(), v)
                }),
                button(text("×").size(13))
                    .on_press(Message::MetaSupprimer(ancienne.clone()))
                    .padding([4, 8]),
            ]
            .spacing(8),
        );
    }

    contenu = contenu.push(
        button(
            palette
                .label("Ajouter une ligne")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::MetaAjouter)
        .padding([5, 12]),
    );

    scrollable(contenu).height(Length::Fill).into()
}

/// Le panneau de mise en page : la page, le texte, les titres, les couleurs.
/// Les formes de numérotation que Typst sait composer.
const FORMES_NUMEROTATION: &[&str] = &["1", "a", "A", "i", "I", "α", "Α", "א"];

/// L'en-tête et le pied de page : leurs trois places, et la forme du
/// numéro.
fn corps_entete_pied<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
    metadonnees: &crate::project::metadonnees::Metadonnees,
) -> iced::widget::Column<'a, Message> {
    let entete_pied = &reglages.entete_pied;

    // Les quick parts : les marqueurs du moteur, puis **les clés de
    // l'auteur** — ses métadonnées s'offrent d'elles-mêmes (demande du
    // 2026-11-04).
    let mut quick: Vec<(String, String)> = vec![
        ("Quick part…".to_string(), String::new()),
        ("Page : {page}".to_string(), "{page}".to_string()),
        ("Total : {pages}".to_string(), "{pages}".to_string()),
        ("Date : {date}".to_string(), "{date}".to_string()),
        ("Titre : {titre}".to_string(), "{titre}".to_string()),
        ("Projet : {projet}".to_string(), "{projet}".to_string()),
        (
            "Document : {document}".to_string(),
            "{document}".to_string(),
        ),
    ];
    for (cle, _) in metadonnees.visibles() {
        quick.push((format!("{{{cle}}}"), format!("{{{cle}}}")));
    }

    let mut corps = column![].spacing(20);

    corps = corps.push(section(
        palette,
        "Images",
        column![ligne(
            palette,
            "Hauteur des images",
            "En pourcentage de la hauteur de l'en-tête ou du pied qui les \
             porte — les proportions se gardent, jamais de débordement \
             (bug-report-8, T4.4).",
            saisie_numerique(
                palette,
                &entete_pied.images_hauteur,
                Champ::ImagesHauteur,
                "80"
            )
        )]
        .spacing(10),
    ));

    // Un lieu — gauche, centre, droite — est une ligne de saisie, et son
    // titre dit la place.
    fn lieu<'a>(
        palette: Palette,
        titre: &'a str,
        valeur: &str,
        champ: Champ,
        quick: Vec<(String, String)>,
    ) -> Element<'a, Message> {
        // Les quick parts : le marqueur choisi s'ajoute à la fin du champ.
        let valeur = valeur.to_string();
        let valeur_pour = valeur.clone();
        let libelles: Vec<String> = quick.iter().map(|(libelle, _)| libelle.clone()).collect();
        let rapide = iced::widget::pick_list(
            libelles,
            Some("Quick part…".to_string()),
            move |choisi: String| {
                let marqueur = quick
                    .iter()
                    .find(|(libelle, _)| *libelle == choisi)
                    .map(|(_, marqueur)| marqueur.clone())
                    .unwrap_or_default();
                Message::MiseEnPageChamp(champ, format!("{valeur_pour}{marqueur}"))
            },
        )
        .text_size(12)
        .padding([4, 6]);

        // L'ajout d'image : une icône, à côté des quick parts
        // (bug-report-8, T4.2-T4.3).
        let image = button(theme::icon(
            lucide_icons::iced::icon_image::<iced::Theme>(),
            14.0,
            palette.text_muted,
        ))
        .padding(4)
        .on_press(Message::EntetePiedImage(champ));

        row![
            palette
                .label(titre)
                .size(theme::jetons::DETAIL)
                .width(Length::Fixed(60.0)),
            saisie(palette, &valeur, champ, "vide"),
            rapide,
            image,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    }

    corps = corps.push(section(
        palette,
        "En-tête",
        column![
            lieu(
                palette,
                "Gauche",
                &entete_pied.entete.gauche,
                Champ::EnteteGauche,
                quick.clone()
            ),
            lieu(
                palette,
                "Centre",
                &entete_pied.entete.centre,
                Champ::EnteteCentre,
                quick.clone()
            ),
            lieu(
                palette,
                "Droite",
                &entete_pied.entete.droite,
                Champ::EnteteDroite,
                quick.clone()
            ),
        ]
        .spacing(8),
    ));

    corps = corps.push(section(
        palette,
        "Pied de page",
        column![
            lieu(palette, "Gauche", &entete_pied.pied.gauche, Champ::PiedGauche, quick.clone()),
            lieu(palette, "Centre", &entete_pied.pied.centre, Champ::PiedCentre, quick.clone()),
            lieu(palette, "Droite", &entete_pied.pied.droite, Champ::PiedDroite, quick.clone()),
            palette.label(
                "Le numéro s'écrit {page}, le total {pages} — « p. {page} », « page {page} sur {pages} », « {page}/{pages} »."
            ).size(theme::jetons::DETAIL),
        ]
        .spacing(8),
    ));

    corps = corps.push(section(
        palette,
        "La forme du numéro",
        column![
            row![
                choix(
                    FORMES_NUMEROTATION,
                    entete_pied.numerotation.as_str(),
                    Champ::EntetePiedNumerotation
                ),
                saisie(
                    palette,
                    &entete_pied.numerotation,
                    Champ::EntetePiedNumerotation,
                    "1.1"
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
            palette
                .label("1, a, A, i, I, α, Α, א — ou votre propre format.")
                .size(theme::jetons::DETAIL),
            palette
                .label(format!(
                    "Aperçu : {}",
                    apercu_ou_defaut(&entete_pied.numerotation)
                ))
                .size(theme::jetons::DETAIL),
        ]
        .spacing(8),
    ));

    corps
}

fn corps_page<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let mut corps = column![].spacing(20);

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
                    saisie_numerique(palette, &reglages.marges.haut, Champ::MargeHaut, "2cm"),
                    saisie_numerique(palette, &reglages.marges.bas, Champ::MargeBas, "2cm"),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Marges (gauche, droite)",
                "La marge horizontale. Toute mesure Typst : 2cm, 1in, 24pt.",
                row![
                    saisie_numerique(palette, &reglages.marges.gauche, Champ::MargeGauche, "2cm"),
                    saisie_numerique(palette, &reglages.marges.droite, Champ::MargeDroite, "2cm"),
                ]
                .spacing(8)
            ),
        ]
        .spacing(10),
    ));

    // NORMS-04-03 : la gestion des modèles est en haut de la fenêtre.

    // ---- Le texte ----

    // ---- Les couleurs ----
    corps
}

fn corps_texte<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
    combo: &'a iced::widget::combo_box::State<String>,
) -> iced::widget::Column<'a, Message> {
    let mut corps = column![].spacing(20);

    corps = corps.push(section(
        palette,
        "Texte",
        column![
            ligne(
                palette,
                "Police",
                "La police de composition, par son nom système.",
                choix_police(&reglages.texte.police, Champ::TextePolice, combo)
            ),
            ligne(
                palette,
                "Corps, langue",
                "Le corps du texte (11pt) et la langue de césure (fr).",
                row![
                    saisie_numerique(palette, &reglages.texte.taille, Champ::TexteTaille, "11pt"),
                    choix(LANGUES, reglages.texte.langue.as_str(), Champ::TexteLangue),
                ]
                .spacing(8)
            ),
            ligne(
                palette,
                "Interligne, espace",
                "L'interligne (0.65em) et l'espace entre les paragraphes (1.2em).",
                row![
                    saisie_numerique(
                        palette,
                        &reglages.texte.interligne,
                        Champ::TexteInterligne,
                        "0.65em"
                    ),
                    saisie_numerique(
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
                saisie_numerique(palette, &reglages.texte.retrait, Champ::TexteRetrait, "0cm")
            ),
            row![case(
                "Paragraphes justifiés",
                reglages.texte.justification,
                Champ::TexteJustification
            ),]
        ]
        .spacing(10),
    ));

    // ---- Les titres ----
    corps
}

fn corps_titres<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let titres = &reglages.titres;

    // ---- La grille : six niveaux, six réglages — tout se voit d'un
    // coup d'œil, colonne par colonne.
    fn largeur(
        element: impl Into<Element<'static, Message>>,
        px: f32,
    ) -> Element<'static, Message> {
        container(element.into()).width(Length::Fixed(px)).into()
    }

    // Une case à bascule : elle montre l'état **réel** du niveau (le
    // réglage global fait défaut) et écrit l'opposé.
    fn bascule(valeur: &str, defaut: bool, champ: Champ) -> Element<'static, Message> {
        let actif = if valeur.is_empty() {
            defaut
        } else {
            valeur == "true"
        };
        checkbox(actif)
            .on_toggle(move |_v| Message::MiseEnPageChamp(champ, (!actif).to_string()))
            .into()
    }

    let mut grille = column![
        row![
            largeur(palette.label("Niveau").size(theme::jetons::DETAIL), 58.0),
            largeur(palette.label("Taille").size(theme::jetons::DETAIL), 66.0),
            largeur(palette.label("Gras").size(theme::jetons::DETAIL), 38.0),
            largeur(palette.label("Italique").size(theme::jetons::DETAIL), 44.0),
            largeur(palette.label("Souligné").size(theme::jetons::DETAIL), 52.0),
            largeur(palette.label("Casse").size(theme::jetons::DETAIL), 118.0),
            largeur(
                palette.label("Espacement").size(theme::jetons::DETAIL),
                74.0
            ),
            largeur(palette.label("Couleur").size(theme::jetons::DETAIL), 34.0),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    ]
    .spacing(6);

    for (niveau, (taille, gras, italique, souligne, casse, espacement)) in [
        (
            1,
            (
                &titres.taille_1,
                &titres.gras_1,
                &titres.italique_1,
                &titres.souligne_1,
                &titres.casse_1,
                &titres.espacement_1,
            ),
        ),
        (
            2,
            (
                &titres.taille_2,
                &titres.gras_2,
                &titres.italique_2,
                &titres.souligne_2,
                &titres.casse_2,
                &titres.espacement_2,
            ),
        ),
        (
            3,
            (
                &titres.taille_3,
                &titres.gras_3,
                &titres.italique_3,
                &titres.souligne_3,
                &titres.casse_3,
                &titres.espacement_3,
            ),
        ),
        (
            4,
            (
                &titres.taille_4,
                &titres.gras_4,
                &titres.italique_4,
                &titres.souligne_4,
                &titres.casse_4,
                &titres.espacement_4,
            ),
        ),
        (
            5,
            (
                &titres.taille_5,
                &titres.gras_5,
                &titres.italique_5,
                &titres.souligne_5,
                &titres.casse_5,
                &titres.espacement_5,
            ),
        ),
        (
            6,
            (
                &titres.taille_6,
                &titres.gras_6,
                &titres.italique_6,
                &titres.souligne_6,
                &titres.casse_6,
                &titres.espacement_6,
            ),
        ),
    ]
    .into_iter()
    {
        let casse_affichee = if casse.as_deref().unwrap_or("").is_empty() {
            titres.casse.as_str()
        } else {
            casse.as_deref().unwrap_or("")
        };
        grille = grille.push(
            row![
                largeur(
                    palette
                        .label(format!("Niveau {niveau}"))
                        .size(theme::jetons::DETAIL),
                    58.0
                ),
                largeur(
                    saisie_numerique(
                        palette,
                        taille,
                        match niveau {
                            1 => Champ::TitresTaille1,
                            2 => Champ::TitresTaille2,
                            3 => Champ::TitresTaille3,
                            4 => Champ::TitresTaille4,
                            5 => Champ::TitresTaille5,
                            _ => Champ::TitresTaille6,
                        },
                        "12pt"
                    ),
                    66.0
                ),
                largeur(
                    bascule(
                        gras.map(|v| v.to_string()).unwrap_or_default().as_str(),
                        titres.gras,
                        match niveau {
                            1 => Champ::TitresGras1,
                            2 => Champ::TitresGras2,
                            3 => Champ::TitresGras3,
                            4 => Champ::TitresGras4,
                            5 => Champ::TitresGras5,
                            _ => Champ::TitresGras6,
                        }
                    ),
                    38.0
                ),
                largeur(
                    bascule(
                        italique.map(|v| v.to_string()).unwrap_or_default().as_str(),
                        false,
                        match niveau {
                            1 => Champ::TitresItalique1,
                            2 => Champ::TitresItalique2,
                            3 => Champ::TitresItalique3,
                            4 => Champ::TitresItalique4,
                            5 => Champ::TitresItalique5,
                            _ => Champ::TitresItalique6,
                        }
                    ),
                    44.0
                ),
                largeur(
                    bascule(
                        souligne.map(|v| v.to_string()).unwrap_or_default().as_str(),
                        false,
                        match niveau {
                            1 => Champ::TitresSouligne1,
                            2 => Champ::TitresSouligne2,
                            3 => Champ::TitresSouligne3,
                            4 => Champ::TitresSouligne4,
                            5 => Champ::TitresSouligne5,
                            _ => Champ::TitresSouligne6,
                        }
                    ),
                    52.0
                ),
                largeur(
                    choix(
                        CASSES,
                        casse_affichee,
                        match niveau {
                            1 => Champ::TitresCasse1,
                            2 => Champ::TitresCasse2,
                            3 => Champ::TitresCasse3,
                            4 => Champ::TitresCasse4,
                            5 => Champ::TitresCasse5,
                            _ => Champ::TitresCasse6,
                        }
                    ),
                    118.0
                ),
                largeur(
                    saisie_numerique(
                        palette,
                        espacement.as_deref().unwrap_or(""),
                        match niveau {
                            1 => Champ::TitresEspacement1,
                            2 => Champ::TitresEspacement2,
                            3 => Champ::TitresEspacement3,
                            4 => Champ::TitresEspacement4,
                            5 => Champ::TitresEspacement5,
                            _ => Champ::TitresEspacement6,
                        },
                        "0.6em"
                    ),
                    74.0
                ),
                largeur(
                    pastille(
                        palette,
                        match niveau {
                            1 => titres.couleur_1.as_deref().unwrap_or(""),
                            2 => titres.couleur_2.as_deref().unwrap_or(""),
                            3 => titres.couleur_3.as_deref().unwrap_or(""),
                            4 => titres.couleur_4.as_deref().unwrap_or(""),
                            5 => titres.couleur_5.as_deref().unwrap_or(""),
                            _ => titres.couleur_6.as_deref().unwrap_or(""),
                        },
                        match niveau {
                            1 => Champ::TitresCouleur1,
                            2 => Champ::TitresCouleur2,
                            3 => Champ::TitresCouleur3,
                            4 => Champ::TitresCouleur4,
                            5 => Champ::TitresCouleur5,
                            _ => Champ::TitresCouleur6,
                        }
                    ),
                    34.0
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }

    let mut corps = column![].spacing(20);

    corps = corps.push(section(
        palette,
        "Numérotation et styles",
        column![
            ligne(
                palette,
                "Numérotation des chapitres",
                "1 = chiffre, a/A = lettre, i/I = romain ; le reste s'écrit \
                 tel quel — « 1.1 », « A.a », « 1) ». Un modèle ou votre \
                 propre format (bug-report-8, T2.3).",
                row![
                    choix(
                        NUMEROTATIONS,
                        titres.numerotation.as_str(),
                        Champ::TitresNumerotation
                    ),
                    saisie(
                        palette,
                        &titres.numerotation,
                        Champ::TitresNumerotation,
                        "1.1"
                    ),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
            ),
            palette
                .label(format!(
                    "Aperçu : {}",
                    crate::mise_en_page::modele::apercu_numerotation(&titres.numerotation)
                ))
                .size(theme::jetons::DETAIL),
            grille,
        ]
        .spacing(12),
    ));

    corps
}

fn corps_figures_dates<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let mut corps = column![].spacing(20);

    corps = corps.push(section(
        palette,
        "Légendes de figures",
        column![
            ligne(
                palette,
                "Numérotation des figures",
                "1, A, a, I, i, 1.1 — ou votre propre format, vide pour le défaut.",
                row![
                    choix_defaut_vide(
                        NUMERATIONS_FIGURES,
                        &reglages.numerotation_figures,
                        Champ::NumerotationFigures
                    ),
                    saisie(
                        palette,
                        &reglages.numerotation_figures,
                        Champ::NumerotationFigures,
                        "1.1"
                    ),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
            ),
            palette
                .label(format!(
                    "Aperçu : {}",
                    apercu_ou_defaut(&reglages.numerotation_figures)
                ))
                .size(theme::jetons::DETAIL),
            ligne(
                palette,
                "Libellé de la légende",
                "« fig. », « Figure », ou autre chose — vide pour le défaut.",
                saisie(
                    palette,
                    &reglages.legende_prefixe,
                    Champ::LegendePrefixe,
                    "fig."
                )
            ),
            ligne(
                palette,
                "Style de la légende",
                "italique, gras, normal — ou le défaut.",
                choix_defaut_vide(STYLES_LEGENDE, &reglages.legende_style, Champ::LegendeStyle)
            ),
        ]
        .spacing(10),
    ));

    corps = corps.push(section(
        palette,
        "Dates — la balise {date:...}",
        column![
            ligne(
                palette,
                "Langue des dates",
                "La langue des noms de mois et de jours — indépendante de \
             celle de l'interface (bug-report-8, T3.2).",
                choix_langue(&reglages.dates.langue, Champ::DatesLangue)
            ),
            ligne(
                palette,
                "Mot du siècle",
                "Le mot du siècle : « siècle », « s. »…",
                saisie(
                    palette,
                    &reglages.dates.siecle,
                    Champ::DatesSiecle,
                    "siècle"
                )
            ),
            ligne(
                palette,
                "Mot du millénaire",
                "Le mot du millénaire.",
                saisie(
                    palette,
                    &reglages.dates.millenaire,
                    Champ::DatesMillenaire,
                    "millénaire"
                )
            ),
            ligne(
                palette,
                "Époque après",
                "La forme longue après : « ap. J.-C. », « EC »…",
                saisie(
                    palette,
                    &reglages.dates.epoque_apres,
                    Champ::DatesEpoqueApres,
                    "ap. J.-C."
                )
            ),
            ligne(
                palette,
                "Époque avant",
                "La forme longue avant : « av. J.-C. », « AEC »…",
                saisie(
                    palette,
                    &reglages.dates.epoque_avant,
                    Champ::DatesEpoqueAvant,
                    "av. J.-C."
                )
            ),
            ligne(
                palette,
                "Format des dates pleines",
                "Les jetons se composent librement — voir la table juste \
                 dessous (bug-report-8, T3.1).",
                saisie(
                    palette,
                    &reglages.dates.format,
                    Champ::DatesFormat,
                    "dd/MM/yyyy"
                )
            ),
            palette
                .label(
                    "yyyy an sur 4 · yy an sur 2 · MMMM mois en lettres · MMM mois abrégé · \
                     MM mois 2 chiffres · M mois · dddd jour de la semaine · ddd abrégé · \
                     dd jour 2 chiffres · d jour",
                )
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
            palette
                .label(
                    "Exemples : « dd/MM/yyyy » → 03/11/2026 · « dddd d MMMM yyyy » → \
                     mardi 3 novembre 2026 · « MMMM yyyy » → novembre 2026. Tout le reste \
                     s'écrit tel quel : barres, points, lettres de votre choix.",
                )
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(10),
    ));

    corps
}

fn corps_couleurs<'a>(
    palette: Palette,
    reglages: &crate::mise_en_page::modele::MiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let mut corps = column![].spacing(20);

    corps = corps.push(section(
        palette,
        "Couleurs",
        column![
            ligne(
                palette,
                "Texte",
                "La couleur du texte composé, en hexadécimal.",
                controle_couleur(
                    palette,
                    &reglages.couleurs.texte,
                    Champ::CouleurTexte,
                    "#000000"
                )
            ),
            ligne(
                palette,
                "Gras",
                "La couleur du texte en gras — vide : celle du texte.",
                controle_couleur(palette, &reglages.couleurs.gras, Champ::CouleurGras, "vide")
            ),
            ligne(
                palette,
                "Italique",
                "La couleur du texte en italique — vide : celle du texte.",
                controle_couleur(
                    palette,
                    &reglages.couleurs.italique,
                    Champ::CouleurItalique,
                    "vide"
                )
            ),
            ligne(
                palette,
                "Titres",
                "La couleur des titres, en hexadécimal.",
                controle_couleur(
                    palette,
                    &reglages.couleurs.titres,
                    Champ::CouleurTitres,
                    "#000000"
                )
            ),
            ligne(
                palette,
                "Liens",
                "La couleur des liens et citations, en hexadécimal.",
                controle_couleur(
                    palette,
                    &reglages.couleurs.liens,
                    Champ::CouleurLiens,
                    "#0000ff"
                )
            ),
            ligne(
                palette,
                "Papier",
                "La couleur du papier, en hexadécimal.",
                controle_couleur(
                    palette,
                    &reglages.couleurs.fond,
                    Champ::CouleurFond,
                    "#ffffff"
                )
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
                controle_couleur(palette, &decor.couleur, Champ::DecorCouleur, "gray")
            ),
            ligne(
                palette,
                "Épaisseur",
                "La graisse du trait : `1pt`, `0.5pt`.",
                saisie_numerique(palette, &decor.epaisseur, Champ::DecorEpaisseur, "1pt")
            ),
            ligne(
                palette,
                "Rayon des coins",
                "L'arrondi : `4pt`, `0pt` pour un angle droit.",
                saisie_numerique(palette, &decor.rayon, Champ::DecorRayon, "4pt")
            ),
            ligne(
                palette,
                "Marge intérieure",
                "L'air entre la bordure et le contenu : `8pt`.",
                saisie_numerique(palette, &decor.marge, Champ::DecorMarge, "8pt")
            ),
            ligne(
                palette,
                "Remplissage",
                "La teinte de fond : `aucun`, une couleur — ou un dégradé, écrit en valeur Typst (gradient.linear(#a, #b)).",
                controle_couleur(
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
                    row![
                        choix(
                            GUILLEMETS_1,
                            &paire_lue(&typo.guillemets, 0),
                            Champ::TypoGuillemets1
                        ),
                        saisie(
                            palette,
                            &paire_lue(&typo.guillemets, 0),
                            Champ::TypoGuillemets1,
                            "«»"
                        )
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ),
                ligne(
                    palette,
                    "Niveau 2",
                    "Ce que la citation de niveau 1 contient — “ ” en français.",
                    row![
                        choix(
                            GUILLEMETS_2,
                            &paire_lue(&typo.guillemets, 1),
                            Champ::TypoGuillemets2
                        ),
                        saisie(
                            palette,
                            &paire_lue(&typo.guillemets, 1),
                            Champ::TypoGuillemets2,
                            "“”"
                        )
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ),
                ligne(
                    palette,
                    "Niveau 3",
                    "Et le suivant — ‘ ’ en français.",
                    row![
                        choix(
                            GUILLEMETS_3,
                            &paire_lue(&typo.guillemets, 2),
                            Champ::TypoGuillemets3
                        ),
                        saisie(
                            palette,
                            &paire_lue(&typo.guillemets, 2),
                            Champ::TypoGuillemets3,
                            "‘’"
                        )
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ),
                // L'aperçu de l'imbrication réelle des trois niveaux.
                palette
                    .label(format!(
                        "Aperçu : {}",
                        crate::mise_en_page::modele::apercu_guillemets(&typo.guillemets)
                    ))
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
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

/// Un onglet vertical : son pictogramme, son nom — et l'élu, marqué.
fn onglet_vertical<'a>(
    palette: Palette,
    pictogramme: impl Into<Element<'a, Message>>,
    nom: &'a str,
    index: usize,
    courant: usize,
) -> Element<'a, Message> {
    let actif = index == courant;

    button(
        row![pictogramme.into(), text(nom).size(13)]
            .spacing(9)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::MiseEnPagePanneau(index))
    .padding([8, 12])
    .width(Length::Fill)
    .style(move |_, _| button::Style {
        background: if actif {
            Some(palette.surface.into())
        } else {
            None
        },
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
            width: 1.5,
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    })
    .into()
}

/// L'onglet Bibliographie : le style des citations et de la liste finale.
///
/// Le choix se fait parmi les styles que le moteur embarqué connaît — la
/// vitrine est validée par lui, jamais tenue à la main — et « le document
/// décide » laisse la directive du fichier faire autorité.
fn corps_bibliographie(
    palette: Palette,
    dialogue: &crate::app::state::DialogueMiseEnPage,
) -> iced::widget::Column<'static, Message> {
    use crate::mise_en_page::modele::Champ;

    let reglages = &dialogue.reglages;

    /// Le choix qui laisse le document décider par sa propre directive.
    const DOCUMENT: &str = "Le document décide (sa directive)";

    let courant = reglages.valeur(Champ::BibliographieStyle);
    let courant = if courant.is_empty() {
        DOCUMENT.to_string()
    } else {
        courant
    };

    let styles = crate::typst::styles::styles_integres();
    let mut options = vec![DOCUMENT.to_string()];
    options.extend(styles.iter().map(|(nom, _, _)| nom.clone()));

    let type_style = styles
        .iter()
        .find(|(nom, _, _)| *nom == courant)
        .map(|(_, type_style, _)| match type_style {
            crate::typst::styles::TypeStyle::Notes => "en notes de bas de page",
            crate::typst::styles::TypeStyle::AuteurDate => "en auteur-date dans le texte",
            crate::typst::styles::TypeStyle::Numerique => "numéroté dans le texte",
        })
        .unwrap_or("fixé par le document");

    let mut corps = column![].spacing(20);

    corps = corps.push(
        palette
            .label("Comment les citations se composent, et à quoi ressemble la liste finale.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    // ---- Le style ----
    corps = corps.push(section(
        palette,
        "Le style",
        column![
            ligne(
                palette,
                "Style des citations",
                "Le style des renvois et de la bibliographie finale.",
                pick_list(options, Some(courant.clone()), |choisi: String| {
                    let valeur = if choisi == DOCUMENT {
                        String::new()
                    } else {
                        choisi
                    };

                    Message::MiseEnPageChamp(Champ::BibliographieStyle, valeur)
                })
                .width(Length::Fixed(170.0))
            ),
            row![
                button(
                    palette
                        .label("Importer un fichier .csl…")
                        .size(theme::jetons::DETAIL)
                )
                .on_press(Message::StyleCslImporter)
                .padding([5, 12]),
                palette
                    .label(format!("Rendu des références : {type_style}."))
                    .size(theme::jetons::DETAIL),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(10),
    ));

    // ---- L'aperçu : juste sous le style, pour voir ce que le choix
    // donne — rendu par le moteur même (R4) ----
    let apercu = dialogue
        .apercu_style
        .clone()
        .unwrap_or_else(|| "Le moteur refuse ce style : rien à prévisualiser.".to_string());

    corps = corps.push(section(
        palette,
        "Ce que cela donne",
        column![
            palette
                .label(format!(
                    "Rendu par le moteur du projet — style « {courant} »."
                ))
                .size(theme::jetons::DETAIL),
            text(apercu).size(theme::jetons::CORPS),
        ]
        .spacing(8),
    ));

    // bug-report-10 : la bibliographie comme on la veut — saut de page
    // avant, et titre de niveau 1.
    corps = corps.push(section(
        palette,
        "Placement de la bibliographie",
        column![
            case(
                "Un saut de page avant la bibliographie",
                reglages.bibliographie.saut_de_page,
                Champ::BiblioSautDePage
            ),
            case(
                "La bibliographie en titre de niveau 1",
                reglages.bibliographie.titre_un,
                Champ::BiblioTitreUn
            ),
        ]
        .spacing(8),
    ));

    // ---- Zotero : ses styles installés, choisis et copiés dans le
    // projet — Zotero n'est jamais modifié. Absents, ils le disent ----
    if dialogue.styles_zotero.is_empty() {
        corps = corps.push(section(
            palette,
            "Depuis Zotero",
            column![
                palette
                    .label("Aucun style installé (ou Zotero absent de cette machine).")
                    .size(theme::jetons::DETAIL)
            ]
            .spacing(8),
        ));
    } else {
        let mut titres: Vec<String> = dialogue
            .styles_zotero
            .iter()
            .map(|style| style.title.clone())
            .collect();
        titres.sort();

        let styles = dialogue.styles_zotero.clone();
        corps = corps.push(section(
            palette,
            "Depuis Zotero",
            column![
                palette
                    .label("Un style installé dans Zotero se copie dans le projet.")
                    .size(theme::jetons::DETAIL),
                pick_list(titres, None::<String>, move |titre: String| {
                    let id = styles
                        .iter()
                        .find(|style| style.title == titre)
                        .map(|style| style.id.clone())
                        .unwrap_or_default();

                    Message::StyleZoteroChoisi(id)
                })
                .width(Length::Fill)
            ]
            .spacing(8),
        ));
    }

    corps
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
    let bulle = container(text(note).size(theme::jetons::DETAIL).color(palette.text))
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

    // Deux colonnes de portions : l'étiquette à gauche, le contrôle qui
    // démarre toujours au même x (l'alignement du point 21) et se
    // resserre avec la fenêtre (le dépassement du point 20).
    row![
        container(etiquette).width(Length::FillPortion(2)),
        container(controle.into()).width(Length::FillPortion(3)),
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
/// Un champ de saisie. `numerique` ajoute les flèches d'incrémentation —
/// NORMS-04-01 : les flèches ne concernent que les valeurs numériques.
/// L'aperçu d'un schéma de numérotation — le défaut s'y montre comme
/// « 1 ».
fn apercu_ou_defaut(valeur: &str) -> String {
    if valeur.trim().is_empty() {
        "1".to_string()
    } else {
        crate::mise_en_page::modele::apercu_numerotation(valeur)
    }
}

fn saisie(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
) -> Element<'static, Message> {
    saisie_interne(palette, valeur, champ, exemple, false)
}

/// Un champ numérique : flèches d'incrémentation et molette.
fn saisie_numerique(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
) -> Element<'static, Message> {
    saisie_interne(palette, valeur, champ, exemple, true)
}

/// Le champ de texte **canonique** (bug-report-8, T8.2) : même coque —
/// même police, même marge — partout où une valeur se tape.
fn champ_texte<'a, F>(
    exemple: &str,
    valeur: &str,
    largeur: Length,
    sur_saisie: F,
) -> Element<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    text_input(exemple, valeur)
        .on_input(sur_saisie)
        .width(largeur)
        .padding(6)
        .size(12)
        .into()
}

fn saisie_interne(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
    numerique: bool,
) -> Element<'static, Message> {
    let champ_molette = champ;
    let champ_haut = champ;
    let champ_bas = champ;
    let fleches: Element<'static, Message> = if numerique {
        iced::widget::column![
            button(text("▲").size(8))
                .on_press(Message::MiseEnPageMolette(champ_haut, 1))
                .padding([1, 3])
                .style(move |_, _| button::Style {
                    background: None,
                    text_color: palette.text_muted,
                    ..Default::default()
                }),
            button(text("▼").size(8))
                .on_press(Message::MiseEnPageMolette(champ_bas, -1))
                .padding([1, 3])
                .style(move |_, _| button::Style {
                    background: None,
                    text_color: palette.text_muted,
                    ..Default::default()
                }),
        ]
        .spacing(0)
        .into()
    } else {
        iced::widget::Space::new().width(0.0).into()
    };
    let champ = champ_texte(exemple, valeur, Length::Fixed(140.0), move |texte| {
        Message::MiseEnPageChamp(champ, texte)
    });
    let contenu: Element<'static, Message> = if numerique {
        iced::widget::mouse_area(
            row![champ, fleches]
                .spacing(2)
                .align_y(iced::Alignment::Center),
        )
        .on_scroll(move |delta| {
            let pas = match delta {
                iced::mouse::ScrollDelta::Lines { y, .. }
                | iced::mouse::ScrollDelta::Pixels { y, .. } => y,
            };
            if pas > 0.0 {
                Message::MiseEnPageMolette(champ_molette, 1)
            } else {
                Message::MiseEnPageMolette(champ_molette, -1)
            }
        })
        .into()
    } else {
        row![champ].align_y(iced::Alignment::Center).into()
    };
    contenu
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

/// Le choix de la langue des dates : le libellé écrit le code.
fn choix_langue(valeur: &str, champ: Champ) -> Element<'static, Message> {
    let libelles: Vec<String> = LANGUES_DATES
        .iter()
        .map(|(libelle, _)| libelle.to_string())
        .collect();
    let courant = LANGUES_DATES
        .iter()
        .find(|(_, code)| *code == valeur)
        .map(|(libelle, _)| libelle.to_string())
        .unwrap_or_else(|| "français".to_string());

    pick_list(libelles, Some(courant), move |choisi: String| {
        let code = LANGUES_DATES
            .iter()
            .find(|(libelle, _)| *libelle == choisi)
            .map(|(_, code)| code.to_string())
            .unwrap_or_else(|| "fr".to_string());
        Message::MiseEnPageChamp(champ, code)
    })
    .width(Length::Fixed(170.0))
    .into()
}

/// Un choix avec une entrée « défaut » : elle mappe sur la valeur vide,
/// qui laisse les défauts du moteur parler.
fn choix_defaut_vide(
    options: &[&'static str],
    valeur: &str,
    champ: Champ,
) -> Element<'static, Message> {
    let mut choix: Vec<String> = vec!["défaut".to_string()];
    choix.extend(options.iter().map(|option| option.to_string()));
    let affiche = if valeur.trim().is_empty() {
        "défaut".to_string()
    } else {
        valeur.to_string()
    };

    pick_list(choix, Some(affiche), move |choisi: String| {
        let valeur = if choisi == "défaut" {
            String::new()
        } else {
            choisi
        };
        Message::MiseEnPageChamp(champ, valeur)
    })
    .width(Length::Fixed(170.0))
    .into()
}

/// Le choix d'une police : un vrai menu déroulant, rechercheable — la
/// liste complète se parcourt et la recherche la filtre (bug-report-9,
/// T1.3). L'aperçu montre la police choisie, composée telle quelle.
///
/// La liste vient du système et du livre de polices que Typst interroge
/// lui-même — ce sont les noms qu'il composerait. Taper filtre et
/// inscrit au passage la valeur tapée (« une saisie valide se garde telle
/// quelle ») ; choisir dans le menu pose le nom exact.
fn choix_police<'a>(
    valeur: &str,
    champ: Champ,
    combo: &'a iced::widget::combo_box::State<String>,
) -> Element<'a, Message> {
    let selection = valeur.to_string();
    let pour_apercu = valeur.to_string();

    let menu = combo_box(
        combo,
        "nom de la police",
        Some(&selection),
        move |famille: String| Message::MiseEnPageChamp(champ, famille),
    )
    .on_input(move |texte| Message::MiseEnPageChamp(champ, texte))
    .padding([4, 8])
    .width(Length::Fixed(220.0));

    let apercu = if pour_apercu.is_empty() {
        text("Aa Bb Cc — le texte se compose ainsi").size(14)
    } else {
        text("Aa Bb Cc — le texte se compose ainsi").font(iced::Font {
            family: iced::font::Family::Name(crate::theming::intern_family(&pour_apercu)),
            ..Default::default()
        })
    };

    row![menu, apercu]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .into()
}

/// La pastille d'une couleur, cliquable : elle ouvre la roue pour ce
/// champ-là (bug-report-8, T6.1).
///
/// Elle ne se teinte que d'une couleur **valide** : une case vide est une
/// case vide, pas un carré noir qui mentirait sur ce qui sera composé.
fn pastille(palette: Palette, valeur: &str, champ: Champ) -> Element<'static, Message> {
    let couleur = couleur_depuis(valeur).unwrap_or(iced::Color::TRANSPARENT);

    button(
        container(Space::new())
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(18.0))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(couleur.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
    )
    .padding(0)
    .on_press(Message::PickerCouleurPourReglage(champ))
    .into()
}

/// Le contrôle de couleur, **partout le même** : une pastille cliquable
/// et un champ texte hexadécimal — la valeur se tape, se colle, ou se
/// choisit à la roue, et les deux se suivent des deux côtés
/// (bug-report-8, T6.2).
fn controle_couleur(
    palette: Palette,
    valeur: &str,
    champ: Champ,
    exemple: &str,
) -> Element<'static, Message> {
    row![
        pastille(palette, valeur, champ),
        saisie(palette, valeur, champ, exemple),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Une couleur `#rrggbb` ou `#rgb`, en couleur d'écran.
/// Une couleur, par son nom ou son hexadécimal — le nuancier la suit.
pub(crate) fn couleur_depuis(valeur: &str) -> Option<iced::Color> {
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
