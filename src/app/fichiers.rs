//! Le domaine des fichiers : les ouvrir, les enregistrer, nommer les chapitres.

use crate::app::message::Message;
use crate::app::state::State;
use iced::Task;

/// Traite les messages de fichiers. L'appelant a déjà vérifié le motif.
pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NewFile => {
            let path = match crate::project::migrate_to_document(&state.project) {
                Ok(path) => path,
                Err(error) => {
                    state.export_status = Some(format!("Création de document.tw : {error}"));
                    return Task::none();
                }
            };

            // Déjà sous les yeux : le bouton le dit — un geste qui ne fait
            // rien en silence a l'air d'un bouton mort.
            if state.editor.path.as_ref() == Some(&path) {
                state.note = Some(crate::app::state::Note::info(
                    "Le document d'écriture est déjà sous les yeux.".to_string(),
                ));
                return Task::none();
            }

            match std::fs::read_to_string(&path) {
                Ok(content) => state.open_document(path, content, false),
                Err(error) => {
                    state.export_status = Some(format!("Ouverture de document.tw : {error}"));
                }
            }

            Task::none()
        }

        Message::OpenFile => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Typst Writer / Typst", &["tw", "typ"])
                    .pick_file()
                    .await
                    .and_then(|file| {
                        let path = file.path().to_path_buf();

                        std::fs::read_to_string(&path)
                            .ok()
                            .map(|content| (path, content))
                    })
            },
            Message::FileOpened,
        ),

        Message::FileOpened(result) => {
            let Some((path, content)) = result else {
                return Task::none();
            };

            if path.extension().is_some_and(|extension| extension == "tw") {
                // Le tampon ouvert n'est jamais remplacé sans préavis : ses
                // changements non sauvegardés sont du texte vivant.
                if state.editor.is_modified() {
                    if state
                        .editor
                        .path
                        .as_ref()
                        .is_some_and(|ouvert| crate::project::same_file(&path, ouvert))
                    {
                        // C'est déjà lui : rien à faire, surtout pas à perdre.
                        state.export_status = Some(
                            "Ce document est déjà ouvert, avec vos changements non sauvegardés."
                                .to_string(),
                        );
                        return Task::none();
                    }
                    // Sinon, le `.tw` rejoint le tampon ouvert, prévisualisé.
                    preparer_import_tw(state, path, &content);
                    return Task::none();
                }

                let document = match crate::project::import_writing_document(&state.project, &path)
                {
                    Ok(document) => document,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        // Le projet a déjà son document : le `.tw` rejoint le
                        // tampon ouvert, après prévisualisation.
                        preparer_import_tw(state, path, &content);
                        return Task::none();
                    }
                    Err(error) => {
                        state.export_status = Some(format!("Ouverture du document : {error}"));
                        return Task::none();
                    }
                };
                let imported = !crate::project::same_file(&document, &path);
                let content = if imported {
                    match std::fs::read_to_string(&document) {
                        Ok(content) => content,
                        Err(error) => {
                            state.export_status = Some(format!("Ouverture du document : {error}"));
                            return Task::none();
                        }
                    }
                } else {
                    content
                };

                state.open_document(document, content, false);

                if imported {
                    state.export_status =
                        Some("Document intermédiaire ajouté au projet.".to_string());
                }

                return Task::none();
            }

            // L'assemblage est le fichier du projet qui n'est pas un chapitre :
            // l'ouvrir doit le laisser où il est, et n'ajouter aucun chapitre.
            if crate::project::same_file(&path, &state.project.main) {
                state.open_document(path, content, false);

                return Task::none();
            }

            // Un chapitre inclus a déjà été migré : l'ouvrir comme source
            // éditable ferait diverger l'archive du document compilé. Un
            // chapitre orphelin, au contraire, n'a pas été migré et doit pouvoir
            // rejoindre le document s'il est explicitement sélectionné.
            let archived_chapter_included = state
                .project
                .chapters
                .iter()
                .find(|chapter| crate::project::same_file(&chapter.path, &path))
                .map(|chapter| chapter.included);

            if let Some(included) = archived_chapter_included {
                if included {
                    // Chapitre inclus : déjà migré, le document unique s'ouvre.
                    let document = match crate::project::migrate_to_document(&state.project) {
                        Ok(document) => document,
                        Err(error) => {
                            state.export_status = Some(format!("Import du chapitre : {error}"));
                            return Task::none();
                        }
                    };

                    match std::fs::read_to_string(&document) {
                        Ok(content) => state.open_document(document, content, false),
                        Err(error) => {
                            state.export_status =
                                Some(format!("Ouverture de document.tw : {error}"));
                            return Task::none();
                        }
                    }

                    state.export_status =
                        Some("Ce chapitre est archivé ; éditez le document unique.".to_string());
                } else {
                    // Chapitre hors assemblage : il rejoint le tampon ouvert,
                    // après prévisualisation de la conversion.
                    preparer_import(state, path, &content);
                }
                return Task::none();
            }

            // Tout autre `.typ` rejoint le tampon ouvert comme fragment, après
            // prévisualisation de la conversion.
            preparer_import(state, path, &content);

            Task::none()
        }

        Message::OpenChapter(path) => {
            let nouvel = state.maj_tenu;
            Task::perform(
                async move {
                    std::fs::read_to_string(&path)
                        .ok()
                        .map(|content| (path, content, nouvel))
                },
                |result| match result {
                    Some((path, content, nouvel)) => Message::ChapterOpened(path, content, nouvel),
                    None => Message::RefreshProject,
                },
            )
        }

        Message::ChapterOpened(path, content, nouvel) => {
            state.renaming = None;
            state.open_document(path, content, nouvel);

            // Le rendez-vous d'un aller à la définition : le chapitre est ouvert,
            // il reste à y poser le curseur.
            match state.jump.take() {
                Some((line, column)) => state.editor.set_cursor(line, column).map(Message::Editor),
                None => Task::none(),
            }
        }

        Message::NewChapter => {
            let title = super::default_chapter_title(&state.project);

            match crate::project::create_chapter(&state.project, &title) {
                Ok(path) => {
                    state.refresh_context();

                    match std::fs::read_to_string(&path) {
                        Ok(content) => state.open_document(path, content, false),
                        Err(error) => {
                            state.export_status = Some(format!("Ouverture du chapitre : {error}"));
                        }
                    }
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du chapitre : {error}"));
                }
            }

            Task::none()
        }

        Message::NewProject => {
            let name = super::next_project_name(&state.projects);

            match crate::project::create(&name) {
                Ok(dir) => {
                    state.project = crate::project::load(&dir);
                    state.projects = crate::project::list_projects();
                    state.editor = crate::editor::EditorState::default();
                    state.render.reset();
                    state.refresh_context();

                    update(state, Message::NewFile)
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du projet : {error}"));

                    Task::none()
                }
            }
        }

        Message::OpenProject(dir) => {
            // Un projet ouvert a son onglet : il reste vivant, avec son
            // monde, même quand un autre prend l'écran.
            if !state.onglets.contains(&dir) {
                state.onglets.push(dir.clone());
            }

            state.project = crate::project::load(&dir);
            state.render.reset();

            if let Err(error) = crate::project::migrate_to_document(&state.project) {
                state.export_status = Some(format!("Ouverture du projet : {error}"));
                return Task::none();
            }

            state.refresh_context();

            match state.project.document.clone().or_else(|| {
                state
                    .project
                    .chapters
                    .first()
                    .map(|chapter| chapter.path.clone())
            }) {
                Some(path) => update(state, Message::OpenChapter(path)),
                None => {
                    state.editor = crate::editor::EditorState::default();
                    state.save_session();

                    Task::none()
                }
            }
        }

        Message::RefreshProject => {
            state.refresh_context();

            Task::none()
        }

        Message::DeleteChapter(path) => {
            match crate::project::remove_chapter(&state.project, &path) {
                Ok(destination) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Chapitre déplacé vers la corbeille : {}",
                        destination.display()
                    )));
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Suppression impossible : {erreur}"
                    )));
                }
            }

            state.refresh_context();

            // Le chapitre ouvert vient d'être supprimé : on ouvre le suivant, ou
            // l'éditeur reste sur un fichier qui n'existe plus.
            if state.editor.path.as_deref() == Some(path.as_path()) {
                match state.project.chapters.first().map(|c| c.path.clone()) {
                    Some(next) => return update(state, Message::OpenChapter(next)),
                    None => {
                        state.editor = crate::editor::EditorState::default();
                        state.save_session();
                    }
                }
            }

            Task::none()
        }

        Message::ImageInserer => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "svg", "webp"])
                    .pick_file()
                    .await
                    .map(|file| file.path().to_path_buf())
            },
            Message::ImageChoisie,
        ),

        Message::StyleCslImporter => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Style CSL", &["csl"])
                    .pick_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::StyleCslChoisi,
        ),

        Message::StyleCslChoisi(result) => {
            let Some(chemin) = result else {
                return Task::none();
            };

            // Entrée non fiable : la taille est bornée avant tout le reste,
            // et le texte doit être de l'UTF-8 lisible.
            let octets = std::fs::read(&chemin).unwrap_or_default();

            if octets.len() > 2 * 1024 * 1024 {
                state.note = Some(crate::app::state::Note::erreur(
                    "Fichier CSL trop grand : 2 Mo au plus.",
                ));

                return Task::none();
            }

            let Ok(texte) = std::str::from_utf8(&octets) else {
                state.note = Some(crate::app::state::Note::erreur(
                    "Fichier CSL illisible : il n'est pas en UTF-8.",
                ));

                return Task::none();
            };

            let nom = chemin
                .file_name()
                .map(|nom| nom.to_string_lossy().into_owned())
                .unwrap_or_else(|| "style.csl".to_string());

            let resultat = crate::project::importer_style_csl(
                &state.project.dir,
                &nom,
                texte,
                crate::zotero::data_dir()
                    .map(|dir| dir.join("styles"))
                    .as_deref(),
            );

            match resultat {
                Ok(resultat) => appliquer_style(state, resultat),
                Err(probleme) => {
                    // Rien ne change : le choix précédent reste.
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }

            Task::none()
        }

        Message::StyleZoteroChoisi(id) => {
            // La troisième porte : un style installé dans Zotero, copié
            // dans le projet comme les autres — Zotero n'est jamais modifié.
            let Some(dialogue) = state.mise_en_page.as_ref() else {
                return Task::none();
            };
            let Some(style) = dialogue
                .styles_zotero
                .iter()
                .find(|style| style.id == id)
                .cloned()
            else {
                // Style manquant : un diagnostic, jamais un remplacement
                // discret.
                state.note = Some(crate::app::state::Note::erreur(
                    "Le style Zotero demandé n'est plus installé : rien n'a changé.",
                ));

                return Task::none();
            };

            let Ok(texte) = std::fs::read_to_string(&style.path) else {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Le style « {} » est illisible : rien n'a changé.",
                    style.title
                )));

                return Task::none();
            };

            let nom = style
                .path
                .file_name()
                .map(|nom| nom.to_string_lossy().into_owned())
                .unwrap_or_else(|| "style.csl".to_string());
            let fratrie = style.path.parent().map(|parent| parent.to_path_buf());

            match crate::project::importer_style_csl(
                &state.project.dir,
                &nom,
                &texte,
                fratrie.as_deref(),
            ) {
                Ok(resultat) => appliquer_style(state, resultat),
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(probleme));
                }
            }

            Task::none()
        }

        Message::StyleParentTelecharger => {
            let Some((titre, parent)) = state.style_dependant_en_attente.clone() else {
                return Task::none();
            };
            state.style_dependant_en_attente = None;

            // Le consentement vient d'être donné par la question. Le
            // téléchargement se dit, et un échec n'empêche rien d'autre.
            state.note = Some(crate::app::state::Note::info(format!(
                "Téléchargement du style parent ({parent})…"
            )));

            match crate::zotero::styles::telecharger_style(&parent) {
                Ok(texte) => {
                    match crate::project::importer_style_csl(
                        &state.project.dir,
                        "parent.csl",
                        &texte,
                        crate::zotero::data_dir()
                            .map(|dir| dir.join("styles"))
                            .as_deref(),
                    ) {
                        Ok(crate::project::ImportationStyle::Choisi {
                            chemin,
                            avertissement: _,
                        }) => {
                            if let Some(dialogue) = state.mise_en_page.as_mut() {
                                dialogue.reglages.modifier(
                                    crate::mise_en_page::modele::Champ::BibliographieStyle,
                                    chemin.display().to_string(),
                                );
                                dialogue.modifie = true;
                            }

                            state.note = Some(crate::app::state::Note::info(format!(
                                "Style dépendant « {titre} » : le moteur utilise son parent, importé ({}) — écart connu : la locale du style dépendant n'est pas appliquée, c'est celle du document qui sert.",
                                chemin.display()
                            )));
                        }
                        Ok(crate::project::ImportationStyle::ParentManquant { .. }) => {
                            state.note = Some(crate::app::state::Note::erreur(
                                "Le parent téléchargé est lui-même dépendant : importez son parent à la main.",
                            ));
                        }
                        Err(probleme) => {
                            state.note = Some(crate::app::state::Note::erreur(probleme));
                        }
                    }
                }
                Err(probleme) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "{probleme} Le choix précédent reste ; récupérez le parent ({parent}) et importez-le avec « Importer un fichier .csl… »."
                    )));
                }
            }

            Task::none()
        }

        Message::ImageChoisie(result) => {
            let Some(chemin) = result else {
                return Task::none();
            };

            // **L'image entre dans le projet.** Un document qui cite une
            // image dehors se casse dès que l'ordinateur change : la copie
            // vit sous `images/`, et le chemin est relatif (bug 9 du
            // rapport 2). Une image déjà dans le projet ne se copie pas.
            let relatif = match chemin.strip_prefix(&state.project.dir) {
                Ok(reste) => reste.to_path_buf(),
                Err(_) => match copier_image_dans_projet(&state.project.dir, &chemin) {
                    Ok(relatif) => relatif,
                    Err(erreur) => {
                        state.note = Some(crate::app::state::Note::erreur(format!(
                            "L'image n'a pas pu entrer dans le projet : {erreur}"
                        )));
                        return Task::none();
                    }
                },
            };

            let legende = chemin
                .file_stem()
                .and_then(|nom| nom.to_str())
                .unwrap_or_default()
                .replace(['_', '-'], " ");

            let image = format!("![{legende}]({})", relatif.display());

            state.editor.insert_at_cursor(&image).map(Message::Editor)
        }

        Message::InsertCitation(key) => state
            .editor
            .insert_at_cursor(&format!("@{key}"))
            .map(Message::Editor),

        // La référence au terme du glossaire : le dialecte du tampon
        // décide de la forme — `{glo:clé}` en `.tw`, `#gls("clé")` en
        // `.typ` (bug-report-8 : l'écrivain ne tape jamais le code).
        // Les sections de fin s'insèrent au curseur, dans le dialecte du
        // tampon (bug-report-9).
        Message::BibliographieSectionInserer => {
            let texte = if sur_typst(&state.editor) {
                "#bibliography(\"references.bib\", full: true)".to_string()
            } else {
                "{bibliographie}".to_string()
            };

            state.editor.insert_at_cursor(&texte).map(Message::Editor)
        }

        Message::SommaireInserer => {
            let texte = if sur_typst(&state.editor) {
                "#outline()".to_string()
            } else {
                "[[TOC]]".to_string()
            };

            state.editor.insert_at_cursor(&texte).map(Message::Editor)
        }

        Message::GlossaireSectionInserer => {
            let texte = if sur_typst(&state.editor) {
                "#print-glossary(tw-glossaire)".to_string()
            } else {
                "{glossaire}".to_string()
            };

            state.editor.insert_at_cursor(&texte).map(Message::Editor)
        }

        Message::GlossaireInserer(cle) => {
            let texte = if sur_typst(&state.editor) {
                format!("#gls(\"{cle}\")")
            } else {
                format!("{{glo:{cle}}}")
            };

            state.editor.insert_at_cursor(&texte).map(Message::Editor)
        }

        // Les gestes d'insertion de R5 : le gabarit entre au curseur, le
        // curseur se pose là où l'on doit écrire — et le dialecte est celui
        // du tampon : le `.tw` écrit ses notes `^[…]`, un chapitre `.typ`
        // parle Typst.
        // Le composeur d'extension (bug-report-7, point 4) : les champs
        // naissent de `parametres()`, et le code se pose au curseur —
        // l'écrivain ne le tape jamais.
        Message::ExtensionOuvrir => {
            let mut valeurs = std::collections::BTreeMap::new();
            if let Some(ext) = crate::extensions::extensions().into_iter().next() {
                for (cle, defaut) in ext.parametres() {
                    valeurs.insert((*cle).to_string(), (*defaut).to_string());
                }
            }
            state.extension = Some(crate::app::state::ExtensionDialogue { choix: 0, valeurs });
            Task::none()
        }

        Message::ExtensionChoisir(choix) => {
            if let Some(dialogue) = state.extension.as_mut() {
                dialogue.choix = choix;
                dialogue.valeurs.clear();
                let exts = crate::extensions::extensions();
                if let Some(ext) = exts.get(choix) {
                    for (cle, defaut) in ext.parametres() {
                        dialogue
                            .valeurs
                            .insert((*cle).to_string(), (*defaut).to_string());
                    }
                }
            }
            Task::none()
        }

        Message::ExtensionParam(cle, valeur) => {
            if let Some(dialogue) = state.extension.as_mut() {
                dialogue.valeurs.insert(cle, valeur);
            }
            Task::none()
        }

        Message::ExtensionInserer => {
            let Some(dialogue) = state.extension.take() else {
                return Task::none();
            };
            let exts = crate::extensions::extensions();
            let Some(ext) = exts.get(dialogue.choix) else {
                return Task::none();
            };
            let code = ext.inserer(&dialogue.valeurs);
            state
                .editor
                .insert_avec_curseur(&code, 0)
                .map(Message::Editor)
        }

        Message::ExtensionAnnuler => {
            state.extension = None;
            Task::none()
        }

        // La note s'écrit dans son panneau, et la seule marque vient dans
        // la ligne — jamais le texte de renvoi sous les doigts
        // (bug-report-7, point 15).
        Message::NoteInserer => {
            state.note_dialogue = Some(String::new());
            Task::none()
        }

        Message::NoteTexte(texte) => {
            if let Some(note) = state.note_dialogue.as_mut() {
                *note = texte;
            }
            Task::none()
        }

        Message::NoteValider => {
            let Some(note) = state.note_dialogue.take() else {
                return Task::none();
            };
            if sur_typst(&state.editor) {
                state
                    .editor
                    .insert_avec_curseur(&format!("#footnote[{note}]"), 0)
                    .map(Message::Editor)
            } else {
                state
                    .editor
                    .insert_avec_curseur(&format!("^[{note}]"), 0)
                    .map(Message::Editor)
            }
        }

        Message::NoteAnnuler => {
            state.note_dialogue = None;
            Task::none()
        }

        Message::FigureInserer => {
            if sur_typst(&state.editor) {
                let suffixe = "],\n) <fig:>\n";

                state
                    .editor
                    .insert_avec_curseur(
                        "#figure(\n  rect(width: 100%),\n  caption: [],\n) <fig:>\n",
                        suffixe.len(),
                    )
                    .map(Message::Editor)
            } else {
                // Sur un `.tw`, une figure est une image légendée : c'est le
                // sélecteur qui la choisit, jamais un gabarit cassé.
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "svg", "webp"])
                            .pick_file()
                            .await
                            .map(|fichier| fichier.path().to_path_buf())
                    },
                    Message::ImageChoisie,
                )
            }
        }

        Message::EtiquetteInserer => {
            // L'étiquette est `<nom>` dans les deux dialectes : le pipeline
            // lit déjà celles du `.tw`, et un renvoi `@fig:nom` les trouve.
            state
                .editor
                .insert_avec_curseur(" <>", ">".len())
                .map(Message::Editor)
        }

        Message::RenvoiInserer => state.editor.insert_at_cursor("@").map(Message::Editor),

        Message::SaveFile => {
            // Une demande explicite remet la question sur le tapis : un refus
            // antérieur ne condamne pas le texte à rester en mémoire.
            state.conflit_refuse = None;
            state.persist();
            state.refresh_context();

            Task::none()
        }

        Message::SauvegardeForcee(_) => {
            let Some(chemin) = state.editor.path.clone() else {
                return Task::none();
            };
            match state.sauvegarder_document(&chemin, true) {
                Ok(()) => {
                    state.save_session();
                    state.note = Some(crate::app::state::Note::info(
                        "Enregistré ; la version remplacée reste dans « sauvegardes/ »."
                            .to_string(),
                    ));
                }
                Err(crate::app::SauvegardeRefusee::Echec(probleme)) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Sauvegarde impossible : {probleme}"
                    )));
                }
                Err(crate::app::SauvegardeRefusee::Conflit(empreinte)) => {
                    // Le disque a encore changé depuis la question : elle se
                    // repose pour ce changement-ci.
                    state.poser_la_question_du_conflit(empreinte);
                }
            }
            Task::none()
        }

        Message::ImportAppliquer => {
            let Some(importation) = state.importation.take() else {
                return Task::none();
            };

            // Le fichier d'origine a pu changer entre l'aperçu et la
            // décision : la conversion montrée n'est plus celle qu'on
            // appliquerait. On ne colle jamais un texte périmé.
            let actuel = std::fs::read(&importation.chemin)
                .map(|octets| crate::project::empreinte_contenu(&octets))
                .ok();
            if actuel != Some(importation.empreinte) {
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "« {} » a changé depuis l'aperçu : l'import est annulé. Relancez-le pour le voir à jour.",
                    importation.chemin.display()
                )));
                return Task::none();
            }
            // Le fragment rejoint la fin du tampon ouvert, d'un seul geste :
            // l'annulation l'emportera d'un seul coup, l'original n'est jamais
            // touché, et rien ne s'écrit qu'à la sauvegarde.
            let taches = vec![
                super::update(state, Message::Editor(iced_code_editor::Message::CtrlEnd)),
                super::update(
                    state,
                    Message::Editor(iced_code_editor::Message::Paste(importation.texte)),
                ),
            ];
            state.export_status = Some(format!(
                "« {} » importé dans votre texte.",
                importation.chemin.display()
            ));
            Task::batch(taches)
        }

        Message::SaveAs => save_as_task(state.editor.is_intermediate()),

        Message::FileSaved(path) => {
            let Some(path) = path else {
                return Task::none();
            };
            if let Err(erreur) = verifier_destination_copie(state, &path) {
                state.note = Some(crate::app::state::Note::erreur(erreur));
                return Task::none();
            }
            // La copie ne devient pas la source du projet et ne marque pas
            // son tampon comme sauvé. On capture les dernières frappes après
            // le dialogue, jamais avant son ouverture.
            match crate::project::ecrire_atomique(&path, &state.editor.content()) {
                Ok(()) => {
                    state.export_status = Some(format!("Copie enregistrée : {}", path.display()))
                }
                Err(erreur) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Copie impossible : {erreur}"
                    )))
                }
            }
            Task::none()
        }

        Message::ExportPdf => {
            let dir = state.document_dir();
            let (entry, overrides) = state.compilation_unit();

            Task::perform(
                async move {
                    let Some(bytes) = crate::typst::render::render_pdf(&dir, &entry, &overrides)
                    else {
                        return Err(
                            "La compilation a échoué : corrigez les erreurs signalées dans le panneau."
                                .to_string(),
                        );
                    };

                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .set_file_name("export.pdf")
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé".to_string());
                    };

                    Ok((file.path().to_path_buf(), bytes))
                },
                Message::PdfDestination,
            )
        }

        Message::PdfDestination(result) => {
            let resultat = result.and_then(|(path, bytes)| {
                verifier_destination_export(state, &path)?;
                crate::project::ecrire_atomique_octets(&path, &bytes)
                    .map(|_| path.display().to_string())
                    .map_err(|error| format!("Écriture du PDF : {error}"))
            });
            update(state, Message::PdfExported(resultat))
        }

        // Le projet voyage : une archive, son manifeste, et ce qui manque
        // déjà dit dans le compte rendu.
        // Le `.tw` unique : le projet entier dans un fichier (jalon 2 du
        // format transportable — bug-report-7, point 3).
        Message::ProjetExportTw => {
            let dir = state.project.dir.clone();
            let nom = dir
                .file_name()
                .map(|nom| nom.to_string_lossy().to_string())
                .unwrap_or_else(|| "projet".into());

            Task::perform(
                async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("Document transportable", &["tw"])
                        .set_file_name(format!("{nom}.tw"))
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé".to_string());
                    };

                    let texte = crate::project::portable::exporter_tw(&dir)?;
                    std::fs::write(file.path(), texte).map_err(|e| e.to_string())?;
                    Ok("Le projet voyage dans un seul fichier .tw.".to_string())
                },
                Message::ProjetTwExporte,
            )
        }

        Message::ProjetTwExporte(resultat) => {
            match resultat {
                Ok(message) => state.note = Some(crate::app::state::Note::info(message)),
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!("Export .tw : {e}")))
                }
            }
            Task::none()
        }

        Message::ProjetImportTw => Task::perform(
            async move {
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Document transportable", &["tw"])
                    .pick_file()
                    .await
                else {
                    return Err("Import annulé".to_string());
                };

                let texte = std::fs::read_to_string(file.path()).map_err(|e| e.to_string())?;
                let nom = file
                    .path()
                    .file_stem()
                    .map(|nom| nom.to_string_lossy().to_string())
                    .unwrap_or_else(|| "projet".into());
                let dir = crate::project::projects_dir().join(nom);
                crate::project::portable::importer_tw(&texte, &dir)?;
                Ok(dir.to_string_lossy().to_string())
            },
            Message::ProjetTwImporte,
        ),

        Message::ProjetTwImporte(resultat) => {
            match resultat {
                Ok(chemin) => {
                    state.projects = crate::project::list_projects();
                    let dir = std::path::PathBuf::from(&chemin);
                    state.ouvrir_projet_espace(dir, crate::app::espace::Ouverture::Remplacer);
                    state.note = Some(crate::app::state::Note::info(
                        "Projet reconstruit depuis le .tw unique.".to_string(),
                    ));
                }
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!("Import .tw : {e}")))
                }
            }
            Task::none()
        }

        Message::ProjetExportPortable => {
            let dir = state.project.dir.clone();
            let nom = dir
                .file_name()
                .map(|nom| nom.to_string_lossy().to_string())
                .unwrap_or_else(|| "projet".into());

            Task::perform(
                async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("Sauvegarde de projet", &["zip"])
                        .set_file_name(format!("{nom}.zip"))
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé".to_string());
                    };

                    crate::project::portable::exporter_projet(&dir, file.path()).map(|inventaire| {
                        let manques = if inventaire.manquantes.is_empty() {
                            String::new()
                        } else {
                            format!(" Manque(nt) : {}.", inventaire.manquantes.join(", "))
                        };
                        format!(
                            "Projet exporté : {} fichier(s).{manques}",
                            inventaire.fichiers().len()
                        )
                    })
                },
                Message::ProjetPortableExporte,
            )
        }

        Message::ProjetPortableExporte(resultat) => {
            match resultat {
                Ok(message) => state.note = Some(crate::app::state::Note::info(message)),
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Export portable : {e}"
                    )))
                }
            }
            Task::none()
        }

        Message::ProjetImportSauvegarde => Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .add_filter("Sauvegarde de projet", &["zip"])
                    .pick_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::ProjetImportChoisi,
        ),

        Message::ProjetImportChoisi(archive) => {
            let Some(archive) = archive else {
                return Task::none();
            };

            // La destination naît du manifeste : le projet porte son nom.
            let ouverture = crate::project::portable::lire_manifeste(&archive).map(|manifeste| {
                let destination = crate::project::projects_dir().join(&manifeste.nom);
                (manifeste.nom, destination)
            });
            let (nom, destination) = match ouverture {
                Ok(destin) => destin,
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!("Import : {e}")));
                    return Task::none();
                }
            };

            match crate::project::portable::importer_projet(
                &archive,
                &destination,
                crate::project::portable::PolitiqueCollision::Refuser,
            ) {
                Ok(rapport) => {
                    state.projects = crate::project::list_projects();
                    state.ouvrir_projet_espace(
                        destination,
                        crate::app::espace::Ouverture::Remplacer,
                    );
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Sauvegarde importée : {} fichier(s).",
                        rapport.ecrits.len()
                    )));
                }
                Err(crate::project::portable::ErreurImport::Collisions(_)) => {
                    // La question est la barrière : rien n'a bougé, et la
                    // réponse se donne en un mot.
                    state.confirmation = Some(crate::app::state::Confirmation {
                        question: format!(
                            "« {nom} » existe déjà. Remplacer ses fichiers en conflit ? Ils resteront dans la corbeille."
                        ),
                        action: Message::ProjetImportConfirmer(archive, destination),
                    });
                }
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!("Import : {e}")))
                }
            }
            Task::none()
        }

        Message::ProjetImportConfirmer(archive, destination) => {
            state.confirmation = None;
            match crate::project::portable::importer_projet(
                &archive,
                &destination,
                crate::project::portable::PolitiqueCollision::VersCorbeille,
            ) {
                Ok(rapport) => {
                    state.projects = crate::project::list_projects();
                    state.ouvrir_projet_espace(
                        destination,
                        crate::app::espace::Ouverture::Remplacer,
                    );
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Sauvegarde importée : {} fichier(s), {} remplacé(s) — dans la corbeille.",
                        rapport.ecrits.len(),
                        rapport.remplaces.len()
                    )));
                }
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!("Import : {e}")))
                }
            }
            Task::none()
        }

        // Les exports décidés par l'expérience DOCX/Markdown : le
        // Markdown au canonique, le DOCX par LibreOffice avec sa limite
        // annoncée — les notes deviennent du texte `[note: …]`.
        Message::ExportMarkdown => Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .add_filter("Markdown", &["md", "markdown"])
                    .set_file_name("document.md")
                    .save_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::ExportMarkdownChoisi,
        ),

        Message::ExportMarkdownChoisi(cible) => {
            let Some(cible) = cible else {
                return Task::none();
            };
            let texte = crate::langage::export::vers_markdown(&state.editor.content());
            match crate::project::ecrire_atomique(&cible, &texte) {
                Ok(()) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Markdown exporté : {}.",
                        cible.display()
                    )))
                }
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Export Markdown : {e}"
                    )))
                }
            }
            Task::none()
        }

        Message::ExportDocx => Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .add_filter("DOCX", &["docx"])
                    .set_file_name("document.docx")
                    .save_file()
                    .await
                    .map(|fichier| fichier.path().to_path_buf())
            },
            Message::ExportDocxChoisi,
        ),

        Message::ExportDocxChoisi(cible) => {
            let Some(cible) = cible else {
                return Task::none();
            };
            let source = state.editor.content();
            match crate::langage::export::vers_docx(&source, &cible) {
                Ok(()) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "DOCX exporté : {}. Les notes y sont du texte [note: …] — la limite annoncée.",
                        cible.display()
                    )))
                }
                Err(e) => {
                    state.note = Some(crate::app::state::Note::erreur(format!(
                        "Export DOCX : {e}"
                    )))
                }
            }
            Task::none()
        }

        Message::PdfExported(result) => {
            state.export_status = Some(match result {
                Ok(path) => format!("PDF exporté : {path}"),
                Err(error) => error,
            });

            Task::none()
        }

        Message::StartRename(path) => {
            // Le titre actuel amorce la saisie : on corrige plus souvent qu'on
            // ne réécrit entièrement.
            let buffer = state
                .project
                .chapters
                .iter()
                .find(|chapter| chapter.path == path)
                .map(|chapter| chapter.title.clone())
                .unwrap_or_default();

            state.renaming = Some(super::ChapterRename { path, buffer });

            // Le focus va à la saisie : sans lui, la frappe part dans
            // l'éditeur et s'insère dans le document.
            iced::widget::operation::focus(crate::ui::rename::id_du_renommage())
        }

        Message::RenameChanged(text) => {
            if let Some(renaming) = state.renaming.as_mut() {
                renaming.buffer = text;
            }

            Task::none()
        }

        Message::RenameCancel => {
            state.renaming = None;

            Task::none()
        }

        Message::SymbolRenameStart => {
            // Le nom actuel n'est pas à demander : c'est le curseur qui le
            // désigne, et le serveur sait de quoi il parle. La case ne reçoit
            // que le **nouveau** nom.
            state.renommage = Some(crate::app::state::Renommage {
                nom: String::new(),
                changements: None,
            });

            Task::none()
        }

        Message::SymbolRenameChanged(text) => {
            if let Some(renommage) = state.renommage.as_mut() {
                renommage.nom = text;
            }

            Task::none()
        }

        Message::SymbolRenameAsk => {
            let Some(renommage) = state.renommage.as_mut() else {
                return Task::none();
            };

            let nom = renommage.nom.trim().to_string();

            if nom.is_empty() {
                state.export_status = Some("Un nom vide ne renomme rien.".to_string());

                return Task::none();
            }

            renommage.nom = nom.clone();
            renommage.changements = None;

            crate::app::assistance::request_rename(state, &nom)
        }

        Message::SymbolRenameApply => appliquer_renommage(state),

        Message::SymbolRenameCancel => {
            state.renommage = None;

            Task::none()
        }

        Message::RenameCommit => {
            let Some(renaming) = state.renaming.clone() else {
                return Task::none();
            };

            let title = renaming.buffer.trim().to_string();

            // Un titre vide laisserait un nom de fichier sans signification.
            if title.is_empty() {
                state.export_status = Some("Le titre ne peut pas être vide.".to_string());

                return Task::none();
            }

            match crate::project::rename_chapter(&state.project, &renaming.path, &title) {
                Ok(new_path) => {
                    state.renaming = None;

                    // Le chapitre renommé peut être celui qui est ouvert : son
                    // chemin a changé, l'éditeur doit suivre.
                    if state.editor.path.as_deref() == Some(renaming.path.as_path()) {
                        state.editor.path = Some(new_path);
                    }

                    state.refresh_context();

                    Task::none()
                }
                Err(error) => {
                    // Le message reste affiché : sans lui, un renommage refusé
                    // (titre déjà pris) serait indiscernable d'un clic perdu.
                    state.export_status = Some(format!("Renommage : {error}"));

                    Task::none()
                }
            }
        }

        // L'appelant ne route ici que les messages de fichiers ; ce bras
        // n'existe que pour que le match soit complet.
        _ => Task::none(),
    }
}

