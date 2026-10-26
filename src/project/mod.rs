//! Gestion du projet : les chapitres, la bibliographie, la session de travail.
//!
//! L'application travaille toujours sur des fichiers locaux : aucun dialogue
//! « où sauvegarder ? » n'intervient pendant l'édition — il ne sert qu'à
//! l'export.
//!
//! Un travail se présente comme un **projet** : un dossier dans
//! `~/.typst-writer/projets/`, avec un fichier par chapitre, sa bibliographie
//! et son assemblage. Voir [`model`] pour la structure exacte.

pub mod model;

pub use model::*;

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BibliographyEntry {
    /// La clé de citation, celle qui suit `@type{`.
    pub key: String,

    /// Le type BibTeX (`article`, `book`, `inproceedings`…).
    pub kind: String,

    /// Les auteurs, `(nom, prénom)` dans l'ordre du fichier.
    ///
    /// BibTeX écrit « Dupont, Jean » ou « Jean Dupont » : les deux formes
    /// sont ramenées au même couple, nom d'abord.
    pub auteurs: Vec<(String, String)>,

    /// Le titre de l'œuvre.
    pub titre: String,

    /// L'année de parution, si le fichier la donne.
    pub annee: Option<String>,
}

impl BibliographyEntry {
    /// La ligne de liste : « Dupont, Jean — Le titre… ».
    ///
    /// C'est ce que l'onglet des références montre à la place de la clé : la
    /// clé est une machine, un nom et un titre se lisent.
    pub fn libelle(&self) -> String {
        let auteur = match self.auteurs.first() {
            Some((nom, prenom)) if !prenom.is_empty() => format!("{nom}, {prenom}"),
            Some((nom, _)) => nom.clone(),
            None => String::new(),
        };

        let corps = if self.titre.is_empty() {
            self.key.clone()
        } else {
            self.titre.clone()
        };

        if auteur.is_empty() {
            corps
        } else {
            format!("{auteur} — {corps}")
        }
    }
}

/// Lit les entrées d'un fichier `.bib`.
///
/// Parseur volontairement tolérant : il cherche les entrées (`@type{clé,`),
/// puis les champs qu'elles portent — auteur, titre, année. Un fichier
/// malformé renvoie les entrées reconnues jusqu'au problème.
pub fn parse_bibliography(path: &Path) -> Vec<BibliographyEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    parse_bibliography_text(&content)
}

/// Met à jour une référence dans un fichier `.bib`.
///
/// Les champs connus — auteur, titre, année — sont remplacés ou ajoutés **dans
/// le bloc de l'entrée** ; tout le reste du fichier ne bouge pas, et la clé
/// reste la sienne : elle est citée dans les documents.
pub fn maj_reference(
    chemin: &std::path::Path,
    cle: &str,
    nom: &str,
    prenom: &str,
    titre: &str,
    annee: &str,
) -> std::io::Result<()> {
    let contenu = std::fs::read_to_string(chemin)?;
    let octets = contenu.as_bytes();
    let mut sortie = String::with_capacity(contenu.len() + 64);
    let mut curseur = 0usize;

    while curseur < contenu.len() {
        let Some(relatif) = contenu[curseur..].find('@') else {
            sortie.push_str(&contenu[curseur..]);
            break;
        };

        let debut = curseur + relatif;
        sortie.push_str(&contenu[curseur..debut]);

        let fin = fin_d_entree(octets, debut).unwrap_or(contenu.len());
        let bloc = &contenu[debut..fin];

        if entree_de(bloc).is_some_and(|entree| entree.key == cle) {
            let auteur = if prenom.is_empty() {
                nom.to_string()
            } else {
                format!("{nom}, {prenom}")
            };

            let mut maj = bloc.to_string();
            maj = remplacer_champ(&maj, "author", &auteur);
            maj = remplacer_champ(&maj, "title", titre);

            if annee.is_empty() {
                maj = retirer_champ(&maj, "year");
            } else {
                maj = remplacer_champ(&maj, "year", annee);
            }

            sortie.push_str(&maj);
        } else {
            sortie.push_str(bloc);
        }

        curseur = fin;
    }

    std::fs::write(chemin, sortie)
}

/// Remplace la valeur d'un champ dans un bloc d'entrée, ou l'y ajoute.
fn remplacer_champ(bloc: &str, nom: &str, valeur: &str) -> String {
    let trouve = champ_plage(bloc, nom);

    match trouve {
        Some((debut, fin)) => {
            let mut sortie = String::with_capacity(bloc.len() + valeur.len());
            sortie.push_str(&bloc[..debut]);
            sortie.push_str(valeur);
            sortie.push_str(&bloc[fin..]);

            sortie
        }
        None => {
            // Avant l'accolade fermante : la dernière du bloc.
            let Some(fin) = bloc.rfind('}') else {
                return bloc.to_string();
            };

            let mut sortie = String::with_capacity(bloc.len() + valeur.len() + 16);
            sortie.push_str(&bloc[..fin]);
            sortie.push_str(&format!(",\n  {nom} = {{{valeur}}}"));
            sortie.push_str(&bloc[fin..]);

            sortie
        }
    }
}

