//! Gestion du projet : les chapitres, la bibliographie, la session de travail.
//!
//! L'application travaille toujours sur des fichiers locaux : aucun dialogue
//! « où sauvegarder ? » n'intervient pendant l'édition — il ne sert qu'à
//! l'export.
//!
//! Un travail se présente comme un **projet** : un dossier dans
//! `~/.typst-writer/projets/`, avec un fichier par chapitre, sa bibliographie
//! et son assemblage. Voir [`project`] pour la structure exacte.

pub mod project;

pub use project::*;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Dossier de travail de l'application : les projets y vivent.
pub fn work_dir() -> PathBuf {
    let dir = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".typst-writer"),
        Err(_) => PathBuf::from(".typst-writer"),
    };

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Session de travail : ce qu'il faut rouvrir au démarrage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Dossier du projet ouvert.
    #[serde(default)]
    pub project: Option<PathBuf>,

    /// Chemin du chapitre ouvert dans ce projet.
    #[serde(default)]
    pub last_document: Option<PathBuf>,

    /// Le mode d'apparence suivi : système, clair ou sombre.
    ///
    /// À défaut, `default` : le mode système, et les thèmes natifs. Une session
    /// enregistrée avant les thèmes se relit donc sans rien perdre.
    #[serde(default)]
    pub theme_mode: crate::ui::theme::ThemeMode,

    /// Les thèmes retenus pour chaque apparence.
    ///
    /// On enregistre le **nom**, jamais une position : la liste change quand on
    /// charge ou supprime un thème, et une position finirait par désigner un
    /// autre thème que celui qu'on avait choisi. Un nom devenu introuvable est
    /// traité par la sélection elle-même, qui retombe sur un thème de la même
    /// apparence.
    #[serde(default)]
    pub themes: crate::theming::Chosen,
}

fn session_path() -> PathBuf {
    work_dir().join("session.json")
}

/// Charge la session enregistrée.
pub fn load_session() -> Session {
    load_session_from(&session_path())
}

/// Charge une session depuis un fichier donné.
///
/// Une session absente, tronquée ou illisible donne la session **vide** : un
/// fichier de configuration cassé ne doit jamais empêcher l'application de
/// démarrer, et repartir des valeurs par défaut est toujours possible.
///
/// Le chemin est un paramètre plutôt qu'une constante pour que la persistance
/// soit vérifiable sur un fichier réel, et non seulement en mémoire.
pub fn load_session_from(path: &Path) -> Session {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Enregistre la session.
pub fn save_session(session: &Session) {
    save_session_to(&session_path(), session);
}

/// Enregistre une session dans un fichier donné.
///
/// Un échec d'écriture est signalé sans interrompre : la session est une
/// commodité, pas une donnée dont dépend la suite du travail.
pub fn save_session_to(path: &Path, session: &Session) {
    let text = match serde_json::to_string_pretty(session) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Sérialisation de la session : {error}");
            return;
        }
    };

    if let Err(error) = std::fs::write(path, text) {
        eprintln!("Enregistrement de la session : {error}");
    }
}

/// Une entrée de bibliographie (fichier `.bib`).
#[derive(Debug, Clone)]
pub struct BibliographyEntry {
    /// La clé de citation, celle qui suit `@type{`.
    pub key: String,

    /// Le type BibTeX (`article`, `book`, `inproceedings`…).
    pub kind: String,
}

/// Lit les entrées d'un fichier `.bib`.
///
/// Parseur volontairement tolérant : il ne cherche que les débuts d'entrée
/// (`@type{clé,`), ce qui suffit pour lister les clés citables. Un fichier
/// malformé renvoie les entrées reconnues jusqu'au problème.
pub fn parse_bibliography(path: &Path) -> Vec<BibliographyEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(|line| parse_bib_entry_line(line))
        .collect()
}

/// Analyse une ligne de fichier BibTeX : `@article{clé,` → entrée.
fn parse_bib_entry_line(line: &str) -> Option<BibliographyEntry> {
    let line = line.trim();

    let rest = line.strip_prefix('@')?;
    let (kind, rest) = rest.split_once('{')?;

    let key = rest.split(',').next()?.trim().to_string();

    if key.is_empty() {
        return None;
    }

    Some(BibliographyEntry {
        kind: kind.trim().to_string(),
        key,
    })
}

/// Le nom du projet créé lors de la reprise des documents existants.
const MIGRATED_PROJECT: &str = "mon-travail";

