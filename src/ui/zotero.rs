//! Le panneau de liaison à Zotero.
//!
//! Un projet se lie à une collection Zotero : on la choisit ici, parmi
//! toutes celles que Zotero connaît. Trois états, pas un de plus — la
//! lecture en cours, le choix, et ce qui n'est pas allé. Une erreur se dit
//! et se répare (« Réessayer »), jamais avalée.
//!
//! Le panneau se conduit au clavier : `Échap` ferme.

use crate::app::state::LiaisonZotero;
use crate::app::Message;
use crate::ui::theme::{self, Palette};
use crate::zotero::client::Collection;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

/// La largeur du panneau.
const LARGEUR: f32 = 460.0;

/// La hauteur de la liste : elle défile, jamais la fenêtre.
const HAUTEUR: f32 = 320.0;

/// La couche de liaison : le panneau quand il est ouvert, rien sinon.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(liaison) = state.liaison.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    fenetre(liaison, state.resolved.palette)
}

/// Le panneau, dans l'état où il est.
fn fenetre<'a>(liaison: &'a LiaisonZotero, palette: Palette) -> Element<'a, Message> {
    let entete = row![
        palette.label("Lier à Zotero").size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        button(text("Fermer").size(12))
            .on_press(Message::ZoteroLiaisonFermer)
            .padding([6, 10]),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let corps: Element<'a, Message> = match liaison {
        LiaisonZotero::Chargement => container(
            palette
                .label("Lecture des collections…")
                .size(theme::jetons::CORPS),
        )
        .padding(12)
        .into(),

        LiaisonZotero::Erreur(motif) => column![
            container(
                column![
                    palette
                        .label("La liaison n'a pas pu se faire")
                        .size(theme::jetons::CORPS),
                    palette
                        .label(motif)
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .spacing(6)
            )
            .padding(12),
            row![
                button(palette.label("Réessayer").size(theme::jetons::CORPS))
                    .on_press(Message::ZoteroLierOuvrir)
                    .padding([6, 12]),
                button(palette.label("Fermer").size(theme::jetons::CORPS))
                    .on_press(Message::ZoteroLiaisonFermer)
                    .padding([6, 12]),
            ]
            .spacing(theme::jetons::PAS),
        ]
        .spacing(theme::jetons::PAS)
        .into(),

        LiaisonZotero::Choix(collections) => liste(collections, palette),
    };

    let pied = row![
        palette
            .label("La collection choisie devient la bibliographie du projet — lue, jamais écrite.")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        Space::new().width(Length::Fill),
        palette
            .label("Échap : fermer")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let fenetre = container(column![entete, corps, pied].spacing(theme::jetons::GRAND_PAS))
        .width(Length::Fixed(LARGEUR))
        .padding(16)
        .style(theme::panneau(palette));

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// La liste des collections, avec leur place dans l'arbre.
fn liste<'a>(collections: &'a [Collection], palette: Palette) -> Element<'a, Message> {
    let noms: Noms = collections
        .iter()
        .map(|collection| {
            (
                collection.cle.as_str(),
                Entree {
                    nom: collection.nom.as_str(),
                    parent: collection.parent.as_deref(),
                },
            )
        })
        .collect();

    let mut colonne = column![].spacing(2);

    for collection in collections {
        let chemin = chemin_de(collection, &noms);
        let nom = collection.nom.clone();
        let cle = collection.cle.clone();

        let ligne = row![
            palette.label(nom.clone()).size(theme::jetons::CORPS),
            Space::new().width(Length::Fill),
            palette
                .label(format!("{} notice(s)", collection.notices))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(theme::jetons::PAS)
        .align_y(Alignment::Center);

        let rang = column![
            ligne,
            palette
                .label(chemin)
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(1);

        colonne = colonne.push(
            button(rang)
                .on_press(Message::ZoteroChoisir(cle, nom))
                .width(Length::Fill)
                .padding([6, 8])
                .style(move |_, _| style_ligne(palette)),
        );
    }

    scrollable(colonne.height(Length::Fixed(HAUTEUR))).into()
}

/// Une ligne de collection : un bouton qui ne se voit pas, jusqu'au geste.
fn style_ligne(palette: Palette) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: None,
        text_color: palette.text,
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 1.0,
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

/// La place d'une collection : « Thèse / Chapitre 2 ».
fn chemin_de(collection: &Collection, noms: &Noms) -> String {
    let mut morceaux = Vec::new();
    let mut parent = collection.parent.as_deref();

    while let Some(cle) = parent {
        let Some(entree) = noms.get(cle) else {
            break;
        };
        morceaux.push(entree.nom);
        parent = entree.parent;
    }

    morceaux.reverse();

    if morceaux.is_empty() {
        "bibliothèque".to_string()
    } else {
        morceaux.join(" / ")
    }
}

/// Ce qu'il faut pour remonter l'arbre : chaque collection par sa clé.
struct Entree<'a> {
    nom: &'a str,
    parent: Option<&'a str>,
}

type Noms<'a> = std::collections::HashMap<&'a str, Entree<'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    /// La place d'une collection se lit à partir de ses parents.
    #[test]
    fn le_chemin_d_une_collection_se_lit() {
        let racine = Collection {
            cle: "AAAA".to_string(),
            nom: "Thèse".to_string(),
            parent: None,
            notices: 0,
        };
        let enfant = Collection {
            cle: "BBBB".to_string(),
            nom: "Chapitre 2".to_string(),
            parent: Some("AAAA".to_string()),
            notices: 3,
        };

        let mut noms = Noms::new();
        noms.insert("AAAA", Entree { nom: "Thèse", parent: None });
        noms.insert(
            "BBBB",
            Entree {
                nom: "Chapitre 2",
                parent: Some("AAAA"),
            },
        );

        assert_eq!(chemin_de(&racine, &noms), "bibliothèque");
        assert_eq!(chemin_de(&enfant, &noms), "Thèse");
    }
}
