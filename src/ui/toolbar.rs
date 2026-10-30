use crate::app::{Message, State};
use crate::ui::theme::{self, Palette};
use iced::widget::{Space, button, container, pick_list, row};
use iced::{Alignment, Element, Length, Theme};

/// Taille des icônes de la barre.
const ICON_SIZE: f32 = 17.0;

/// Espacement entre les boutons.
const SPACING: f32 = 2.0;

/// Côté d'un bouton-icône : carré, pour que la cible de clic soit généreuse
/// sans que l'icône elle-même prenne de la place.
const BUTTON: f32 = 30.0;

/// Hauteur de la barre.
pub(crate) const BAR_HEIGHT: f32 = 42.0;

/// Les couleurs proposées pour teinter du texte, avec leur nom Typst.
///
/// Ce sont des noms reconnus par Typst : le générateur les passe tels quels
/// dans `#text(fill: …)`, donc aucun choix ici ne peut produire une couleur
/// inconnue du compilateur.
const COULEURS: [(&str, &str, &str); 7] = [
    ("Rouge", "red", "#e74c3c"),
    ("Bleu", "blue", "#3498db"),
    ("Vert", "green", "#27ae60"),
    ("Orange", "orange", "#e67e22"),
    ("Violet", "purple", "#9b59b6"),
    ("Noir", "black", "#1a1a2e"),
    ("Gris", "gray", "#7f8c8d"),
];


