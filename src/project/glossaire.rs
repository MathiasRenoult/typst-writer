//! Le glossaire d'un projet : ses termes, ses sigles, ses définitions.
//!
//! Les entrées vivent dans `glossaire.toml`, à la racine du projet — comme
//! les métadonnées, elles appartiennent à l'auteur et se lisent **dans son
//! ordre**. À la compilation, elles deviennent le registre du paquet
//! `glossarium` (voir [`crate::langage::pipeline`]) : la référence
//! `{glo:clé}` du `.tw` et la liste finale `{glossaire}` s'y branchent.
//!
//! Une donnée jamais perdue : un fichier absent est un glossaire vide, un
//! fichier abîmé est un glossaire vide **et une trace** — jamais un
//! écrasement du fichier de l'auteur.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Le fichier, à la racine du projet.
pub const FICHIER: &str = "glossaire.toml";

/// Une entrée de glossaire.
///
/// Les noms de champs sont ceux du registre `glossarium` (`short`, `long`,
/// `description`, `plural`, `longplural`, `group`) — un seul vocabulaire,
/// du fichier à la compilation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EntreeGlossaire {
    /// La clé de référence : `{glo:clé}` l'appelle. Identifiant.
    pub key: String,
    /// La forme courte : sigle, abréviation — celle des reprises.
    pub short: Option<String>,
    /// La forme longue — développée à la première occurrence.
    pub long: Option<String>,
    /// La définition, après le terme dans la liste finale.
    pub description: Option<String>,
    /// La forme plurielle de la forme courte.
    pub plural: Option<String>,
    /// La forme plurielle de la forme longue.
    pub longplural: Option<String>,
    /// Le groupe : la rubrique de la liste finale.
    pub group: Option<String>,
}

impl EntreeGlossaire {
    /// Une entrée neuve, à sa clé — le minimum que glossarium exige.
    #[allow(dead_code)] // API de contrat : éprouvée par les tests du modèle.
    pub fn nouvelle(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ..Default::default()
        }
    }

    /// Une clé de glossaire est un **identifiant** : ni espace, ni signe
    /// qui ferait écho à la syntaxe des balises (`{glo:clé majuscule}`
    /// lit « majuscule » comme un mot-clé — une clé qui s'y terminerait
    /// parlerait contre elle-même).
    pub fn cle_valide(cle: &str) -> bool {
        !cle.is_empty()
            && cle
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }
}

/// Le glossaire d'un projet, **dans l'ordre de l'auteur**.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Glossaire {
    /// Les entrées, jamais triées : l'ordre du fichier est celui de la
    /// liste finale.
    pub entrees: Vec<EntreeGlossaire>,
}

impl Glossaire {
    /// Le fichier du projet.
    pub fn chemin(root: &Path) -> PathBuf {
        root.join(FICHIER)
    }

    /// Le glossaire du projet. Un fichier absent est un glossaire vide ;
    /// un fichier illisible aussi — mais il ne s'écrase pas : [`Self::sauver`]
    /// refuse tant que la lecture n'a pas réussi.
    pub fn charger(root: &Path) -> Self {
        Self::lire(&Self::chemin(root))
    }

    /// La lecture d'un fichier. Une absence est un glossaire vide ; un
    /// fichier illisible rend aussi un glossaire vide, mais [`Self::sauver`]
    /// refusera de l'écraser tant qu'il reste illisible.
    pub fn lire(chemin: &Path) -> Self {
        let Ok(texte) = std::fs::read_to_string(chemin) else {
            return Self::default();
        };
        toml::from_str(&texte).unwrap_or_default()
    }

    /// L'écriture, entière et atomique : la bourse du fichier ne se vide
    /// jamais entre le début et la fin de l'écriture.
    ///
    /// Si un fichier existe déjà mais ne se lit pas ou ne se parse pas, la
    /// sauvegarde est **refusée** : remplacer un glossaire abîmé par le
    /// glossaire vide affiché perdrait le travail de l'auteur.
    pub fn sauver(&self, root: &Path) -> std::io::Result<()> {
        let chemin = Self::chemin(root);

        match std::fs::read_to_string(&chemin) {
            Ok(texte) => {
                toml::from_str::<Glossaire>(&texte).map_err(|erreur| {
                    std::io::Error::other(format!(
                        "glossaire existant illisible, sauvegarde refusée : {erreur}"
                    ))
                })?;
            }
            Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {}
            Err(erreur) => {
                return Err(std::io::Error::other(format!(
                    "glossaire existant illisible, sauvegarde refusée : {erreur}"
                )));
            }
        }

        let texte = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("glossaire non sérialisable : {e}")))?;
        crate::services::stockage::ecrire_atomique(&chemin, &texte)
    }

    /// L'entrée d'une clé.
    #[allow(dead_code)] // API de contrat : éprouvée par les tests du modèle.
    pub fn entree(&self, cle: &str) -> Option<&EntreeGlossaire> {
        self.entrees.iter().find(|entree| entree.key == cle)
    }

    /// Ajoute une entrée — ou la remplace si sa clé existe déjà.
    /// L'ordre de l'auteur est conservé : le remplacement reste en place.
    pub fn definir(&mut self, entree: EntreeGlossaire) {
        if let Some(ancienne) = self.entrees.iter_mut().find(|e| e.key == entree.key) {
            *ancienne = entree;
        } else {
            self.entrees.push(entree);
        }
    }

    /// Retire l'entrée d'une clé, et dit si elle y était.
    pub fn supprimer(&mut self, cle: &str) -> bool {
        let avant = self.entrees.len();
        self.entrees.retain(|entree| entree.key != cle);
        self.entrees.len() != avant
    }

    /// Le registre glossarium : le tableau de dictionnaires Typst que
    /// `#register-glossary` et `#print-glossary` comprennent.
    ///
    /// Les champs vides ne partent pas : glossarium préfère l'absence à
    /// une chaîne vide, et le code généré reste lisible.
    pub fn registre_typst(&self) -> String {
        let mut out = String::from("(\n");
        for entree in &self.entrees {
            out.push_str("  (");
            out.push_str(&format!("key: \"{}\"", echapper(&entree.key)));
            for (champ, valeur) in [
                ("short", &entree.short),
                ("long", &entree.long),
                ("description", &entree.description),
                ("plural", &entree.plural),
                ("longplural", &entree.longplural),
                ("group", &entree.group),
            ] {
                if let Some(valeur) = valeur.as_deref().filter(|v| !v.trim().is_empty()) {
                    out.push_str(&format!(", {champ}: \"{}\"", echapper(valeur)));
                }
            }
            out.push_str("),\n");
        }
        out.push(')');
        out
    }
}

