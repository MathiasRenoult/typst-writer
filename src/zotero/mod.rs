//! La collaboration avec Zotero, si l'utilisateur la désire.
//!
//! Zotero tourne chez lui, avec sa propre bibliothèque, et expose un petit
//! serveur local (port 23119). C'est par là que l'application lit les
//! collections, cherche dans celle qui est pointée, et reçoit ce qu'il faut
//! pour tenir `references-zotero.bib` à jour. Rien de tout cela n'est
//! obligatoire : sans Zotero, sans collection pointée, l'application ignore ce
//! dossier.
//!
//! **Better BibTeX est recommandé, jamais requis.** Avec lui, les clés de
//! citation viennent de lui et ne bougent plus jamais — c'est la stabilité
//! qu'attendent des documents qui vivent des mois. Sans lui, l'application
//! fabrique ses propres clés et les **garde** : une clé, une fois attribuée à
//! une notice, ne change plus quand la notice change. C'est écrit noir sur
//! blanc dans `citations.json`, à côté du projet.
//!
//! Ce qui vit ici, pour l'instant, ne parle à personne : la découverte de la
//! bibliothèque et les styles enregistrés sont des lectures de disque.

// Pièces posées avant d'être raccordées : les étapes suivantes du chantier
// les emploient (le client, la synchronisation, le sélecteur). Cet attribut
// partira avec le raccordement.
#![allow(dead_code)]

pub mod client;
pub mod keys;

use serde::{Deserialize, Serialize};
pub mod styles;

use std::path::PathBuf;

/// Le dossier de données de Zotero, là où vit sa bibliothèque.
///
/// Le profil de Zotero le dit (`prefs.js`), et la convention sert de repli.
/// Ni l'un ni l'autre : Zotero n'est pas là, et c'est un état, pas une panne.
pub fn data_dir() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var("HOME").ok()?);

    if let Some(dir) = data_dir_from_profiles(&home) {
        return Some(dir);
    }

    let convention = home.join("Zotero");

    convention.is_dir().then_some(convention)
}

/// Ce que les profils de Zotero disent de son dossier de données.
fn data_dir_from_profiles(home: &std::path::Path) -> Option<PathBuf> {
    let profiles = home.join(".zotero").join("zotero");

    for entry in std::fs::read_dir(profiles).ok()? {
        // Un profil sans `prefs.js` lisible n'est pas une raison de
        // renoncer : le suivant porte peut-être le réglage.
        let Ok(entry) = entry else {
            continue;
        };
        let prefs = entry.path().join("prefs.js");
        let Ok(text) = std::fs::read_to_string(prefs) else {
            continue;
        };

        for line in text.lines() {
            let Some(rest) = line
                .trim()
                .strip_prefix("user_pref(\"extensions.zotero.dataDir\", ")
            else {
                continue;
            };

            // La valeur suit, entre guillemets. Un chemin de données n'en
            // porte pas dans sa chair : les échappements JavaScript ne sont
            // pas traités, et c'est délibéré.
            let path = rest.split('"').nth(1)?;

            return Some(PathBuf::from(path));
        }
    }

    None
}

/// D'où vient la bibliothèque affichée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Source {
    /// Une bibliothèque locale, non synchronisée : elle s'écrit.
    #[default]
    Locale,
    /// La collection Zotero pointée : elle se lit, et la synchronisation la
    /// reprend — jamais d'édition par ici.
    Zotero,
}

/// La collection Zotero à laquelle un projet est lié.
///
/// C'est tout le contrat : sa clé — celle de Zotero, qui ne bouge plus — et
/// son nom, pour se rappeler à quoi on a lié son travail. La notice vit dans
/// `zotero.json`, à côté du projet : elle voyage avec lui.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lien {
    /// La clé de collection, l'identifiant stable.
    pub cle: String,

    /// Son nom, tel que Zotero le montre.
    pub nom: String,
}

/// Le nom du fichier qui tient la liaison.
pub const FICHIER: &str = "zotero.json";

/// La liaison d'un projet, s'il en a une.
/// La liaison d'un projet — `None` s'il n'en a pas.
///
/// Un fichier corrompu n'est pas « pas de liaison » : c'est un problème,
/// et il le dit. Ce qui est perdu se sait.
pub fn lien_du_projet(dir: &std::path::Path) -> Result<Option<Lien>, String> {
    let chemin = dir.join(FICHIER);

    let texte = match std::fs::read_to_string(&chemin) {
        Ok(texte) => texte,
        Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(erreur) => return Err(format!("{} : {erreur}", chemin.display())),
    };

    serde_json::from_str(&texte)
        .map(Some)
        .map_err(|erreur| format!("{} : {erreur}", chemin.display()))
}

