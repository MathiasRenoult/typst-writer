//! L'index des personnes d'un projet : le registre des noms cités.
//!
//! Les entrées vivent dans `personnes.toml`, à la racine du projet. Elles
//! sont **explicites** : une personne n'entre dans l'index que si le texte
//! la nomme par `{{p clé}}` ou `{{personne clé}}`. Aucun nom n'est deviné
//! dans la prose : l'index ne doit jamais inventer une mention.
//!
//! Le fichier appartient à l'auteur. Un fichier absent est un registre vide ;
//! un fichier abîmé est un registre vide **et une trace**, jamais un
//! écrasement silencieux.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mise_en_page::modele::{FormeDatesPersonne, FormePersonne};

/// Le fichier, à la racine du projet.
pub const FICHIER: &str = "personnes.toml";

/// Une personne citée dans le document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EntreePersonne {
    /// La clé de référence : `{{p clé}}` l'appelle. Identifiant stable.
    pub cle: String,

    /// Le nom complet : « Augustin d'Hippone ».
    pub nom: String,

    /// Le nom simple : « Augustin ». Vide : le nom complet sert de repli.
    pub nom_simple: Option<String>,

    /// L'alias ou le nom de plume : « Saint Augustin ». Vide : le nom
    /// simple sert de repli.
    pub alias: Option<String>,

    /// La clé de classement dans l'index. Vide : le nom sert de clé.
    ///
    /// Elle permet de classer « Thomas d'Aquin » à « Aquin » sans imposer
    /// une forme d'écriture au nom affiché.
    pub tri: Option<String>,

    /// Les dates, telles que l'auteur les écrit : « 354-430 ». Champ libre
    /// historique, conservé comme repli quand naissance et mort manquent.
    pub dates: Option<String>,

    /// L'année de naissance : « 354 ».
    pub naissance: Option<String>,

    /// L'année de mort : « 430 ».
    pub mort: Option<String>,

    /// La qualité ou la fonction : « évêque d'Hippone ».
    pub qualite: Option<String>,

    /// Les formes cherchables dans l'interface. Elles ne servent **jamais**
    /// à reconnaître automatiquement une mention dans le texte.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variantes: Vec<String>,
}

impl EntreePersonne {
    /// Une entrée neuve, réduite à sa clé.
    #[allow(dead_code)] // API de contrat : éprouvée par les tests du modèle.
    pub fn nouvelle(cle: impl Into<String>) -> Self {
        Self {
            cle: cle.into(),
            ..Default::default()
        }
    }

    /// Une clé de personne est un identifiant : lettres ASCII, chiffres,
    /// tiret, souligné et point. Elle ne doit pas contenir d'espace ni de
    /// délimiteur de balise.
    pub fn cle_valide(cle: &str) -> bool {
        !cle.is_empty()
            && cle
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }

    /// Le nom affiché, ou la clé quand le registre est incomplet.
    pub fn nom_affiche(&self) -> &str {
        if self.nom.trim().is_empty() {
            &self.cle
        } else {
            &self.nom
        }
    }

    /// Le nom simple, ou le nom complet quand il n'est pas renseigné.
    pub fn nom_simple_affiche(&self) -> &str {
        self.nom_simple
            .as_deref()
            .map(str::trim)
            .filter(|nom| !nom.is_empty())
            .unwrap_or_else(|| self.nom_affiche())
    }

    /// L'alias, ou le nom simple, ou le nom complet — jamais du vide.
    pub fn alias_affiche(&self) -> &str {
        self.alias
            .as_deref()
            .map(str::trim)
            .filter(|alias| !alias.is_empty())
            .unwrap_or_else(|| self.nom_simple_affiche())
    }

    /// Le nom dans la forme demandée par les normes.
    pub fn nom_pour(&self, forme: FormePersonne) -> &str {
        match forme {
            FormePersonne::Complet => self.nom_affiche(),
            FormePersonne::Simple => self.nom_simple_affiche(),
            FormePersonne::Alias => self.alias_affiche(),
        }
    }

    /// La clé de classement : `tri` si elle est renseignée, sinon le nom.
    pub fn cle_de_tri(&self) -> &str {
        self.tri
            .as_deref()
            .map(str::trim)
            .filter(|tri| !tri.is_empty())
            .unwrap_or_else(|| self.nom_affiche())
    }

    /// La qualité, si elle est renseignée et non vide.
    pub fn qualite_affichee(&self) -> Option<&str> {
        self.qualite
            .as_deref()
            .map(str::trim)
            .filter(|qualite| !qualite.is_empty())
    }

    /// La naissance, si elle est renseignée et non vide.
    pub fn naissance_affichee(&self) -> Option<&str> {
        self.naissance
            .as_deref()
            .map(str::trim)
            .filter(|date| !date.is_empty())
    }

    /// La mort, si elle est renseignée et non vide.
    pub fn mort_affichee(&self) -> Option<&str> {
        self.mort
            .as_deref()
            .map(str::trim)
            .filter(|date| !date.is_empty())
    }

