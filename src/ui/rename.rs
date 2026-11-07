//! La saisie de renommage d'un chapitre ou d'un projet.
//!
//! Le renommage de symboles Typst est retiré avec le LSP (feuille de
//! route, phase 1) : reste la saisie, dont le focus se règle ici.

/// L'identifiant de la saisie : la ligne de renommage s'y ouvre.
pub fn id_du_renommage() -> iced::widget::Id {
    iced::widget::Id::new("renommage")
}
