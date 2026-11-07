//! Les quick parts d'un document : ce que l'auteur décide d'y mettre.
//!
//! C'est la demande 9 de `bug-report-7.md` — revisité le 2026-11-04 :
//! la liste est **celle de l'auteur**, dans **son ordre**, sans champ
//! imposé. `titre` et `date` n'ont rien à faire là : le titre vient du
//! projet et la date de la balise `{date:}` ; ils voyagent en plomberie
//! pour la page de titre mais ne s'affichent pas parmi les quick parts.
//! Elles vivent dans `metadonnees.toml`, à côté du projet — jamais dans
//! les normes, qui se réécrivent tout entières à chaque application.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Le fichier, à la racine du projet.
pub const FICHIER: &str = "metadonnees.toml";

/// Les noms de balises que la langue s'est réservés : une métadonnée ne
/// les porte pas, sous peine de confondre sa balise `{clé}` avec une
/// balise système (bug-report-10, 4.5). Ce sont les tiges que le parseur
/// reconnaît — `{date:}`, `{glo:}`, `{glossaire}`, `{graphique`,
/// `{bibliographie}`, la couleur, `[[TOC]]` — plus `titre` et `date`,
/// déjà tenus ailleurs.
const CLES_RESERVEES: [&str; 8] = [
    "date",
    "glo",
    "glossaire",
    "graphique",
    "bibliographie",
    "couleur",
    "toc",
    "titre",
];

/// Une clé de métadonnée est-elle réservée par la langue ?
pub fn cle_reservee(cle: &str) -> bool {
    CLES_RESERVEES.contains(&cle.trim().to_lowercase().as_str())
}

/// Les clés qui ne s'affichent pas : tenues ailleurs.
const CLEES_CACHEES: [&str; 2] = ["titre", "date"];

/// Une paire clé → valeur, dans l'ordre où l'auteur l'a posée.
type Paire = (String, String);

/// Un profil de métadonnées : ses valeurs, sous un nom (bug-report-10,
/// 4.1) — le même mot y porte une valeur différente (4.2).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profil {
    /// Le nom du profil.
    pub nom: String,

    /// Ses paires, dans l'ordre de l'auteur.
    #[serde(with = "en_ordre")]
    pub paires: Vec<Paire>,
}

/// Les métadonnées : les quick parts de l'auteur, **jamais triées** — son
/// ordre est le sien.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Metadonnees {
    /// Les paires, dans l'ordre de saisie de l'auteur — celles du profil
    /// actif : les balises `{clé}` les lisent (bug-report-10, 4.3).
    #[serde(with = "en_ordre")]
    pub paires: Vec<Paire>,

    /// Les profils nommés.
    pub profils: Vec<Profil>,

    /// Le profil actif — vide : aucune, les paires seules.
    pub profil_actif: String,

    /// Ce projet est un **modèle** : en créer un nouveau, c'est le
    /// prendre pour base (demande de l'auteur). Un modèle reste un
    /// projet comme un autre — ouvrable, modifiable, effaçable.
    pub est_modele: bool,
}

/// Le format de fichier reste une table TOML (`clé = "valeur"`) — les
/// projets existants se lisent — mais l'ordre du fichier est gardé, et
/// l'alphabet n'a rien à dire.
mod en_ordre {
    use super::Paire;
    use serde::de::{MapAccess, Visitor};
    use serde::ser::SerializeMap;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S: Serializer>(paires: &[Paire], serialiseur: S) -> Result<S::Ok, S::Error> {
        let mut map = serialiseur.serialize_map(Some(paires.len()))?;
        for (cle, valeur) in paires {
            map.serialize_entry(cle, valeur)?;
        }
        map.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserialiseur: D,
    ) -> Result<Vec<Paire>, D::Error> {
        struct Lecteur;

        impl<'de> Visitor<'de> for Lecteur {
            type Value = Vec<Paire>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("une table de quick parts")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut paires = Vec::new();
                while let Some((cle, valeur)) = map.next_entry::<String, String>()? {
                    paires.push((cle, valeur));
                }
                Ok(paires)
            }
        }

        deserialiseur.deserialize_map(Lecteur)
    }
}

impl Metadonnees {
    /// Un document neuf : aucune quick part imposée — l'auteur pose les
    /// siennes.
    pub fn base() -> Self {
        Self::default()
    }

    /// Lit les métadonnées du projet. Un projet qui n'en a pas encore
    /// reçoit une liste vide.
    pub fn charger(projet: &Path) -> Self {
        match std::fs::read_to_string(projet.join(FICHIER)) {
            Ok(texte) => toml::from_str(&texte).unwrap_or_default(),
            Err(_) => Self::base(),
        }
    }

