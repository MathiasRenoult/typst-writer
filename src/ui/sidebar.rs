use crate::app::state::ChapterRename;
use crate::app::{Message, State};
use crate::project::{BibliographyEntry, Chapter};
use crate::ui::theme::{self, Palette};
use iced::widget::canvas;
use iced::widget::{
    Space, button, column, container, mouse_area, pick_list, row, scrollable, text_input, tooltip,
};
use iced::{Alignment, Element, Fill, Length, Theme};

/// Hauteur d'une ligne.
const ENTRY_HEIGHT: f32 = 32.0;

/// Le pas d'un niveau de sommaire : une colonne de seize pixels par
/// profondeur. Il est volontairement plus large qu'un simple retrait de
/// texte : c'est l'écart qui rend la branche lisible, comme la colonne
/// d'un fil de commentaires.
const PAS_HIERARCHIE: f32 = 16.0;

/// Largeur de la colonne des comptes de mots.
///
/// Fixe, et alignée à droite : c'est ce qui fait qu'on parcourt la colonne des
/// chiffres d'un seul regard, sans que les nombres dansent d'une ligne à
/// l'autre.
const COUNT_WIDTH: f32 = 58.0;

/// L'identifiant du champ de recherche biblique : le panneau lui donne le
/// clavier dès qu'il s'ouvre, au lieu de laisser la frappe à l'éditeur.
pub(crate) fn id_recherche_biblique() -> iced::widget::Id {
    iced::widget::Id::new("statistiques-bibliques-recherche")
}

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

    // Les repères sont en tête : le nom du document, son volume, et ce
    // qu'il mobilise. Le reste du panneau (sommaire, notices, projets) vient
    // après — on doit savoir où l'on est avant de chercher une section.
    content = content.push(
        container(
            column![
                palette.label(&project.name).size(theme::jetons::SAILLANT),
                row![
                    palette
                        .label(format!("{} mots", format_count(project.words())))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                    Space::new().width(Length::Fill),
                    palette
                        .label(format!(
                            "{} références bibliques",
                            state.plugins.bible.stats.par_reference.len()
                        ))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .align_y(Alignment::Center),
                row![
                    palette
                        .label(format!("{} notices", project.bibliography_entries))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                    Space::new().width(Length::Fill),
                    palette
                        .label(format!("{} termes", state.plugins.glossaire.entrees.len()))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(3),
        )
        .padding(8)
        .style(move |_| iced::widget::container::Style {
            background: Some(palette.surface_hover.into()),
            border: iced::Border {
                color: palette.border,
                width: 1.0,
                radius: crate::ui::theme::jetons::RAYON.into(),
            },
            ..Default::default()
        }),
    );
    content = content.push(Space::new().height(10));

    // TOPBAR-06-01 : le bouton de fermeture est dans la top bar,
    // pas dans le panneau. Le sommaire commence directement.
    content = content.push(Space::new().height(2));

    // SIDEBAR-06-02 : le sommaire est le premier élément du panneau.
    if !state.render.outline.is_empty() {
        content = content.push(section_header(
            "Sommaire",
            state.sommaire_replie,
            Message::SommaireReplier,
            palette,
        ));

        if state.sommaire_replie {
            // Replié : le titre seul reste, et la section suivante reprend
            // sa place sans laisser un grand vide.
        } else {
            let mut sommaire = column![].spacing(0);
            let niveaux: Vec<usize> = state
                .render
                .outline
                .iter()
                .map(|entree| entree.level.saturating_sub(1))
                .collect();

            for (index, entry) in state.render.outline.iter().enumerate() {
                let level = niveaux[index];
                // La coude ne descend jusqu'au bas de la rangée que si
                // l'entrée suivante est **son enfant**. Une feuille suivie
                // d'une sœur n'a personne à rejoindre : son trait s'arrête au
                // milieu, sinon il laisse un moignon au bord de la rangée et
                // la ligne paraît coupée.
                let est_dernier = !a_un_enfant(&niveaux, index);

                // Les guides des ancêtres : un trait vertical là où la branche
                // continue en dessous, rien là où elle s'achève. Les **traits**,
                // pas les glyphes de boîte — dans une police proportionnelle
                // ils dérivent, et la liste partait en diagonale
                // (bug-report-7, point 13).
                let guides = guides_du_sommaire(&niveaux, index);

                // Du **texte cliquable**, pas un bouton : les boutons
                // s'espacent entre eux et la hiérarchie part en escalier. Les
                // traits sont dessinés, le texte est nu — et sous la main, il
                // se teinte, le curseur pointe (bug-report 2026-11-03).
                let survole = state.survol_sommaire == Some(index);

                // Un titre de niveau 1 est la tête du fil : plus grand, plus
                // dense. Les réponses s'allègent à mesure qu'elles
                // s'enfoncent, sans jamais perdre leur lisibilité. C'est la
                // même lecture qu'un fil de commentaires : la branche se
                // reconnaît à sa colonne avant de se reconnaître à sa taille.
                let (taille, couleur, police) = if level == 0 {
                    (
                        theme::jetons::SAILLANT,
                        if survole {
                            palette.accent
                        } else {
                            palette.text
                        },
                        iced::Font {
                            weight: iced::font::Weight::Semibold,
                            ..palette.interface
                        },
                    )
                } else if level == 1 {
                    (
                        theme::jetons::CORPS,
                        if survole {
                            palette.accent
                        } else {
                            palette.text
                        },
                        palette.interface,
                    )
                } else {
                    (
                        theme::jetons::DETAIL,
                        if survole {
                            palette.accent
                        } else {
                            palette.text_muted
                        },
                        palette.interface,
                    )
                };

                let ligne = container(
                    row![
                        barres_hiérarchiques(&guides, est_dernier, level, palette),
                        palette
                            .label(entry.text.trim())
                            .size(taille)
                            .font(police)
                            .color(couleur),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .height(Length::Fixed(ENTRY_HEIGHT)),
                )
                .width(Length::Fill)
                .padding([0, 6])
                .style(move |_| iced::widget::container::Style {
                    background: survole.then_some(palette.surface_hover.into()),
                    border: iced::Border {
                        radius: crate::ui::theme::jetons::RAYON.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                sommaire = sommaire.push(
                    mouse_area(ligne)
                        .on_press(Message::GotoHeading(index))
                        .on_enter(Message::SommaireSurvol(Some(index)))
                        .on_exit(Message::SommaireSurvol(None))
                        .interaction(iced::mouse::Interaction::Pointer),
                );
            }

            content = content.push(sommaire);
            content = content.push(Space::new().height(12));
        }
    }

    // LEFT-04-02 : la section « Document » est supprimée — le nom du
    // document est en en-tête, le compte de mots dans la barre d'état.
    if let Some(document) = project.document.as_ref() {
        let _ = document;
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
        content = content.push(section_header(
            "Chapitres",
            state.chapitres_replie,
            Message::ChapitresReplier,
            palette,
        ));

        if !state.chapitres_replie {
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
    }

    if let Some(cliquee) = &state.plugins.bible.reference_cliquee {
        content = content.push(separation(palette));
        content = content.push(Space::new().height(12));
        content = content.push(reference_cliquee_section(state, cliquee, palette));
    }

    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_REFERENCES_BIBLIQUES)
    {
        // ---- Les statistiques des versets cités (l'add-on biblique) ----
        let replie = state.plugins.bible.repliees;
        content = content.push(separation(palette));
        content = content.push(Space::new().height(12));
        content = content.push(section_header(
            "Statistiques bibliques",
            replie,
            Message::StatsReplier,
            palette,
        ));
        if !replie {
            let mut gestes = row![].spacing(2).align_y(Alignment::Center);
            gestes = gestes.push(stat_icon(
                lucide_icons::iced::icon_search::<Theme>(),
                "Chercher une référence",
                Message::BibleRechercheBascule,
                palette,
                state.plugins.bible.recherche_active,
            ));
            gestes = gestes.push(stat_icon(
                lucide_icons::iced::icon_list_ordered::<Theme>(),
                "Trier dans l'ordre du document",
                Message::BibleTri(crate::plugins::bible::TriReferences::Apparition),
                palette,
                state.plugins.bible.tri == crate::plugins::bible::TriReferences::Apparition,
            ));
            gestes = gestes.push(stat_icon(
                lucide_icons::iced::icon_book_open::<Theme>(),
                "Trier dans l'ordre canonique des livres",
                Message::BibleTri(crate::plugins::bible::TriReferences::Canonique),
                palette,
                state.plugins.bible.tri == crate::plugins::bible::TriReferences::Canonique,
            ));
            content = content.push(gestes);
            if state.plugins.bible.recherche_active {
                content = content.push(
                    text_input("Chercher une référence", &state.plugins.bible.recherche)
                        .id(id_recherche_biblique())
                        .on_input(Message::BibleFiltrer)
                        .padding([4, 6]),
                );
            }
            content = content.push(Space::new().height(6));
            content = content.push(crate::ui::statistiques::view(state));
        }
    }

    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_BIBLIOGRAPHIE)
    {
        // ---- La bibliographie ----
        content = content.push(separation(palette));
        content = content.push(Space::new().height(12));

        // Le titre replie la liste, et le filtre la cherche : une thèse a
        // des centaines de notices, et une liste qui ne se cherche pas est un
        // mur.
        content = content.push(section_header(
            "Bibliographie",
            state.plugins.bibliographie.repliees,
            Message::ReferencesReplier,
            palette,
        ));

        // Les gestes de la bibliographie, **en icônes** (bug-report
        // 2026-11-03) : création, import, recherche — et le lien Zotero
        // rejoint la rangée. Les gestes d'insertion (note, figure, étiquette,
        // renvoi) sont retirés : ils ne servaient à rien ici.
        let mut gestes = row![].spacing(2).align_y(Alignment::Center);
        if state.project.bibliography.is_none() {
            gestes = gestes.push(tooltip(
                button(theme::icon(
                    lucide_icons::iced::icon_file_plus::<Theme>(),
                    13.0,
                    palette.text,
                ))
                .on_press(Message::BibliographieCreer)
                .padding([4, 6])
                .style(move |t, status| entry_style(t, status, palette, false)),
                palette
                    .label("Créer la bibliographie locale")
                    .size(theme::jetons::DETAIL),
                tooltip::Position::Bottom,
            ));
        }
        gestes = gestes.push(tooltip(
            button(theme::icon(
                lucide_icons::iced::icon_upload::<Theme>(),
                13.0,
                palette.text,
            ))
            .on_press(Message::BibliographieImporter)
            .padding([4, 6])
            .style(move |t, status| entry_style(t, status, palette, false)),
            palette
                .label("Importer un fichier .bib…")
                .size(theme::jetons::DETAIL),
            tooltip::Position::Bottom,
        ));
        gestes = gestes.push(tooltip(
            button(theme::icon(
                lucide_icons::iced::icon_search::<Theme>(),
                13.0,
                palette.text,
            ))
            .on_press(Message::ReferencesRechercheBascule)
            .padding([4, 6])
            .style(move |t, status| entry_style(t, status, palette, false)),
            palette
                .label("Chercher une référence")
                .size(theme::jetons::DETAIL),
            tooltip::Position::Bottom,
        ));
        if state.zotero.source == crate::zotero::Source::Locale {
            gestes = gestes.push(tooltip(
                button(theme::icon(
                    lucide_icons::iced::icon_link::<Theme>(),
                    13.0,
                    palette.text,
                ))
                .on_press(Message::ZoteroLierOuvrir)
                .padding([4, 6])
                .style(move |t, status| entry_style(t, status, palette, false)),
                palette.label("Lier à Zotero…").size(theme::jetons::DETAIL),
                tooltip::Position::Bottom,
            ));
        }
        content = content.push(gestes);

        // La dernière notice supprimée se ravise d'un geste : sa copie est à
        // la corbeille, et c'est elle qu'on remet.
        if let Some(copie) = state.derniere_notice_supprimee.clone() {
            content = content.push(
                button(
                    palette
                        .label("Restaurer la dernière notice supprimée")
                        .size(theme::jetons::DETAIL),
                )
                .on_press(Message::ReferenceRestaurer(copie))
                .padding([5, 10])
                .width(Fill),
            );
        }

        // La recherche ne se montre que sur son icône (bug-report 2026-11-03).
        if !state.plugins.bibliographie.repliees
            && (state.plugins.bibliographie.recherche_visible
                || !state.plugins.bibliographie.filtre.is_empty())
        {
            content = content.push(
                text_input(
                    "chercher une référence",
                    &state.plugins.bibliographie.filtre,
                )
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
            "Entrées bibliographiques".to_string(),
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
        }
        // Le lien Zotero s'est joint à la rangée de gestes (bug-report
        // 2026-11-03) — l'icône, comme les autres.

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
        let filtre = state.plugins.bibliographie.filtre.trim().to_lowercase();

        let visibles = state.bibliography.iter().filter(|entry| {
            filtre.is_empty()
                || entry.key.to_lowercase().contains(&filtre)
                || entry.titre.to_lowercase().contains(&filtre)
                || entry
                    .annee
                    .as_deref()
                    .map(|a| a.to_lowercase().contains(&filtre))
                    .unwrap_or(false)
                || entry
                    .auteurs
                    .iter()
                    .any(|(nom, prenom)| format!("{nom} {prenom}").to_lowercase().contains(&filtre))
        });

        // La recherche trouve ses résultats même la section repliée
        // (bug-report-10, 2.2).
        if !state.plugins.bibliographie.repliees || !filtre.is_empty() {
            for (rang, entry) in visibles.enumerate() {
                if rang > 0 {
                    content = content.push(separation(palette));
                }

                let survolee =
                    state.plugins.bibliographie.survol.as_deref() == Some(entry.key.as_str());
                content = content.push(citation_row(entry, palette, survolee));
            }
        }
    }

    if state
        .reglages
        .fonctionnalites
        .actif(crate::base::valeurs::CLE_GLOSSAIRE)
    {
        // ---- Le glossaire ----
        content = content.push(separation(palette));
        content = content.push(Space::new().height(12));

        // Le titre replie la liste, et le filtre la cherche.
        content = content.push(section_header(
            "Glossaire",
            state.plugins.glossaire.replie,
            Message::GlossaireReplier,
            palette,
        ));

        // Les gestes : un terme s'ajoute, la liste se cherche — en icônes,
        // comme la bibliographie.
        let mut gestes = row![].spacing(2).align_y(Alignment::Center);
        gestes = gestes.push(tooltip(
            button(theme::icon(
                lucide_icons::iced::icon_plus::<Theme>(),
                13.0,
                palette.text,
            ))
            .on_press(Message::GlossaireCreer)
            .padding([4, 6])
            .style(move |t, status| entry_style(t, status, palette, false)),
            palette.label("Nouveau terme").size(theme::jetons::DETAIL),
            tooltip::Position::Bottom,
        ));
        gestes = gestes.push(tooltip(
            button(theme::icon(
                lucide_icons::iced::icon_search::<Theme>(),
                13.0,
                palette.text,
            ))
            .on_press(Message::GlossaireRechercheBascule)
            .padding([4, 6])
            .style(move |t, status| entry_style(t, status, palette, false)),
            palette
                .label("Chercher un terme")
                .size(theme::jetons::DETAIL),
            tooltip::Position::Bottom,
        ));
        content = content.push(gestes);

        // La recherche ne se montre que sur son icône (bug-report 2026-11-03).
        if !state.plugins.glossaire.replie
            && (state.plugins.glossaire.recherche_visible
                || !state.plugins.glossaire.filtre.is_empty())
        {
            content = content.push(
                text_input("chercher un terme", &state.plugins.glossaire.filtre)
                    .on_input(Message::GlossaireFiltrer)
                    .padding(5)
                    .size(theme::jetons::DETAIL)
                    .width(Fill),
            );
        }

        // L'état : combien de termes — et le vide dit quoi faire.
        content = content.push(info_row(
            theme::icon(
                lucide_icons::iced::icon_book::<Theme>(),
                14.0,
                palette.text_muted,
            ),
            "Termes du glossaire".to_string(),
            if state.plugins.glossaire.entrees.is_empty() {
                "aucun".to_string()
            } else {
                format!("{} terme(s)", state.plugins.glossaire.entrees.len())
            },
            state.plugins.glossaire.entrees.is_empty(),
            palette,
        ));

        if state.plugins.glossaire.entrees.is_empty() {
            content = content.push(
                palette
                    .label(
                        "Le glossaire se gère ici : un terme porte sa clé, ses formes et sa \
                     définition. Un clic sur un terme de la liste l'insère au curseur.",
                    )
                    .size(theme::jetons::DETAIL)
                    .color(palette.text_muted),
            );
        }

        // Les termes citables : un clic insère `{glo:clé}` (ou `#gls("clé")`,
        // selon le dialecte du tampon). Le filtre cherche la clé, les formes
        // ou la définition.
        let filtre = state.plugins.glossaire.filtre.trim().to_lowercase();

        let visibles = state.plugins.glossaire.entrees.iter().filter(|entree| {
            filtre.is_empty()
                || entree.key.to_lowercase().contains(&filtre)
                || entree
                    .short
                    .as_deref()
                    .map(|forme| forme.to_lowercase().contains(&filtre))
                    .unwrap_or(false)
                || entree
                    .long
                    .as_deref()
                    .map(|forme| forme.to_lowercase().contains(&filtre))
                    .unwrap_or(false)
                || entree
                    .description
                    .as_deref()
                    .map(|forme| forme.to_lowercase().contains(&filtre))
                    .unwrap_or(false)
        });

        if !state.plugins.glossaire.replie {
            for (rang, entree) in visibles.enumerate() {
                if rang > 0 {
                    content = content.push(separation(palette));
                }

                let survolee =
                    state.plugins.glossaire.survol.as_deref() == Some(entree.key.as_str());
                content = content.push(glossaire_row(entree, palette, survolee));
            }
        }
    }

    // ---- Les projets ----

    content = content.push(Space::new().height(18));
    content = content.push(section_header(
        "Projets",
        state.projets_section_replie,
        Message::ProjetsSectionReplier,
        palette,
    ));

    if !state.projets_section_replie {
        // La liste des projets : l'étagère se choisit **au menu** (pas une
        // bascule), et la recherche ne se montre que sur son icône
        // (bug-report 2026-11-03).
        let etageres = &state.etageres;
        let mut choix_etages = vec!["Mon étagère".to_string(), "Tous".to_string()];
        let mut noms: Vec<String> = state.etageres_ordre.clone();
        noms.extend(
            etageres
                .values()
                .filter(|etagere| !etagere.trim().is_empty())
                .cloned(),
        );
        let mut vus = std::collections::HashSet::new();
        noms.retain(|nom| !nom.trim().is_empty() && vus.insert(nom.trim().to_string()));
        choix_etages.extend(noms);

        content = content.push(
            row![
                button(theme::icon(
                    lucide_icons::iced::icon_search::<Theme>(),
                    13.0,
                    palette.text,
                ))
                .on_press(Message::ProjetsRechercheBascule)
                .padding([4, 6])
                .style(move |t, status| entry_style(t, status, palette, false)),
                // La bibliothèque, vue étagères — l’accès depuis la
                // section Projets (bug-report-10, 2.3).
                tooltip(
                    button(theme::icon(
                        lucide_icons::iced::icon_library::<Theme>(),
                        13.0,
                        if state.projets_vue {
                            palette.accent
                        } else {
                            palette.text
                        },
                    ))
                    .on_press(Message::ProjetsVue)
                    .padding([4, 6])
                    .style(move |t, status| { entry_style(t, status, palette, state.projets_vue) }),
                    container(palette.label("La bibliothèque, vue étagères").size(12))
                        .padding([4, 8]),
                    tooltip::Position::Bottom,
                ),
                pick_list(
                    choix_etages,
                    Some(state.projets_etage.clone()),
                    Message::ProjetsEtageChoisie
                )
                .text_size(theme::jetons::DETAIL)
                .padding([3, 6])
                .width(Fill),
            ]
            .spacing(6),
        );
        if state.projets_recherche_visible || !state.projets_recherche.is_empty() {
            content = content.push(
                text_input("chercher un projet…", &state.projets_recherche)
                    .on_input(Message::ProjetsRecherche)
                    .padding([3, 8])
                    .size(theme::jetons::DETAIL)
                    .width(Fill),
            );
        }

        let visibles = projets_visibles(
            &state.projects,
            project,
            etageres,
            &state.projets_etage,
            &state.projets_recherche,
        );

        // Le projet courant en tête : la liste sert à en changer, pas à se repérer.
        for (rang, other) in visibles.iter().enumerate() {
            if rang > 0 {
                content = content.push(separation(palette));
            }

            let is_current = other.dir == project.dir;
            let survole = state.survol_projet.as_deref() == Some(other.dir.as_path());

            let mut ligne = row![
                button(
                    row![palette.label(other.name.clone()).size(theme::jetons::CORPS)]
                        .align_y(Alignment::Center)
                        .height(Length::Fill),
                )
                .on_press(Message::OpenProject(other.dir.clone()))
                .width(Fill)
                .height(Length::Fixed(ENTRY_HEIGHT))
                .padding([4, 8])
                .style(move |t, status| entry_style(t, status, palette, is_current)),
            ]
            .spacing(2);

            if survole {
                ligne = ligne.push(
                    button(theme::icon(
                        lucide_icons::iced::icon_pencil::<Theme>(),
                        12.0,
                        palette.text_muted,
                    ))
                    .on_press(Message::ProjetRenommer(other.dir.clone()))
                    .padding([4, 6])
                    .style(move |t, status| entry_style(t, status, palette, false)),
                );
                ligne = ligne.push(
                    button(theme::icon(
                        lucide_icons::iced::icon_trash_2::<Theme>(),
                        12.0,
                        palette.text_muted,
                    ))
                    .on_press(Message::ProjetSupprimer(other.dir.clone()))
                    .padding([4, 6])
                    .style(move |t, status| delete_style(t, status, palette)),
                );
            }

            let dir_clone = other.dir.clone();
            content = content.push(
                iced::widget::mouse_area(ligne)
                    .on_enter(Message::SurvolProjet(Some(dir_clone.clone())))
                    .on_exit(Message::SurvolProjet(None)),
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
                .align_y(Alignment::Center)
                .height(Length::Fill),
            )
            .on_press(Message::ProjetCreer)
            .width(Fill)
            .height(Length::Fixed(ENTRY_HEIGHT))
            .padding([4, 8])
            .style(move |t, status| entry_style(t, status, palette, false)),
        );
    }

    let sortie: Element<'_, Message> = container(scrollable(content).width(Fill).height(Fill))
        .width(Length::Fill)
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(palette.surface.into()),
            ..Default::default()
        })
        .into();
    sortie
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

    // Du texte cliquable, pas un bouton : les boutons s'espacent et la
    // liste part en escalier (bug-report 2026-11-03).
    let title = mouse_area(label.width(Fill).height(Length::Fixed(ENTRY_HEIGHT)))
        .on_press(Message::OpenChapter(chapter.path.clone()))
        .interaction(iced::mouse::Interaction::Pointer);

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

/// Une commande de statistiques réduite à une icône : recherche ou tri.
fn stat_icon<'a>(
    glyph: iced::widget::Text<'a, Theme>,
    tooltip_text: &'static str,
    message: Message,
    palette: Palette,
    active: bool,
) -> Element<'a, Message> {
    tooltip(
        button(theme::icon(
            glyph,
            13.0,
            if active {
                palette.accent
            } else {
                palette.text_muted
            },
        ))
        .on_press(message)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .padding([3, 3])
        .style(move |_, status| button::Style {
            background: match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(palette.surface_hover.into())
                }
                _ => None,
            },
            border: iced::Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        container(
            palette
                .label(tooltip_text)
                .size(theme::jetons::DETAIL)
                .color(palette.text),
        )
        .padding([4, 8])
        .style(move |_| tooltip_style(palette)),
        tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// La référence cliquée : ses citations séparées, leur livre, leur chapitre,
/// l'édition de l'une d'elles et la traduction de l'extrait.
fn reference_cliquee_section<'a>(
    state: &'a State,
    cliquee: &'a crate::plugins::bible::ReferenceCliquee,
    palette: Palette,
) -> Element<'a, Message> {
    let mut liste = column![].spacing(2);
    for (index, element) in cliquee.elements.iter().enumerate() {
        match element {
            crate::plugins::bible::ElementCitation::Reference(reference) => {
                let livre = crate::langage::references::nom_du_livre(
                    &reference.livre,
                    crate::langage::references::FormeLivre::FrancaiseLongue,
                );
                let nombres = crate::langage::references::afficher_nombres(
                    reference,
                    &state.reglages.references,
                );
                let mention = crate::langage::references::mention_traduction(reference);
                let extrait = crate::plugins::bible::extrait(
                    std::slice::from_ref(reference),
                    state.plugins.bible.traduction,
                );
                let mut entete = row![
                    palette.label(livre).size(theme::jetons::CORPS),
                    palette
                        .label(nombres)
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                ]
                .spacing(4)
                .align_y(Alignment::Center);
                if !mention.is_empty() {
                    entete = entete.push(
                        palette
                            .label(mention)
                            .size(theme::jetons::DETAIL)
                            .color(palette.text_muted),
                    );
                }
                entete = entete
                    .push(Space::new().width(Length::Fill))
                    .push(icon_action(
                        lucide_icons::iced::icon_pencil::<Theme>(),
                        "Modifier cette référence",
                        Message::BibleReferenceModifier(index),
                        palette,
                    ));
                liste = liste.push(
                    column![
                        entete.width(Length::Fill),
                        palette
                            .label(extrait)
                            .size(theme::jetons::DETAIL)
                            .color(palette.text_muted),
                    ]
                    .spacing(4),
                );
            }
            crate::plugins::bible::ElementCitation::Texte(texte) => {
                liste = liste.push(
                    palette
                        .label(format!("\"{texte}\""))
                        .size(theme::jetons::DETAIL)
                        .color(palette.text_muted),
                );
            }
        }
    }

    let corps = column![
        palette.label("La référence cliquée").size(13),
        liste,
        pick_list(
            vec!["Louis Segond 1910", "J.N. Darby"],
            Some(state.plugins.bible.traduction.nom()),
            |choix| {
                Message::BibleTraduction(if choix == "J.N. Darby" {
                    crate::plugins::bible::Traduction::Darby
                } else {
                    crate::plugins::bible::Traduction::Lsg
                })
            },
        )
        .width(Length::Fill),
    ]
    .spacing(6);
    corps.into()
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

/// Un terme du glossaire : un clic insère sa référence au curseur,
/// l'icône l'ouvre pour le corriger.
fn glossaire_row<'a>(
    entree: &'a crate::plugins::glossaire::EntreeGlossaire,
    palette: Palette,
    survolee: bool,
) -> Element<'a, Message> {
    let info = format!("{{glo:{}}} · glossaire", entree.key);

    let forme_courte = entree
        .short
        .as_deref()
        .map(str::trim)
        .filter(|forme| !forme.is_empty());
    let forme_longue = entree
        .long
        .as_deref()
        .map(str::trim)
        .filter(|forme| !forme.is_empty());

    // Deux registres : la forme courte (ou la clé) d'abord, la forme
    // longue ensuite en retrait — l'œil suit.
    let tete = forme_courte.unwrap_or(&entree.key);

    let mut registres = column![palette.label(tete).size(theme::jetons::CORPS)].spacing(1);

    if forme_courte.is_some()
        && let Some(longue) = forme_longue
    {
        registres = registres.push(
            palette
                .label(tronquer(longue, 48))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    } else if forme_courte.is_some()
        && let Some(description) = entree
            .description
            .as_deref()
            .map(str::trim)
            .filter(|texte| !texte.is_empty())
    {
        registres = registres.push(
            palette
                .label(tronquer(description, 48))
                .size(theme::jetons::DETAIL)
                .color(palette.text_muted),
        );
    }

    // Les gestes — éditer, supprimer — ne paraissent qu'au survol : une
    // liste de termes est un texte à lire, pas une barre d'outils.
    let mut ligne = row![
        button(registres)
            .on_press(Message::GlossaireInserer(entree.key.clone()))
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
            .on_press(Message::GlossaireModifier(entree.key.clone()))
            .padding(6)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );

        ligne = ligne.push(
            button(theme::icon(
                lucide_icons::iced::icon_trash_2::<Theme>(),
                12.0,
                palette.danger,
            ))
            .on_press(Message::GlossaireSupprimer(entree.key.clone()))
            .padding(6)
            .style(move |t, status| entry_style(t, status, palette, false)),
        );
    }

    let ligne = mouse_area(ligne)
        .on_enter(Message::SurvolGlossaire(Some(entree.key.clone())))
        .on_exit(Message::SurvolGlossaire(None));

    tooltip(
        ligne,
        palette.label(info).size(theme::jetons::DETAIL),
        tooltip::Position::Right,
    )
    .gap(6)
    .into()
}

/// Une référence citable : un clic insère `@clé`, l'icône l'ouvre pour la
/// modifier — dans une bibliothèque locale seulement.
fn citation_row<'a>(
    entry: &'a BibliographyEntry,
    palette: Palette,
    survolee: bool,
) -> Element<'a, Message> {
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

    // Le titre paraît toujours : le repli est celui de la section
    // entière, pas celui de chaque entrée (bug-report-10).
    if !entry.titre.trim().is_empty() {
        let titre = if let Some(annee) = entry.annee.as_deref().filter(|a| !a.trim().is_empty()) {
            format!("{} ({})", tronquer(&entry.titre, 48), annee.trim())
        } else {
            tronquer(&entry.titre, 48)
        };
        registres = registres.push(
            palette
                .label(titre)
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

/// Le titre d'une section, avec son chevron et son geste de repli.
///
/// Sommaire, Chapitres, Bibliographie, Glossaire et Projets partagent
/// exactement la même typographie, le même espacement et le même indicateur
/// d'état : c'est ce qui fait du panneau un ensemble, pas cinq interfaces.
fn section_header<'a>(
    title: &'a str,
    replie: bool,
    action: Message,
    palette: Palette,
) -> Element<'a, Message> {
    button(
        row![
            theme::icon(
                if replie {
                    lucide_icons::iced::icon_chevron_right::<Theme>()
                } else {
                    lucide_icons::iced::icon_chevron_down::<Theme>()
                },
                12.0,
                palette.text_muted,
            ),
            palette.label(title).size(theme::jetons::SAILLANT),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press(action)
    .padding([2, 4])
    .style(move |t, status| entry_style(t, status, palette, false))
    .into()
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

/// Les projets visibles dans la liste : l'étagère du projet courant par
/// défaut, tous sur demande, et la recherche filtre par nom.
///
/// Fonction pure — c'est la règle du point 18, éprouvée telle quelle.
pub fn projets_visibles<'a>(
    projects: &'a [crate::project::Project],
    projet_courant: &crate::project::Project,
    etageres: &std::collections::BTreeMap<String, String>,
    choix: &str,
    recherche: &str,
) -> Vec<&'a crate::project::Project> {
    let etagere_courante = etageres
        .get(&projet_courant.name)
        .cloned()
        .unwrap_or_default();
    let recherche = recherche.trim().to_lowercase();
    projects
        .iter()
        .filter(|p| {
            let etagere = etageres.get(&p.name).cloned().unwrap_or_default();
            // Le menu d'étagère : « Tous », « Mon étagère », ou une
            // étagère nommée (bug-report 2026-11-03).
            let ok_etage = match choix {
                "Tous" => true,
                "Mon étagère" => etagere == etagere_courante,
                nom => etagere == nom,
            };
            ok_etage && (recherche.is_empty() || p.name.to_lowercase().contains(&recherche))
        })
        .collect()
}

/// La géométrie des barres hiérarchiques — **pure** : un niveau = une
/// colonne de `PAS_HIERARCHIE` pixels, toujours au même x, quoi que fasse la
/// police. Chaque segment est `(x1, y1, x2, y2)` dans le canevas.
///
/// C'est la réponse au bug-report du 2026-11-03 : les lignes étaient
/// « pas alignées et difformes » — dessinées dans la marge des widgets,
/// elles dérivaient. Ici, la position ne dépend que des nombres.
///
/// Les entrées de niveau 1 n'ont pas de branche : leur pastille est dessinée
/// par [`Barres`]. Cette fonction ne décrit donc que les réponses.
fn segments(guides: &[bool], est_dernier: bool, hauteur: f32) -> Vec<(f32, f32, f32, f32)> {
    let mut segments = Vec::new();

    // Les guides des ancêtres : pleins là où la branche continue.
    for (k, guide) in guides.iter().enumerate() {
        if *guide {
            let x = k as f32 * PAS_HIERARCHIE + PAS_HIERARCHIE / 2.0;
            segments.push((x, 0.0, x, hauteur));
        }
    }

    // La coude de l'entrée : le vertical — demi pour la dernière
    // branche — et la barre qui va jusqu'au bord de la colonne, là où
    // commence le texte après l'espacement de la rangée.
    let x = guides.len() as f32 * PAS_HIERARCHIE + PAS_HIERARCHIE / 2.0;
    let fin = if est_dernier { hauteur / 2.0 } else { hauteur };
    segments.push((x, 0.0, x, fin));
    segments.push((
        x,
        hauteur / 2.0,
        x + PAS_HIERARCHIE / 2.0 + 8.0,
        hauteur / 2.0,
    ));
    segments
}

/// Les guides d'une entrée de sommaire, à partir des niveaux (0 = niveau 1).
///
/// Un guide d'ancêtre continue tant que l'entrée suivante reste dans le
/// sous-arbre ; dès qu'on remonte au niveau de l'ancêtre ou au-dessus, il
/// s'arrête. C'est cette règle, et non « toutes les entrées suivantes », qui
/// raccorde les branches entre deux enfants d'un même parent.
fn guides_du_sommaire(niveaux: &[usize], index: usize) -> Vec<bool> {
    let level = niveaux.get(index).copied().unwrap_or(0);

    (0..level)
        .map(|ancetre| {
            niveaux[index + 1..]
                .iter()
                .take_while(|suivant| **suivant > ancetre)
                .next()
                .is_some()
        })
        .collect()
}

/// L'entrée a-t-elle une entrée enfant juste après elle ?
///
/// C'est la seule chose qui doit prolonger son trait vertical jusqu'à la
/// rangée suivante : une feuille suivie d'une sœur n'a personne à rejoindre.
fn a_un_enfant(niveaux: &[usize], index: usize) -> bool {
    let level = niveaux.get(index).copied().unwrap_or(0);

    niveaux
        .get(index + 1)
        .is_some_and(|suivant| *suivant > level)
}

/// La largeur de la zone de barres, pour un nombre de guides.
fn largeur_barres(guides: usize) -> f32 {
    guides as f32 * PAS_HIERARCHIE + PAS_HIERARCHIE + 8.0
}

/// Le canevas des barres : la géométrie, rien d'autre. Il **possède**
/// ses guides — le widget ne se loue pas à une pile qui meurt.
struct Barres {
    guides: Vec<bool>,
    est_dernier: bool,
    niveau: usize,
    palette: Palette,
}

impl canvas::Program<Message> for Barres {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, iced::Size::new(bounds.width, bounds.height));

        // Un titre de niveau 1 est la **tête du fil** : une pastille au lieu
        // d'une coude. Sans elle, la racine ressemblait à une réponse sortie
        // d'un parent invisible ; la hiérarchie commençait par un trait qui
        // ne menait nulle part.
        if self.niveau == 0 {
            let centre = iced::Point::new(PAS_HIERARCHIE / 2.0, bounds.height / 2.0);
            frame.fill(
                &canvas::Path::circle(centre, 3.5),
                canvas::Fill::from(self.palette.accent),
            );

            // Quand une branche descend, le fil part de la pastille et
            // rejoint la rangée suivante : c'est ce qui fait tenir le fil
            // d'un commentaire à ses réponses.
            if !self.est_dernier {
                frame.fill(
                    &canvas::Path::rectangle(
                        iced::Point::new(centre.x.floor(), centre.y),
                        iced::Size::new(1.0, bounds.height - centre.y),
                    ),
                    canvas::Fill::from(self.palette.border),
                );
            }

            return vec![frame.into_geometry()];
        }

        // Des **rectangles pleins** au pixel, pas des traits tracés : un
        // trait de 1 px à la demi-pixel s'antialiasse et reste flou —
        // c'est le « pas propre » que l'on voyait.
        let pinceau = canvas::Fill::from(self.palette.border);
        for (x1, y1, x2, y2) in segments(&self.guides, self.est_dernier, bounds.height) {
            let rectangle = if (x1 - x2).abs() < 0.5 {
                // Vertical : un pixel de large, à l'entier près.
                canvas::Path::rectangle(
                    iced::Point::new(x1.floor(), y1.min(y2)),
                    iced::Size::new(1.0, (y2 - y1).abs()),
                )
            } else {
                // Horizontal : un pixel de haut.
                canvas::Path::rectangle(
                    iced::Point::new(x1.min(x2), y1.floor()),
                    iced::Size::new((x2 - x1).abs(), 1.0),
                )
            };
            frame.fill(&rectangle, pinceau);
        }
        vec![frame.into_geometry()]
    }
}