/// Reprend les documents à plat dans un premier projet.
///
/// Les versions précédentes rangeaient les documents directement à la racine de
/// `~/.typst-writer/`. Puisqu'un travail est désormais un projet, ces fichiers
/// deviennent les chapitres d'un projet « mon-travail ».
///
/// Ne fait rien si un projet existe déjà, ou s'il n'y a rien à reprendre :
/// la migration ne s'exécute qu'une fois, sur un dossier vierge de projets.
///
/// Renvoie le dossier du projet créé, le cas échéant.
pub fn migrate_flat_documents() -> Option<PathBuf> {
    // Un projet existe déjà : le dossier a été migré, ou l'utilisateur a
    // commencé à travailler dans la nouvelle organisation.
    if !project::list_projects().is_empty() {
        return None;
    }

    let flat = locate_flat_documents();

    if flat.is_empty() {
        return None;
    }

    let dir = project::create(MIGRATED_PROJECT).ok()?;

    // Une bibliographie restée à la racine doit rejoindre le projet : sans
    // cela, un chapitre qui la cite perdrait sa référence.
    adopt_stray_bibliography(&dir);

    let chapters_dir = dir.join("chapitres");

    let session = load_session();
    let mut moved_session = session.project.is_none();

    for (index, path) in flat.iter().enumerate() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("chapitre.typ");

        let destination = chapters_dir.join(format!("{:02}-{}", index + 1, name));

        if std::fs::rename(path, &destination).is_err() {
            // Un fichier qu'on n'a pas pu déplacer est laissé sur place plutôt
            // que perdu : la migration reprendra au prochain démarrage.
            continue;
        }

        let _ = project::add_chapter_to_assembly(&dir.join("main.typ"), &destination);

        if session.last_document.as_deref() == Some(path.as_path()) {
            moved_session = true;
        }
    }

    // La session pointait vers un fichier à plat : elle pointe maintenant vers
    // le projet, sinon le démarrage suivant chercherait un document disparu.
    if moved_session {
        save_session(&Session {
            project: Some(dir.clone()),
            last_document: None,
            // Le thème retenu ne regarde pas la migration des documents à plat :
            // il survit tel quel, et le reste de la session avec lui.
            ..session.clone()
        });
    }

    Some(dir)
}

/// Fait rejoindre au projet une bibliographie laissée à la racine.
///
/// Le fichier créé par `project::create` est un modèle vide : on ne le remplace
/// que s'il est effectivement vide, pour ne jamais écraser des références
/// existantes par un exemple commenté.
fn adopt_stray_bibliography(project_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(work_dir()) else {
        return;
    };

    let Some(stray) = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().map(|ext| ext == "bib").unwrap_or(false))
    else {
        return;
    };

    let destination = project_dir.join("references.bib");

    let template_is_untouched = std::fs::read_to_string(&destination)
        // Le modèle n'a que des lignes vides ou commentées : aucun `@`.
        .map(|content| !content.contains('@'))
        .unwrap_or(false);

    if template_is_untouched {
        let _ = std::fs::rename(&stray, &destination);
    }
}