    /// Écrit les métadonnées dans le projet.
    pub fn sauver(&self, projet: &Path) -> Result<(), String> {
        let texte = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(projet.join(FICHIER), texte).map_err(|e| e.to_string())
    }

    /// Les quick parts visibles : l'ordre de l'auteur, sans les clés
    /// tenues ailleurs (`titre`, `date`).
    pub fn visibles(&self) -> impl Iterator<Item = &Paire> {
        self.paires
            .iter()
            .filter(|(cle, _)| !CLEES_CACHEES.contains(&cle.as_str()))
    }

    /// La valeur d'une clé, visible ou cachée.
    #[allow(dead_code)] // éprouvée par les épreuves, tenue pour les appelants.
    pub fn valeur(&self, cle: &str) -> Option<&str> {
        self.paires
            .iter()
            .find(|(c, _)| c == cle)
            .map(|(_, v)| v.as_str())
    }

    /// Pose une valeur sous une clé — ou renomme la clé `ancienne`, en
    /// gardant la place de la paire dans la liste.
    pub fn definir(&mut self, ancienne: &str, nouvelle: String, valeur: String) {
        if let Some(paire) = self.paires.iter_mut().find(|(cle, _)| cle == ancienne) {
            *paire = (nouvelle, valeur);
            return;
        }
        self.paires.push((nouvelle, valeur));
    }

    /// Ajoute une ligne neuve, sans jamais écraser un nom existant.
    pub fn ajouter(&mut self) {
        let mut rang = self.paires.len() + 1;
        while self
            .paires
            .iter()
            .any(|(cle, _)| cle == &format!("champ{rang}"))
        {
            rang += 1;
        }
        self.paires.push((format!("champ{rang}"), String::new()));
    }

    /// Retire une ligne.
    pub fn supprimer(&mut self, cle: &str) {
        self.paires.retain(|(c, _)| c != cle);
    }

    /// Les paires pour `sys.inputs` : les quick parts y appellent
    /// `{clé}`… — les vides sont omises, une quick part sans valeur n'a
    /// rien à dire.
    /// Active un profil : les valeurs courantes se rangent sous l'ancien
    /// nom, celles du profil choisi deviennent les valeurs.
    pub fn activer_profil(&mut self, nom: &str) {
        self.ranger_les_valeurs();

        self.paires = self
            .profils
            .iter()
            .find(|profil| profil.nom == nom)
            .map(|profil| profil.paires.clone())
            .unwrap_or_default();
        self.profil_actif = nom.to_string();
    }

    /// Enregistre les valeurs courantes sous ce nom — le profil devient
    /// l'actif.
    pub fn enregistrer_profil(&mut self, nom: &str) {
        // Les valeurs courantes vont sous CE nom — jamais sous l'ancien
        // profil, qui garde les siennes.
        if let Some(profil) = self.profils.iter_mut().find(|profil| profil.nom == nom) {
            profil.paires = self.paires.clone();
        } else {
            self.profils.push(Profil {
                nom: nom.to_string(),
                paires: self.paires.clone(),
            });
        }

        self.profil_actif = nom.to_string();
    }

    /// Supprime un profil — les valeurs courantes ne sont pas atteintes.
    pub fn supprimer_profil(&mut self, nom: &str) {
        self.profils.retain(|profil| profil.nom != nom);

        if self.profil_actif == nom {
            self.profil_actif.clear();
        }
    }

    /// Les valeurs courantes vont sous le profil actif.
    fn ranger_les_valeurs(&mut self) {
        if self.profil_actif.is_empty() {
            return;
        }

        if let Some(profil) = self
            .profils
            .iter_mut()
            .find(|profil| profil.nom == self.profil_actif)
        {
            profil.paires = self.paires.clone();
        }
    }