/// La barre d'outils : des icônes, pas des mots.
///
/// Chaque bouton porte une infobulle — sans elle une icône seule est une
/// devinette, et le gain de place serait payé en friction. Les couleurs
/// viennent de la palette du thème : la barre suit le clair et le sombre.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let mut bar = row![].spacing(SPACING).align_y(Alignment::Center);

    // TOPBAR-04-10 : les normes tout à gauche — la mise en page est le
    // cadre du travail, pas une commande parmi d'autres.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file_cog::<Theme>(),
        "Les normes du document : papier, marges, titres, couleurs (Ctrl+P)",
        Message::MiseEnPageOuvrir,
        palette,
        state.mise_en_page.is_some(),
    ));

    bar = bar.push(separation(palette));

    // Fichier : les commandes essentielles. L'ouverture, l'import et la
    // sauvegarde se font par le clavier ou la bibliothèque — TOPBAR-04-02.
    bar = bar.push(icon_button(
        lucide_icons::iced::icon_file::<Theme>(),
        "Ouvrir le document d'écriture du projet (Ctrl+N)",
        Message::NewFile,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_library::<Theme>(),
        "Les projets et leurs étagères",
        Message::ProjetsVue,
        palette,
        state.projets_vue,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_download::<Theme>(),
        "Exporter en PDF (Ctrl+E)",
        Message::ExportPdf,
        palette,
        false,
    ));

    bar = bar.push(separation(palette));

    // TOPBAR-04-01 : les commandes de format sont centrées dans la barre.
    bar = bar.push(spacer());

    if state.editor.is_intermediate() {
        bar = bar.push(format_button(
            "B",
            "Gras : formater la sélection",
            crate::app::message::FormatAction::Gras,
            palette,
        ));
        bar = bar.push(format_button(
            "I",
            "Italique : formater la sélection",
            crate::app::message::FormatAction::Italique,
            palette,
        ));
        bar = bar.push(titre_menu(palette));

        bar = bar.push(format_button(
            "•",
            "Basculer la ligne en liste à puces",
            crate::app::message::FormatAction::ListePuce,
            palette,
        ));
        bar = bar.push(format_button(
            "1.",
            "Basculer la ligne en liste numérotée",
            crate::app::message::FormatAction::ListeNumero,
            palette,
        ));
        bar = bar.push(format_button(
            ">",
            "Basculer la ligne en citation de bloc",
            crate::app::message::FormatAction::CitationBloc,
            palette,
        ));
        // TOPBAR-06 : la pastille applique la dernière couleur choisie,
        // le picker ouvre le choix. La pastille mémorise la couleur courante.
        let derniere = state.derniere_couleur.clone().unwrap_or_else(|| "red".to_string());
        // TOPBAR-04-06 : la pastille montre la vraie couleur, pas un nom.
        let couleur_actuelle = COULEURS
            .iter()
            .find(|(_, nom, _)| *nom == derniere)
            .and_then(|(_, _, hex)| crate::theming::HexColor::parse(hex).ok().map(|c| c.0))
            .or_else(|| crate::theming::HexColor::parse(&derniere).ok().map(|c| c.0));

        let pastille = iced::widget::button(
            iced::widget::container(
                iced::widget::Space::new()
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0)),
            )
            .style(move |_t| iced::widget::container::Style {
                background: couleur_actuelle.map(iced::Background::from),
                border: iced::Border {
                    width: 1.0,
                    color: palette.border,
                    radius: 3.0.into(),
                },
                ..Default::default()
            }),
        )
        .on_press(Message::Format(crate::app::message::FormatAction::Couleur(
            derniere.clone(),
        )))
        .padding(4)
        .style(move |_, s| button_style(s, palette));

        bar = bar.push(iced::widget::tooltip(
            pastille,
            container(palette.label("Appliquer la dernière couleur choisie").size(12).color(palette.text))
                .padding([4, 8])
                .style(move |_| container::Style {
                    background: Some(palette.surface_hover.into()),
                    border: iced::Border {
                        color: palette.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
            iced::widget::tooltip::Position::Bottom,
        ).gap(6));

        // TOPBAR-04-08 : le picker de l'éditeur — palette de base.
        // Le choix libre passe par l'éditeur de thème (picker RVB).
        bar = bar.push(format_picker(
            COULEURS
                .iter()
                .map(|(libelle, _, _)| (*libelle).to_string())
                .collect(),
            "Couleur",
            |choisi| {
                let nom = couleur_typst(&choisi).unwrap_or("red").to_string();
                Message::Format(crate::app::message::FormatAction::Couleur(nom))
            },
            palette,
            80.0,
        ));
        bar = bar.push(format_button(
            "N",
            "Note de bas de page sur la sélection",
            crate::app::message::FormatAction::Note,
            palette,
        ));
        bar = bar.push(format_button(
            "@",
            "Insérer une citation bibliographique",
            crate::app::message::FormatAction::Citation,
            palette,
        ));
        bar = bar.push(format_button(
            "T",
            "Entourer la sélection d'un bloc Typst brut",
            crate::app::message::FormatAction::BlocTypst,
            palette,
        ));
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_image::<Theme>(),
            "Insérer une image",
            Message::ImageInserer,
            palette,
            false,
        ));

        bar = bar.push(icon_button(
            lucide_icons::iced::icon_table::<Theme>(),
            "Insérer un tableau",
            Message::TableauOuvrir,
            palette,
            false,
        ));

        bar = bar.push(separation(palette));
    }

    // Le remplissage pousse les clusters de droite au bord : la
    // lecture va du document à son rendu, et rien ne flotte entre deux.
    bar = bar.push(spacer());

    // Panneaux : ces trois-là gouvernent la disposition, ils vont ensemble.
    // En mode focus ils disparaissent — les afficher donnerait à voir des
    // commandes sans effet, et le mode focus n'a de sens que s'il ne reste que
    // le texte.
    if !state.layout.focus_mode {
        // LEFT-04-01 : le panneau latéral se rouvre d'ici — le bouton vit
        // dans la barre, pas seulement dans le panneau qu'il ferme.
        bar = bar.push(icon_button(
            if state.layout.sidebar_visible {
                lucide_icons::iced::icon_panel_left_close::<Theme>()
            } else {
                lucide_icons::iced::icon_panel_left_open::<Theme>()
            },
            if state.layout.sidebar_visible {
                "Masquer le panneau de travail"
            } else {
                "Afficher le panneau de travail"
            },
            Message::ToggleSidebar,
            palette,
            state.layout.sidebar_visible,
        ));

        // TOPBAR-04-09 : les commandes d'aperçu sont supprimées — la
        // fenêtre de rendu se gère depuis l'espace de travail.
        // Les normes sont à gauche (TOPBAR-04-10), pas ici.
    }

    bar = bar.push(icon_button(
        if state.layout.focus_mode {
            lucide_icons::iced::icon_minimize_2::<Theme>()
        } else {
            lucide_icons::iced::icon_maximize_2::<Theme>()
        },
        if state.layout.focus_mode {
            "Quitter le mode focus (F11)"
        } else {
            "Mode focus — le texte seul (F11)"
        },
        Message::ToggleFocusMode,
        palette,
        state.layout.focus_mode,
    ));

    bar = bar.push(separation(palette));

    // Thèmes : la liste, puis le mode d'apparence, puis le chargement et la
    // suppression. Les quatre vont ensemble, ils sont donc côte à côte.
    let names: Vec<String> = state
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect();

    let current = state.current_theme().map(|theme| theme.name.clone());

    // Les commandes de thème se poussent à droite — TOPBAR-09/13.
    bar = bar.push(Space::new().width(Length::Fill));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_palette::<Theme>(),
        "L'éditeur de thème",
        Message::ThemeEditeurOuvrir,
        palette,
        state.editeur_theme.is_some(),
    ));

    bar = bar.push(
        pick_list(names, current, Message::SelectTheme)
            .text_size(12)
            // La liste suit la police du thème comme le reste de l'interface :
            // sans cela, choisir une famille laissait ce seul contrôle dans
            // celle du moteur.
            .font(palette.interface)
            .padding([4, 8])
            .width(Length::Fixed(168.0)),
    );

    bar = bar.push(icon_button(
        theme_icon(state.theme),
        state.theme.label(),
        Message::CycleTheme,
        palette,
        false,
    ));

    bar = bar.push(icon_button(
        lucide_icons::iced::icon_download::<Theme>(),
        "Charger un thème (fichier .toml)",
        Message::LoadTheme,
        palette,
        false,
    ));

    // Supprimer n'a de sens que pour un thème chargé : les thèmes natifs n'ont
    // pas de fichier, et un bouton qui ne ferait rien serait un mensonge.
    if let Some(theme) = state.current_theme()
        && !theme.is_native()
    {
        bar = bar.push(icon_button(
            lucide_icons::iced::icon_trash_2::<Theme>(),
            "Supprimer ce thème",
            Message::DeleteTheme(theme.name.clone()),
            palette,
            false,
        ));
    }

    // La barre déborde parfois la fenêtre : plutôt que rogner des boutons
    // inatteignables, elle défile — sans barre de défilement visible, le
    // reste est au diapason de la sobriété de la fenêtre.
    let bar = iced::widget::scrollable(bar)
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::new()
                .width(0)
                .scroller_width(0),
        ))
        .height(Length::Fixed(BAR_HEIGHT));

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(BAR_HEIGHT))
        .padding([0, 8])
        .center_y(BAR_HEIGHT)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Un bouton-icône, sans bordure et sans fond au repos.
