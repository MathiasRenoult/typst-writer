//! Le panneau de gestion des fichiers de normes.
//!
//! Une norme est un **fichier** : elle porte la mise en page traditionnelle
//! (papier, marges, texte, titres, décor) et le paramétrage des plugins qui
//! doit s'appliquer avec elle à l'ouverture du document. Ce panneau montre les
//! fichiers de la bibliothèque, les charge dans la copie de travail, les
//! applique au document, les exporte et les supprime. Il ne double pas les
//! onglets de réglage : il gère les **fichiers** que ces réglages composent.

use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Length};

use crate::app::message::Message;
use crate::app::state::DialogueMiseEnPage;
use crate::ui::theme::{self, Palette};

/// Le corps du panneau : l'import, la copie courante, puis la bibliothèque.
pub(crate) fn corps<'a>(
    palette: Palette,
    dialogue: &'a DialogueMiseEnPage,
) -> iced::widget::Column<'a, Message> {
    let noms = crate::project::noms_de_normes();

    let entete = row![
        palette
            .label("Fichiers de normes")
            .size(theme::jetons::SAILLANT),
        Space::new().width(Length::Fill),
        button(
            palette
                .label("Importer un fichier…")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::NormesFichierImporter)
        .padding([5, 12]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let explication = palette
        .label(
            "Un fichier de normes porte la mise en page et le paramétrage des plugins. \
             Choisir une norme la charge ici ; « Appliquer » l'écrit dans le document.",
        )
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    let nom_courant = row![
        palette.label("Nom de la copie").size(theme::jetons::DETAIL),
        text_input("nom du fichier de normes", &dialogue.norme)
            .on_input(Message::NormesNom)
            .on_submit(Message::NormesEnregistrer(dialogue.norme.clone()))
            .padding(5)
            .width(Length::Fixed(220.0)),
        button(
            palette
                .label("Enregistrer une copie")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::NormesEnregistrer(dialogue.norme.clone()))
        .padding([5, 12]),
        Space::new().width(Length::Fill),
        button(
            palette
                .label("Exporter la norme courante")
                .size(theme::jetons::DETAIL),
        )
        .on_press(Message::MiseEnPageExporter)
        .padding([5, 12]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut liste = column![].spacing(8);

    if noms.is_empty() {
        liste = liste.push(
            palette
                .label("Aucun fichier de normes dans la bibliothèque.")
                .size(theme::jetons::CORPS)
                .color(palette.text_muted),
        );
    }

    for nom in noms {
        let Some(norme) = crate::project::norme(&nom) else {
            continue;
        };
        let choisi = dialogue.norme_originale == nom;
        let resume = resume_norme(&norme);

        let titre = row![
            palette.label(nom.clone()).size(theme::jetons::CORPS),
            if choisi {
                palette
                    .label("chargée")
                    .size(theme::jetons::DETAIL)
                    .color(palette.accent)
            } else {
                palette.label("").size(theme::jetons::DETAIL)
            },
            Space::new().width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let actions = row![
            button(palette.label("Charger").size(theme::jetons::DETAIL),)
                .on_press(Message::NormesAppliquer(nom.clone()))
                .padding([4, 10]),
            button(
                palette
                    .label("Appliquer au document")
                    .size(theme::jetons::DETAIL),
            )
            .on_press(Message::NormesAppliquerAuDocument(nom.clone()))
            .padding([4, 10]),
            Space::new().width(Length::Fill),
            button(theme::icon(
                lucide_icons::iced::icon_download::<iced::Theme>(),
                13.0,
                palette.text_muted,
            ))
            .on_press(Message::NormesExporter(nom.clone()))
            .padding([4, 6]),
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<iced::Theme>(),
                13.0,
                palette.text_muted,
            ))
            .on_press(Message::NormesSupprimerDemande(nom.clone()))
            .padding([4, 6]),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let carte = container(
            column![
                titre,
                palette
                    .label(resume)
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
                actions,
            ]
            .spacing(6),
        )
        .padding(10)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(
                if choisi {
                    palette.surface_hover
                } else {
                    palette.surface
                }
                .into(),
            ),
            border: iced::Border {
                color: if choisi {
                    palette.accent
                } else {
                    palette.border
                },
                width: 1.0,
                radius: theme::jetons::RAYON.into(),
            },
            ..Default::default()
        });

        liste = liste.push(carte);
    }

    let chemin = palette
        .label(format!(
            "Bibliothèque : {}",
            crate::project::bibliotheque_de_normes().display()
        ))
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    column![entete, explication, nom_courant, chemin, liste].spacing(10)
}

/// Le résumé d'un fichier : ce que la norme compose et comment ses plugins
/// se comportent. Il sert aussi de preuve lisible que les deux familles de
/// réglages vivent bien dans le même fichier.
pub(crate) fn resume_norme(norme: &crate::mise_en_page::modele::MiseEnPage) -> String {
    let f = &norme.fonctionnalites;
    let biblique = if f.references_bibliques {
        format!(
            "biblique {} · {}",
            if norme.parametres_plugins.biblique.traduction == "darby" {
                "Darby"
            } else {
                "LSG"
            },
            if norme.parametres_plugins.biblique.tri == "canonique" {
                "ordre canonique"
            } else {
                "ordre d'apparition"
            }
        )
    } else {
        "biblique désactivé".to_string()
    };
    let bibliographie = if f.bibliographie {
        format!(
            "bibliographie {}",
            norme
                .bibliographie
                .style
                .as_deref()
                .unwrap_or("style du document")
        )
    } else {
        "bibliographie désactivée".to_string()
    };
    let glossaire = if f.glossaire {
        "glossaire actif"
    } else {
        "glossaire désactivé"
    };
    let callouts = if f.callouts {
        "callouts actifs"
    } else {
        "callouts désactivés"
    };

    format!(
        "{} · {} {} · {} · {} · {} · {}",
        norme.papier.format.to_uppercase(),
        norme.texte.police,
        norme.texte.taille,
        bibliographie,
        biblique,
        glossaire,
        callouts
    )
}

#[cfg(test)]
mod tests {
    use super::resume_norme;
    use crate::mise_en_page::modele::MiseEnPage;

    /// Le résumé d'un fichier dit les deux familles de réglages : la mise en
    /// page et le paramétrage des plugins.
    #[test]
    fn le_resume_dit_la_mise_en_page_et_les_plugins() {
        let mut norme = MiseEnPage::default();
        norme.papier.format = "a5".to_string();
        norme.bibliographie.style = Some("apa".to_string());
        norme.parametres_plugins.biblique.traduction = "darby".to_string();
        norme.parametres_plugins.biblique.tri = "canonique".to_string();

        let resume = resume_norme(&norme);

        assert!(resume.contains("A5"));
        assert!(resume.contains("bibliographie apa"));
        assert!(resume.contains("biblique Darby"));
        assert!(resume.contains("ordre canonique"));
        assert!(resume.contains("glossaire actif"));
        assert!(resume.contains("callouts actifs"));
    }

    /// Une fonctionnalité débranchée se lit dans le résumé au lieu d'être
    /// passée sous silence.
    #[test]
    fn le_resume_dit_les_plugins_debranches() {
        let mut norme = MiseEnPage::default();
        norme.fonctionnalites.references_bibliques = false;
        norme.fonctionnalites.glossaire = false;
        norme.fonctionnalites.callouts = false;

        let resume = resume_norme(&norme);

        assert!(resume.contains("biblique désactivé"));
        assert!(resume.contains("glossaire désactivé"));
        assert!(resume.contains("callouts désactivés"));
        assert!(!resume.contains("ordre canonique"));
    }
}
