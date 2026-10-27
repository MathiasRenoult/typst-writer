//! Le domaine des thèmes : les lire, les choisir, les appliquer.

use crate::app::message::Message;
use crate::app::state::State;
use iced::Task;

/// Traite les messages de thème. L'appelant a déjà vérifié le motif.
pub(super) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SystemTheme(mode) => {
            state.system_theme = mode;
            retheme(state);

            Task::none()
        }

        Message::CycleTheme => {
            state.theme = state.theme.next();
            retheme(state);
            state.save_session();

            Task::none()
        }

        Message::RefreshThemes => {
            refresh_themes(state);

            Task::none()
        }

        Message::OpenThemesFolder => {
            let dir = crate::theming::themes_dir();

            open_folder(&dir);

            // Le chemin est dit dans tous les cas : sur un système où
            // l'ouverture n'est pas prévue, il reste au moins de quoi y aller.
            state.export_status = Some(format!("Dossier des thèmes : {}", dir.display()));

            Task::none()
        }

        Message::SelectTheme(name) => {
            let Some(theme) = state.themes.iter().find(|theme| theme.name == name) else {
                return Task::none();
            };

            let appearance = theme.appearance;
            let name = theme.name.clone();

            state.chosen.set(appearance, &name);

            // Choisir un thème fixe le mode sur son apparence. Sans cela, en
            // mode système, le thème clair choisi disparaîtrait dès que le
            // système annonce une apparence sombre — on croirait que le clic
            // n'a rien fait.
            state.theme = match appearance {
                crate::theming::Appearance::Light => crate::ui::theme::ThemeMode::Light,
                crate::theming::Appearance::Dark => crate::ui::theme::ThemeMode::Dark,
            };

            retheme(state);
            state.save_session();

            Task::none()
        }

        Message::LoadTheme => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Thème", &["toml"])
                    .pick_file()
                    .await
                    .map(|file| file.path().to_path_buf())
            },
            Message::ThemeLoaded,
        ),

        Message::ThemeLoaded(result) => {
            let Some(path) = result else {
                return Task::none();
            };

            match crate::theming::import(&path) {
                Ok(loaded) => {
                    let name = loaded.meta.name.clone();

                    // Un thème du même nom existait déjà : le fichier a été
                    // remplacé, et le dire évite de croire à un doublon.
                    let updated = state.themes.iter().any(|theme| theme.name == name);

                    refresh_themes(state);

                    // On applique le thème qu'on vient de charger : c'est ce
                    // qu'on veut voir, et sans cela il faudrait le chercher
                    // dans la liste après l'avoir chargé.
                    if let Some(theme) = state.themes.iter().find(|theme| theme.name == name) {
                        let appearance = theme.appearance;

                        state.chosen.set(appearance, &name);

                        state.theme = match appearance {
                            crate::theming::Appearance::Light => crate::ui::theme::ThemeMode::Light,
                            crate::theming::Appearance::Dark => crate::ui::theme::ThemeMode::Dark,
                        };

                        retheme(state);
                        state.save_session();
                    }

                    // L'auteur, quand il s'est nommé, est une politesse qu'on
                    // lui rend : c'est la seule raison d'écrire ce champ.
                    let verb = if updated { "mis à jour" } else { "chargé" };

                    state.export_status = Some(match loaded.meta.author.as_deref() {
                        Some(author) => format!("Thème « {name} », par {author}, {verb}."),
                        None => format!("Thème « {name} » {verb}."),
                    });
                }
                Err(error) => {
                    state.export_status = Some(format!("Chargement du thème : {error}"));
                }
            }

            Task::none()
        }

        Message::DeleteTheme(name) => {
            match crate::theming::remove(&name) {
                Ok(destination) => {
                    state.note = Some(crate::app::state::Note::info(format!(
                        "Thème déplacé vers la corbeille : {}",
                        destination.display()
                    )));

                    // Le thème supprimé ne doit pas rester « retenu » : on
                    // l'oublie, sinon le repli désignerait éternellement un
                    // fichier absent.
                    for appearance in [
                        crate::theming::Appearance::Light,
                        crate::theming::Appearance::Dark,
                    ] {
                        if state.chosen.get(appearance) == name {
                            let native = crate::theming::natives()
                                .into_iter()
                                .find(|theme| theme.appearance == appearance);

                            if let Some(native) = native {
                                state.chosen.set(appearance, &native.name);
                            }
                        }
                    }

                    refresh_themes(state);
                    state.save_session();

                    state.export_status = Some(format!("Thème « {name} » supprimé."));
                }
                Err(error) => {
                    state.export_status = Some(format!("Suppression du thème : {error}"));
                }
            }

            Task::none()
        }

        // L'appelant ne route ici que les messages de thème ; ce bras n'existe
        // que pour que le match soit complet.
        _ => Task::none(),
    }
}

/// Réapplique le thème aux éléments qui en dépendent mais ne le recoivent pas
/// à chaque rendu — en l'occurrence l'éditeur, dont les couleurs sont une
/// copie prise au moment du réglage.
pub(super) fn retheme(state: &mut State) {
    let appearance = state.theme.appearance(state.system_theme);

    let index = crate::theming::select(&state.themes, appearance, &state.chosen);

    let Some(theme) = state.themes.get(index) else {
        return;
    };

    state.resolved = crate::ui::theme::Resolved::of(theme);

    state.editor.retheme(&state.resolved);
}

/// Ouvre un dossier dans le gestionnaire de fichiers du système.
///
/// Sous Linux seulement : c'est la seule plateforme où l'application est
/// utilisée, et deviner la commande des autres serait du code jamais vérifié.
/// L'appelant dit le chemin de toute façon, ce qui reste une issue.
#[cfg(target_os = "linux")]
fn open_folder(dir: &std::path::Path) {
    let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
}

#[cfg(not(target_os = "linux"))]
fn open_folder(_dir: &std::path::Path) {}

/// Relit les thèmes du disque, puis réapplique celui qui est retenu.
///
/// Appelé au démarrage et après tout changement dans le dossier : un thème
/// écrit à la main pendant que l'application tourne doit pouvoir être rechargé
/// sans la relancer.
pub(super) fn refresh_themes(state: &mut State) {
    // Le thème d'exemple sert de documentation : il est écrit dans le dossier
    // au premier besoin, pour que le format soit sous les yeux de qui veut en
    // écrire un, plutôt que dans un fichier de référence à retrouver.
    let _ = crate::theming::write_example();

    let (themes, problems) = crate::theming::list();

    state.themes = themes;
    state.theme_problems = problems;

    retheme(state);
}