///
/// `active` ne signale pas une action mais un état : panneau ouvert,
/// document modifié.
fn icon_button<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    tooltip: &'static str,
    message: Message,
    palette: Palette,
    active: bool,
) -> Element<'a, Message> {
    let color = if active {
        palette.accent
    } else {
        palette.text_muted
    };

    let button = button(theme::icon(glyph, ICON_SIZE, color))
        .on_press(message)
        .padding([6, 7])
        .width(Length::Fixed(BUTTON))
        .height(Length::Fixed(BUTTON))
        .style(move |_, status| button_style(status, palette));

    // L'infobulle est ce qui rend une icône seule acceptable : sans elle, un
    // bouton de 30 px est une devinette, et le gain de place serait payé en
    // friction.
    iced::widget::tooltip(
        button,
        container(palette.label(tooltip).size(12).color(palette.text))
            .padding([4, 8])
            .style(move |_| container::Style {
                background: Some(palette.surface_hover.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Bouton compact de mise en forme, affiché pour `document.tw`.
fn format_button<'a>(
    label: &'static str,
    help: &'static str,
    action: crate::app::message::FormatAction,
    palette: Palette,
) -> Element<'a, Message> {
    toolbar_text_button(label, help, Message::Format(action), palette)
}

/// Le nom Typst de la couleur désignée par son libellé français.
fn couleur_typst(label: &str) -> Option<&'static str> {
    COULEURS
        .iter()
        .find(|(libelle, _, _)| *libelle == label)
        .map(|(_, nom, _)| *nom)
}


/// Le menu des niveaux de titre : un `pick_list`, pas un `tooltip`.
///
/// Un `tooltip` se ferme dès que la souris quitte le bouton — impossible
/// de choisir un niveau. Un `pick_list` reste ouvert jusqu'au choix ou au
/// clic extérieur, et se conduit au clavier. TOPBAR-04-04.
fn titre_menu(palette: Palette) -> Element<'static, Message> {
    let niveaux: Vec<String> = (1..=6).map(|n| format!("H{n}")).collect();

    pick_list(niveaux, None::<String>, |choisi: String| {
        let niveau: u8 = choisi
            .strip_prefix('H')
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        Message::Format(crate::app::message::FormatAction::Titre(niveau))
    })
    .placeholder("Titre")
    .text_size(12)
    .font(palette.interface)
    .padding([4, 8])
    .width(Length::Fixed(56.0))
    .into()
}