    /// Les dates libres, si elles sont renseignées et non vides.
    pub fn dates_affichees(&self) -> Option<&str> {
        self.dates
            .as_deref()
            .map(str::trim)
            .filter(|dates| !dates.is_empty())
    }

    /// Les dates composées selon la forme demandée.
    ///
    /// Naissance et mort restent des champs séparés dans le registre : la
    /// norme décide seulement comment les réunir. Un ancien `dates` libre
    /// reste le repli des deux bornes, pour ne perdre aucun document.
    pub fn dates_pour(&self, forme: FormeDatesPersonne) -> Option<String> {
        let naissance = self
            .naissance
            .as_deref()
            .map(str::trim)
            .filter(|date| !date.is_empty());
        let mort = self
            .mort
            .as_deref()
            .map(str::trim)
            .filter(|date| !date.is_empty());
        let libre = self.dates_affichees();

        match forme {
            FormeDatesPersonne::NaissanceMort => match (naissance, mort) {
                (Some(naissance), Some(mort)) => Some(format!("{naissance}-{mort}")),
                (Some(naissance), None) => Some(naissance.to_string()),
                (None, Some(mort)) => Some(mort.to_string()),
                (None, None) => libre.map(str::to_string),
            },
            FormeDatesPersonne::Naissance => naissance
                .map(str::to_string)
                .or_else(|| libre.map(str::to_string)),
            FormeDatesPersonne::Mort => mort
                .map(str::to_string)
                .or_else(|| libre.map(str::to_string)),
            FormeDatesPersonne::Libelle => libre.map(str::to_string),
        }
    }
}

/// Le registre des personnes d'un projet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Personnes {
    /// Les entrées, dans l'ordre de l'auteur.
    pub entrees: Vec<EntreePersonne>,
}

impl Personnes {
    /// Le fichier du projet.
    pub fn chemin(root: &Path) -> PathBuf {
        root.join(FICHIER)
    }

    /// Le registre du projet. Un fichier absent est un registre vide ; un
    /// fichier illisible aussi, mais [`Self::sauver`] refuse alors de
    /// l'écraser.
    pub fn charger(root: &Path) -> Self {
        Self::lire(&Self::chemin(root))
    }

    /// La lecture d'un fichier.
    pub fn lire(chemin: &Path) -> Self {
        let Ok(texte) = std::fs::read_to_string(chemin) else {
            return Self::default();
        };
        toml::from_str(&texte).unwrap_or_default()
    }

    /// L'écriture atomique, qui refuse de remplacer un fichier illisible.
    pub fn sauver(&self, root: &Path) -> std::io::Result<()> {
        let chemin = Self::chemin(root);

        match std::fs::read_to_string(&chemin) {
            Ok(texte) => {
                toml::from_str::<Personnes>(&texte).map_err(|erreur| {
                    std::io::Error::other(format!(
                        "registre des personnes illisible, sauvegarde refusée : {erreur}"
                    ))
                })?;
            }
            Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {}
            Err(erreur) => {
                return Err(std::io::Error::other(format!(
                    "registre des personnes illisible, sauvegarde refusée : {erreur}"
                )));
            }
        }

        let texte = toml::to_string_pretty(self).map_err(|erreur| {
            std::io::Error::other(format!("registre non sérialisable : {erreur}"))
        })?;
        crate::services::stockage::ecrire_atomique(&chemin, &texte)
    }

    /// L'entrée d'une clé.
    pub fn entree(&self, cle: &str) -> Option<&EntreePersonne> {
        self.entrees.iter().find(|entree| entree.cle == cle)
    }

    /// Ajoute une entrée, ou remplace celle qui porte déjà la clé, en place.
    pub fn definir(&mut self, entree: EntreePersonne) {
        if let Some(ancienne) = self.entrees.iter_mut().find(|e| e.cle == entree.cle) {
            *ancienne = entree;
        } else {
            self.entrees.push(entree);
        }
    }

    /// Retire une entrée et dit si elle y était.
    pub fn supprimer(&mut self, cle: &str) -> bool {
        let avant = self.entrees.len();
        self.entrees.retain(|entree| entree.cle != cle);
        self.entrees.len() != avant
    }