/// Les documents `.typ` restés à la racine du dossier de travail.
fn locate_flat_documents() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(work_dir()) else {
        return Vec::new();
    };

    let mut documents: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().map(|ext| ext == "typ").unwrap_or(false))
        .collect();

    // L'ordre de lecture du dossier n'est pas garanti : on trie pour que la
    // numérotation des chapitres soit reproductible.
    documents.sort();

    documents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bib_entries() {
        let dir = std::env::temp_dir().join("typst-writer-bib");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{smith2020,\n  title = {Un titre},\n}\n@book{doe2019, author = {Doe}}\n",
        )
        .unwrap();

        let entries = parse_bibliography(&bib);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "smith2020");
        assert_eq!(entries[0].kind, "article");
        assert_eq!(entries[1].key, "doe2019");
        assert_eq!(entries[1].kind, "book");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une ligne sans `@` ou sans clé est ignorée, pas une erreur.
    #[test]
    fn ignores_malformed_bib_lines() {
        assert!(parse_bib_entry_line("ceci n'est pas une entrée").is_none());
        assert!(parse_bib_entry_line("@article{,").is_none());
        assert!(parse_bib_entry_line("@article{clé,").is_some());
    }

    /// Une session d'une version précédente, qui ne connaissait que
    /// `last_document`, doit se lire sans erreur.
    #[test]
    fn an_older_session_still_loads() {
        let session: Session =
            serde_json::from_str(r#"{"last_document": "/tmp/ancien.typ"}"#).unwrap();

        assert_eq!(
            session.last_document,
            Some(PathBuf::from("/tmp/ancien.typ"))
        );
        assert!(session.project.is_none());

        // Les champs de thème, absents, retombent sur le mode système et les
        // thèmes natifs : une session écrite avant les thèmes se relit donc
        // sans rien perdre, et sans démarrer sur un thème inexistant.
        assert_eq!(session.theme_mode, crate::ui::theme::ThemeMode::System);
        assert_eq!(session.themes, crate::theming::Chosen::default());
    }

    /// Le thème retenu survit à un redémarrage.
    ///
    /// C'est la raison d'être de ces champs : sans eux, l'application repartait
    /// sur les thèmes natifs à chaque lancement, et choisir un thème revenait à
    /// le rechoisir tous les matins.
    #[test]
    fn a_session_keeps_the_chosen_theme() {
        let session = Session {
            project: Some(PathBuf::from("/tmp/projet")),
            last_document: None,
            theme_mode: crate::ui::theme::ThemeMode::Light,
            themes: crate::theming::Chosen {
                light: "Moka clair".to_string(),
                dark: "Moka sombre".to_string(),
            },
        };

        let text = serde_json::to_string_pretty(&session).unwrap();

        // Le fichier reste lisible : c'est ce qu'on veut pouvoir inspecter et
        // corriger à la main quand quelque chose ne se rouvre pas comme prévu.
        assert!(text.contains("\"light\""), "session écrite :\n{text}");

        let restored: Session = serde_json::from_str(&text).unwrap();

        assert_eq!(restored.theme_mode, crate::ui::theme::ThemeMode::Light);
        assert_eq!(restored.themes.light, "Moka clair");
        assert_eq!(restored.themes.dark, "Moka sombre");
    }

    /// Le thème retenu survit à un aller-retour par un vrai fichier.
    ///
    /// C'est la chaîne complète du redémarrage : écrire la session, la relire,
    /// retrouver le thème qu'elle nomme dans la liste, et le sélectionner. Les
    /// trois maillons sont ici, dans l'ordre où l'application les exécute.
    #[test]
    fn the_chosen_theme_survives_a_real_restart() {
        let dir = std::env::temp_dir().join("typst-writer-session-theme");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("session.json");

        // 1. L'utilisateur choisit un thème clair et un thème sombre.
        save_session_to(
            &path,
            &Session {
                project: None,
                last_document: None,
                theme_mode: crate::ui::theme::ThemeMode::Dark,
                themes: crate::theming::Chosen {
                    light: "Clair".to_string(),
                    dark: "Mon thème".to_string(),
                },
            },
        );

        // 2. L'application redémarre et relit la session.
        let restored = load_session_from(&path);

        assert_eq!(restored.theme_mode, crate::ui::theme::ThemeMode::Dark);

        // 3. Les thèmes disponibles sont relus du dossier, et le choix retrouvé.
        let mut themes = crate::theming::natives();

        let file = crate::theming::ThemeFile::parse(crate::theming::EXAMPLE, None).unwrap();
        let name = file.meta.name.clone();

        themes.push(crate::theming::Theme {
            name: name.clone(),
            appearance: file.meta.appearance,
            file: Some(file),
        });

        let index =
            crate::theming::select(&themes, crate::theming::Appearance::Dark, &restored.themes);

        assert_eq!(
            themes[index].name, "Mon thème",
            "le thème retenu avant le redémarrage doit être celui qu'on applique après"
        );

        // 4. Une session absente retombe sur les défauts, sans échouer.
        let missing = load_session_from(&dir.join("absent.json"));

        assert_eq!(missing.theme_mode, crate::ui::theme::ThemeMode::System);
        assert_eq!(missing.themes, crate::theming::Chosen::default());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un mode de thème s'écrit en minuscules et se relit tel quel.
    #[test]
    fn the_theme_mode_is_written_in_lowercase() {
        for (mode, expected) in [
            (crate::ui::theme::ThemeMode::System, "\"system\""),
            (crate::ui::theme::ThemeMode::Light, "\"light\""),
            (crate::ui::theme::ThemeMode::Dark, "\"dark\""),
        ] {
            let text = serde_json::to_string(&mode).unwrap();

            assert_eq!(text, expected, "mode {mode:?}");
        }
    }
}
