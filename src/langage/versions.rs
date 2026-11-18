//! Les versions bibliques ajoutées par l'auteur.
//!
//! L'application embarque deux traductions ; ce module ouvre la porte aux
//! autres. Une version est un fichier JSON **local**, déposé dans
//! `<données>/bibles/`, qui décrit ses livres et ses chapitres. Aucun texte
//! n'est téléchargé ni livré avec le programme : l'auteur n'installe que ce
//! qu'il a le droit d'utiliser.
//!
//! Le format est volontairement explicite et sans héritage implicite :
//!
//! ```json
//! {
//!   "id": "vulgate",
//!   "nom": "Vulgate",
//!   "langue": "la",
//!   "livres": [
//!     {
//!       "osis": "Gen",
//!       "nom": "Genesis",
//!       "abreviation": "Gn",
//!       "chapitres": [["In principio…", "…"]]
//!     }
//!   ]
//! }
//! ```
//!
//! `chapitres` se lit dans l'ordre : le premier tableau est le chapitre 1,
//! son premier élément le verset 1. Une version peut donc avoir le nombre
//! de livres qu'elle veut, dans l'ordre qu'elle veut ; son `id` et les codes
//! `osis` de ses livres sont uniques. Ces codes sont ceux qui paraissent
//! dans les citations `{{Jn 3.16|vulgate}}` et dans l'index.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// Le fichier d'une version, tel qu'il est écrit par son auteur.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VersionBiblique {
    /// Le code court utilisé dans les citations, par exemple `vulgate`.
    pub id: String,
    /// Le nom complet affiché dans les listes.
    pub nom: String,
    /// La langue, facultative : `fr`, `la`, `grc`…
    #[serde(default)]
    pub langue: String,
    /// Une note libre sur la provenance ou les droits.
    #[serde(default)]
    pub source: String,
    /// Les livres de **cette** version, dans l'ordre où elle les présente.
    pub livres: Vec<LivreVersion>,
}

/// Un livre d'une version ajoutée.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivreVersion {
    /// Le code OSIS du livre (`Gen`, `Tob`, `1En`…).
    pub osis: String,
    /// Le nom lisible.
    pub nom: String,
    /// L'abréviation lisible.
    pub abreviation: String,
    /// Chapitres puis versets, sans numérotation implicite autre que l'ordre.
    #[serde(default)]
    pub chapitres: Vec<Vec<String>>,
}

impl VersionBiblique {
    /// Lit et vérifie une version depuis ses octets.
    pub fn depuis_octets(octets: &[u8]) -> Result<Self, String> {
        let version: VersionBiblique =
            serde_json::from_slice(octets).map_err(|erreur| format!("JSON invalide : {erreur}"))?;
        version.valider()?;
        Ok(version)
    }

    /// Vérifie les invariants qui empêchent une ambiguïté.
    pub fn valider(&self) -> Result<(), String> {
        if !code_sur(&self.id) {
            return Err(
                "L'identifiant doit commencer par une lettre ou un chiffre, puis ne contenir \
                 que des minuscules, chiffres, tirets, points ou soulignés."
                    .to_string(),
            );
        }
        if matches!(
            self.id.as_str(),
            "lsg" | "darby" | "segond" | "louis_segond_1910" | "jnd"
        ) {
            return Err("Cet identifiant est réservé aux traductions embarquées.".to_string());
        }
        if self.nom.trim().is_empty() {
            return Err("Le nom de la version est vide.".to_string());
        }
        if self.livres.is_empty() {
            return Err("Une version doit contenir au moins un livre.".to_string());
        }
        let mut codes = std::collections::HashSet::new();
        for livre in &self.livres {
            if !osis_sur(&livre.osis) {
                return Err(format!(
                    "Le code OSIS « {} » n'est pas un code sûr (lettres, chiffres, `-`, `_`, `.`).",
                    livre.osis
                ));
            }
            if !codes.insert(livre.osis.clone()) {
                return Err(format!("Le livre « {} » apparaît deux fois.", livre.osis));
            }
            if livre.nom.trim().is_empty() || livre.abreviation.trim().is_empty() {
                return Err(format!(
                    "Le livre « {} » n'a pas de nom ou d'abréviation.",
                    livre.osis
                ));
            }
        }
        Ok(())
    }

    /// Le dossier où vivent les versions ajoutées.
    pub fn dossier() -> PathBuf {
        crate::services::stockage::dossier_donnees().join("bibles")
    }
}

/// Une version vérifiée, avec sa table de chapitres déjà construite.
#[derive(Clone, Debug)]
struct VersionChargee {
    version: VersionBiblique,
    chapitres: HashMap<(String, u32), Vec<(u32, String)>>,
}