    /// Le registre trié pour l'index : les entrées sans doublon, classées
    /// selon leur clé de tri, puis selon leur clé stable.
    #[allow(dead_code)] // API de contrat : l'index a sa propre collecte.
    pub fn pour_index(&self) -> Vec<&EntreePersonne> {
        let mut entrees: Vec<&EntreePersonne> = self.entrees.iter().collect();
        entrees.sort_by(|a, b| {
            a.cle_de_tri()
                .to_lowercase()
                .cmp(&b.cle_de_tri().to_lowercase())
                .then_with(|| a.cle.cmp(&b.cle))
        });
        entrees
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn racine(nom: &str) -> PathBuf {
        let chemin = std::env::temp_dir().join(format!(
            "typst-writer-personnes-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&chemin);
        std::fs::create_dir_all(&chemin).expect("racine du test");
        chemin
    }

    #[test]
    fn un_registre_se_persiste_dans_l_ordre() {
        let root = racine("persistance");
        let mut personnes = Personnes::default();
        personnes.definir(EntreePersonne {
            cle: "augustin".into(),
            nom: "Augustin d'Hippone".into(),
            nom_simple: Some("Augustin".into()),
            alias: Some("Saint Augustin".into()),
            tri: Some("Augustin".into()),
            dates: Some("354-430".into()),
            naissance: Some("354".into()),
            mort: Some("430".into()),
            qualite: Some("évêque d'Hippone".into()),
            variantes: vec!["saint Augustin".into()],
        });
        personnes.definir(EntreePersonne::nouvelle("aquin"));

        personnes.sauver(&root).expect("sauvegarde");
        let relu = Personnes::charger(&root);

        assert_eq!(relu, personnes, "tout revient, dans l'ordre");
        assert_eq!(relu.entrees[0].cle, "augustin");
        assert_eq!(
            relu.entree("augustin")
                .and_then(EntreePersonne::qualite_affichee),
            Some("évêque d'Hippone")
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn un_ancien_registre_se_lit_sans_les_nouveaux_champs() {
        let root = racine("ancien");
        std::fs::write(
            root.join(FICHIER),
            "[[entrees]]\ncle = \"augustin\"\nnom = \"Augustin d'Hippone\"\ndates = \"354-430\"\n",
        )
        .expect("ancien registre");

        let personnes = Personnes::charger(&root);
        let entree = personnes.entree("augustin").expect("entrée");
        assert!(entree.nom_simple.is_none());
        assert!(entree.alias.is_none());
        assert!(entree.naissance.is_none());
        assert!(entree.mort.is_none());
        assert_eq!(
            entree.dates_pour(FormeDatesPersonne::NaissanceMort),
            Some("354-430".to_string())
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn un_fichier_abime_ne_craque_ni_ne_s_ecrase() {
        let root = racine("abime");
        std::fs::write(root.join(FICHIER), "ceci n'est pas du toml [[[").expect("fichier fautif");

        let personnes = Personnes::charger(&root);
        assert!(personnes.entrees.is_empty());
        assert!(personnes.sauver(&root).is_err());
        assert_eq!(
            std::fs::read_to_string(root.join(FICHIER)).expect("fichier"),
            "ceci n'est pas du toml [[["
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn la_cle_de_tri_est_celle_du_registre_puis_le_nom() {
        let mut personnes = Personnes::default();
        personnes.definir(EntreePersonne {
            cle: "aquin".into(),
            nom: "Thomas d'Aquin".into(),
            tri: Some("Aquin".into()),
            ..Default::default()
        });
        personnes.definir(EntreePersonne {
            cle: "augustin".into(),
            nom: "Augustin d'Hippone".into(),
            ..Default::default()
        });

        let triees: Vec<_> = personnes
            .pour_index()
            .iter()
            .map(|e| e.cle.as_str())
            .collect();
        assert_eq!(triees, vec!["aquin", "augustin"]);
    }

    #[test]
    fn les_formes_du_nom_et_les_dates_se_composent() {
        let entree = EntreePersonne {
            cle: "augustin".into(),
            nom: "Augustin d'Hippone".into(),
            nom_simple: Some("Augustin".into()),
            alias: Some("Saint Augustin".into()),
            naissance: Some("354".into()),
            mort: Some("430".into()),
            ..Default::default()
        };

        assert_eq!(
            entree.nom_pour(FormePersonne::Complet),
            "Augustin d'Hippone"
        );
        assert_eq!(entree.nom_pour(FormePersonne::Simple), "Augustin");
        assert_eq!(entree.nom_pour(FormePersonne::Alias), "Saint Augustin");
        assert_eq!(
            entree.dates_pour(FormeDatesPersonne::NaissanceMort),
            Some("354-430".to_string())
        );
        assert_eq!(
            entree.dates_pour(FormeDatesPersonne::Naissance),
            Some("354".to_string())
        );
        assert_eq!(
            entree.dates_pour(FormeDatesPersonne::Mort),
            Some("430".to_string())
        );

        let sans_alias = EntreePersonne {
            alias: None,
            ..entree
        };
        assert_eq!(sans_alias.nom_pour(FormePersonne::Alias), "Augustin");

        let ancien = EntreePersonne {
            cle: "ancien".into(),
            nom: "Ancien".into(),
            dates: Some("XIIe siècle".into()),
            ..Default::default()
        };
        assert_eq!(
            ancien.dates_pour(FormeDatesPersonne::NaissanceMort),
            Some("XIIe siècle".to_string()),
            "le champ libre reste le repli des anciens registres"
        );
    }

    #[test]
    fn une_cle_est_un_identifiant() {
        assert!(EntreePersonne::cle_valide("augustin"));
        assert!(EntreePersonne::cle_valide("thomas-aquin"));
        assert!(!EntreePersonne::cle_valide(""));
        assert!(!EntreePersonne::cle_valide("saint augustin"));
        assert!(!EntreePersonne::cle_valide("a|b"));
    }
}