/// La barre hiérarchique d'une entrée de sommaire — dessinée, jamais
/// composée de glyphes ni de marge de widgets.
fn barres_hiérarchiques(
    guides: &[bool],
    est_dernier: bool,
    niveau: usize,
    palette: Palette,
) -> Element<'static, Message> {
    canvas::Canvas::new(Barres {
        guides: guides.to_vec(),
        est_dernier,
        niveau,
        palette,
    })
    .width(Length::Fixed(largeur_barres(guides.len())))
    .height(Length::Fill)
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
            "/home/quelqu-un/.typst-writer/projets/mon-travail/chapitres/01-intro.typ",
        );

        assert_eq!(format_path(path), "chapitres/01-intro.typ");
    }
}

#[cfg(test)]
mod tests_filtre {
    use super::projets_visibles;
    use crate::project::Project;

    fn projet(nom: &str) -> Project {
        Project {
            dir: std::path::PathBuf::from(format!("/p/{nom}")),
            problemes: Vec::new(),
            name: nom.to_string(),
            main: std::path::PathBuf::from(format!("/p/{nom}/main.typ")),
            document: None,
            document_words: 0,
            chapters: Vec::new(),
            bibliography: None,
            bibliography_entries: 0,
            zotero: None,
        }
    }

    /// La règle du point 18 : l'étagère courante par défaut, tous sur
    /// demande, la recherche filtre par nom.
    #[test]
    fn la_liste_ne_va_que_son_etagere() {
        let courant = projet("these");
        let tous = vec![projet("these"), projet("devoir"), projet("article")];
        let etageres = std::collections::BTreeMap::from([
            ("these".to_string(), "travail".to_string()),
            ("devoir".to_string(), "travail".to_string()),
            ("article".to_string(), "revue".to_string()),
        ]);

        // Par défaut : l'étagère du projet courant, rien d'autre.
        let visibles = projets_visibles(&tous, &courant, &etageres, "Mon étagère", "");
        assert_eq!(visibles.len(), 2, "l'étagère du courant : {visibles:?}");
        assert!(visibles.iter().all(|p| p.name != "article"));

        // Sur demande : tous.
        let visibles = projets_visibles(&tous, &courant, &etageres, "Tous", "");
        assert_eq!(visibles.len(), 3);

        // La recherche filtre par nom, sur ce qui est montré.
        let visibles = projets_visibles(&tous, &courant, &etageres, "Tous", "ART");
        assert_eq!(visibles.len(), 1);
        assert_eq!(visibles[0].name, "article");
    }