/// c'est ce qui rend le choix d'un niveau de titre ou d'une couleur immédiat,
/// au lieu d'une suite de clics à l'aveugle. Comme c'est une action et non un
/// état, rien n'y est présélectionné : le libellé vide dit ce qu'elle propose.
fn format_picker<'a>(
    options: Vec<String>,
    placeholder: &'static str,
    map: impl Fn(String) -> Message + 'a,
    palette: Palette,
    width: f32,
) -> Element<'a, Message> {
    pick_list(options, None::<String>, map)
        .placeholder(placeholder)
        .text_size(12)
        .font(palette.interface)
        .padding([4, 8])
        .width(Length::Fixed(width))
        .into()
}

/// Bouton compact avec infobulle, partagé par les commandes de texte et zoom.
fn toolbar_text_button<'a>(
    label: &'static str,
    help: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    // TOPBAR-04-03 : le libellé est centré dans le bouton.
    let button = button(
        container(palette.label(label).size(12).color(palette.text))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(message)
    .padding([4, 6])
    .width(Length::Fixed(BUTTON))
    .height(Length::Fixed(BUTTON))
    .style(move |_, status| button_style(status, palette));

    iced::widget::tooltip(
        button,
        container(palette.label(help).size(12).color(palette.text))
            .padding([4, 8])
            .style(move |_| container::Style {
                background: Some(palette.surface_hover.into()),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Bouton-icône : invisible au repos, un fond doux au survol.
fn button_style(status: button::Status, palette: Palette) -> button::Style {
    let background = match status {
        button::Status::Active => None,
        button::Status::Hovered => Some(palette.surface_hover),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => None,
    };

    button::Style {
        background: background.map(Into::into),
        text_color: palette.text_muted,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Espacement élastique : pousse ce qui suit vers la droite.
/// Un filet entre deux clusters : les sépare sans les couper.
///
/// Un trait de la couleur des bordures, haut comme les boutons : la barre
/// se lit en groupes, et chaque groupe a sa raison d'être ensemble.
fn separation(palette: Palette) -> Element<'static, Message> {
    iced::widget::container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(18.0))
        .style(move |_| container::Style {
            background: Some(palette.border.into()),
            ..Default::default()
        })
        .into()
}

fn spacer() -> Space {
    Space::new().width(Length::Fill).height(Length::Shrink)
}

/// Icône du thème : on voit d'un coup d'œil ce qui est actif.
fn theme_icon(mode: theme::ThemeMode) -> iced::widget::Text<'static, Theme> {
    match mode {
        theme::ThemeMode::System => lucide_icons::iced::icon_sun_moon::<Theme>(),
        theme::ThemeMode::Light => lucide_icons::iced::icon_sun::<Theme>(),
        theme::ThemeMode::Dark => lucide_icons::iced::icon_moon::<Theme>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque libellé de couleur donne son nom Typst, et rien d'autre.
    #[test]
    fn colour_labels_map_to_typst_names() {
        assert_eq!(couleur_typst("Rouge"), Some("red"));
        assert_eq!(couleur_typst("Violet"), Some("purple"));
        assert_eq!(couleur_typst("Inexistant"), None);
    }
}
