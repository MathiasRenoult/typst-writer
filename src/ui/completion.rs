//! La fenêtre de suggestions de citation.
//!
//! Une fenêtre **fixe, au centre de l'écran**, ouverte par un `@`. À la
//! différence d'une bulle accrochée au curseur, elle n'a pas à calculer sa
//! place : elle est au centre, toujours, et rien ne peut la recouvrir ni la
//! faire sortir de l'écran. C'est ce qui la rend simple, et ce qui rend les
//! clics prévisibles.
//!
//! Elle a deux visages. Le **composeur de notes** pour les références — choisir,
//! donner la page, ajouter une autre référence à la même note — et la liste de
//! complétion du serveur pour les commandes `#`.

use iced::widget::{Space, button, column, container, row, text_input};
use iced::{Alignment, Element, Length};

use crate::app::composeur::{Composeur, Etape};
use crate::app::message::Message;
use crate::app::state::State;
use crate::ui::editor::{bubble_style, completion_popup};
use crate::ui::theme::{self, Palette};

/// Largeur de la fenêtre de suggestions.
const LARGEUR: f32 = 400.0;

/// La fenêtre, ou sa place vide.
///
/// Sa place est **toujours** dans la page, comme celle de la fenêtre de mise en
/// page : la forme de l'arbre de widgets ne change jamais quand elle s'ouvre.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    if let Some(composeur) = state.composeur.as_ref() {
        return fenetre_composeur(composeur, palette, state.editor.is_intermediate());
    }

    let Some(open) = state.lsp.completion.as_ref() else {
        return container(Space::new()).into();
    };

    let entete = row![
        palette.label("Choisir une référence").size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label("Échap pour fermer")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let pied = palette
        .label("↑↓ naviguer · Tab ou Entrée insérer")
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    let fenetre = container(
        column![
            entete,
            completion_popup(open, palette, LARGEUR, state.lsp.awaiting),
            pied,
        ]
        .spacing(10),
    )
    .width(Length::Fixed(LARGEUR + 32.0))
    .padding(16)
    .style(bubble_style(palette));

    // La fenêtre est centrée dans toute la page, et le fond reste la page
    // elle-même : cliquer hors de la fenêtre tombe dans l'éditeur, ce qui
    // déplace le curseur et ferme la liste — le geste naturel de sortie.
    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Le visage composeur : la note se choisit, se pagine, et s'écrit à la fin.
fn fenetre_composeur<'a>(
    composeur: &'a Composeur,
    palette: Palette,
    intermediaire: bool,
) -> Element<'a, Message> {
    let (titre, invite, raccourcis) = match composeur.etape {
        Etape::Reference => (
            if composeur.references.is_empty() {
                "Choisir une référence"
            } else {
                "Ajouter une référence à la même note"
            },
            "Chercher une clé…",
            "↑↓ naviguer · Tab : numéro de page · Entrée : insérer · Échap : fermer",
        ),
        Etape::Page => (
            "Numéro de page",
            "p. 45, pp. 12-15, chap. 3…",
            "Tab : une autre référence · Entrée : insérer · Échap : fermer",
        ),
    };

    let entete = row![
        palette.label(titre).size(theme::jetons::TITRE),
        Space::new().width(Length::Fill),
        palette
            .label("Échap pour fermer")
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    // Un seul champ pour les deux étapes : la forme de la fenêtre ne change
    // jamais quand l'étape change, et le focus y demeure.
    let champ = text_input(invite, &composeur.saisie)
        .id(Composeur::champ_id())
        .on_input(Message::ComposeurSaisie)
        .padding(6)
        .width(Length::Fixed(LARGEUR));

    let note = lignes_note(composeur, palette);

    let corps = match composeur.etape {
        Etape::Reference => liste_references(composeur, palette),
        Etape::Page => container(
            container(
                palette
                    .label("La page s'écrit comme elle apparaîtra dans la note.")
                    .size(theme::jetons::CORPS)
                    .color(palette.text_muted),
            )
            .width(Length::Fill)
            .height(Length::Fixed(28.0))
            .padding([5, 8]),
        )
        .width(Length::Fixed(LARGEUR))
        .style(bubble_style(palette))
        .into(),
    };

    // La note telle qu'elle se compose : c'est exactement ce que le document
    // recevra, et c'est ce qui rend le composeur honnête.
    let apercu = row![
        palette.label("Note").size(theme::jetons::DETAIL).color(palette.text_muted),
        palette
            .label(composeur.apercu(intermediaire))
            .size(theme::jetons::CORPS)
            .color(palette.text),
    ]
    .spacing(theme::jetons::PAS)
    .align_y(Alignment::Center);

    let pied = palette
        .label(raccourcis)
        .size(theme::jetons::DETAIL)
        .color(palette.text_muted);

    let fenetre = container(
        column![entete, champ, note, corps, apercu, pied].spacing(10),
    )
    .width(Length::Fixed(LARGEUR + 32.0))
    .padding(16)
    .style(bubble_style(palette));

    container(fenetre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Le fond d'un bouton au survol : une brume, jamais un bloc.
///
/// La sélection ne colore pas la boîte — c'est sa **bordure** qui se teinte.
/// Le survol n'est qu'un soupçon de surface, pour dire que ça se clique.
fn survol(status: button::Status, palette: Palette) -> Option<iced::Background> {
    match status {
        button::Status::Hovered | button::Status::Pressed => Some(palette.surface_hover.into()),
        _ => None,
    }
}

/// Les lignes de la note : chacune se reprend, à son rang.
///
/// Cliquer une ligne remet sa clé et sa page sous les doigts — c'est la
/// première, la deuxième, ou celle qu'on veut — et la réécriture la remet à
/// sa place. L'ordre de la note ne bouge jamais.
fn lignes_note<'a>(composeur: &'a Composeur, palette: Palette) -> Element<'a, Message> {
    let mut liste = column![].spacing(0.0).width(Length::Fixed(LARGEUR));

    for (index, reference) in composeur.references.iter().enumerate() {
        let tenue = composeur.rang == Some(index);
        let page = reference.page.as_deref().unwrap_or("");

        let mut ligne = row![]
            .width(Length::Fill)
            .padding([4, 8])
            .spacing(theme::jetons::PAS)
            .align_y(Alignment::Center);

        ligne = ligne.push(
            palette
                .label(format!("{}", index + 1))
                .size(theme::jetons::DETAIL)
                .color(if tenue { palette.accent } else { palette.text_muted }),
        );

        ligne = ligne.push(palette.label(&reference.cle).size(theme::jetons::SAILLANT).color(if tenue {
            palette.accent
        } else {
            palette.text
        }));

        ligne = ligne.push(Space::new().width(Length::Fill));

        if !page.is_empty() {
            ligne = ligne.push(palette.label(page).size(theme::jetons::DETAIL).color(if tenue {
                palette.accent
            } else {
                palette.text_muted
            }));
        }

        liste = liste.push(
            button(ligne)
                .width(Length::Fill)
                .padding([6, 10])
                .style(move |_, status| button::Style {
                    background: survol(status, palette),
                    text_color: if tenue { palette.accent } else { palette.text },
                    border: iced::Border {
                        color: if tenue { palette.accent } else { iced::Color::TRANSPARENT },
                        width: if tenue { 1.5 } else { 1.0 },
                        radius: theme::jetons::RAYON.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::ComposeurChoisir(index)),
        );
    }

    if composeur.references.is_empty() {
        return Space::new()
            .width(Length::Fixed(LARGEUR))
            .height(Length::Fixed(0.0))
            .into();
    }

    container(liste)
        .width(Length::Fixed(LARGEUR))
        .style(bubble_style(palette))
        .into()
}

/// Les clés citables, filtrées par ce qui est tapé dans le champ.
fn liste_references<'a>(composeur: &'a Composeur, palette: Palette) -> Element<'a, Message> {
    let visibles = composeur.visibles();

    if visibles.is_empty() {
        return container(
            container(
                palette
                    .label(if composeur.cles.is_empty() {
                        "Aucune référence dans la bibliographie"
                    } else {
                        "Aucune référence correspondante"
                    })
                    .size(theme::jetons::SAILLANT)
                    .color(palette.text_muted),
            )
            .width(Length::Fill)
            .height(Length::Fixed(28.0))
            .padding([5, 8]),
        )
        .width(Length::Fixed(LARGEUR))
        .style(bubble_style(palette))
        .into();
    }

    let mut liste = column![].spacing(0.0).width(Length::Fixed(LARGEUR));

    for (position, index) in visibles.iter().enumerate() {
        let (cle, genre) = &composeur.cles[*index];
        let selected = composeur.sur_palette() && position == composeur.selection;

        let mut ligne = row![]
            .width(Length::Fill)
            .padding([5, 8])
            .spacing(theme::jetons::PAS)
            .align_y(Alignment::Center);

        ligne = ligne.push(palette.label(cle).size(theme::jetons::SAILLANT).color(if selected {
            palette.accent
        } else {
            palette.text
        }));

        ligne = ligne.push(Space::new().width(Length::Fill));

        ligne = ligne.push(palette.label(genre).size(theme::jetons::DETAIL).color(if selected {
            palette.accent
        } else {
            palette.text_muted
        }));

        liste = liste.push(
            button(ligne)
                .width(Length::Fill)
                .padding([6, 10])
                .style(move |_, status| button::Style {
                    background: survol(status, palette),
                    text_color: if selected { palette.accent } else { palette.text },
                    border: iced::Border {
                        color: if selected { palette.accent } else { iced::Color::TRANSPARENT },
                        width: if selected { 1.5 } else { 1.0 },
                        radius: theme::jetons::RAYON.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::ComposeurPrendre(position)),
        );
    }

    container(liste)
        .width(Length::Fixed(LARGEUR))
        .style(bubble_style(palette))
        .into()
}