/// Prévisualise l'import d'un fragment : la conversion est calculée d'avance
/// et montrée — ce qui s'est converti, ce qui reste brut — avant d'entrer
/// dans le tampon. Le fichier d'origine ne se touche jamais.
fn preparer_import(
    state: &mut crate::app::state::State,
    chemin: std::path::PathBuf,
    contenu: &str,
) {
    let (document, rapport) = crate::langage::migration::importer_typst_avec_rapport(contenu);
    let texte = crate::langage::serialiser(&document);
    let nom = chemin
        .file_name()
        .map(|nom| nom.to_string_lossy().into_owned())
        .unwrap_or_else(|| "le fragment".to_string());
    let mut question = format!(
        "Importer « {nom} » à la fin de votre texte ? {} convertis, {} restés en Typst brut.",
        rapport.elements_convertis, rapport.blocs_bruts
    );
    if !rapport.avertissements.is_empty() {
        question.push(' ');
        question.push_str(&rapport.avertissements.join(" "));
    }
    let empreinte = crate::project::empreinte_contenu(&std::fs::read(&chemin).unwrap_or_default());
    state.importation = Some(crate::app::state::Importation {
        chemin,
        texte,
        empreinte,
    });
    state.confirmation = Some(crate::app::state::Confirmation {
        question,
        action: Message::ImportAppliquer,
    });
}

