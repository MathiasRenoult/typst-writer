use crate::app::state::ChapterRename;
use crate::app::{Message, State};
use crate::project::{BibliographyEntry, Chapter};
use crate::ui::theme::{self, Palette};
use iced::widget::{mouse_area, Space, button, column, container, row, scrollable, text, text_input, tooltip};
use iced::{Alignment, Element, Fill, Length, Theme};

/// Hauteur d'une ligne.
const ENTRY_HEIGHT: f32 = 32.0;

/// Largeur de la colonne des comptes de mots.
///
/// Fixe, et alignée à droite : c'est ce qui fait qu'on parcourt la colonne des
/// chiffres d'un seul regard, sans que les nombres dansent d'une ligne à
/// l'autre.
const COUNT_WIDTH: f32 = 58.0;

/// Le panneau de travail : où en est le projet.
///
/// Il répond à une seule question — « où j'en suis ? » — et l'ordonne du plus
/// général au plus précis : le projet et son volume, ses chapitres, sa
/// bibliographie, les autres projets. Chaque chapitre porte son compte de mots,
/// qui est le repère de progression.
pub fn view(state: &State) -> Element<'_, Message> {
    let palette = state.resolved.palette;

    let project = &state.project;

    let mut content = column![].spacing(2).padding([14, 12]);

    // ---- Le projet et son volume ----
    content = content.push(
        palette
            .label(project.name.to_uppercase())
            .size(theme::jetons::CHAPEAU)
            .color(palette.text_muted),
    );

    // Le compte de mots vit à la barre d'état, où il est vivant — le
    // répéter en tête ne fait pas une information de plus.
    content = content.push(
        row![
            Space::new().width(Length::Fill),
            palette
                .label(if project.document.is_some() {
                    "1 document".to_string()
                } else {
                    format!("{} chapitre(s)", project.chapters.len())
                })
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .align_y(Alignment::Center),
    );

    content = content.push(Space::new().height(14));

    // Le jalon 2 réduit le projet à un document d'écriture unique. Les anciens
    // chapitres restent conservés sur disque ; ils ne sont plus la navigation
    // principale après leur migration.
    if let Some(document) = project.document.as_ref() {
        content = content.push(section_title("Document", palette));

        let active = state.editor.path.as_deref() == Some(document.as_path());
        let line = row![
            theme::icon(
                lucide_icons::iced::icon_file_text::<Theme>(),
                14.0,
                if active {
                    palette.accent
                } else {
                    palette.text_muted
                },
            ),
            palette
                .label("document.tw")
                .size(theme::jetons::SAILLANT)
                .color(if active { palette.accent } else { palette.text }),
            Space::new().width(Length::Fill),
            palette
                .label(format_count(state.word_count()))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        ]
        .spacing(7)
        .align_y(Alignment::Center);

        // La liaison du bouton à l'éditeur fait de ce fichier l'unique point
        // d'entrée pour écrire ; les anciens chapitres ne sont plus édités par
        // erreur depuis la liste.
        content = content.push(
            button(line)
                .on_press(Message::OpenChapter(document.clone()))
                .width(Fill)
                .height(Length::Fixed(ENTRY_HEIGHT))
                .padding([4, 8])
                .style(move |t, status| entry_style(t, status, palette, active)),
        );

        if !project.chapters.is_empty() {
            content = content.push(
                palette
                    .label(format!(
                        "{} ancien(s) chapitre(s) conservé(s) en archive",
                        project.chapters.len()
                    ))
                    .size(theme::jetons::CHAPEAU)
                    .color(palette.text_muted),
            );
        }

        let orphan_count = project.orphan_chapters().count();
        if orphan_count > 0 {
            content = content.push(
                palette
                    .label(format!(
                        "{orphan_count} archive(s) hors assemblage — importez-les via Ouvrir"
                    ))
                    .size(theme::jetons::CHAPEAU)
                    .color(palette.warning),
            );
        }
    } else {
        // Projet ancien en cours de migration : il reste navigable tant que le
        // document unique n'a pas pu être créé.
        content = content.push(section_title("Chapitres", palette));

        if project.chapters.is_empty() {
            content = content.push(
                palette
                    .label("Aucun chapitre.")
                    .size(theme::jetons::CORPS)
                    .color(palette.text_muted),
            );
        }

        let active = state.editor.path.clone();

        for (rang, chapter) in project.chapters.iter().enumerate() {
            if rang > 0 {
                content = content.push(separation(palette));
            }

            content = content.push(chapter_row(
                chapter,
                state.renaming.as_ref(),
                active.as_deref() == Some(chapter.path.as_path()),
                palette,
            ));
        }

        let orphans = project.orphan_chapters().count();

        if orphans > 0 {
            content = content.push(Space::new().height(8));
            content = content.push(
                palette
                    .label(format!(
                        "{orphans} chapitre(s) hors assemblage — absent(s) du PDF"
                    ))
                    .size(theme::jetons::DETAIL)
                    .color(palette.warning),
            );
        }

        content = content.push(Space::new().height(8));
        content = content.push(
            button(
                row![
                    theme::icon(lucide_icons::iced::icon_plus::<Theme>(), 15.0, palette.text),
                    palette
                        .label("Nouveau chapitre")
                        .size(theme::jetons::SAILLANT),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .on_press(Message::NewChapter)
            .width(Fill)
            .padding([7, 10])
            .style(move |t, status| primary_style(t, status, palette)),
        );
    }

    // ---- Le sommaire ----
    //
    // Les titres du document, avec le retrait de leur niveau : c'est la table
    // des matières qu'on consulte en écrivant, et un clic y va — dans les deux
    // vues à la fois.
    if !state.render.outline.is_empty() {
        content = content.push(Space::new().height(18));
        content = content.push(section_title("Sommaire", palette));

        for (index, entry) in state.render.outline.iter().enumerate() {
            if index > 0 {
                content = content.push(separation(palette));
            }

            let mut label = "    ".repeat(entry.level.saturating_sub(1));
            label.push_str(entry.text.trim());

            content = content.push(
                button(palette.label(label).size(theme::jetons::CORPS))
                    .on_press(Message::GotoHeading(index))
                    .width(Fill)
                    .height(Length::Fixed(ENTRY_HEIGHT))
                    .padding([4, 8])
                    .style(move |t, status| entry_style(t, status, palette, false)),
            );
        }
    }

    // ---- La bibliographie ----
    content = content.push(Space::new().height(18));

    // Le titre replie la liste, et le filtre la cherche : une thèse a
    // des centaines de notices, et une liste qui ne se cherche pas est un
    // mur.
    content = content.push(
        button(
            row![
                theme::icon(
                    if state.references_repliees {
                        lucide_icons::iced::icon_chevron_right::<Theme>()
                    } else {
                        lucide_icons::iced::icon_chevron_down::<Theme>()
                    },
                    12.0,
                    palette.text_muted,
                ),
                palette.label("Bibliographie").size(theme::jetons::SAILLANT),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ReferencesReplier)
        .padding([2, 4])
        .style(move |t, status| entry_style(t, status, palette, false)),
    );

    // Un projet sans bibliographie la voit naître d'un geste.
    if state.project.bibliography.is_none() {
        content = content.push(
            button(
                palette
                    .label("Créer la bibliographie locale")
                    .size(theme::jetons::DETAIL)
            )
            .on_press(Message::BibliographieCreer)
            .padding([5, 10])
            .width(Fill),
        );
    }

    if !state.references_repliees {
        content = content.push(
            text_input("chercher une référence", &state.references_filtre)
                .on_input(Message::ReferencesFiltrer)
                .padding(5)
                .size(theme::jetons::DETAIL)
                .width(Fill),
        );
    }

    let missing = project.bibliography.is_none();

    content = content.push(info_row(
        theme::icon(
            lucide_icons::iced::icon_book::<Theme>(),
            14.0,
            palette.text_muted,
        ),
        "references.bib".to_string(),
        if missing {
            "absent".to_string()
        } else {
            format!("{} entrée(s)", project.bibliography_entries)
        },
        missing,
        palette,
    ));

    // Un projet lié vit de sa collection : on le voit, et on peut s'en
    // dédire. Sinon, la liaison se propose — pour une bibliothèque locale
    // seulement, car une notice Zotero déjà pointée n'a pas de deuxième chef.
    if let Some(lien) = state.project.zotero.as_ref() {
        content = content.push(Space::new().height(6));
        content = content.push(
            row![
                theme::icon(
                    lucide_icons::iced::icon_book::<Theme>(),
                    11.0,
                    palette.success,
                ),
                palette
                    .label(format!("Zotero · {}", lien.nom))
                    .size(theme::jetons::CORPS),
                Space::new().width(Fill),
                button(
                    palette
                        .label("Synchroniser")
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted)
                )
                .on_press(Message::ZoteroSynchroniser)
                .padding([4, 8]),
                button(
                    palette
                        .label("Délier")
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted)
                )
                .on_press(Message::ZoteroDelier)
                .padding([4, 8]),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    } else if state.zotero.source == crate::zotero::Source::Locale {
        content = content.push(Space::new().height(6));
        content = content.push(
            button(
                row![
                    theme::icon(
                        lucide_icons::iced::icon_link::<Theme>(),
                        11.0,
                        palette.text_muted,
                    ),
                    palette.label("Lier à Zotero…").size(theme::jetons::CORPS),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::ZoteroLierOuvrir)
            .padding([4, 8])
            .width(Fill)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );
    }

    // Une notice s'ajoute quand la bibliothèque est la sienne : seule une
    // bibliothèque locale s'écrit.
    let locale = state.zotero.source == crate::zotero::Source::Locale;

    if locale && !missing {
        content = content.push(Space::new().height(6));
        content = content.push(
            button(
                row![
                    theme::icon(
                        lucide_icons::iced::icon_plus::<Theme>(),
                        11.0,
                        palette.text_muted,
                    ),
                    palette
                        .label("Nouvelle référence")
                        .size(theme::jetons::CORPS),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::EditionReferenceNouveau)
            .padding([4, 8])
            .width(Fill)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );
    }

    // Les clés citables, quand la bibliographie en contient : un clic insère
    // `@clé` au curseur. Le filtre cherche l'auteur, le titre ou la clé.
    let filtre = state.references_filtre.trim().to_lowercase();

    let visibles = state.bibliography.iter().filter(|entry| {
        filtre.is_empty()
            || entry.key.to_lowercase().contains(&filtre)
            || entry.titre.to_lowercase().contains(&filtre)
            || entry
                .auteurs
                .iter()
                .any(|(nom, prenom)| format!("{nom} {prenom}").to_lowercase().contains(&filtre))
    });

    if !state.references_repliees {
        for (rang, entry) in visibles.enumerate() {
            if rang > 0 {
                content = content.push(separation(palette));
            }

            let survolee = state.survol_reference.as_deref() == Some(entry.key.as_str());
            content = content.push(citation_row(entry, palette, survolee));
        }
    }

    // ---- Les projets ----
    content = content.push(Space::new().height(18));
    content = content.push(section_title("Projets", palette));

    // Le projet courant en tête : la liste sert à en changer, pas à se repérer.
    for (rang, other) in state.projects.iter().enumerate() {
        if rang > 0 {
            content = content.push(separation(palette));
        }

        let is_current = other.dir == project.dir;

        content = content.push(
            row![
                button(palette.label(other.name.clone()).size(theme::jetons::CORPS))
                    .on_press(Message::OpenProject(other.dir.clone()))
                    .width(Fill)
                    .height(Length::Fixed(ENTRY_HEIGHT))
                    .padding([4, 8])
                    .style(move |t, status| entry_style(t, status, palette, is_current)),
                button(theme::icon(
                    lucide_icons::iced::icon_pencil::<Theme>(),
                    12.0,
                    palette.text_muted,
                ))
                .on_press(Message::ProjetRenommer(other.dir.clone()))
                .padding([4, 6])
                .style(move |t, status| entry_style(t, status, palette, false)),
                button(theme::icon(
                    lucide_icons::iced::icon_trash_2::<Theme>(),
                    12.0,
                    palette.text_muted,
                ))
                .on_press(Message::ProjetSupprimer(other.dir.clone()))
                .padding([4, 6])
                .style(move |t, status| delete_style(t, status, palette)),
            ]
            .spacing(2),
        );
    }

    // Un seul bouton : la création ouvre sa fenêtre, plutôt que de tenir
    // champs et boutons en permanence dans le panneau.
    content = content.push(
        button(
            row![
                theme::icon(
                    lucide_icons::iced::icon_plus::<Theme>(),
                    13.0,
                    palette.text_muted
                ),
                palette.label("Nouveau projet").size(theme::jetons::CORPS),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ProjetCreer)
        .width(Fill)
        .height(Length::Fixed(ENTRY_HEIGHT))
        .padding([4, 8])
        .style(move |t, status| entry_style(t, status, palette, false)),
    );

    container(scrollable(content).width(Fill).height(Fill))
        .width(Length::Fill)
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            ..Default::default()
        })
        .into()
}

/// Une ligne de chapitre : son numéro, son titre, son compte de mots, et de
/// quoi le renommer ou le supprimer.
///
/// Quand ce chapitre est celui qu'on renomme, la ligne devient un champ de
/// saisie : l'édition se fait là où on lit, sans boîte de dialogue.
fn chapter_row<'a>(
    chapter: &'a Chapter,
    renaming: Option<&'a ChapterRename>,
    is_active: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let being_renamed = renaming
        .filter(|renaming| renaming.path == chapter.path)
        .map(|renaming| renaming.buffer.clone());

    if let Some(buffer) = being_renamed {
        return renaming_row(chapter, buffer, palette);
    }

    let mut label = row![].spacing(8).align_y(Alignment::Center);

    if is_active {
        // Le repère d'activité est un point, pas une flèche en texte : même
        // information, aucun bruit typographique.
        label = label.push(theme::icon(
            lucide_icons::iced::icon_dot::<Theme>(),
            9.0,
            palette.accent,
        ));
    }

    // Le numéro passe en avertissement quand le chapitre n'est pas assemblé :
    // la couleur suffit à repérer l'oubli dans la liste.
    label = label.push(
        palette
            .label(format!("{:02}", chapter.order))
            .size(theme::jetons::DETAIL)
            .color(if chapter.included {
                palette.text_muted
            } else {
                palette.warning
            }),
    );

    label = label.push(
        palette
            .label(chapter.title.clone())
            .size(theme::jetons::SAILLANT)
            .color(palette.text),
    );

    let title = button(label)
        .on_press(Message::OpenChapter(chapter.path.clone()))
        .width(Fill)
        .height(Length::Fixed(ENTRY_HEIGHT))
        .padding([4, 8])
        .style(move |t, status| entry_style(t, status, palette, is_active));

    row![
        tooltip(
            title,
            chapter_details(chapter, palette),
            tooltip::Position::Right
        )
        .gap(6),
        container(
            palette
                .label(format_count(chapter.words))
                .size(theme::jetons::CORPS)
                .color(palette.text_muted)
        )
        .width(Length::Fixed(COUNT_WIDTH))
        .height(Length::Fixed(ENTRY_HEIGHT))
        .align_x(Alignment::End)
        .center_y(ENTRY_HEIGHT),
        icon_action(
            lucide_icons::iced::icon_pencil::<Theme>(),
            "Renommer le chapitre",
            Message::StartRename(chapter.path.clone()),
            palette,
        ),
        icon_action(
            lucide_icons::iced::icon_trash_2::<Theme>(),
            "Supprimer le chapitre",
            Message::DeleteChapter(chapter.path.clone()),
            palette,
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .into()
}

/// La ligne d'un chapitre en cours de renommage : un champ de saisie, et de
/// quoi valider ou abandonner.
fn renaming_row<'a>(
    chapter: &'a Chapter,
    buffer: String,
    palette: Palette,
) -> Element<'a, Message> {
    row![
        palette
            .label(format!("{:02}", chapter.order))
            .size(theme::jetons::DETAIL)
            .color(palette.text_muted),
        text_input::TextInput::new("Titre du chapitre", &buffer)
            .on_input(Message::RenameChanged)
            .on_submit(Message::RenameCommit)
            .size(theme::jetons::SAILLANT)
            .padding([4, 8])
            .width(Fill),
        icon_action(
            lucide_icons::iced::icon_check::<Theme>(),
            "Valider",
            Message::RenameCommit,
            palette,
        ),
        icon_action(
            lucide_icons::iced::icon_x::<Theme>(),
            "Abandonner",
            Message::RenameCancel,
            palette,
        ),
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .height(Length::Fixed(ENTRY_HEIGHT))
    .into()
}

/// Une action discrète, réduite à une icône.
fn icon_action<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    tooltip_text: &'static str,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    tooltip(
        button(theme::icon(glyph, 13.0, palette.text_muted))
            .on_press(message)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(ENTRY_HEIGHT))
            .padding([4, 4])
            .style(move |t, status| delete_style(t, status, palette)),
        container(
            palette
                .label(tooltip_text)
                .size(theme::jetons::DETAIL)
                .color(palette.text),
        )
        .padding([4, 8])
        .style(move |_| tooltip_style(palette)),
        tooltip::Position::Right,
    )
    .gap(6)
    .into()
}

/// L'infobulle d'un chapitre : son état d'assemblage et son chemin.
fn chapter_details<'a>(chapter: &'a Chapter, palette: Palette) -> Element<'a, Message> {
    let (status, color) = if chapter.included {
        ("Inclus dans main.typ".to_string(), palette.text_muted)
    } else {
        (
            "Hors assemblage : n'apparaîtra pas dans le PDF".to_string(),
            palette.warning,
        )
    };

    container(
        column![
            palette
                .label(chapter.title.clone())
                .size(theme::jetons::CORPS),
            palette
                .label(status)
                .size(theme::jetons::DETAIL)
                .color(color),
            palette
                .label(format_path(&chapter.path))
                .size(theme::jetons::CHAPEAU)
                .color(palette.text_muted),
        ]
        .spacing(2),
    )
    .padding([6, 10])
    .style(move |_| tooltip_style(palette))
    .into()
}

