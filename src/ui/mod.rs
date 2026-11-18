pub mod aide;
pub mod apercu;
pub mod balise;
pub mod bible;
pub mod bible_edition;
pub mod callout;
pub mod chart;
pub mod completion;
pub mod confirmation;
pub mod corbeille;
pub mod editor;
pub mod extension;
pub mod glossaire;
pub mod glossaire_fenetre;
pub mod layout;
pub mod normes_fichiers;
pub mod normes_lot;
pub mod note;
pub mod onglets;
pub mod palette;
pub mod picker_couleur;
pub mod plugins;
pub mod preview;
pub mod projet;
pub mod projets;
pub mod recherche;
pub mod reference;
pub(crate) mod reglages;
pub mod restauration;
pub(crate) mod sidebar;
mod statistiques;
pub mod status_bar;
pub mod tableau;
pub use crate::design::theme;
pub mod theme_editeur;
pub mod toolbar;
pub mod zotero;

use crate::app::{Message, State};
use iced::widget::{column, stack};
use iced::{Element, Length};

/// Compose l'interface : barre d'outils, espace de travail, barre d'état.
///
/// La barre d'outils et la barre d'état sont fixes ; tout l'espace qui reste
/// appartient à la disposition des panneaux, qui gère seule ce qui est
/// redimensionnable. La fenêtre de suggestions et celle de mise en page sont
/// des **couches** posées par-dessus, toujours en place dans l'arbre.
/// Le voile d'une fenêtre : il avale les clics.
///
/// Sans lui, un clic non traité par la fenêtre passait à ce qui est
/// derrière — le texte, un bouton de la barre — et la fenêtre ouverte
/// faisait travailler l'application à travers elle.
pub fn voile<'a>(arriere_plan: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    // Un vrai écran : **tout** ce qui est pointeur s'y arrête — clics
    // gauche, droit, molette, défilement — sinon l'interface derrière
    // reste jouable à travers les fenêtres (bug-report-9, T1.1).
    iced::widget::mouse_area(arriere_plan.into())
        .on_press(Message::Rien)
        .on_right_press(Message::Rien)
        .on_middle_press(Message::Rien)
        .on_scroll(|_| Message::Rien)
        .into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    // En mode focus, la barre d'outils et la barre d'état disparaissent :
    // elles n'ont plus d'objet, les panneaux étant masqués et les commandes
    // de fichier n'étant pas ce qu'on vient chercher en écrivant. On en sort
    // par F11 ou Échap.
    if state.layout.focus_mode && !state.projets_vue {
        return stack![
            iced::widget::container(layout::view(state))
                .width(Length::Fill)
                .height(Length::Fill),
            palette::view(state),
            balise::view(state),
            completion::view(state),
            recherche::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    // Les fenêtres flottantes sont des **couches** : leur place existe même
    // quand elles sont vides, la forme de la page ne change donc jamais quand
    // elles s'ouvrent — la leçon du défilement perdu à l'ouverture d'une bulle
    // se paie encore.
    stack![
        column![
            toolbar::view(state),
            layout::view(state),
            status_bar::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        palette::view(state),
        balise::view(state),
        completion::view(state),
        projets::view(state),
        normes_lot::view(state),
        reference::view(state),
        bible_edition::view(state),
        glossaire_fenetre::view(state),
        glossaire::view(state),
        zotero::view(state),
        recherche::view(state),
        restauration::view(state),
        corbeille::view(state),
        reglages::view(state),
        aide::view(state),
        picker_couleur::view(state, state.resolved.palette),
        confirmation::view(state),
        projet::view(state),
        note::view(state),
        extension::view(state),
        callout::view(state),
        chart::view(state),
        tableau::view(state),
        theme_editeur::view(state),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub mod rename;
