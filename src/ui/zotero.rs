//! Le panneau de liaison à Zotero.
//!
//! Un projet se lie à une collection Zotero : on la choisit ici, parmi
//! toutes celles que Zotero connaît. Trois états, pas un de plus — la
//! lecture en cours, le choix, et ce qui n'est pas allé. Une erreur se dit
//! et se répare (« Réessayer »), jamais avalée.
//!
//! Le panneau se conduit au clavier : `Échap` ferme.

use crate::app::Message;
use crate::app::state::LiaisonZotero;
use crate::ui::theme::{self, Palette};
use crate::zotero::client::Collection;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
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

    crate::ui::voile(fenetre(
        liaison,
        state.resolved.palette,
        &state.zotero_recherche,
        state.project.zotero.is_some(),
    ))
}

/// Le panneau, dans l'état où il est.
fn fenetre<'a>(
    liaison: &'a LiaisonZotero,
    palette: Palette,
    recherche: &'a str,
    remplacement: bool,
) -> Element<'a, Message> {
    let (titre, explication) = if remplacement {
        (
            "Changer de collection Zotero",
            "La nouvelle collection remplace la bibliographie Zotero du projet — lue, jamais écrite.",
        )
    } else {
        (
            "Lier à Zotero",
            "La collection choisie devient la bibliographie du projet — lue, jamais écrite.",
        )
    };

    let entete = row![
        palette.label(titre).size(theme::jetons::TITRE),
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

        LiaisonZotero::Erreur { motif, reglable } => {
            let mut actions = row![].spacing(theme::jetons::PAS);

            // Le réglage d'API est le seul désordre qu'on puisse réparer
            // soi-même : le geste est là, sous la main du nouvel arrivant.
            if *reglable {
                actions = actions.push(
                    button(
                        palette
                            .label("Activer l'API locale")
                            .size(theme::jetons::CORPS),
                    )
                    .on_press(Message::ZoteroActiverApi)
                    .padding([6, 12]),
                );
            }

            actions = actions.push(
                button(palette.label("Réessayer").size(theme::jetons::CORPS))
                    .on_press(Message::ZoteroLierOuvrir)
                    .padding([6, 12]),
            );
            actions = actions.push(
                button(palette.label("Fermer").size(theme::jetons::CORPS))
                    .on_press(Message::ZoteroLiaisonFermer)
                    .padding([6, 12]),
            );

            column![
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
                actions,
            ]
            .spacing(theme::jetons::PAS)
            .into()
        }

        LiaisonZotero::Choix(collections) => liste(collections, palette, recherche),
    };

    let pied = row![
        palette
            .label(explication)
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

/// La liste des collections : **l'arbre**, respecté — les enfants se
/// posent sous leurs parents, indentés — et cherchable (bug 1 du
/// rapport 2). Une bibliothèque de collectionneur a des dizaines de
/// collections : une liste à plat qui les masque en ombre est une liste
/// qui ment.
fn liste<'a>(
    collections: &'a [Collection],
    palette: Palette,
    recherche: &str,
) -> Element<'a, Message> {
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

    // Le filtre cherche le nom, ou le chemin entier.
    let filtre = recherche.trim().to_lowercase();

    // L'arbre : les racines d'abord, puis les enfants sous leurs parents.
    let mut par_parent: std::collections::BTreeMap<Option<&str>, Vec<&Collection>> =
        std::collections::BTreeMap::new();

    for collection in collections {
        par_parent
            .entry(collection.parent.as_deref())
            .or_default()
            .push(collection);
    }

    fn pousser<'a>(
        rangs: &mut Vec<Element<'a, Message>>,
        par_parent: &std::collections::BTreeMap<Option<&'a str>, Vec<&'a Collection>>,
        parent: Option<&'a str>,
        profondeur: usize,
        palette: Palette,
        noms: &Noms<'a>,
        filtre: &str,
    ) {
        let Some(enfants) = par_parent.get(&parent) else {
            return;
        };

        for collection in enfants {
            let passe = filtre.is_empty()
                || collection.nom.to_lowercase().contains(filtre)
                || chemin_de(collection, noms).to_lowercase().contains(filtre);

            // Le filtre garde aussi les parents des collections trouvées :
            // un enfant trouvé se voit avec sa place.
            if passe {
                let nom = collection.nom.clone();
                let cle = collection.cle.clone();

                let ligne = row![
                    Space::new().width(Length::Fixed(14.0 * profondeur as f32)),
                    palette.label(nom.clone()).size(theme::jetons::CORPS),
                    Space::new().width(Length::Fill),
                    palette
                        .label(format!("{} notice(s)", collection.notices))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .spacing(theme::jetons::PAS)
                .align_y(Alignment::Center);

                rangs.push(
                    button(ligne)
                        .on_press(Message::ZoteroChoisir(cle, nom))
                        .width(Length::Fill)
                        .padding([6, 8])
                        .style(move |_, _| style_ligne(palette))
                        .into(),
                );
            }

            pousser(
                rangs,
                par_parent,
                Some(collection.cle.as_str()),
                profondeur + 1,
                palette,
                noms,
                filtre,
            );
        }
    }

    let mut rangs = Vec::new();
    pousser(&mut rangs, &par_parent, None, 0, palette, &noms, &filtre);

    let mut colonne: iced::widget::Column<'_, Message> = column![].spacing(2);

    for rang in rangs {
        colonne = colonne.push(rang);
    }

    let corps = column![
        text_input("chercher une collection", recherche)
            .on_input(Message::ZoteroRecherche)
            .padding(5),
        scrollable(colonne).height(Length::Fixed(HAUTEUR - 34.0)),
    ]
    .spacing(6);

    corps.into()
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
        noms.insert(
            "AAAA",
            Entree {
                nom: "Thèse",
                parent: None,
            },
        );
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
