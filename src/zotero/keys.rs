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

    /// Réécrit un export BibTeX en clés stables, et dit le nombre de notices vues.
    ///
    /// **Une clé attribuée ne bouge plus.** L'identité d'une notice est celle
    /// que l'export porte : sa clé Zotero quand Better BibTeX l'écrit dans le
    /// champ `zotero`, sinon sa clé d'export — auquel cas la stabilité est
    /// celle de l'export lui-même, et elle le reste dite. Chaque notice vue
    /// une fois reçoit sa clé, et la garde.
    ///
    /// `pris` porte les clés déjà données ailleurs — la bibliothèque locale,
    /// d'abord : une clé de notice ne vient jamais heurter celle d'un livre
    /// que l'écrivain tient lui-même.
    pub fn stabiliser(
        &mut self,
        bibtex: &str,
        pris: &std::collections::BTreeSet<String>,
    ) -> String {
        let mut sortie = String::new();
        let mut vues = 0usize;

        for entree in split_entries(bibtex) {
            let Some(ancienne) = key_of(entree) else {
                sortie.push_str(entree);
                continue;
            };

            let identite = item_of(entree).unwrap_or_else(|| ancienne.clone());
            let (auteur, annee, titre) = champs_of(entree);
            let nouvelle = self.key_for(
                &identite,
                auteur.as_deref(),
                annee.as_deref(),
                titre.as_deref().unwrap_or_default(),
                pris,
            );

            vues += 1;

            // La tête change — `@type{ancienne,` — et rien d'autre.
            let ouverture = entree.find('{').map(|i| i + 1).unwrap_or(0);
            let virgule = entree[ouverture..]
                .find(',')
                .map(|i| ouverture + i)
                .unwrap_or(entree.len());

            sortie.push_str(&entree[..ouverture]);
            sortie.push_str(&nouvelle);
            sortie.push_str(&entree[virgule..]);
        }

        let _ = vues;
        sortie
    }
}

/// Découpe un export en entrées : chacune commence par `@` en tête de ligne.
fn split_entries(bibtex: &str) -> Vec<&str> {
    let mut entrees = Vec::new();
    let mut courant = 0usize;

    for (position, _) in bibtex.match_indices('\n') {
        let suivant = position + 1;

        if bibtex[suivant..].starts_with('@') {
            entrees.push(&bibtex[courant..suivant]);
            courant = suivant;
        }
    }

    entrees.push(&bibtex[courant..]);
    entrees.retain(|e| !e.trim().is_empty());
    entrees
}

/// La clé de la tête d'une entrée : `@type{clé,`.
fn key_of(entree: &str) -> Option<String> {
    let ouverture = entree.find('{')?;
    let virgule = entree[ouverture..].find(',').map(|i| ouverture + i)?;

    let cle = entree[ouverture + 1..virgule].trim().to_string();

    (!cle.is_empty()).then_some(cle)
}

/// L'identifiant Zotero de la notice, quand l'export le porte.
fn item_of(entree: &str) -> Option<String> {
    let bas = entree.to_lowercase();
    let position = bas.find("zotero = {")?;
    let debut = position + "zotero = {".len();
    let fin = entree[debut..].find('}')?;

    let identite = entree[debut..debut + fin].trim().to_string();

    (!identite.is_empty()).then_some(identite)
}