impl VersionChargee {
    fn nouvelle(version: VersionBiblique) -> Self {
        let mut chapitres = HashMap::new();
        for livre in &version.livres {
            for (index, versets) in livre.chapitres.iter().enumerate() {
                let numero = u32::try_from(index + 1).unwrap_or(u32::MAX);
                let versets = versets
                    .iter()
                    .enumerate()
                    .filter(|(_, texte)| !texte.trim().is_empty())
                    .map(|(rang, texte)| {
                        (u32::try_from(rang + 1).unwrap_or(u32::MAX), texte.clone())
                    })
                    .collect();
                chapitres.insert((livre.osis.clone(), numero), versets);
            }
        }
        Self { version, chapitres }
    }
}

static REGISTRE: OnceLock<RwLock<Vec<VersionChargee>>> = OnceLock::new();

fn registre() -> &'static RwLock<Vec<VersionChargee>> {
    REGISTRE.get_or_init(|| RwLock::new(charger_depuis_le_disque()))
}

/// Relit le dossier des versions. À appeler après une installation.
pub fn recharger() {
    let versions = charger_depuis_le_disque();
    if let Ok(mut registre) = registre().write() {
        *registre = versions;
    }
}

fn charger_depuis_le_disque() -> Vec<VersionChargee> {
    let dossier = VersionBiblique::dossier();
    let Ok(entrees) = std::fs::read_dir(&dossier) else {
        return Vec::new();
    };
    let mut versions = Vec::new();
    let mut identifiants = std::collections::HashSet::new();
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(octets) = std::fs::read(&chemin) else {
            continue;
        };
        if let Ok(version) = VersionBiblique::depuis_octets(&octets) {
            // Deux fichiers pour le même identifiant rendraient le choix
            // ambigu : le premier fichier lisible gagne, les autres sont
            // ignorés jusqu'à ce que l'auteur nettoie le dossier.
            if identifiants.insert(version.id.clone()) {
                versions.push(VersionChargee::nouvelle(version));
            }
        }
    }
    versions.sort_by(|a, b| a.version.id.cmp(&b.version.id));
    versions
}

/// Le nombre de versions ajoutées actuellement lues.
pub fn nombre() -> usize {
    registre().read().map(|r| r.len()).unwrap_or(0)
}

/// L'index d'une version à partir de son code de citation.
pub fn index_de_code(code: &str) -> Option<u16> {
    let code = code.trim().to_lowercase();
    registre().read().ok().and_then(|r| {
        r.iter()
            .position(|v| v.version.id == code)
            .and_then(|index| u16::try_from(index).ok())
    })
}

/// Les métadonnées d'une version par son index stable dans le registre.
///
/// Seuls les champs d'affichage sont copiés : le texte des chapitres reste
/// dans le registre et n'est jamais cloné pour un simple libellé.
pub fn nom(index: u16) -> Option<String> {
    registre()
        .read()
        .ok()
        .and_then(|r| r.get(index as usize).map(|v| v.version.nom.clone()))
}

/// Le code de citation d'une version ajoutée.
pub fn code(index: u16) -> Option<String> {
    registre()
        .read()
        .ok()
        .and_then(|r| r.get(index as usize).map(|v| v.version.id.clone()))
}

/// Le nom lisible d'un livre, sans son texte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivreResume {
    pub osis: String,
    pub nom: String,
    pub abreviation: String,
}

/// Résout un nom ou une abréviation de livre dans les versions ajoutées.
///
/// La réponse n'est rendue que si elle est **unique** : deux versions qui
/// nomment différemment le même code ne doivent pas faire choisir au hasard.
/// Les livres embarqués sont résolus avant d'arriver ici.
pub fn code_pour_nom(nom: &str) -> Option<String> {
    let cherche = normaliser(nom);
    let mut codes = std::collections::HashSet::new();
    if let Ok(registre) = registre().read() {
        for version in registre.iter() {
            for livre in &version.version.livres {
                if normaliser(&livre.osis) == cherche
                    || normaliser(&livre.nom) == cherche
                    || normaliser(&livre.abreviation) == cherche
                {
                    codes.insert(livre.osis.clone());
                }
            }
        }
    }
    if codes.len() == 1 {
        codes.into_iter().next()
    } else {
        None
    }
}

/// Le premier livre d'une version ajoutée qui porte ce code OSIS.
pub fn livre_pour_osis(osis: &str) -> Option<LivreResume> {
    registre().read().ok().and_then(|registre| {
        registre.iter().find_map(|version| {
            version
                .version
                .livres
                .iter()
                .find(|livre| livre.osis == osis)
                .map(|livre| LivreResume {
                    osis: livre.osis.clone(),
                    nom: livre.nom.clone(),
                    abreviation: livre.abreviation.clone(),
                })
        })
    })
}

