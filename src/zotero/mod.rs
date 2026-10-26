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

pub mod keys;
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
        let prefs = entry.ok()?.path().join("prefs.js");
        let text = std::fs::read_to_string(prefs).ok()?;

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