/// Lie un projet à une collection — et jamais à la place de l'utilisateur.
pub fn lier(dir: &std::path::Path, lien: &Lien) -> std::io::Result<()> {
    let texte = serde_json::to_string_pretty(lien).map_err(std::io::Error::other)?;
    crate::project::ecrire_atomique(&dir.join(FICHIER), &texte)
}

/// Délie un projet : il retrouve sa bibliothèque locale.
///
/// La liaison part vers la corbeille, pas dans le vide : se délier est un
/// geste qu'on fait parfois à tort.
pub fn delier(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    crate::project::vers_la_corbeille(&dir.join(FICHIER)).ok()
}

/// Les préférences qui ouvrent l'API locale de Zotero.
///
/// Ce sont elles que la case « Permettre aux autres applications sur cet
/// ordinateur de communiquer avec Zotero » coche — Réglages → Avancé.
const PREFERENCES: &[&str] = &[
    "extensions.zotero.httpServer.enabled",
    "extensions.zotero.httpServer.localAPI.enabled",
];

/// Ouvre l'API locale de Zotero, dans ses profils.
///
/// C'est le geste de la case, écrit une fois pour toutes dans le
/// `user.js` de chaque profil : le mécanisme standard des profils, qui
/// survit aux réécritures de `prefs.js`. **C'est réversible** : retirer
/// les lignes remet l'état d'avant.
///
/// Le réglage prend effet au prochain démarrage de Zotero ; la case, elle,
/// s'applique tout de suite. Rien de tout cela ne ferme Zotero.
pub fn activer_l_api_locale() -> Result<Vec<std::path::PathBuf>, String> {
    let home = std::path::PathBuf::from(
        std::env::var("HOME").map_err(|_| "aucun dossier personnel".to_string())?,
    );

    activer_l_api_locale_depuis(&home)
}

/// La même ouverture, depuis un dossier personnel — celle des tests.
pub fn activer_l_api_locale_depuis(
    home: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    let profils = home.join(".zotero").join("zotero");
    let entrees = std::fs::read_dir(&profils)
        .map_err(|_| "Aucun profil Zotero trouvé — Zotero est-il installé ?".to_string())?;

    let mut touches = Vec::new();

    for entree in entrees.filter_map(Result::ok) {
        let dossier = entree.path();

        if !dossier.is_dir() {
            continue;
        }

        // Un profil est un dossier qui porte des préférences ; les autres
        // — caches, sauvegardes — ne se touchent pas.
        if !dossier.join("prefs.js").is_file() {
            continue;
        }

        let user = dossier.join("user.js");
        let existant = std::fs::read_to_string(&user).unwrap_or_default();

        let mut nouveau = existant.clone();

        for preference in PREFERENCES {
            let ligne = format!("user_pref(\"{preference}\", true);");

            if !nouveau.contains(&ligne) {
                nouveau.push_str(&ligne);
                nouveau.push('\n');
            }
        }

        if nouveau != existant {
            std::fs::write(&user, nouveau)
                .map_err(|erreur| format!("{} : {erreur}", user.display()))?;
        }

        touches.push(dossier);
    }

    if touches.is_empty() {
        return Err("Aucun profil Zotero trouvé — Zotero est-il installé ?".to_string());
    }

    Ok(touches)
}

/// L'état de la collaboration avec Zotero.
#[derive(Debug, Clone, Default)]
pub struct Etat {
    /// D'où viennent les références montrées.
    pub source: Source,
    /// Zotero est-il lié ? Sa collection est pointée, sauf décision
    /// contraire : on peut toujours revenir à une bibliothèque locale.
    pub lie: bool,
}

#[cfg(test)]
mod tests {