/// La normalisation des noms de livres, la même que celle des références.
fn normaliser(texte: &str) -> String {
    texte
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Les livres d'une version, dans son ordre propre, sans leur texte.
pub fn livres(index: u16) -> Vec<LivreResume> {
    registre()
        .read()
        .ok()
        .and_then(|r| {
            r.get(index as usize).map(|v| {
                v.version
                    .livres
                    .iter()
                    .map(|livre| LivreResume {
                        osis: livre.osis.clone(),
                        nom: livre.nom.clone(),
                        abreviation: livre.abreviation.clone(),
                    })
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Les versets d'un chapitre, déjà numérotés à partir de 1.
pub fn chapitres(index: u16, livre: &str, chapitre: u32) -> Vec<(u32, String)> {
    registre()
        .read()
        .ok()
        .and_then(|r| {
            r.get(index as usize)
                .and_then(|v| v.chapitres.get(&(livre.to_string(), chapitre)).cloned())
        })
        .unwrap_or_default()
}

/// Installe une version depuis un fichier JSON.
///
/// Le fichier est vérifié **avant** d'être copié. Un identifiant déjà présent
/// est remplacé par la nouvelle version : l'identifiant reste unique, et une
/// mise à jour ne crée pas deux entrées ambiguës.
pub fn installer(chemin: &Path) -> Result<VersionBiblique, String> {
    let octets =
        std::fs::read(chemin).map_err(|erreur| format!("Lecture impossible : {erreur}"))?;
    let version = VersionBiblique::depuis_octets(&octets)?;

    let dossier = VersionBiblique::dossier();
    std::fs::create_dir_all(&dossier)
        .map_err(|erreur| format!("Dossier des bibles impossible : {erreur}"))?;
    let destination = dossier.join(format!("{}.json", version.id));

    let temporaire = dossier.join(format!(".{}.json.tmp", version.id));
    std::fs::write(&temporaire, &octets)
        .map_err(|erreur| format!("Écriture impossible : {erreur}"))?;
    std::fs::rename(&temporaire, &destination)
        .map_err(|erreur| format!("Installation impossible : {erreur}"))?;

    // Le succès précède le nettoyage : un ancien fichier au même identifiant
    // ne disparaît qu'une fois la nouvelle version posée. Les fichiers
    // illisibles ou étrangers restent intacts.
    if let Ok(entrees) = std::fs::read_dir(&dossier) {
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin == destination || chemin.extension().and_then(|e| e.to_str()) != Some("json")
            {
                continue;
            }
            let Ok(octets) = std::fs::read(&chemin) else {
                continue;
            };
            if VersionBiblique::depuis_octets(&octets)
                .is_ok_and(|ancienne| ancienne.id == version.id)
            {
                let _ = std::fs::remove_file(&chemin);
            }
        }
    }

    recharger();
    Ok(version)
}

/// Un code sûr pour un identifiant ou un code OSIS.
fn code_sur(code: &str) -> bool {
    let mut caracteres = code.chars();
    let Some(premier) = caracteres.next() else {
        return false;
    };
    (premier.is_ascii_lowercase() || premier.is_ascii_digit())
        && caracteres
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

/// Un code OSIS sûr : il garde sa casse, mais reste un seul mot.
fn osis_sur(code: &str) -> bool {
    let mut caracteres = code.chars();
    let Some(premier) = caracteres.next() else {
        return false;
    };
    (premier.is_ascii_alphanumeric())
        && caracteres.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod epreuves {
    use super::*;

    fn version(id: &str) -> VersionBiblique {
        VersionBiblique {
            id: id.to_string(),
            nom: id.to_string(),
            langue: "fr".to_string(),
            source: "domaine public".to_string(),
            livres: vec![LivreVersion {
                osis: "Tob".to_string(),
                nom: "Tobie".to_string(),
                abreviation: "Tb".to_string(),
                chapitres: vec![vec!["Premier verset.".to_string(), "Second.".to_string()]],
            }],
        }
    }

    #[test]
    fn une_version_se_lit_et_se_valide() {
        let octets = serde_json::to_vec(&version("vulgate")).unwrap();
        let lue = VersionBiblique::depuis_octets(&octets).expect("version lisible");
        assert_eq!(lue.id, "vulgate");
        assert_eq!(lue.livres[0].chapitres[0].len(), 2);

        let chargee = VersionChargee::nouvelle(lue);
        assert_eq!(
            chargee
                .chapitres
                .get(&("Tob".to_string(), 1))
                .map(|v| v.len()),
            Some(2)
        );
    }

    #[test]
    fn un_doublon_de_livre_est_refuse() {
        let mut v = version("vulgate");
        v.livres.push(v.livres[0].clone());
        assert!(v.valider().is_err(), "deux fois Tobie est ambigu");
    }

    #[test]
    fn un_identifiant_embarque_est_reserve() {
        for id in ["lsg", "darby", "segond", "jnd"] {
            let v = version(id);
            assert!(v.valider().is_err(), "{id} est réservé");
        }
    }

    #[test]
    fn un_nombre_de_livres_quelconque_est_permis() {
        let mut v = version("courte");
        v.livres = vec![LivreVersion {
            osis: "Matt".to_string(),
            nom: "Matthieu".to_string(),
            abreviation: "Mt".to_string(),
            chapitres: vec![vec!["Seul verset.".to_string()]],
        }];
        assert!(
            v.valider().is_ok(),
            "une version d'un seul livre est valide"
        );
    }

    #[test]
    fn le_dossier_est_sous_les_donnees_de_l_application() {
        let dossier = VersionBiblique::dossier();
        assert!(dossier.ends_with("bibles"), "{dossier:?}");
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn une_version_installee_se_lit_et_se_cite() {
        assert_version_installee();
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn installer_retire_un_ancien_fichier_du_meme_identifiant() {
        let v = version("vulgate");
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();
        let ancien = dossier.join("ancienne-copie.json");
        std::fs::write(&ancien, serde_json::to_vec(&v).unwrap()).unwrap();

        let chemin = std::env::temp_dir().join(format!(
            "version-biblique-doublon-{}.json",
            std::process::id()
        ));
        std::fs::write(&chemin, serde_json::to_vec(&v).unwrap()).unwrap();
        installer(&chemin).expect("installation");
        let _ = std::fs::remove_file(&chemin);

        assert!(!ancien.exists(), "le doublon est retiré");
        assert!(
            dossier.join("vulgate.json").exists(),
            "la version canonique est à sa place"
        );
    }

    fn assert_version_installee() {
        let mut version = version("vulgate");
        version.livres = vec![
            LivreVersion {
                osis: "Gen".to_string(),
                nom: "Genesis".to_string(),
                abreviation: "Gn".to_string(),
                chapitres: vec![vec!["In principio.".to_string()]],
            },
            LivreVersion {
                osis: "Tob".to_string(),
                nom: "Tobie".to_string(),
                abreviation: "Tb".to_string(),
                chapitres: vec![vec!["Premier.".to_string(), "Second.".to_string()]],
            },
            LivreVersion {
                osis: "1Clem".to_string(),
                nom: "Première épître de Clément".to_string(),
                abreviation: "1 Cl".to_string(),
                chapitres: vec![vec!["Église de Dieu.".to_string()]],
            },
        ];
        let chemin = std::env::temp_dir().join(format!(
            "version-biblique-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&chemin, serde_json::to_vec(&version).unwrap()).unwrap();

        let installee = installer(&chemin).expect("installation");
        assert_eq!(installee.livres.len(), 3);
        let _ = std::fs::remove_file(&chemin);

        let traduction =
            crate::langage::references::Traduction::depuis_code("vulgate").expect("la version");
        let livres = crate::plugins::bible::livres(traduction);
        assert_eq!(livres.len(), 3);
        assert!(livres.iter().any(|livre| livre.osis == "Tob"));

        let reference = crate::plugins::bible::Reference::verset("Tob", 1, 1);
        let versets = crate::plugins::bible::versets(&reference, traduction);
        assert_eq!(versets, vec![(1, "Premier.".to_string())]);
        let extrait = crate::plugins::bible::extrait(&[reference], traduction);
        assert!(extrait.contains("Premier."), "{extrait}");
        assert!(extrait.contains("vulgate"), "{extrait}");

        assert_eq!(
            crate::langage::references::code_du_livre("Première épître de Clément").as_deref(),
            Some("1Clem")
        );
        assert_eq!(
            crate::langage::references::nom_du_livre(
                "1Clem",
                crate::langage::references::FormeLivre::FrancaiseLongue
            ),
            "Première épître de Clément"
        );
        let reference =
            crate::langage::references::depuis_texte("Première épître de Clément 1.1|vulgate")
                .expect("un livre propre à la version se cite");
        assert_eq!(reference.livre, "1Clem");
        assert_eq!(
            reference.traduction,
            Some(crate::langage::references::Traduction::Externe(0))
        );

        let nom = traduction.nom();
        assert_eq!(nom, "vulgate");
        assert_eq!(traduction.abreviation(), "VULGATE");

        let mut normes = crate::mise_en_page::modele::MiseEnPage::default();
        normes.parametres_plugins.biblique.traduction = "vulgate".to_string();
        assert!(
            normes.problemes().is_empty(),
            "une version installée est une traduction valide"
        );
        let plugins = crate::plugins::Plugins::depuis_les_normes(&normes);
        assert_eq!(plugins.bible.traduction, traduction);
    }
}
