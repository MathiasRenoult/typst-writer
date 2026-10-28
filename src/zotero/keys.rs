//! Les clés de citation, stables avec ou sans Better BibTeX.
//!
//! Une clé de citation est ce que le document écrit — `@dupont2023intro` — et
//! ce qu'il ne faut **jamais** casser : un travail de théologie se cite
//! lui-même des mois durant. Better BibTeX sait faire cette stabilité ; sans
//! lui, c'est ici que cela se décide.
//!
//! La règle tient en une phrase : **une clé attribuée ne bouge plus.** Elle
//! est demandée une fois par notice, écrite dans `citations.json`, et y
//! survit à tous les changements de la notice — un titre corrigé, une année
//! complétée, un auteur ajouté ne renomment pas ce qui est déjà cité. La
//! dérivation n'intervient qu'au **premier** regard porté sur une notice.

use std::collections::BTreeMap;
use std::path::Path;

/// Les clés déjà attribuées, par notice Zotero.
#[derive(Debug, Default)]
pub struct Keyring {
    /// `identifiant de notice` → `clé de citation`.
    keys: BTreeMap<String, String>,
}

impl Keyring {
    /// Le trousseau écrit par une version précédente ; vide s'il n'y en a pas.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };

        // Le fichier est la carte elle-même : `{"ABCD": "dupont2023titre"}`.
        let keys = serde_json::from_str(&text).unwrap_or_default();

        Self { keys }
    }

    /// Écrit le trousseau, tel qu'il sera relu.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let texte = serde_json::to_string_pretty(&self.keys).map_err(std::io::Error::other)?;

        crate::project::ecrire_atomique(path, &texte)
    }

    /// La clé d'une notice, qu'elle soit déjà attribuée ou à attribuer.
    ///
    /// `pris` porte les clés déjà données à d'autres notices : deux notices
    /// d'un même auteur, la même année, ne portent pas la même clé — elles
    /// reçoivent un suffixe, dans l'ordre où elles se présentent.
    pub fn key_for(
        &mut self,
        item: &str,
        author: Option<&str>,
        year: Option<&str>,
        title: &str,
        pris: &std::collections::BTreeSet<String>,
    ) -> String {
        if let Some(key) = self.keys.get(item) {
            return key.clone();
        }

        let base = derive(author, year, title);
        let mut key = base.clone();
        let mut rang = 0u32;

        while pris.contains(&key) || self.keys.values().any(|k| k == &key) {
            rang += 1;
            key = format!("{base}{}", suffix(rang));
        }

        self.keys.insert(item.to_string(), key.clone());
        key
    }
}

/// La clé qu'une notice mérite : `nom2023premiermot`.
///
/// Ce n'est qu'une proposition — elle n'est utilisée qu'au premier regard,
/// puis figée. Elle se veut reconnaissable à la lecture d'un document, pas
/// parfaite : un nom, une année, un mot du titre suffisent à retrouver la
/// notice dans une bibliographie.
fn derive(author: Option<&str>, year: Option<&str>, title: &str) -> String {
    let mut key = fold(author.unwrap_or_default());
    let annee: String = year
        .unwrap_or_default()
        .chars()
        .filter(char::is_ascii_digit)
        .take(4)
        .collect();

    key.push_str(&annee);
    key.push_str(&first_word(title));

    // Une clé ne commence pas par un chiffre et n'est jamais vide : sans
    // auteur ni année, le mot du titre porte seule, et sinon la notice.
    if key.is_empty() {
        return "notice".to_string();
    }

    if key.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        key.insert(0, 'n');
    }

    key
}

/// Le premier mot du titre qui en vaille un.
fn first_word(title: &str) -> String {
    title
        .split(|c: char| !c.is_alphanumeric())
        .find(|mot| mot.chars().count() > 2)
        .map(fold)
        .unwrap_or_default()
}

/// La forme ordinaire d'une clé : minuscules, sans accent, sans espace.
///
/// Plié à la main : les clés se lisent dans le texte, et la moindre
/// dépendance de plus pour vingt caractères d'accents serait un mauvais
/// marché.
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' | 'ì' => 'i',
            'ô' | 'ö' | 'ó' | 'ò' | 'õ' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ÿ' | 'ý' => 'y',
            'ñ' => 'n',
            'œ' => 'o',
            'æ' => 'a',
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
}

/// Un suffixe de désambiguïsation : a, b, … z, aa, ab, …
fn suffix(rang: u32) -> String {
    let mut out = String::new();
    let mut rang = rang;

    while rang > 0 {
        rang -= 1;
        out.insert(0, (b'a' + (rang % 26) as u8) as char);
        rang /= 26;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une clé lisible : nom, année, mot du titre.
    #[test]
    fn a_key_is_readable() {
        assert_eq!(
            derive(Some("Éric Dupont"), Some("2023"), "Introduction à la grâce"),
            "ericdupont2023introduction"
        );
    }

    /// La promesse : une clé attribuée ne bouge plus, même quand la notice
    /// change — c'est tout l'intérêt de la noter.
    #[test]
    fn an_assigned_key_never_moves() {
        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();

        let première =
            trousseau.key_for("ABCD", Some("Dupont"), Some("2023"), "Premier titre", &pris);

        // La notice change : nouveau titre, nouvelle année, autre auteur.
        let seconde = trousseau.key_for("ABCD", Some("Martin"), Some("2024"), "Autre titre", &pris);

        assert_eq!(première, seconde, "la clé attribuée tient bon");
        assert_eq!(première, "dupont2023premier");
    }

    /// Deux notices jumelles ne portent pas la même clé.
    #[test]
    fn twin_notices_do_not_share_a_key() {
        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();

        let a = trousseau.key_for("A", Some("Dupont"), Some("2023"), "Titre", &pris);
        let b = trousseau.key_for("B", Some("Dupont"), Some("2023"), "Titre", &pris);

        assert_eq!(a, "dupont2023titre");
        assert_eq!(b, "dupont2023titrea", "un suffixe les sépare");
    }

    /// Le trousseau survit à la fermeture de l'application.
    #[test]
    fn the_keyring_is_written_and_read_back() {
        let path = std::env::temp_dir().join(format!("tw-cles-{}.json", std::process::id()));
        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();

        trousseau.key_for("ABCD", Some("Dupont"), None, "Titre", &pris);
        trousseau.save(&path).expect("écrit");

        let relu = Keyring::load(&path);
        assert_eq!(
            relu.keys.get("ABCD").map(String::as_str),
            Some("duponttitre"),
            "la clé revient telle quelle"
        );

        std::fs::remove_file(&path).ok();
    }
}