/// Le même aperçu pour un fichier `.tw` : la transformation est la
/// normalisation — lecture puis écriture — et aucune pièce n'y perd rien.
fn preparer_import_tw(
    state: &mut crate::app::state::State,
    chemin: std::path::PathBuf,
    contenu: &str,
) {
    let document = crate::langage::parse(contenu);
    let texte = crate::langage::serialiser(&document);
    let nom = chemin
        .file_name()
        .map(|nom| nom.to_string_lossy().into_owned())
        .unwrap_or_else(|| "le fichier".to_string());
    let question = format!(
        "Importer « {nom} » à la fin de votre texte ? {} éléments.",
        document.blocs.len()
    );
    let empreinte = crate::project::empreinte_contenu(&std::fs::read(&chemin).unwrap_or_default());
    state.importation = Some(crate::app::state::Importation {
        chemin,
        texte,
        empreinte,
    });
    state.confirmation = Some(crate::app::state::Confirmation {
        question,
        action: Message::ImportAppliquer,
    });
}

/// Importe une image sans remplacer une image homonyme déjà présente.
fn copier_image_dans_projet(
    projet: &std::path::Path,
    chemin: &std::path::Path,
) -> std::io::Result<std::path::PathBuf> {
    use std::io::{Error, ErrorKind};

    let dossier = projet.join("images");
    std::fs::create_dir_all(&dossier)?;
    let racine = projet.canonicalize()?;
    let images = dossier.canonicalize()?;
    if images.parent() != Some(racine.as_path()) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Le dossier images se trouve hors du projet.",
        ));
    }
    let nom = chemin
        .file_name()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Nom d'image absent"))?;
    let base = chemin.file_stem().unwrap_or(nom);
    let extension = chemin.extension();
    let mut source = std::fs::File::open(chemin)?;
    for indice in 1u64.. {
        let nom = if indice == 1 {
            nom.to_os_string()
        } else {
            let mut nom = base.to_os_string();
            nom.push(format!("-{indice}"));
            if let Some(extension) = extension {
                nom.push(".");
                nom.push(extension);
            }
            nom
        };
        let destination = dossier.join(&nom);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(mut copie) => {
                std::io::copy(&mut source, &mut copie)
                    .and_then(|_| copie.sync_all())
                    .map_err(|erreur| {
                        Error::new(
                            erreur.kind(),
                            format!(
                                "{erreur}. Copie éventuellement incomplète : {}",
                                destination.display()
                            ),
                        )
                    })?;
                return Ok(std::path::PathBuf::from("images").join(nom));
            }
            Err(erreur) if erreur.kind() == ErrorKind::AlreadyExists => continue,
            Err(erreur) => return Err(erreur),
        }
    }
    Err(Error::other("Aucun nom disponible pour l'image."))
}

