//! Le domaine des fichiers : les ouvrir, les enregistrer, nommer les chapitres.

use crate::app::message::Message;
use crate::app::state::State;
use iced::Task;

/// Traite les messages de fichiers. L'appelant a déjà vérifié le motif.
pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NewFile => {
            // Un « nouveau document » est désormais un nouveau chapitre : c'est
            // l'unité de travail du projet.
            update(state, Message::NewChapter)
        }

        Message::OpenFile => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Typst", &["typ"])
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

            // L'assemblage est le fichier du projet qui n'est pas un chapitre :
            // l'ouvrir doit le laisser où il est, et n'ajouter aucun chapitre.
            if crate::project::same_file(&path, &state.project.main) {
                state.open_document(path, content);

                return Task::none();
            }

            // Charger un fichier, c'est l'**ajouter au projet** : un fichier
            // simplement affiché ne serait ni compté, ni inclus dans
            // l'assemblage, donc jamais rendu — il n'existerait que dans
            // l'éditeur.
            let chapter = match crate::project::import_chapter(&state.project, &path) {
                Ok(chapter) => chapter,
                Err(error) => {
                    state.export_status = Some(format!("Ajout au projet : {error}"));

                    return Task::none();
                }
            };

            // Un chapitre déjà installé dans le projet se charge avec ce qu'on
            // vient de lire ; une copie doit se lire à sa nouvelle adresse.
            let imported = !crate::project::same_file(&chapter, &path);

            let content = if imported {
                match std::fs::read_to_string(&chapter) {
                    Ok(content) => content,
                    Err(error) => {
                        state.export_status = Some(format!("Ouverture du chapitre : {error}"));

                        return Task::none();
                    }
                }
            } else {
                content
            };

            state.open_document(chapter.clone(), content);

            // Le message se pose après `open_document`, qui efface le statut et
            // relit le projet : c'est le seul moment où le titre du chapitre
            // fraîchement importé est connu. Copier un fichier est un effet de
            // bord qu'il faut dire, sans quoi le chapitre apparaîtrait dans le
            // panneau sans qu'on sache pourquoi.
            if imported {
                let title = state
                    .project
                    .chapters
                    .iter()
                    .find(|entry| crate::project::same_file(&entry.path, &chapter))
                    .map(|entry| entry.title.clone())
                    .unwrap_or_else(|| {
                        chapter
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("chapitre")
                            .to_string()
                    });

                state.export_status = Some(format!("« {title} » ajouté au projet."));
            }

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

                    update(state, Message::NewChapter)
                }
                Err(error) => {
                    state.export_status = Some(format!("Création du projet : {error}"));

                    Task::none()
                }
            }
        }

        Message::OpenProject(dir) => {
            state.project = crate::project::load(&dir);
            state.projects = crate::project::list_projects();
            state.render.reset();
            state.refresh_context();

            // On repart sur le premier chapitre du nouveau projet : rouvrir le
            // chapitre d'un autre projet n'aurait pas de sens.
            match state.project.chapters.first().map(|c| c.path.clone()) {
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
            let _ = crate::project::remove_chapter(&state.project, &path);

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
                state.editor.path = Some(path);
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

                    std::fs::write(&path, &bytes)
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

            Task::none()
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

            match std::fs::write(&path, content) {
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
