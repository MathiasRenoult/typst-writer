//! L'architecture des extensions (bug-report-7, point 4).
//!
//! « Il faudrait que ce soit facile pour nous de prévoir ce genre
//! d'extensions dans le futur. » La promesse : **ajouter un paquet Typst
//! et sa GUI sans toucher au reste**. Voici la maison :
//!
//! 1. **Le paquet se résout localement** — `packages_dir()` et
//!    [`crate::typst::render`] : un `#import "@local/cetz:…"` compile
//!    depuis le dossier des paquets, sans réseau ;
//! 2. **Un trait décrit l'extension** — [`ExtensionPaquet`] : son
//!    identité, son insertion (les paramètres → le code Typst) ;
//! 3. **Un registre les rassemble** — [`extensions`] : l'application ne
//!    connaît que lui. Ajouter une extension = un type de plus dans le
//!    registre.
//!
//! Le troisième jalon — la **GUI** de réglage (le composeur cetz) —
//! s'ajoutera comme une vue par extension ; le trait grandira d'une
//! méthode `vue` quand la première arrivera, pas avant.

use std::collections::BTreeMap;

/// Un paquet Typst intégré à l'application.
pub trait ExtensionPaquet: Send + Sync {
    /// Son identité Typst — `@local/cetz:0.3.4`.
    fn paquet(&self) -> &'static str;

    /// Son nom à l'écran.
    fn nom(&self) -> &'static str;

    /// Les paramètres qu'il propose : nom → défaut. C'est le cahier de
    /// la future GUI : une saisie par ligne.
    fn parametres(&self) -> &'static [(&'static str, &'static str)];

    /// Les paramètres choisis deviennent le code Typst posé dans le
    /// document — l'écrivain ne tape jamais ce code.
    fn inserer(&self, parametres: &BTreeMap<String, String>) -> String;
}

/// Le registre : ce que l'application offre. Ajouter une extension, c'est
/// une ligne ici.
pub fn extensions() -> Vec<Box<dyn ExtensionPaquet>> {
    vec![Box::new(Cetz)]
}

/// La première extension — la preuve que la maison tient : ceTZ, le
/// graphique, réglé par paramètres plutôt que tapé.
pub struct Cetz;

impl ExtensionPaquet for Cetz {
    fn paquet(&self) -> &'static str {
        "@local/cetz:0.3.4"
    }

    fn nom(&self) -> &'static str {
        "Graphique (cetz)"
    }

    fn parametres(&self) -> &'static [(&'static str, &'static str)] {
        &[("largeur", "8cm"), ("hauteur", "5cm"), ("axes", "oui")]
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
        let largeur = valeur(parametres, "largeur", "8cm");
        let hauteur = valeur(parametres, "hauteur", "5cm");
        let axes = valeur(parametres, "axes", "oui") == "oui";
        format!(
            "#{{\\n  import \\\"{}\\\": canvas, draw\\n  canvas(width: {}, height: {}, {{\\n    if {axes} {{ draw.set-style(mark: (fill: black)) }}\\n  }})\\n}}\\n",
            self.paquet(),
            largeur,
            hauteur
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Cetz, ExtensionPaquet, extensions};

    /// La maison tient : le registre parle, et l'extension écrit le code
    /// à la place de l'écrivain.
    #[test]
    fn une_extension_ecrit_le_code_a_sa_place() {
        let registre = extensions();
        assert_eq!(registre.len(), 1, "la première extension est là");
        assert_eq!(registre[0].nom(), "Graphique (cetz)");

        let mut choix = std::collections::BTreeMap::new();
        choix.insert("largeur".to_string(), "12cm".to_string());
        let code = Cetz.inserer(&choix);
        assert!(code.contains("width: 12cm"), "le choix vient : {code}");
        assert!(
            code.contains("@local/cetz:0.3.4"),
            "le paquet est nommé : {code}"
        );
    }
}
