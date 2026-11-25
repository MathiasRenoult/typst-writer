//! Le panneau d'insertion d'une personne citée.
//!
//! Une seule liste cherchable pour choisir la personne ; ses champs de
//! registre se remplissent ici ; la balise n'écrit que la clé, avec une
//! expression d'affichage facultative après `|`.

use crate::app::Message;
use crate::app::state::State;
use crate::plugins::personnes::PersonnePanneau;
use crate::ui::theme::{self, Palette};
use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length};

/// La largeur du panneau.
const LARGEUR: f32 = 560.0;

/// La hauteur de la liste déroulante.
const HAUTEUR_LISTE: f32 = 240.0;

/// Les expressions proposées par la liste ; la valeur `None` est le nom
/// simple par défaut.
const EXPRESSIONS: [&str; 5] = [
    "Selon les normes (défaut)",
    "Nom simple",
    "Nom complet",
    "Alias",
    "Champs personnalisés",
];

/// La couche : le panneau quand il est ouvert, rien sinon.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(panneau) = state.plugins.personnes.panneau.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(
        panneau,
        &state.plugins.personnes.entrees,
        state.resolved.palette,
        state.window_size,
    )
}

fn fenetre<'a>(
    panneau: &'a PersonnePanneau,
    entrees: &'a [crate::project::personnes::EntreePersonne],
    palette: Palette,
    fenetre: iced::Size,
) -> Element<'a, Message> {
    let recherche = text_input("Chercher une personne", &panneau.recherche)
        .id(id_recherche())
        .on_input(Message::PersonnePanneauRecherche)
        .on_submit(Message::PersonnePanneauValiderRecherche)
        .padding(6)
        .width(Length::Fill);

    let options = panneau.options(entrees);
    let mut resultats = column![].spacing(2);
    if options.is_empty() {
        resultats = resultats.push(
            palette
                .label("Aucune personne ne correspond.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    } else {
        for option in &options {
            resultats = resultats.push(
                button(
                    palette
                        .label(option.libelle.clone())
                        .size(theme::jetons::DETAIL),
                )
                .on_press(Message::PersonnePanneauChoisir(option.cle.clone()))
                .padding([5, 8])
                .width(Length::Fill),
            );
        }
    }
    // La liste garde une hauteur courte quand le filtre ne laisse que
    // quelques entrées ; au-delà, elle défile.
    let hauteur_liste = ((options.len() as f32 * 28.0) + 8.0).clamp(44.0, HAUTEUR_LISTE);
    let liste = container(scrollable(resultats).height(Length::Fixed(hauteur_liste)))
        .width(Length::Fill)
        .padding(4)
        .style(theme::panneau(palette));

    let selection = panneau
        .choix_courant(entrees)
        .map(|choix| format!("Sélection : {}", choix.libelle))
        .unwrap_or_else(|| "Aucune personne sélectionnée.".to_string());

    let gestes = row![
        button(
            palette
                .label("Nouvelle personne")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::PersonnePanneauNouvelle)
        .padding([5, 10]),
        palette
            .label("La balise ne retient que la clé ; l'expression est facultative.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut formulaire = column![].spacing(6);

    // La clé n'est saisie qu'à la création : ensuite, elle est déjà citée
    // dans les textes et ne doit pas bouger.
    if panneau.edition.creation {
        formulaire = formulaire.push(champ("Clé", &panneau.edition.cle, "cle", palette));
    } else {
        formulaire = formulaire.push(
            palette
                .label(format!("Clé — citée : {{{{p {}}}}}", panneau.edition.cle))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    formulaire = formulaire.push(champ("Nom complet", &panneau.edition.nom, "nom", palette));
    formulaire = formulaire.push(champ(
        "Nom simple / nom court",
        &panneau.edition.nom_simple,
        "nom_simple",
        palette,
    ));
    formulaire = formulaire.push(champ(
        "Alias / nom de plume",
        &panneau.edition.alias,
        "alias",
        palette,
    ));
    formulaire = formulaire.push(champ(
        "Clé de classement",
        &panneau.edition.tri,
        "tri",
        palette,
    ));
    formulaire = formulaire.push(champ(
        "Naissance",
        &panneau.edition.naissance,
        "naissance",
        palette,
    ));
    formulaire = formulaire.push(champ("Mort", &panneau.edition.mort, "mort", palette));
    formulaire = formulaire.push(champ(
        "Dates libres (repli)",
        &panneau.edition.dates,
        "dates",
        palette,
    ));
    formulaire = formulaire.push(champ(
        "Qualité",
        &panneau.edition.qualite,
        "qualite",
        palette,
    ));
    formulaire = formulaire.push(champ(
        "Variantes cherchables",
        &panneau.edition.variantes,
        "variantes",
        palette,
    ));

    let expression = expression_libelle(&panneau.expression);
    let options_expression = EXPRESSIONS
        .iter()
        .map(|expression| expression.to_string())
        .collect::<Vec<_>>();
    let expression_actuelle = panneau.expression.clone();
    let choix_expression = pick_list(
        options_expression,
        Some(expression.to_string()),
        move |choix: String| {
            if choix == "Champs personnalisés" {
                Message::PersonnePanneauExpression(expression_actuelle.clone())
            } else {
                Message::PersonnePanneauExpression(expression_code(&choix).to_string())
            }
        },
    )
    .width(Length::Fixed(240.0));

    let expression_libre = text_input("nom+dates, simple+naissance+mort…", &panneau.expression)
        .on_input(Message::PersonnePanneauExpression)
        .padding(6)
        .width(Length::Fill);

    let apercu = panneau
        .texte()
        .unwrap_or_else(|erreur| format!("— {erreur}"));

    let pied = row![
        button(
            palette
                .label("Enregistrer la fiche")
                .size(theme::jetons::DETAIL)
        )
        .on_press(Message::PersonnePanneauEnregistrer)
        .padding([6, 12]),
        button(palette.label("Fermer").size(theme::jetons::DETAIL))
            .on_press(Message::PersonnePanneauFermer)
            .padding([6, 12]),
        Space::new().width(Length::Fill),
        button(palette.label("Insérer").size(theme::jetons::CORPS))
            .on_press(Message::PersonnePanneauInserer)
            .padding([6, 12]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let corps = column![
        palette
            .label("Insérer une personne")
            .size(theme::jetons::TITRE),
        recherche,
        liste,
        palette
            .label(selection)
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        gestes,
        formulaire,
        palette.label("Affichage").size(theme::jetons::SAILLANT),
        row![
            palette
                .label("Expression")
                .size(theme::jetons::DETAIL)
                .width(Length::Fill),
            choix_expression,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        expression_libre,
        palette
            .label(
                "Formes : complet, simple, alias. Champs : nom, naissance, \
                 mort, dates, qualite, séparés par +. auto applique les règles \
                 de première mention.",
            )
            .size(theme::jetons::CHAPEAU)
            .color(palette.text_muted),
        palette
            .label("Aperçu de la balise")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        text(apercu).size(theme::jetons::DETAIL),
        pied,
    ]
    .spacing(10);

    let hauteur = (fenetre.height - 32.0).clamp(360.0, 680.0);

    let carte = container(scrollable(corps).height(Length::Fill))
        .width(Length::Fixed(LARGEUR))
        .height(Length::Fixed(hauteur))
        .padding(18)
        .style(theme::panneau(palette));

    crate::ui::voile(
        container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}

fn champ<'a>(
    libelle: &'a str,
    valeur: &'a str,
    cle: &'static str,
    palette: Palette,
) -> Element<'a, Message> {
    let champ = text_input(libelle, valeur)
        .on_input(move |texte| Message::PersonnePanneauChamp(cle.to_string(), texte))
        .on_submit(Message::PersonnePanneauEnregistrer)
        .padding(6)
        .width(Length::Fill);

    row![
        palette
            .label(libelle)
            .size(theme::jetons::DETAIL)
            .width(Length::Fixed(150.0)),
        champ,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn expression_libelle(expression: &str) -> &'static str {
    match expression.trim() {
        "" | "auto" => "Selon les normes (défaut)",
        "simple" => "Nom simple",
        "complet" => "Nom complet",
        "alias" => "Alias",
        _ => "Champs personnalisés",
    }
}

fn expression_code(libelle: &str) -> &'static str {
    match libelle {
        "Nom simple" => "simple",
        "Nom complet" => "complet",
        "Alias" => "alias",
        "Champs personnalisés" => "",
        _ => "",
    }
}

/// L'identifiant du champ de recherche, pour lui donner le clavier à
/// l'ouverture du panneau.
pub fn id_recherche() -> iced::widget::Id {
    iced::widget::Id::new("personne-panneau-recherche")
}
