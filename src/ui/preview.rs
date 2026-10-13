use crate::app::{Message, State};
use iced::widget::{button, column, container, image, row, text};
use iced::{Alignment, ContentFit, Element, Fill};

pub fn view(state: &State) -> Element<'_, Message> {
    let Some(preview) = state.preview.as_ref() else {
        return container(
            text("Prévisualisation en direct\n\nLe PDF est recompilé automatiquement\nquelques instants après la saisie.")
                .size(16)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .center_x(Fill)
        .center_y(Fill)
        .width(Fill)
        .height(Fill)
        .into();
    };

    let pages = &preview.pages;
    let current = preview.current.min(pages.len().saturating_sub(1));

    // Barre d'état : message de compilation et ouverture dans le lecteur externe.
    let header = row![
        text(&preview.status).size(13).width(Fill),
        button("Ouvrir le PDF").on_press(Message::OpenPreview),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(6);

    // Navigation entre les pages.
    let navigation = row![
        button("◀").on_press_maybe((current > 0).then_some(Message::PreviewPagePrev)),
        text(format!("Page {} / {}", current + 1, pages.len().max(1)))
            .size(13)
            .width(Fill)
            .align_x(iced::alignment::Horizontal::Center),
        button("▶").on_press_maybe((current + 1 < pages.len()).then_some(Message::PreviewPageNext)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(6);

    // Corps : la page courante du PDF, mise à l'échelle pour tenir dans le panneau.
    let body: Element<'_, Message> = match pages.get(current) {
        Some(page) => image::viewer(image::Handle::from_path(page))
            .content_fit(ContentFit::Contain)
            .width(Fill)
            .height(Fill)
            .into(),
        None => container(
            text(&preview.status)
                .size(14)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .center_x(Fill)
        .center_y(Fill)
        .into(),
    };

    column![header, navigation, body]
        .width(Fill)
        .height(Fill)
        .spacing(4)
        .into()
}