/// Une ligne d'information : une icône, un nom, un détail à droite.
fn info_row<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    name: String,
    detail: String,
    missing: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let color = if missing {
        palette.warning
    } else {
        palette.text
    };

    row![
        container(glyph).padding([0, 4]).center_y(ENTRY_HEIGHT),
        palette
            .label(name)
            .size(theme::jetons::SAILLANT)
            .color(color),
        Space::new().width(Length::Fill),
        palette
            .label(detail)
            .size(theme::jetons::DETAIL)
            .color(if missing {
                palette.warning
            } else {
                palette.text_muted
            }),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .height(Length::Fixed(ENTRY_HEIGHT))
    .into()
}

/// Une référence citable : un clic insère `@clé`, l'icône l'ouvre pour la
/// modifier — dans une bibliothèque locale seulement.
fn citation_row<'a>(entry: &'a BibliographyEntry, palette: Palette, survolee: bool) -> Element<'a, Message> {
    // Deux registres plutôt qu'une ligne coupée : l'auteur au premier,
    // l'ouvrage au second en retrait — l'œil suit, au lieu de deviner ce
    // que les trois points ont emporté. La clé reste dans l'infobulle,
    // avec son type — une machine, pas un nom.
    let info = format!("@{} · {}", entry.key, entry.kind);

    let auteurs = entry
        .auteurs
        .iter()
        .map(|(nom, prenom)| {
            if prenom.trim().is_empty() {
                nom.clone()
            } else {
                format!("{nom}, {prenom}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ; ");

    let tete = if auteurs.trim().is_empty() {
        entry.key.clone()
    } else {
        auteurs
    };

    let mut registres = column![palette.label(tete).size(theme::jetons::CORPS)].spacing(1);

    if !entry.titre.trim().is_empty() {
        registres = registres.push(
            palette
                .label(tronquer(&entry.titre, 48))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    // Les gestes — éditer, supprimer — ne paraissent qu'au survol : une
    // liste de références est un texte à lire, pas une barre d'outils
    // (bug 13 du rapport 2).
    let mut ligne = row![
        button(registres)
            .on_press(Message::InsertCitation(entry.key.clone()))
            .width(Fill)
            .padding([7, 8])
            .style(move |t, status| entry_style(t, status, palette, false)),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    if survolee {
        ligne = ligne.push(
            button(theme::icon(
                lucide_icons::iced::icon_pencil::<Theme>(),
                12.0,
                palette.text_muted,
            ))
            .on_press(Message::ModifierReference(entry.key.clone()))
            .padding(6)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );

        ligne = ligne.push(
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<Theme>(),
                12.0,
                palette.danger,
            ))
            .on_press(Message::ReferenceSupprimer(entry.key.clone()))
            .padding(6)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );
    }

    let ligne = mouse_area(ligne)
        .on_enter(Message::SurvolReference(Some(entry.key.clone())))
        .on_exit(Message::SurvolReference(None));

    tooltip(
        ligne,
        palette.label(info).size(theme::jetons::DETAIL),
        tooltip::Position::Right,
    )
    .gap(6)
    .into()
}

/// Raccourcit un libellé trop long : « … » marque la coupure.
fn tronquer(texte: &str, largeur: usize) -> String {
    if texte.chars().count() <= largeur {
        return texte.to_string();
    }

    let coupe: String = texte.chars().take(largeur.saturating_sub(1)).collect();

    format!("{coupe}…")
}

/// Titre de section, en petites capitales discrètes.
fn section_title(title: &str, palette: Palette) -> text::Text<'_> {
    palette
        .label(title.to_uppercase())
        .size(theme::jetons::CHAPEAU)
        .color(palette.text_muted)
}

/// Nombre de mots, groupé par milliers pour rester lisible.
fn format_count(words: usize) -> String {
    let digits = words.to_string();

    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);

    for (count, character) in digits.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            // Une espace fine insécable : le groupe ne doit pas se couper en fin
            // de ligne.
            grouped.push('\u{202f}');
        }

        grouped.push(character);
    }

    grouped.chars().rev().collect()
}

