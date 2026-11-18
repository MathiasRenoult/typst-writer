//! L'architecture des extensions (bug-report-7, point 4).
//!
//! « Il faudrait que ce soit facile pour nous de prévoir ce genre
//! d'extensions dans le futur. » La promesse : **ajouter un paquet Typst
//! et sa GUI sans toucher au reste**. Voici la maison :
//!
//! 1. **Le paquet se résout localement** — `packages_dir()` et
//!    [`crate::typst::render`] : un `#import "@local/primaviz:…"` compile
//!    depuis le dossier des paquets, sans réseau ;
//! 2. **Un trait décrit l'extension** — [`ExtensionPaquet`] : son
//!    identité, son insertion (les paramètres → la source `.tw`) ;
//! 3. **Un registre les rassemble** — [`extensions`] : l'application ne
//!    connaît que lui. Ajouter une extension = un type de plus dans le
//!    registre.
//!
//! Le composeur générique reste le chemin rapide ; le panneau `{chart}`
//! (bouton de la barre d'outils) est le chemin complet, avec le CSV en
//! plusieurs lignes et l'import de fichier.

use std::collections::BTreeMap;

/// Un paquet Typst intégré à l'application.
pub trait ExtensionPaquet: Send + Sync {
    /// Son identité Typst — `@local/primaviz:0.12.1`.
    fn paquet(&self) -> &'static str;

    /// Son nom à l'écran.
    fn nom(&self) -> &'static str;

    /// Les paramètres qu'il propose : nom → défaut. C'est le cahier de
    /// la GUI : une saisie par ligne.
    fn parametres(&self) -> &'static [(&'static str, &'static str)];

    /// Les paramètres choisis deviennent la source posée dans le
    /// document — l'écrivain ne tape jamais le CSV.
    fn inserer(&self, parametres: &BTreeMap<String, String>) -> String;
}

/// Le registre : ce que l'application offre. Ajouter une extension, c'est
/// une ligne ici.
pub fn extensions() -> Vec<Box<dyn ExtensionPaquet>> {
    vec![Box::new(Primaviz)]
}

/// La première extension — primaviz, le graphique réglé par paramètres
/// plutôt que tapé.
pub struct Primaviz;

impl ExtensionPaquet for Primaviz {
    fn paquet(&self) -> &'static str {
        "@local/primaviz:0.12.1"
    }

    fn nom(&self) -> &'static str {
        "Graphique (primaviz)"
    }

    fn parametres(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("type", "bar"),
            ("titre", ""),
            ("largeur", "10cm"),
            ("hauteur", "6cm"),
            ("placement", "centre"),
            ("legende", "oui"),
            ("valeurs", "oui"),
            ("donnees", "A,1;B,2"),
        ]
    }

    fn inserer(&self, parametres: &BTreeMap<String, String>) -> String {
        fn valeur<'a>(
            parametres: &'a BTreeMap<String, String>,
            cle: &str,
            defaut: &'a str,
        ) -> &'a str {
            parametres
                .get(cle)
                .map(String::as_str)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or(defaut)
        }

        let placement = match valeur(parametres, "placement", "centre")
            .trim()
            .to_lowercase()
            .as_str()
        {
            "gauche" | "left" => "gauche",
            "droite" | "right" => "droite",
            "pleine-largeur" | "pleine" | "full" => "pleine-largeur",
            _ => "centre",
        };
        let chart = crate::langage::schema::Chart {
            type_: crate::langage::chart::normaliser_type(valeur(parametres, "type", "bar"))
                .to_string(),
            titre: valeur(parametres, "titre", "").to_string(),
            largeur: valeur(parametres, "largeur", "10cm").to_string(),
            hauteur: valeur(parametres, "hauteur", "6cm").to_string(),
            placement: placement.to_string(),
            legende: oui(valeur(parametres, "legende", "oui")),
            valeurs: oui(valeur(parametres, "valeurs", "oui")),
            donnees: valeur(parametres, "donnees", "A,1;B,2").replace(';', "\n"),
            ..crate::langage::schema::Chart::default()
        };
        crate::langage::chart::source(&chart)
    }
}

fn oui(valeur: &str) -> bool {
    matches!(
        valeur.trim().to_lowercase().as_str(),
        "oui" | "true" | "1" | "vrai"
    )
}

#[cfg(test)]
mod tests {
    use super::{ExtensionPaquet, Primaviz, extensions};

    /// Le registre parle, et l'extension écrit une balise primaviz à la
    /// place de l'écrivain.
    #[test]
    fn une_extension_ecrit_la_balise_a_sa_place() {
        let registre = extensions();
        assert_eq!(registre.len(), 1, "la première extension est là");
        assert_eq!(registre[0].nom(), "Graphique (primaviz)");

        let mut choix = std::collections::BTreeMap::new();
        choix.insert("largeur".to_string(), "12cm".to_string());
        choix.insert("donnees".to_string(), "A,1;B,2".to_string());
        let code = Primaviz.inserer(&choix);
        assert!(
            code.contains("{chart bar 12cm×6cm"),
            "la balise compacte est posée : {code}"
        );
        assert!(
            code.contains("A,1\nB,2"),
            "le CSV reprend ses lignes : {code}"
        );
        assert!(!code.contains("cetz"), "primaviz a remplacé cetz : {code}");
    }
}