    /// Un projet sans étagère forme le groupe des non-filés.
    #[test]
    fn les_sans_etagere_se_tiennent() {
        let courant = projet("these");
        let tous = vec![projet("these"), projet("brouillon")];
        let etageres = std::collections::BTreeMap::new();
        let visibles = projets_visibles(&tous, &courant, &etageres, "Mon étagère", "");
        assert_eq!(visibles.len(), 2, "les non-filés ensemble");
    }
}

#[cfg(test)]
mod tests_menu_etagere {
    use super::projets_visibles;
    use crate::project::Project;

    fn projet(nom: &str) -> Project {
        Project {
            dir: std::path::PathBuf::from(format!("/p/{nom}")),
            problemes: Vec::new(),
            name: nom.to_string(),
            main: std::path::PathBuf::from(format!("/p/{nom}/main.typ")),
            document: None,
            document_words: 0,
            chapters: Vec::new(),
            bibliography: None,
            bibliography_entries: 0,
            zotero: None,
        }
    }

    /// Le menu d'étagère (bug-report 2026-11-03) : une étagère nommée
    /// montre la sienne, « Tous » montre tout.
    #[test]
    fn une_etagore_nomme_montre_la_sienne() {
        let courant = projet("these");
        let tous = vec![projet("these"), projet("devoir"), projet("article")];
        let etageres = std::collections::BTreeMap::from([
            ("these".to_string(), "travail".to_string()),
            ("devoir".to_string(), "travail".to_string()),
            ("article".to_string(), "revue".to_string()),
        ]);

        let visibles = projets_visibles(&tous, &courant, &etageres, "revue", "");
        assert_eq!(visibles.len(), 1, "l'étagère nommée : {visibles:?}");
        assert_eq!(visibles[0].name, "article");

        let visibles = projets_visibles(&tous, &courant, &etageres, "Tous", "");
        assert_eq!(visibles.len(), 3);
    }
}