/// Chemin réduit à ses deux derniers éléments.
///
/// Le préfixe `~/.typst-writer/projets/<nom>/` est le même pour tout le monde :
/// l'afficher dans une infobulle n'apprend rien.
fn format_path(path: &std::path::Path) -> String {
    let parts: Vec<String> = path
        .components()
        .rev()
        .take(2)
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();

    parts.into_iter().rev().collect::<Vec<_>>().join("/")
}

/// Bouton principal : plein, pour qu'il se détache du reste.
fn primary_style(_t: &Theme, status: button::Status, palette: Palette) -> button::Style {
    let background = match status {
        button::Status::Active | button::Status::Hovered => Some(palette.accent),
        button::Status::Pressed => Some(palette.surface_active),
        button::Status::Disabled => Some(palette.border),
    };

    button::Style {
        background: background.map(Into::into),
        // Le texte doit rester lisible sur le fond réel : un thème écrit son
        // propre accent, celui d'iced n'est pas le bon — on mesure.
        text_color: theme::lisible_sur(background.unwrap_or(palette.surface)),
        border: iced::Border {
            radius: crate::ui::theme::jetons::RAYON.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Entrée de liste : rien au repos, une brume au survol, et la sélection
/// **par la bordure** — le bloc de couleur ne se teinte pas.
/// Un filet entre deux entrées : les listes se lisent ligne à ligne.
///
/// Sans lui, une bibliothèque de notices est un mur de texte ; avec lui,
/// chaque entrée a sa place, et l'œil ne se perd pas entre deux clés.
fn separation(palette: Palette) -> Element<'static, Message> {
    iced::widget::container(iced::widget::Space::new())
        .width(Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| iced::widget::container::Style {
            background: Some(palette.border.into()),
            ..Default::default()
        })
        .into()
}

fn entry_style(
    _t: &Theme,
    status: button::Status,
    palette: Palette,
    active: bool,
) -> button::Style {
    let background = if active {
        None
    } else {
        match status {
            button::Status::Active => None,
            button::Status::Hovered => Some(palette.surface_hover),
            button::Status::Pressed => Some(palette.surface_active),
            button::Status::Disabled => None,
        }
    };

    button::Style {
        background: background.map(Into::into),
        text_color: if active { palette.accent } else { palette.text },
        border: iced::Border {
            color: if active {
                palette.accent
            } else {
                iced::Color::TRANSPARENT
            },
            width: if active { 1.5 } else { 1.0 },
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

/// Corbeille : discrète jusqu'au survol, où sa bordure se teinte de danger.
fn delete_style(_t: &Theme, status: button::Status, palette: Palette) -> button::Style {
    let (lisiere, text_color) = match status {
        button::Status::Active => (None, palette.text_muted),
        button::Status::Hovered | button::Status::Pressed => (Some(palette.danger), palette.danger),
        button::Status::Disabled => (None, palette.text_muted),
    };

    button::Style {
        background: None,
        text_color,
        border: iced::Border {
            color: lisiere.unwrap_or(iced::Color::TRANSPARENT),
            width: if lisiere.is_some() { 1.5 } else { 1.0 },
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

/// Fond d'une infobulle.
fn tooltip_style(palette: Palette) -> container::Style {
    container::Style {
        background: Some(palette.surface_hover.into()),
        border: iced::Border {
            color: palette.border,
            width: 1.0,
            radius: crate::ui::theme::jetons::RAYON.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les comptes de mots sont groupés par milliers : un projet de 12 000 mots
    /// doit se lire d'un coup d'œil, pas se déchiffrer.
    #[test]
    fn word_counts_are_grouped() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_000), "1\u{202f}000");
        assert_eq!(format_count(12_345), "12\u{202f}345");
        assert_eq!(format_count(1_234_567), "1\u{202f}234\u{202f}567");
    }

    /// Un chemin affiché se réduit à ses deux derniers éléments.
    #[test]
    fn displayed_paths_keep_only_the_last_components() {
        let path = std::path::Path::new(
            "/home/mat/.typst-writer/projets/mon-travail/chapitres/01-intro.typ",
        );

        assert_eq!(format_path(path), "chapitres/01-intro.typ");
    }
}