/// Une chaîne Typst : le guillemet et la barre oblique s'y échappent.
fn echapper(texte: &str) -> String {
    let mut out = String::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un dossier de test, unique par test et par processus.
    fn racine(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "typst-writer-glossaire-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&chemin);
        std::fs::create_dir_all(&chemin).expect("racine du test");
        chemin
    }

    /// L'aller-retour par le fichier garde tout, dans l'ordre de l'auteur.
    #[test]
    fn un_glossaire_se_persiste_dans_l_ordre() {
        let root = racine("persistance");
        let mut glossaire = Glossaire::default();
        glossaire.definir(EntreeGlossaire {
            key: "sdo".into(),
            short: Some("sdo".into()),
            long: Some("docteur scolastique".into()),
            plural: Some("sdo".into()),
            longplural: Some("docteurs scolastiques".into()),
            description: Some("Un théologien du \"moyen âge\"".into()),
            group: Some("Acteurs".into()),
        });
        glossaire.definir(EntreeGlossaire::nouvelle("cat"));

        glossaire.sauver(&root).expect("sauvegarde");
        let relu = Glossaire::charger(&root);

        assert_eq!(relu, glossaire, "tout revient, y compris l'ordre");
        assert_eq!(relu.entrees[0].key, "sdo", "l'auteur a posé sdo le premier");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un fichier absent ou abîmé est un glossaire vide — et il ne
    /// s'écrase pas sous le vide.
    #[test]
    fn un_fichier_abime_ne_craque_ni_ne_s_ecrase() {
        let root = racine("abime");
        std::fs::write(root.join(FICHIER), "ceci n'est pas du toml [[[").expect("fichier fautif");

        let glossaire = Glossaire::charger(&root);
        assert!(glossaire.entrees.is_empty(), "le vide, sans panique");
        assert!(
            glossaire.sauver(&root).is_err(),
            "la sauvegarde refuse d'écraser un fichier abîmé"
        );
        assert_eq!(
            std::fs::read_to_string(root.join(FICHIER)).expect("fichier"),
            "ceci n'est pas du toml [[[",
            "le fichier de l'auteur survit intact"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// `definir` remplace en place, `supprimer` retire — la clé est le
    /// repère.
    #[test]
    fn definir_remplace_et_supprimer_retire() {
        let mut glossaire = Glossaire::default();
        glossaire.definir(EntreeGlossaire::nouvelle("a"));
        glossaire.definir(EntreeGlossaire::nouvelle("b"));
        glossaire.definir(EntreeGlossaire {
            key: "a".into(),
            short: Some("A".into()),
            ..Default::default()
        });

        assert_eq!(glossaire.entrees.len(), 2, "pas de doublon");
        assert_eq!(glossaire.entrees[0].key, "a", "remplacé en place");
        assert_eq!(
            glossaire.entree("a").and_then(|e| e.short.as_deref()),
            Some("A")
        );

        assert!(glossaire.supprimer("a"), "elle y était");
        assert!(!glossaire.supprimer("a"), "plus maintenant");
        assert_eq!(glossaire.entrees.len(), 1);
    }

    /// Le registre Typst est lisible, échappé, et muet sur les champs
    /// vides.
    #[test]
    fn le_registre_typst_ecrit_le_strict_necessaire() {
        let mut glossaire = Glossaire::default();
        glossaire.definir(EntreeGlossaire {
            key: "cite".into(),
            short: Some("Cité \"idéale\"".into()),
            long: None,
            description: Some("100% \\ conjecture".into()),
            ..Default::default()
        });

        let registre = glossaire.registre_typst();
        assert!(registre.contains("key: \"cite\""), "{registre}");
        assert!(
            registre.contains("short: \"Cité \\\"idéale\\\"\""),
            "{registre}"
        );
        assert!(
            registre.contains("description: \"100% \\\\ conjecture\""),
            "{registre}"
        );
        assert!(!registre.contains("long:"), "le champ vide ne part pas");
        assert!(!registre.contains("group:"), "le champ absent non plus");
    }

    /// Une clé de glossaire est un identifiant — la grammaire des balises
    /// en dépend.
    #[test]
    fn une_cle_est_un_identifiant() {
        assert!(EntreeGlossaire::cle_valide("sdo"));
        assert!(EntreeGlossaire::cle_valide("fig-croix.2"));
        assert!(!EntreeGlossaire::cle_valide(""), "jamais vide");
        assert!(!EntreeGlossaire::cle_valide("deux mots"), "jamais d'espace");
        assert!(
            !EntreeGlossaire::cle_valide("fig:croix"),
            "le : est aux balises"
        );
        assert!(!EntreeGlossaire::cle_valide("clé"), "sans diacritique");
    }
}
