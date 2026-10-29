pub mod aide;
pub mod completion;
pub mod confirmation;
pub mod editor;
pub mod layout;
pub mod preview;
pub mod projet;
pub mod onglets;
pub mod projets;
pub mod theme_editeur;
pub mod recherche;
pub mod reference;
mod reglages;
pub mod rename;
pub mod sidebar;
pub mod status_bar;
pub mod tableau;
pub mod theme;
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
    iced::widget::mouse_area(arriere_plan.into())
        .on_press(Message::Rien)
        .into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    // En mode focus, la barre d'outils disparaît : elle n'a plus d'objet, les
    // panneaux étant masqués et les commandes de fichier n'étant pas ce qu'on
    // vient chercher en écrivant. On en sort par F11 ou Échap.
    if state.layout.focus_mode && !state.projets_vue {
        return stack![
            column![layout::view(state), status_bar::view(state)]
                .width(Length::Fill)
                .height(Length::Fill),
            completion::view(state),
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
            rename::view(state),
            layout::view(state),
            status_bar::view(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        completion::view(state),
        projets::view(state),
        reference::view(state),
        zotero::view(state),
        recherche::view(state),
        reglages::view(state),
        aide::view(state),
        confirmation::view(state),
        projet::view(state),
        tableau::view(state),
        theme_editeur::view(state),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
