//! La barre de recherche et de remplacement.
//!
//! C'est un **outil**, pas une fenêtre modale : elle se pose au-dessus du
//! document sans intercepter les clics qui ne la touchent pas. On peut donc
//! continuer à parcourir et à cliquer dans l'éditeur pendant qu'elle est
//! ouverte, et `Échap` la ferme.
//!
//! Le motif est une expression régulière : `$1` dans le remplacement rappelle
//! un groupe.

use crate::app::Message;
use crate::ui::theme;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

/// L'identifiant du champ du motif : la recherche s'ouvre sur lui.
pub fn id_du_motif() -> iced::widget::Id {
    iced::widget::Id::new("recherche-motif")
}

/// La barre, quand la recherche est ouverte.
pub fn view(state: &crate::app::State) -> Element<'_, Message> {
    let Some(etat) = state.recherche.as_ref() else {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    };

    let palette = state.resolved.palette;

    // `Entrée` dans le motif cherche le suivant ; dans le remplacement,
    // elle remplace l'occurrence courante. Les deux gestes sont ceux qu'on
    // attend en tapant — ils ne se cherchent pas.
    let motif = text_input(
        if etat.mode_regex {
            "motif — expression régulière"
        } else {
            "motif — texte littéral"
        },
        &etat.motif,
    )
    .on_submit(Message::RechercheSuivant)
    .id(id_du_motif())
    .on_input(Message::RechercheMotif)
    .padding(6)
    .width(Length::Fill);

    // Le mode de recherche : littéral ou regex — FIND-01.
    let mode_btn = button(text(if etat.mode_regex { ".*" } else { "Aa" }).size(11))
        .on_press(Message::RechercheBasculerMode)
        .padding([4, 8]);

    let portee_btn = button(
        text(if etat.portee_projet {
            "Projet"
        } else {
            "Document"
        })
        .size(11),
    )
    .on_press(Message::RechercheBasculerPortee)
    .padding([4, 8]);

    let remplacement = text_input("remplacement — $1 rappelle un groupe", &etat.remplacement)
        .on_submit(Message::RechercheRemplacer)
        .on_input(Message::RechercheRemplacement)
        .padding(6)
        .width(Length::Fill);

    let compte = if etat.portee_projet {
        if etat.motif.is_empty() {
            String::new()
        } else {
            format!("{} résultat(s)", etat.resultats_projet.len())
        }
    } else if etat.occurrences.is_empty() {
        if etat.motif.is_empty() {
            String::new()
        } else {
            "aucune".to_string()
        }
    } else {
        format!("{}/{}", etat.courante + 1, etat.occurrences.len())
    };

    let mut haut = row![mode_btn, portee_btn, motif, remplacement]
        .spacing(theme::jetons::PAS)
        .align_y(Alignment::Center);

    haut = haut.push(
        palette
            .label(compte)
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    );

    let mut actions = row![].spacing(6);

    if !etat.portee_projet {
        for (libelle, message) in [
            ("Précédent", Message::RecherchePrecedent),
            ("Suivant", Message::RechercheSuivant),
            ("Remplacer", Message::RechercheRemplacer),
            ("Tout remplacer", Message::RechercheRemplacerTout),
        ] {
            actions = actions.push(
                button(text(libelle).size(11))
                    .on_press(message)
                    .padding([4, 8]),
            );
        }
    }

    actions = actions.push(
        button(text("Fermer").size(11))
            .on_press(Message::RechercheFermer)
            .padding([4, 8]),
    );

    let mut corps = column![haut, actions].spacing(8);

    if etat.portee_projet {
        for resultat in etat.resultats_projet.iter().take(10) {
            corps = corps.push(
                button(
                    palette
                        .label(format!(
                            "{}:{} — {}",
                            resultat.chemin, resultat.ligne, resultat.extrait
                        ))
                        .size(theme::jetons::DETAIL),
                )
                .on_press(Message::RechercheProjetOuvrir(
                    resultat.chemin.clone(),
                    resultat.debut,
                    resultat.fin,
                ))
                .padding([2, 4])
                .width(Length::Fill)
                .style(button::text),
            );
        }
        if etat.resultats_projet.len() > 10 {
            corps = corps.push(
                palette
                    .label(format!(
                        "… et {} autre(s)",
                        etat.resultats_projet.len() - 10
                    ))
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
            );
        }
        if !etat.motif.is_empty() && etat.resultats_projet.is_empty() {
            corps = corps.push(
                palette
                    .label("Aucune occurrence dans les sources du projet.")
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
            );
        }
    }

    // L'aperçu des changements : ce que l'occurrence courante devient —
    // voir avant de remplacer (R6).
    if let Some((avant, apres)) = etat.apercu.get(etat.courante) {
        corps = corps.push(
            palette
                .label(format!("« {avant} » devient « {apres} »"))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    if let Some(erreur) = etat.erreur.as_ref() {
        corps = corps.push(
            palette
                .label(erreur)
                .size(theme::jetons::DETAIL)
                .color(palette.danger),
        );
    }

    let fenetre = container(corps)
        .padding(12)
        .width(Length::Fill)
        .style(theme::panneau(palette));

    // Pas de voile : la barre ne prend que sa propre surface. Cliquer dans
    // le texte, faire défiler, sélectionner — tout reste possible pendant
    // qu'elle est ouverte. Le décalage la laisse sous la barre d'outils.
    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(iced::Padding {
            top: 56.0,
            right: 16.0,
            bottom: 16.0,
            left: 16.0,
        })
        .into()
}