/// Auteur, année, titre — pour la proposition de clé.
fn champs_of(entree: &str) -> (Option<String>, Option<String>, Option<String>) {
    fn champ(entree: &str, nom: &str) -> Option<String> {
        let bas = entree.to_lowercase();
        let motif = format!("{nom} = {{");
        let position = bas.find(&motif)?;
        let debut = position + motif.len();
        let fin = entree[debut..].find('}')?;

        let valeur = entree[debut..debut + fin].trim().to_string();
        (!valeur.is_empty()).then_some(valeur)
    }

    (
        champ(entree, "author"),
        champ(entree, "year").or_else(|| champ(entree, "date")),
        champ(entree, "title"),
    )
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
    /// Une clé attribuée ne bouge plus : la notice revient avec une clé
    /// d'export différente, la sienne reste.
    #[test]
    fn une_cle_attribuee_ne_bouge_plus() {
        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();

        let premier = trousseau.stabiliser(
            "@book{zotero01,\n  zotero = {ABCD1234},\n  author = {Dupont},\n  title = {Le titre}\n}\n",
            &pris,
        );

        // La collection bouge : la clé d'export change, la notice non.
        let second = trousseau.stabiliser(
            "@book{autreCle,\n  zotero = {ABCD1234},\n  author = {Dupont},\n  title = {Le titre corrigé}\n}\n",
            &pris,
        );

        let cle = |texte: &str| {
            let ouverture = texte.find('{').unwrap() + 1;
            let virgule = texte[ouverture..].find(',').unwrap() + ouverture;
            texte[ouverture..virgule].to_string()
        };

        assert_eq!(
            cle(&premier),
            cle(&second),
            "la clé suit la notice : {second}"
        );
    }

    /// La clé ne vient jamais heurter celle de la bibliothèque locale.
    #[test]
    fn la_cle_locale_est_respectee() {
        let mut trousseau = Keyring::default();
        let mut pris = std::collections::BTreeSet::new();
        pris.insert("dupont2023titre".to_string());

        let rendu = trousseau.stabiliser(
            "@book{x,\n  zotero = {ABCD1234},\n  author = {Dupont},\n  year = {2023},\n  title = {Le titre}\n}\n",
            &pris,
        );

        assert!(
            !rendu.contains("@book{dupont2023titre,"),
            "la clé locale est prise : {rendu}"
        );
    }

    /// Plusieurs notices d'un même auteur, la même année, ne portent pas
    /// la même clé.
    #[test]
    fn deux_notices_ne_partagent_pas_une_cle() {
        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();

        let rendu = trousseau.stabiliser(
            "@book{a,\n  zotero = {A1},\n  author = {Dupont},\n  year = {2023},\n  title = {Le titre}\n}\n\n@book{b,\n  zotero = {B2},\n  author = {Dupont},\n  year = {2023},\n  title = {Le titre}\n}\n",
            &pris,
        );

        let premier = rendu.matches('@').count();
        assert_eq!(premier, 2, "deux entrées : {rendu}");

        // Les clés sont dans les têtes : `@book{clé,`.
        let cles: Vec<&str> = rendu
            .match_indices("@book{")
            .map(|(i, _)| {
                let ouverture = i + "@book{".len();
                let fin = rendu[ouverture..].find(',').unwrap() + ouverture;
                &rendu[ouverture..fin]
            })
            .collect();

        assert_eq!(cles.len(), 2, "deux têtes : {rendu}");
        assert_ne!(cles[0], cles[1], "deux clés, jamais une : {rendu}");
    }

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

#[cfg(test)]
mod tests_de_stabilisation {
    use super::*;

    /// La stabilisation ne casse jamais le BibTeX : l'export d'un lien de
    /// collection, réécrit en clés stables, se relit et se compile. C'est
    /// le bug 2 du rapport 2 — « expecting an opening brace » — pris à la
    /// racine.
    #[test]
    fn la_stabilisation_ne_casse_jamais_le_bibtex() {
        let export = "% This file was created with Better BibTeX\n\n@book{dupont2020titre,\n  title = {Le titre {{Protégé}} d'un livre},\n  author = {Dupont, Jean},\n  date = {2020},\n  zotero = {ABCD1234},\n}\n\n@article{martin2021autre,\n  title = {Un second titre},\n  author = {Martin, Marie},\n  date = {2021},\n  zotero = {EFGH5678},\n}\n";

        let mut trousseau = Keyring::default();
        let pris = std::collections::BTreeSet::new();
        let sortie = trousseau.stabiliser(export, &pris);

        // Chaque notice garde son corps : le titre protégé s'y trouve,
        // accolades comprises.
        assert!(sortie.contains("{{Protégé}}"), "le titre protégé survit : {sortie}");

        // Le fichier se compile : c'est le compilateur qui juge.
        let dossier = std::env::temp_dir().join("typst-writer-epreuve-bib");
        let _ = std::fs::create_dir_all(&dossier);
        let chemin = dossier.join("references-zotero.bib");
        std::fs::write(&chemin, &sortie).expect("le bib s'écrit");

        // La clé stabilisée est celle du fichier : on cite ce qui s'y
        // trouve, pas ce qui s'y trouvait.
        let cle = sortie
            .lines()
            .find_map(|ligne| {
                let ouverture = ligne.find('{')?;
                let virgule = ligne[ouverture..].find(',').map(|i| ouverture + i)?;
                ligne.get(ouverture + 1..virgule).map(str::to_string)
            })
            .expect("une clé dans la sortie");

        std::fs::write(
            dossier.join("document.typ"),
            format!(
                "#set text(lang: \"fr\")\n#bibliography(\"references-zotero.bib\")\nUn test @{cle}.\n"
            ),
        )
        .expect("le document s'écrit");

        let rendu = std::process::Command::new("typst")
            .args(["compile", dossier.join("document.typ").to_str().unwrap()])
            .output()
            .expect("typst s'exécute");

        assert!(
            rendu.status.success(),
            "le bib se compile toujours : {}",
            String::from_utf8_lossy(&rendu.stderr)
        );
    }
}