/// Retire un champ d'un bloc d'entrée, s'il y est.
fn retirer_champ(bloc: &str, nom: &str) -> String {
    let Some((debut_valeur, fin_valeur)) = champ_plage(bloc, nom) else {
        return bloc.to_string();
    };

    // Le champ entier : sa virgule précédente jusqu'à la fin de la valeur.
    let avant = bloc[..debut_valeur].rfind(',').map_or(0, |index| index);
    let mut sortie = String::with_capacity(bloc.len());
    sortie.push_str(&bloc[..avant]);
    sortie.push_str(&bloc[fin_valeur..]);

    sortie
}

/// La plage de la **valeur** d'un champ : entre accolades, sans elles.
fn champ_plage(bloc: &str, nom: &str) -> Option<(usize, usize)> {
    let mut base = 0usize;

    while let Some(relatif) = bloc[base..].find('=') {
        let egal = base + relatif;
        let debut_valeur = egal + 1 + bloc[egal + 1..].len() - bloc[egal + 1..].trim_start().len();
        let avant = bloc[..egal].trim();
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            let valeur = &bloc[debut_valeur..];

            if let Some(ouvrante) = valeur.find('{') {
                let mut profondeur = 1usize;

                for (relatif, caractere) in valeur[ouvrante + 1..].char_indices() {
                    match caractere {
                        '{' => profondeur += 1,
                        '}' => {
                            profondeur -= 1;

                            if profondeur == 0 {
                                return Some((
                                    debut_valeur + ouvrante + 1,
                                    debut_valeur + ouvrante + 1 + relatif,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }

            if let Some(ouvrante) = valeur.find('"')
                && let Some(fermante) = valeur[ouvrante + 1..].find('"')
            {
                return Some((
                    debut_valeur + ouvrante + 1,
                    debut_valeur + ouvrante + 1 + fermante,
                ));
            }

            // Valeur nue : jusqu'à la virgule ou l'accolade.
            let fin = valeur
                .find([',', '}'])
                .map_or(valeur.len(), |index| debut_valeur + index);

            return Some((debut_valeur, fin));
        }

        base = egal + 1;
    }

    None
}

/// Les entrées d'un texte BibTeX.
pub fn parse_bibliography_text(content: &str) -> Vec<BibliographyEntry> {
    let mut entrees = Vec::new();
    let octets = content.as_bytes();
    let mut curseur = 0usize;

    while let Some(debut) = content[curseur..].find('@') {
        let debut = curseur + debut;

        let Some(fin) = fin_d_entree(octets, debut) else {
            break;
        };

        if let Some(entree) = entree_de(&content[debut..fin]) {
            entrees.push(entree);
        }

        curseur = fin;
    }

    entrees
}

/// La fin d'une entrée : l'accolade qui referme celle ouverte en `debut`.
fn fin_d_entree(octets: &[u8], debut: usize) -> Option<usize> {
    let mut profondeur = 0usize;

    for (relatif, octet) in octets[debut..].iter().enumerate() {
        match octet {
            b'{' => profondeur += 1,
            b'}' => {
                profondeur = profondeur.saturating_sub(1);

                if profondeur == 0 {
                    return Some(debut + relatif + 1);
                }
            }
            _ => {}
        }
    }

    None
}

/// Une entrée, lue dans son bloc entier : `@type{clé, champs… }`.
fn entree_de(bloc: &str) -> Option<BibliographyEntry> {
    let reste = bloc.trim().strip_prefix('@')?;
    let (kind, reste) = reste.split_once('{')?;

    let (cle, champs) = reste.split_once(',').unwrap_or((reste, ""));
    let key = cle.trim().to_string();

    if key.is_empty() {
        return None;
    }

    Some(BibliographyEntry {
        kind: kind.trim().to_string(),
        key,
        auteurs: auteurs_de(champ(champs, "author")),
        titre: champ(champs, "title").unwrap_or_default(),
        annee: champ(champs, "year"),
    })
}

/// La valeur d'un champ BibTeX, sans ses accolades ni ses guillemets.
fn champ(champs: &str, nom: &str) -> Option<String> {
    let mut reste = champs;

    while let Some(egal) = reste.find('=') {
        let avant = reste[..egal].trim();
        let apres = &reste[egal + 1..];

        // Le nom du champ est ce qui suit la virgule précédente.
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            return Some(valeur_de(apres));
        }

        reste = apres;
    }

    None
}

/// La valeur qui suit un `=` : entre accolades, entre guillemets, ou jusqu'à
/// la virgule suivante.
fn valeur_de(texte: &str) -> String {
    let texte = texte.trim_start();

    if let Some(reste) = texte.strip_prefix('{') {
        let mut profondeur = 1usize;

        for (index, caractere) in reste.char_indices() {
            match caractere {
                '{' => profondeur += 1,
                '}' => {
                    profondeur -= 1;

                    if profondeur == 0 {
                        return reste[..index].trim().to_string();
                    }
                }
                _ => {}
            }
        }

        return reste.trim().to_string();
    }

    if let Some(reste) = texte.strip_prefix('"') {
        return reste
            .split_once('"')
            .map(|(valeur, _)| valeur.trim().to_string())
            .unwrap_or_else(|| reste.trim().to_string());
    }

    texte.split(',').next().unwrap_or(texte).trim().to_string()
}

/// Les auteurs d'un champ `author` : « et » sépare, la virgule dit le nom.
fn auteurs_de(champ: Option<String>) -> Vec<(String, String)> {
    let Some(texte) = champ else {
        return Vec::new();
    };

    texte
        .replace(" AND ", " and ")
        .split(" and ")
        .filter(|auteur| !auteur.trim().is_empty())
        .map(|auteur| {
            let auteur = auteur.trim();

            if let Some((nom, prenom)) = auteur.split_once(',') {
                (nom.trim().to_string(), prenom.trim().to_string())
            } else {
                // « Jean Dupont » : le nom est le dernier mot.
                let mut mots = auteur.rsplitn(2, ' ');
                let nom = mots.next().unwrap_or(auteur).to_string();
                let prenom = mots.next().unwrap_or("").to_string();

                (nom, prenom)
            }
        })
        .collect()
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
    if !model::list_projects().is_empty() {
        return None;
    }

    let flat = locate_flat_documents();

    if flat.is_empty() {
        return None;
    }

    let dir = model::create(MIGRATED_PROJECT).ok()?;

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

        let _ = model::add_chapter_to_assembly(&dir.join("main.typ"), &destination);

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
/// Le fichier créé par `model::create` est un modèle vide : on ne le remplace
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

    /// Les entrées se lisent avec leurs auteurs, leur titre, leur année —
    /// c'est ce que l'onglet des références montrera à la place des clés.
    #[test]
    fn la_bibliographie_lit_les_auteurs_et_les_titres() {
        let texte = r#"
@article{dupont2023,
  author = {Dupont, Jean and Curie, Marie},
  title = {Une {étude} des notes},
  year = {2023},
}
@book{martin2020,
  author = "Paul Martin",
  title = "Le grand traité",
  year = "2020"
}
"#;
        let entrees = parse_bibliography_text(texte);

        assert_eq!(entrees.len(), 2, "deux entrées, même sur plusieurs lignes");
        assert_eq!(entrees[0].key, "dupont2023");
        assert_eq!(
            entrees[0].auteurs,
            vec![
                ("Dupont".to_string(), "Jean".to_string()),
                ("Curie".to_string(), "Marie".to_string())
            ],
            "« et » sépare les auteurs, la virgule dit le nom"
        );
        assert_eq!(entrees[0].titre, "Une {étude} des notes");
        assert_eq!(entrees[0].annee.as_deref(), Some("2023"));

        assert_eq!(
            entrees[1].libelle(),
            "Martin, Paul — Le grand traité",
            "la ligne de liste se lit : nom, prénom, titre"
        );
    }

    /// Sans auteur, la ligne montre le titre seul ; sans titre, la clé.
    #[test]
    fn le_libelle_tient_debout_sans_auteur() {
        let entree = BibliographyEntry {
            key: "anonyme".to_string(),
            kind: "book".to_string(),
            titre: "L'œuvre anonyme".to_string(),
            ..Default::default()
        };

        assert_eq!(entree.libelle(), "L'œuvre anonyme");
    }

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

    /// Une entrée sans `@` ou sans clé est ignorée, pas une erreur ; une
    /// entrée jamais refermée est le « problème » où s'arrête la lecture.
    #[test]
    fn ignores_malformed_bib_lines() {
        assert!(parse_bibliography_text("ceci n'est pas une entrée").is_empty());
        assert!(parse_bibliography_text("@article{,}").is_empty());
        assert!(parse_bibliography_text("@article{clé,").is_empty());
        assert_eq!(parse_bibliography_text("@article{clé,}").len(), 1);
    }

    /// Modifier une référence réécrit ses champs, sans toucher au reste du
    /// fichier — et la clé ne bouge pas, elle est citée dans les documents.
    #[test]
    fn modifier_une_reference_ne_reecrit_que_ses_champs() {
        let dir = std::env::temp_dir().join(format!("tw-maj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{dupont2023,\n  author = {Dupont, Jean},\n  title = {Un titre},\n  journal = {Revue},\n}\n@book{martin2020,\n  author = {Martin},\n  title = {L'autre},\n}\n",
        )
        .unwrap();

        maj_reference(&bib, "dupont2023", "Curie", "Marie", "Un autre titre", "2024").unwrap();

        let entrees = parse_bibliography(&bib);

        assert_eq!(entrees.len(), 2, "les deux entrées sont là");
        assert_eq!(entrees[0].key, "dupont2023", "la clé ne bouge pas");
        assert_eq!(
            entrees[0].auteurs,
            vec![("Curie".to_string(), "Marie".to_string())]
        );
        assert_eq!(entrees[0].titre, "Un autre titre");
        assert_eq!(entrees[0].annee.as_deref(), Some("2024"));
        assert_eq!(
            entrees[1].titre, "L'autre",
            "la seconde entrée n'a pas bougé"
        );

        let _ = std::fs::remove_dir_all(&dir);
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