#[cfg(test)]
mod tests_barres {
    use super::{a_un_enfant, guides_du_sommaire, largeur_barres, segments};

    /// Une feuille suivie d'une sœur ne continue pas sa branche : le trait
    /// s'arrête au milieu, sinon il laisse un moignon au bord de la rangée.
    #[test]
    fn seuls_les_parents_prolongent_le_trait() {
        let niveaux = [0, 1, 1, 0];

        assert!(a_un_enfant(&niveaux, 0), "la racine a un enfant");
        assert!(
            !a_un_enfant(&niveaux, 1),
            "la première feuille n'a pas d'enfant : elle est suivie d'une sœur"
        );
        assert!(!a_un_enfant(&niveaux, 2));
        assert!(!a_un_enfant(&niveaux, 3));

        let avec_petit_enfant = [0, 1, 2, 1];
        assert!(
            a_un_enfant(&avec_petit_enfant, 1),
            "le niveau 2 a un enfant"
        );
        assert!(!a_un_enfant(&avec_petit_enfant, 2));
    }

    /// AMÉLIORATION-005 : deux enfants d'un même parent gardent le trait de
    /// leur parent ; le trait s'arrête quand on quitte le sous-arbre.
    #[test]
    fn les_guides_ne_coupent_pas_entre_deux_enfants() {
        // Niveau 1, deux enfants (niveau 2), puis un autre niveau 1.
        let niveaux = [0, 1, 1, 1, 0];

        assert_eq!(guides_du_sommaire(&niveaux, 0), Vec::<bool>::new());
        assert_eq!(guides_du_sommaire(&niveaux, 1), vec![true]);
        assert_eq!(
            guides_du_sommaire(&niveaux, 2),
            vec![true],
            "le parent continue vers son second enfant"
        );
        assert_eq!(
            guides_du_sommaire(&niveaux, 3),
            vec![false],
            "le dernier enfant ferme la branche"
        );
    }

