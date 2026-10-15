use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::State;
use crate::ui::editor::bubble_style;
use crate::ui::theme::Palette;

/// La case de renommage d'un symbole, et l'aperçu de ce qu'il changerait.
///
/// Elle n'a que deux états : la saisie du nom, puis les modifications que le
/// serveur propose. Tant que les secondes ne sont pas **montrées**, le bouton
/// d'application n'existe pas : un renommage touche des fichiers qu'on n'a pas
/// ouverts, et il se voit avant de se faire.
pub fn view(state: &State) -> Element<'_, Message> {
    let Some(renommage) = state.renommage.as_ref() else {
        // Le dialogue a une place fixe dans la page : vide, il ne bouge rien.
        return container(Space::new()).into();
    };

    let palette: Palette = state.resolved.palette;

    let entete = row![
        palette.label("Renommer le symbole").size(14),
        Space::new().width(Length::Fill),
        button(text("Annuler").size(12))
            .on_press(Message::SymbolRenameCancel)
            .padding([6, 10]),
    ]
    .spacing(8);

    let saisie = text_input("nouveau nom", &renommage.nom)
        .on_input(Message::SymbolRenameChanged)
        .on_submit(Message::SymbolRenameAsk)
        .padding(8)
        .size(13);

    let apercu = match &renommage.changements {
        None => column![
            palette
                .label("Le serveur dira ce que cela changerait, fichier par fichier.")
                .size(11),
        ],

        Some(changements) => {
            let total: usize = changements.iter().map(|fichier| fichier.edits.len()).sum();

            let mut col = column![
                palette
                    .label(format!(
                        "{} fichier(s), {} modification(s)",
                        changements.len(),
                        total
                    ))
                    .size(12),
            ];

            for fichier in changements {
                col = col.push(
                    palette
                        .label(format!("{} — {}", fichier.path.display(), fichier.edits.len()))
                        .size(11),
                );
            }

            col
        }
    };

    let boutons = row![
        button(text("Demander").size(12))
            .on_press(Message::SymbolRenameAsk)
            .padding([6, 10]),

        button(text("Appliquer").size(12))
            .on_press_maybe(
                renommage
                    .changements
                    .as_ref()
                    .map(|_| Message::SymbolRenameApply)
            )
            .padding([6, 10]),
    ]
    .spacing(8);

    container(column![entete, saisie, apercu, boutons].spacing(10))
        .padding(14)
        .style(bubble_style(palette))
        .width(Length::Fill)
        .into()
}