/// Empêche un export d'écraser un document ouvert ou un projet sans verrou.
pub(super) fn verifier_destination_export(
    state: &State,
    chemin: &std::path::Path,
) -> Result<(), String> {
    let parent = chemin
        .parent()
        .ok_or("Destination sans dossier parent")?
        .canonicalize()
        .map_err(|e| format!("Dossier de destination inaccessible : {e}"))?;
    let nom = chemin.file_name().ok_or("Nom de destination absent")?;
    let destination = parent.join(nom);
    let est_ouvert = state
        .editor
        .path
        .as_ref()
        .is_some_and(|p| crate::project::same_file(p, &destination))
        || state.espace.vues.values().any(|v| {
            v.editor
                .path
                .as_ref()
                .is_some_and(|p| crate::project::same_file(p, &destination))
        });
    if est_ouvert {
        return Err(
            "La copie remplacerait un document ouvert. Choisissez un autre nom.".to_string(),
        );
    }
    let mut racines: Vec<_> = crate::project::list_projects()
        .into_iter()
        .map(|p| p.dir)
        .collect();
    racines.push(state.project.dir.clone());
    racines.extend(state.espace.vues.values().map(|v| v.project.dir.clone()));
    for ancetre in parent.ancestors() {
        let marqueur = ancetre
            .join(".verrou-natif")
            .try_exists()
            .map_err(|e| format!("Destination inaccessible : {e}"))?
            || ancetre
                .join(".verrou")
                .try_exists()
                .map_err(|e| format!("Destination inaccessible : {e}"))?;
        // Un dossier qui porte la signature de l'appli (`document.tw`) est un
        // de ses projets, même sans verrou : elle ne le détient pas, elle
        // n'écrit pas dedans. Un dossier Typst ordinaire, lui, reste libre :
        // `main.typ` seul n'est pas notre marque.
        let structure_projet = ancetre
            .join("document.tw")
            .try_exists()
            .map_err(|e| format!("Destination inaccessible : {e}"))?;
        if marqueur || structure_projet {
            racines.push(ancetre.to_path_buf());
        }
    }
    for racine in racines {
        let Ok(resolue) = racine.canonicalize() else {
            if chemin.starts_with(&racine) {
                return Err("Le projet de destination est inaccessible.".to_string());
            }
            continue;
        };
        if !destination.starts_with(&resolue) {
            continue;
        }
        let possede_verrou = state
            .espace
            .verrous
            .keys()
            .any(|p| p.canonicalize().is_ok_and(|p| p == resolue))
            || (state.verrou.is_some()
                && state.project.dir.canonicalize().is_ok_and(|p| p == resolue));
        if !possede_verrou {
            return Err("Le projet de destination n'est pas ouvert en écriture. Choisissez un autre dossier ou ouvrez d'abord ce projet.".to_string());
        }
        let existe = match std::fs::symlink_metadata(&destination) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(format!("Destination inaccessible : {e}")),
        };
        if existe
            && (destination.extension().is_some_and(|e| {
                ["tw", "typ", "bib", "toml", "json"]
                    .iter()
                    .any(|x| e.eq_ignore_ascii_case(x))
            }) || nom == ".verrou"
                || nom == ".verrou-natif")
        {
            return Err(
                "Cette copie remplacerait un fichier du projet. Choisissez un autre nom."
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn verifier_destination_copie(state: &State, chemin: &std::path::Path) -> Result<(), String> {
    let extension_attendue = if state.editor.is_intermediate() {
        "tw"
    } else {
        "typ"
    };
    if chemin
        .extension()
        .is_none_or(|extension| !extension.eq_ignore_ascii_case(extension_attendue))
    {
        return Err(format!(
            "La copie de cette source doit porter l'extension .{extension_attendue}. Choisissez un autre nom."
        ));
    }
    verifier_destination_export(state, chemin)
}

fn save_as_task(intermediaire: bool) -> Task<Message> {
    let (filtre, extension, nom) = if intermediaire {
        ("Typst Writer", "tw", "copie.tw")
    } else {
        ("Typst", "typ", "copie.typ")
    };
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .add_filter(filtre, &[extension])
                .set_file_name(nom)
                .save_file()
                .await
                .map(|file| file.path().to_path_buf())
        },
        Message::FileSaved,
    )
}

/// Applique les modifications du renommage aux fichiers.
///
/// Chaque fichier est écrit par **remplacement** : le contenu nouveau est
/// complet avant d'exister, jamais à moitié — une coupure au milieu laisse
/// l'ancien ou le nouveau, jamais entre les deux. Le document ouvert, s'il est
/// touché, est rouvert sur son contenu neuf : la même opération qu'à
/// l'ouverture, et le même sort pour le compte de mots et le rendu.
fn appliquer_renommage(state: &mut State) -> Task<Message> {
    let Some(renommage) = state.renommage.take() else {
        return Task::none();
    };

    let Some(changements) = renommage.changements else {
        // Le serveur n'a pas répondu : tant que l'aperçu n'existe pas, rien
        // ne s'écrit.
        state.renommage = Some(renommage);

        return Task::none();
    };

    // Valider toutes les cibles avant la première écriture. Le serveur peut
    // renvoyer un fichier étranger ou un lien sortant du projet courant : le
    // verrou détenu pour ce projet ne protège pas ces autres emplacements.
    let racine = state.project.dir.canonicalize();
    let toutes_dans_le_projet = racine.as_ref().is_ok_and(|racine| {
        changements.iter().all(|fichier| {
            fichier
                .path
                .canonicalize()
                .is_ok_and(|chemin| chemin.starts_with(racine))
        })
    });
    if !toutes_dans_le_projet {
        state.renommage = Some(crate::app::state::Renommage {
            nom: renommage.nom,
            changements: Some(changements),
        });
        state.note = Some(crate::app::state::Note::erreur(
            "Renommage annulé : un fichier est absent ou extérieur au projet. Relancez le renommage dans le bon projet.",
        ));
        return Task::none();
    }

    // Ne jamais recharger depuis le disque par-dessus une projection qui
    // contient des frappes non sauvegardées, y compris dans un autre onglet.
    let touche_un_tampon_modifie = changements.iter().any(|fichier| {
        (state.editor.is_modified()
            && state
                .editor
                .path
                .as_ref()
                .is_some_and(|path| crate::project::same_file(path, &fichier.path)))
            || state.espace.vues.values().any(|vue| {
                vue.editor.is_modified()
                    && vue
                        .editor
                        .path
                        .as_ref()
                        .is_some_and(|path| crate::project::same_file(path, &fichier.path))
            })
    });
    if touche_un_tampon_modifie {
        state.renommage = Some(crate::app::state::Renommage {
            nom: renommage.nom,
            changements: None,
        });
        state.note = Some(crate::app::state::Note::erreur(
            "Renommage annulé : un document concerné contient des modifications non sauvegardées. Enregistrez-le puis redemandez le renommage.",
        ));
        return Task::none();
    }

    let mut fichiers = 0usize;
    let mut modifications = 0usize;
    let mut touches = Vec::new();

    for fichier in &changements {
        let texte = match std::fs::read_to_string(&fichier.path) {
            Ok(texte) => texte,
            Err(erreur) => {
                // Un fichier qu'on ne peut pas lire est un renommage
                // incomplet : il s'arrête là, et le dit — glisser ce fichier
                // en silence laisserait un renommage « appliqué » qui ne
                // l'est pas tout à fait.
                state.note = Some(crate::app::state::Note::erreur(format!(
                    "Renommage interrompu : {} est illisible ({erreur}). {fichiers} fichier(s) déjà modifié(s).",
                    fichier.path.display()
                )));
                return Task::none();
            }
        };

        let nouveau = crate::lsp::messages::apply_edits(&texte, &fichier.edits);
        if let Err(erreur) = crate::project::ecrire_atomique(&fichier.path, &nouveau) {
            state.note = Some(crate::app::state::Note::erreur(format!(
                "Renommage interrompu à {} : {erreur}. {fichiers} fichier(s) déjà modifié(s).",
                fichier.path.display()
            )));
            return Task::none();
        }

        fichiers += 1;
        modifications += fichier.edits.len();
        touches.push(
            fichier
                .path
                .strip_prefix(&state.project.dir)
                .unwrap_or(&fichier.path)
                .display()
                .to_string(),
        );

        if state.editor.path.as_deref() == Some(fichier.path.as_path()) {
            state.editor = crate::editor::EditorState::new_file(fichier.path.clone(), nouveau);
            state.editor.retheme(&state.resolved);
            state.render.reset();
            state.render.words = Some(crate::project::count_words(&state.editor.content()));
        }
    }

    state.export_status = Some(format!(
        "Renommage appliqué : {} — {fichiers} fichier(s), {modifications} modification(s)",
        touches.join(", ")
    ));

    Task::none()
}

/// Les trois issues d'un import de style, tenues en un seul endroit :
/// choisi (avec l'écart éventuel à dire), parent manquant (le geste se
/// propose), ou refusé (rien ne change).
fn appliquer_style(state: &mut State, resultat: crate::project::ImportationStyle) {
    match resultat {
        crate::project::ImportationStyle::Choisi {
            chemin,
            avertissement,
        } => {
            if let Some(dialogue) = state.mise_en_page.as_mut() {
                dialogue.reglages.modifier(
                    crate::mise_en_page::modele::Champ::BibliographieStyle,
                    chemin.display().to_string(),
                );
                dialogue.modifie = true;
            }

            let message = match avertissement {
                Some(ecart) => format!("Style importé : {} — {ecart}", chemin.display()),
                None => format!(
                    "Style importé : {} est une copie du projet, l'original n'a pas bougé.",
                    chemin.display()
                ),
            };
            state.note = Some(crate::app::state::Note::info(message));
        }
        crate::project::ImportationStyle::ParentManquant {
            chemin,
            titre,
            parent,
        } => {
            // Le style dépendant est conservé, mais rien n'est choisi : le
            // parent manque, et le geste se propose — jamais un
            // téléchargement silencieux.
            state.style_dependant_en_attente = Some((titre.clone(), parent.clone()));
            state.note = Some(crate::app::state::Note::info(format!(
                "« {titre} » est conservé ({}) mais son parent ({parent}) n'est pas sur cette machine : le choix précédent reste.",
                chemin.display()
            )));
            state.confirmation = Some(crate::app::state::Confirmation {
                question: format!(
                    "Le style « {titre} » a besoin de son parent ({parent}). Le télécharger depuis le dépôt des styles ?"
                ),
                action: Message::StyleParentTelecharger,
            });
        }
    }
}

/// Le tampon est-il un chapitre `.typ` (Typst brut) plutôt qu'un document
/// `.tw` ? Le dialecte des gestes d'insertion en dépend (R5).
fn sur_typst(editor: &crate::editor::EditorState) -> bool {
    editor
        .path
        .as_deref()
        .and_then(|chemin| chemin.extension())
        .is_some_and(|extension| extension == "typ")
}