    /// La géométrie des barres (bug-report 2026-11-03) : un niveau = une
    /// colonne, tous les verticaux au **même x** d'une ligne à l'autre,
    /// et la barre part du vertical vers le texte.
    #[test]
    fn les_barres_sont_alignees_et_regulieres() {
        let h = 32.0;

        // Un guide : le vertical d'ancêtre au milieu de sa colonne.
        let segs = segments(&[true], false, h);
        assert_eq!(segs[0], (8.0, 0.0, 8.0, h), "le guide occupe sa colonne");

        // La coude : le vertical de l'entrée à la colonne suivante, la
        // barre part de lui.
        let coude = segs[1];
        assert_eq!(coude, (24.0, 0.0, 24.0, h), "la coude est pleine");
        let barre = segs[2];
        assert_eq!(barre.0, coude.0, "la barre part du vertical");
        assert_eq!(barre.1, h / 2.0, "la barre est au milieu");
        assert_eq!(barre.2, 40.0, "la barre rejoint le bord de la colonne");

        // La dernière branche : la coude s'arrête au milieu.
        let segs = segments(&[true], true, h);
        assert_eq!(segs[1], (24.0, 0.0, 24.0, h / 2.0), "demi-trait");

        // La largeur suit le même pas ; la racine n'a que sa pastille.
        assert_eq!(largeur_barres(0), 24.0);
        assert_eq!(largeur_barres(2), 56.0);
    }
}