    /// La procédure s'écrit toute seule : le `user.js` des profils porte
    /// le réglage, et deux passages ne le disent pas deux fois.
    #[test]
    fn la_procedure_s_ecrit_elle_meme() {
        let home = std::env::temp_dir().join(format!("tw-api-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);

        let profil = home.join(".zotero/zotero/abcd1234.default");
        std::fs::create_dir_all(&profil).expect("profil");
        std::fs::write(profil.join("prefs.js"), "// les préférences\n").expect("prefs");

        // Un dossier qui n'est pas un profil ne se touche pas.
        std::fs::create_dir_all(home.join(".zotero/zotero/cache")).expect("cache");

        let touches = activer_l_api_locale_depuis(&home).expect("ouverture");
        assert_eq!(touches, vec![profil.clone()]);

        let user = std::fs::read_to_string(profil.join("user.js")).expect("user.js");
        assert!(user.contains("httpServer.localAPI.enabled"), "vu : {user}");

        // Deuxième passage : rien ne se dédouble.
        activer_l_api_locale_depuis(&home).expect("ouverture");
        let second = std::fs::read_to_string(profil.join("user.js")).expect("user.js");

        assert_eq!(
            second.matches("localAPI.enabled").count(),
            1,
            "une seule ligne, jamais deux : {second}"
        );

        let _ = std::fs::remove_dir_all(&home);
    }

    /// Sans profil, la procédure le dit — pas un silence.
    #[test]
    fn sans_profil_la_procedure_le_dit() {
        let home = std::env::temp_dir().join(format!("tw-api-2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("dossier");

        let erreur = activer_l_api_locale_depuis(&home).expect_err("personne");
        assert!(erreur.contains("Zotero"), "vu : {erreur}");

        let _ = std::fs::remove_dir_all(&home);
    }

    /// La liaison se garde à côté du projet, et se relit telle quelle.
    #[test]
    fn une_liaison_se_garde_et_se_relit() {
        let dir = std::env::temp_dir().join(format!("tw-liaison-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier");

        assert_eq!(
            lien_du_projet(&dir).expect("lecture"),
            None,
            "rien, tant qu'on n'a rien lié"
        );

        let lien = Lien {
            cle: "AAAA1111".to_string(),
            nom: "Ma thèse".to_string(),
        };
        lier(&dir, &lien).expect("liaison écrite");
        assert_eq!(lien_du_projet(&dir).expect("lecture"), Some(lien));

        delier(&dir);
        assert_eq!(
            lien_du_projet(&dir).expect("lecture"),
            None,
            "délié, il n'y a plus rien"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Un fichier de liaison corrompu n'est pas « pas de liaison » : c'est
    /// un problème, et il le dit. Ce qui est perdu se sait.
    #[test]
    fn un_fichier_corrompu_dit_son_probleme() {
        let dir = std::env::temp_dir().join(format!("tw-liaison-2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier");
        std::fs::write(dir.join(FICHIER), "n'importe quoi").expect("fichier");

        let probleme = lien_du_projet(&dir).expect_err("corrompu");
        assert!(
            probleme.contains(FICHIER),
            "le fichier est nommé : {probleme}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    use super::*;

    /// Le profil dit où vit la bibliothèque ; sans profil, la convention.
    #[test]
    fn the_data_dir_comes_from_the_profile_and_falls_back_to_the_convention() {
        let home = std::env::temp_dir().join(format!("tw-zotero-{}", std::process::id()));
        let profile = home.join(".zotero/zotero/abcd1234.default");
        std::fs::create_dir_all(&profile).expect("profil");

        assert_eq!(
            data_dir_from_profiles(&home),
            None,
            "un profil sans réglage ne dit rien"
        );

        let prefs = concat!(
            "// un préambule\n",
            "user_pref(\"browser.startup.homepage\", \"about:blank\");\n",
            "user_pref(\"extensions.zotero.dataDir\", \"/chemin/vers/Zotero\");\n",
        );

        std::fs::write(profile.join("prefs.js"), prefs).expect("prefs");

        assert_eq!(
            data_dir_from_profiles(&home),
            Some(PathBuf::from("/chemin/vers/Zotero")),
            "le réglage du profil fait foi"
        );

        std::fs::remove_dir_all(&home).ok();
    }

    /// Ce que le Zotero de cette machine contient réellement comme styles.
    ///
    /// Les tests ne voient que des fichiers fabriqués ; ceci lit la
    /// bibliothèque du jour, et dit si le sélecteur aura de quoi parler.
    #[test]
    #[ignore = "lit le dossier de données de Zotero sur cette machine"]
    fn diagnose_styles() {
        let Some(dir) = data_dir() else {
            println!("aucun dossier de données Zotero trouvé");

            return;
        };

        println!("dossier de données : {}", dir.display());

        let styles = styles::installed(&dir.join("styles"));

        println!("styles enregistrés : {}", styles.len());

        for style in styles.iter().take(25) {
            println!("  {} | {}", style.title, style.id);
        }
    }
}
