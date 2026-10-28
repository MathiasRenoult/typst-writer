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

            match std::fs::read_to_string(&path) {
                Ok(content) => state.open_document(path, content),
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
                let document = match crate::project::import_writing_document(&state.project, &path)
                {
                    Ok(document) => document,
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

                state.open_document(document, content);

                if imported {
                    state.export_status =
                        Some("Document intermédiaire ajouté au projet.".to_string());
                }

                return Task::none();
            }

            // L'assemblage est le fichier du projet qui n'est pas un chapitre :
            // l'ouvrir doit le laisser où il est, et n'ajouter aucun chapitre.
            if crate::project::same_file(&path, &state.project.main) {
                state.open_document(path, content);

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
                let document = if included {
                    crate::project::migrate_to_document(&state.project)
                } else {
                    crate::project::append_typst_to_document(&state.project, &content)
                };
                let document = match document {
                    Ok(document) => document,
                    Err(error) => {
                        state.export_status = Some(format!("Import du chapitre : {error}"));
                        return Task::none();
                    }
                };

                match std::fs::read_to_string(&document) {
                    Ok(content) => state.open_document(document, content),
                    Err(error) => {
                        state.export_status = Some(format!("Ouverture de document.tw : {error}"));
                        return Task::none();
                    }
                }

                state.export_status = Some(if included {
                    "Ce chapitre est archivé ; éditez le document unique.".to_string()
                } else {
                    "Chapitre hors assemblage ajouté à document.tw.".to_string()
                });
                return Task::none();
            }

            // Tout autre `.typ` est importé comme fragment dans le document
            // unique ; il ne devient plus un chapitre parallèle.
            let document = match crate::project::append_typst_to_document(&state.project, &content)
            {
                Ok(document) => document,
                Err(error) => {
                    state.export_status = Some(format!("Import Typst : {error}"));
                    return Task::none();
                }
            };
            match std::fs::read_to_string(&document) {
                Ok(content) => state.open_document(document, content),
                Err(error) => {
                    state.export_status = Some(format!("Ouverture de document.tw : {error}"));
                    return Task::none();
                }
            }
            state.export_status = Some("Fragment Typst converti dans document.tw.".to_string());

            Task::none()
        }

        Message::OpenChapter(path) => Task::perform(
            async move {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            },
            |result| match result {
                Some((path, content)) => Message::ChapterOpened(path, content),
                None => Message::RefreshProject,
            },
        ),

        Message::ChapterOpened(path, content) => {
            state.renaming = None;
            state.open_document(path, content);

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
                        Ok(content) => state.open_document(path, content),
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

        Message::InsertCitation(key) => state
            .editor
            .insert_at_cursor(&format!("@{key}"))
            .map(Message::Editor),

        Message::SaveFile => {
            state.persist();
            state.refresh_context();

            Task::none()
        }

        Message::SaveAs => save_as_task(state),

        Message::FileSaved(path) => {
            if let Some(path) = path {
                state.editor.set_path(path);
                state.editor.editor.mark_saved();
            }

            state.refresh_context();

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

                    let path = file.path().to_path_buf();

                    crate::project::ecrire_atomique_octets(&path, &bytes)
                        .map(|_| path.display().to_string())
                        .map_err(|error| format!("Écriture du PDF : {error}"))
                },
                Message::PdfExported,
            )
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
                        state.editor.path = Some(new_path.clone());
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

fn save_as_task(state: &State) -> Task<Message> {
    let content = state.editor.content().to_owned();

    Task::perform(
        async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("Typst", &["typ"])
                .set_file_name("export.typ")
                .save_file()
                .await?;

            let path = file.path().to_path_buf();

            match crate::project::ecrire_atomique(&path, &content) {
                Ok(_) => Some(path),
                Err(error) => {
                    eprintln!("Erreur lors de l'export : {error}");
                    None
                }
            }
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

    let mut fichiers = 0usize;
    let mut modifications = 0usize;

    for fichier in &changements {
        let Ok(texte) = std::fs::read_to_string(&fichier.path) else {
            continue;
        };

        let nouveau = crate::lsp::messages::apply_edits(&texte, &fichier.edits);
        let temp = fichier.path.with_file_name(".typst-writer-tmp");

        if std::fs::write(&temp, &nouveau).is_err() {
            continue;
        }

        if std::fs::rename(&temp, &fichier.path).is_err() {
            let _ = std::fs::remove_file(&temp);

            continue;
        }

        fichiers += 1;
        modifications += fichier.edits.len();

        if state.editor.path.as_deref() == Some(fichier.path.as_path()) {
            state.editor = crate::editor::EditorState::new_file(fichier.path.clone(), nouveau);
            state.editor.retheme(&state.resolved);
            state.render.reset();
            state.render.words = Some(crate::project::count_words(&state.editor.content()));
        }
    }

    state.export_status = Some(format!(
        "Renommage appliqué : {fichiers} fichier(s), {modifications} modification(s)"
    ));

    Task::none()
}
