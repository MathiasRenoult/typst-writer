//! Le formulaire générique d'une balise simple.
//!
//! Il ne remplace pas les panneaux des graphiques ou des callouts : il
//! couvre les balises dont les champs tiennent en une ligne. Le même schéma
//! produit le texte canonique inséré, donc l'aperçu ne peut pas mentir.

use iced::widget::{Space, button, column, container, row, scrollable, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::app::palette::{BaliseDialogue, TagId};
use crate::app::state::State;
use crate::ui::theme::{self, Palette};

pub fn view(state: &State) -> Element<'_, Message> {
    let Some(dialogue) = state.balise_dialogue.as_ref() else {
        return Space::new().into();
    };
    let palette: Palette = state.resolved.palette;

    let mut champs = column![].spacing(8);
    for (rang, champ) in dialogue.champs.iter().enumerate() {
        champs = champs.push(
            column![
                palette
                    .label(champ.libelle)
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
                text_input(champ.libelle, &champ.valeur)
                    .id(BaliseDialogue::id_champ(rang))
                    .on_input(move |valeur| Message::BaliseChamp(champ.cle.to_string(), valeur))
                    .padding(6)
                    .width(Length::Fill),
            ]
            .spacing(2),
        );
    }

    match dialogue.tag {
        TagId::Couleur => {
            champs = champs.push(choix_couleur(palette));
        }
        TagId::Date => {
            champs = champs.push(choix_date(palette));
        }
        TagId::Glossaire => {
            champs = champs.push(termes_glossaire(state, palette));
        }
        _ => {}
    }

    let apercu = match dialogue.texte() {
        Ok(texte) => palette
            .label(format!("Aperçu : {texte}"))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Err(motif) => palette
            .label(motif)
            .size(theme::jetons::DETAIL)
            .color(palette.danger),
    };

    let carte = container(
        column![
            palette
                .label(titre(dialogue.tag))
                .size(theme::jetons::TITRE),
            champs,
            apercu,
            row![
                button(
                    palette
                        .label(if dialogue.remplacement.is_some() {
                            "Remplacer"
                        } else {
                            "Insérer"
                        })
                        .size(theme::jetons::DETAIL),
                )
                .on_press(Message::BaliseInserer)
                .padding([5, 12]),
                button(palette.label("Annuler").size(theme::jetons::DETAIL))
                    .on_press(Message::BaliseAnnuler)
                    .padding([5, 12]),
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .width(Length::Fixed(500.0))
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

fn titre(tag: TagId) -> &'static str {
    match tag {
        TagId::Couleur => "Teinter un passage",
        TagId::Date => "Écrire une date",
        TagId::Glossaire => "Appeler un terme du glossaire",
        TagId::Meta => "Appeler une métadonnée",
        TagId::Lien => "Écrire un lien",
        TagId::Brut => "Modifier la balise",
        _ => "Compléter la balise",
    }
}

/// Les couleurs rapides du formulaire : un clic remplit le champ, la roue
/// reste disponible pour une teinte exacte.
fn choix_couleur(palette: Palette) -> Element<'static, Message> {
    let mut pastilles = row![].spacing(6);
    for (nom, valeur) in crate::ui::picker_couleur::RAPIDES {
        let couleur = crate::theming::HexColor::parse(valeur)
            .ok()
            .map(|couleur| couleur.0);
        pastilles = pastilles.push(
            iced::widget::tooltip(
                button(
                    container(
                        Space::new()
                            .width(Length::Fixed(18.0))
                            .height(Length::Fixed(18.0)),
                    )
                    .style(move |_| container::Style {
                        background: couleur.map(iced::Background::from),
                        border: iced::Border {
                            width: 1.0,
                            color: palette.border,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }),
                )
                .on_press(Message::BaliseChamp(
                    "couleur".to_string(),
                    valeur.to_string(),
                ))
                .padding(3),
                palette.label(nom).size(theme::jetons::DETAIL),
                iced::widget::tooltip::Position::Top,
            )
            .gap(4),
        );
    }

    column![
        palette
            .label("Couleurs rapides")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        pastilles,
        button(
            palette
                .label("Choisir une couleur précise…")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::PickerCouleurPourBalise)
        .padding([4, 8]),
    ]
    .spacing(5)
    .into()
}

/// Les formes de date les plus courantes : le champ garde la main pour une
/// date ou un format sur mesure.
fn choix_date(palette: Palette) -> Element<'static, Message> {
    const FORMES: [(&str, &str); 9] = [
        ("Date", "1789.07.14"),
        ("Date longue", "1789.07.14 \"d MMMM yyyy\""),
        ("Date complète", "1789.07.14 \"dddd d MMMM yyyy\""),
        ("Mois et année", "1789.07.14 \"MMMM yyyy\""),
        ("Aujourd'hui", "aujourd'hui"),
        ("Maintenant", "maintenant"),
        ("Siècle", "XIXs"),
        ("Siècle long", "XIXs?"),
        ("Avant J.-C.", "-IVs?"),
    ];

    let mut boutons = column![].spacing(4);
    for groupe in FORMES.chunks(3) {
        let mut ligne = row![].spacing(6);
        for (libelle, spec) in groupe {
            ligne = ligne.push(
                button(palette.label(*libelle).size(theme::jetons::DETAIL))
                    .on_press(Message::BaliseChamp(
                        "spec".to_string(),
                        (*spec).to_string(),
                    ))
                    .padding([4, 8]),
            );
        }
        boutons = boutons.push(ligne);
    }

    column![
        palette
            .label("Formes courantes — ou écris ta propre date ci-dessus")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        boutons,
    ]
    .spacing(5)
    .into()
}

/// Les termes réellement présents dans le glossaire du projet : un clic
/// remplit la clé, sans avoir à la retenir.
fn termes_glossaire<'a>(state: &'a State, palette: Palette) -> Element<'a, Message> {
    let mut liste = column![].spacing(3);

    if state.plugins.glossaire.entrees.is_empty() {
        liste = liste.push(
            palette
                .label("Aucun terme actif dans ce projet.")
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    } else {
        for entree in state.plugins.glossaire.entrees.iter().take(12) {
            let forme = entree
                .long
                .as_deref()
                .or(entree.short.as_deref())
                .unwrap_or("");
            let libelle = if forme.is_empty() {
                entree.key.clone()
            } else {
                format!("{} — {}", entree.key, forme)
            };
            liste = liste.push(
                button(palette.label(libelle).size(theme::jetons::DETAIL))
                    .on_press(Message::BaliseChamp("cle".to_string(), entree.key.clone()))
                    .padding([3, 6])
                    .width(Length::Fill),
            );
        }
    }

    column![
        palette
            .label("Termes du glossaire")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        scrollable(liste).height(Length::Fixed(120.0)),
    ]
    .spacing(4)
    .into()
}