    pub fn entrees(&self) -> Vec<Paire> {
        self.paires
            .iter()
            .filter(|(_, valeur)| !valeur.trim().is_empty())
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Metadonnees;

    /// Le va-et-vient : ce qui s'écrit se relit, les quick parts
    /// appellent les mêmes clés, et **l'ordre de l'auteur est gardé**.
    #[test]
    fn les_metadonnées_vivent_et_alimentent_les_quick_parts() {
        let racine = std::env::temp_dir().join(format!("tw-meta-{}", std::process::id()));
        std::fs::create_dir_all(&racine).expect("dossier");

        let mut meta = Metadonnees::base();
        meta.definir("", "zeta".to_string(), "Z".to_string());
        meta.definir("", "alpha".to_string(), "A".to_string());
        meta.definir("", "titre".to_string(), "Ma thèse".to_string());
        meta.sauver(&racine).expect("enregistrement");

        let relue = Metadonnees::charger(&racine);
        assert_eq!(relue, meta, "ce qui s'écrit se relit");
        assert_eq!(
            relue.paires,
            vec![
                ("zeta".to_string(), "Z".to_string()),
                ("alpha".to_string(), "A".to_string()),
                ("titre".to_string(), "Ma thèse".to_string()),
            ],
            "l'ordre de l'auteur tient — jamais l'alphabet"
        );
        let entrees = relue.entrees();
        assert!(
            entrees.contains(&("titre".to_string(), "Ma thèse".to_string())),
            "la quick part titre a sa valeur"
        );
        assert!(
            !entrees.iter().any(|(cle, _)| cle == "auteur"),
            "une valeur vide n'alimente rien"
        );

        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }

    /// `titre` et `date` voyagent pour la page de titre mais ne se
    /// montrent pas parmi les quick parts.
    #[test]
    fn titre_et_date_ne_se_montrent_pas() {
        let mut meta = Metadonnees::base();
        meta.definir("", "titre".to_string(), "T".to_string());
        meta.definir("", "date".to_string(), "D".to_string());
        meta.definir("", "directeur".to_string(), "M. X".to_string());

        let visibles: Vec<_> = meta.visibles().map(|(c, _)| c.as_str()).collect();
        assert_eq!(
            visibles,
            vec!["directeur"],
            "seule la quick part de l'auteur"
        );
    }
}

/// Remplace les balises `{clé}` du texte par les valeurs des
/// métadonnées (bug-report-10, 4.4) — une balise inconnue garde ses
/// accolades : elle est du texte, pas un trou.
pub fn remplacer_les_balises(texte: &str, metadonnees: &Metadonnees) -> String {
    let mut out = String::new();
    let mut reste = texte;

    while let Some(debut) = reste.find('{') {
        let apres = &reste[debut + 1..];
        let Some(fin) = apres.find('}') else {
            break;
        };

        let cle = &apres[..fin];

        if cle.is_empty() || cle.contains(['{', ' ', '\n', '\t']) {
            out.push_str(&reste[..debut + 1]);
            reste = apres;
            continue;
        }

        let valeur = metadonnees
            .paires
            .iter()
            .find(|(c, _)| c == cle)
            .map(|(_, valeur)| valeur.trim());

        match valeur {
            Some(valeur) if !valeur.is_empty() => {
                out.push_str(&reste[..debut]);
                out.push_str(valeur);
            }
            _ => {
                out.push_str(&reste[..debut + fin + 2]);
            }
        }

        reste = &apres[fin + 1..];
    }

    out.push_str(reste);
    out
}

#[cfg(test)]
mod tests_des_profils {
    use super::*;

    #[test]
    fn les_profils_portent_leurs_valeurs() {
        let mut metadonnees = Metadonnees {
            paires: vec![("auteur".to_string(), "Jean".to_string())],
            ..Default::default()
        };

        // Un profil se nomme, puis un second porte l'autre valeur.
        metadonnees.enregistrer_profil("mémoire");
        metadonnees.paires = vec![("auteur".to_string(), "Jacques".to_string())];
        metadonnees.enregistrer_profil("thèse");

        // Le profil actif donne ses valeurs aux balises.
        assert_eq!(
            metadonnees.entrees(),
            vec![("auteur".to_string(), "Jacques".to_string())],
            "les balises lisent le profil actif"
        );

        // Revenir au premier : ses valeurs reviennent.
        metadonnees.activer_profil("mémoire");
        assert_eq!(
            metadonnees.entrees(),
            vec![("auteur".to_string(), "Jean".to_string())],
            "le même mot porte une valeur selon le profil"
        );
        assert_eq!(metadonnees.profil_actif, "mémoire", "le profil actif suit");
    }

    #[test]
    fn les_cles_systeme_sont_reservees() {
        assert!(cle_reservee("date"), "date est au système");
        assert!(cle_reservee("Glo"), "glo est au système, casse comprise");
        assert!(
            cle_reservee("bibliographie"),
            "la bibliographie est au système"
        );
        assert!(!cle_reservee("directeur"), "une clé de l'auteur passe");
        assert!(!cle_reservee("matiere"), "une clé de l'auteur passe");
    }

    #[test]
    fn les_balises_se_remplacent_par_leurs_valeurs() {
        let metadonnees = Metadonnees {
            paires: vec![
                ("auteur".to_string(), "Jean".to_string()),
                ("vide".to_string(), String::new()),
            ],
            ..Default::default()
        };

        let texte = "Par {auteur}, et {inconnue} — {vide} non plus.";
        let rempli = remplacer_les_balises(texte, &metadonnees);

        assert_eq!(
            rempli, "Par Jean, et {inconnue} — {vide} non plus.",
            "la valeur entre, l'inconnue garde ses accolades"
        );
    }
}
