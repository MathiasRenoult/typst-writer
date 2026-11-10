//! La fenêtre de projet : créer, ou renommer.
//!
//! Un champ, une action, et `Échap` qui annule — le panneau de gauche
//! garde un seul bouton, et c'est ici que le nom s'écrit.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, pick_list, row, text_input};
use iced::{Element, Length};

/// L'identifiant du champ du nom : la fenêtre s'ouvre dessus.
pub fn id_du_nom() -> iced::widget::Id {
    iced::widget::Id::new("projet-nom")
}

/// La fenêtre, quand un projet attend son nom.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(dialogue) = state.projet_dialogue.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;
    let creation = dialogue.cible.is_none();

    // Le accueil : quelques mots qui donnent envie, pas un formulaire
    // qui attend (demande de l'auteur : un onglet plus chaleureux).
    let mut corps = column![
        palette
            .label(if creation {
                "Un nouveau projet"
            } else {
                "Renommer le projet"
            })
            .size(theme::jetons::TITRE),
        palette
            .label(if creation {
                "Un document neuf, à votre image. Donnez-lui un nom — le reste se règle plus tard, quand vous le voudrez."
            } else {
                "Un nouveau nom, et le projet continue exactement comme avant."
            })
            .size(theme::jetons::DETAIL),
    ]
    .spacing(6);

    corps = corps.push(Space::new().height(6));
    corps = corps.push(
        palette
            .label("Comment s'appelle votre projet ?")
            .size(theme::jetons::DETAIL),
    );
    corps = corps.push(
        text_input("par exemple : ma thèse", &dialogue.nom)
            .id(id_du_nom())
            .on_input(Message::ProjetNom)
            .on_submit(Message::ProjetValider),
    );

    if creation {
        corps = corps.push(Space::new().height(4));
        corps = corps.push(palette.label("Il naîtra de…").size(theme::jetons::DETAIL));

        let mut bases: Vec<String> = vec!["Projet vide".to_string()];
        bases.extend(dialogue.bases.iter().cloned());
        let choisie = if dialogue.base.trim().is_empty() {
            Some("Projet vide".to_string())
        } else {
            Some(dialogue.base.clone())
        };
        corps = corps.push(
            pick_list(bases, choisie, |choix| {
                if choix == "Projet vide" {
                    Message::ProjetBase(String::new())
                } else {
                    Message::ProjetBase(choix)
                }
            })
            .width(Length::Fixed(260.0)),
        );

        corps = corps.push(Space::new().height(4));
        corps = corps.push(
            palette
                .label("Ce projet utilisera…")
                .size(theme::jetons::DETAIL),
        );
        for desc in crate::base::valeurs::PLUGINS {
            let actif = dialogue.fonctionnalites.actif(desc.cle);
            let nom = desc.cle;
            let libelle = desc.nom;
            corps = corps.push(
                iced::widget::checkbox(actif)
                    .label(libelle)
                    .on_toggle(move |v| Message::ProjetFonctionnalite(nom.to_string(), v)),
            );
        }
    }

    corps = corps.push(Space::new().height(6));
    corps = corps.push(
        palette
            .label("Pour l'entête et les citations")
            .size(theme::jetons::DETAIL),
    );
    corps = corps.push(
        text_input("titre du document", &dialogue.titre)
            .on_input(|v| Message::ProjetMeta("titre".to_string(), v))
            .on_submit(Message::ProjetValider),
    );
    corps = corps.push(
        text_input("votre nom", &dialogue.auteur)
            .on_input(|v| Message::ProjetMeta("auteur".to_string(), v))
            .on_submit(Message::ProjetValider)
            .padding(6)
            .width(Length::Fixed(260.0)),
    );
    corps = corps.push(
        pick_list(
            crate::project::noms_de_normes(),
            Some(dialogue.norme.clone()),
            Message::ProjetNorme,
        )
        .width(Length::Fixed(260.0)),
    );
    corps = corps.push(
        palette
            .label("Les normes du projet — vide : celles de la base choisie.")
            .size(theme::jetons::DETAIL),
    );

    corps = corps.push(Space::new().height(10));
    corps = corps.push(
        row![
            button(
                palette
                    .label(if creation {
                        "Créer le projet"
                    } else {
                        "Renommer"
                    })
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::ProjetValider)
            .padding([5, 12]),
            button(palette.label("Annuler").size(theme::jetons::DETAIL))
                .on_press(Message::ProjetAnnuler)
                .padding([5, 12]),
        ]
        .spacing(theme::jetons::PAS),
    );

    let corps = corps.spacing(theme::jetons::PAS);
    let fenetre = container(corps).padding(20).style(theme::panneau(palette));

    crate::ui::voile(
        container(fenetre)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
}
