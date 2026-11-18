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

static REGISTRE: OnceLock<RwLock<Vec<Option<VersionChargee>>>> = OnceLock::new();

fn registre() -> &'static RwLock<Vec<Option<VersionChargee>>> {
    REGISTRE.get_or_init(|| RwLock::new(charger_depuis_le_disque().into_iter().map(Some).collect()))
}

/// Relit le dossier des versions. À appeler après une installation.
///
/// Les indices existants ne bougent pas : une version retirée laisse un trou,
/// une nouvelle version s'ajoute à la fin. Une référence `Traduction::Externe`
/// déjà ouverte dans la session reste donc attachée à la même version.
pub fn recharger() {
    let nouvelles = charger_depuis_le_disque();
    if let Ok(mut registre) = registre().write() {
        // Les emplacements connus gardent leur index ; une version disparue
        // laisse un trou, une nouvelle est ajoutée à la fin. On déplace les
        // versions au lieu de les cloner : leur texte peut peser plusieurs
        // méga-octets, et le registre le stocke déjà deux fois.
        let mut restantes: Vec<Option<VersionChargee>> = nouvelles.into_iter().map(Some).collect();
        let mut fusion: Vec<Option<VersionChargee>> = Vec::with_capacity(registre.len());
        for ancienne in registre.iter() {
            if let Some(ancienne) = ancienne.as_ref() {
                let position = restantes.iter().position(|nouvelle| {
                    nouvelle
                        .as_ref()
                        .is_some_and(|nouvelle| nouvelle.version.id == ancienne.version.id)
                });
                match position {
                    Some(position) => fusion.push(restantes[position].take()),
                    None => fusion.push(None),
                }
            } else {
                fusion.push(None);
            }
        }
        for nouvelle in restantes.into_iter().flatten() {
            fusion.push(Some(nouvelle));
        }
        *registre = fusion;
    }
}

fn charger_depuis_le_disque() -> Vec<VersionChargee> {
    let Ok(versions) = fichiers_de_versions() else {
        return Vec::new();
    };
    let mut chargees = Vec::new();
    let mut identifiants = std::collections::HashSet::new();
    for (_, version) in versions {
        // Deux fichiers pour le même identifiant rendraient le choix
        // ambigu : le premier fichier lisible gagne, les autres sont
        // ignorés jusqu'à ce que l'auteur nettoie le dossier.
        if identifiants.insert(version.id.clone()) {
            chargees.push(VersionChargee::nouvelle(version));
        }
    }
    chargees.sort_by(|a, b| a.version.id.cmp(&b.version.id));
    chargees
}

/// Tous les JSON lisibles du dossier, avec leur chemin.
fn fichiers_de_versions() -> std::io::Result<Vec<(PathBuf, VersionBiblique)>> {
    let mut versions = Vec::new();
    for entree in std::fs::read_dir(VersionBiblique::dossier())?.flatten() {
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(octets) = std::fs::read(&chemin) else {
            continue;
        };
        if let Ok(version) = VersionBiblique::depuis_octets(&octets) {
            versions.push((chemin, version));
        }
    }
    Ok(versions)
}

/// Les indices occupés du registre, dans leur ordre stable.
pub fn indices() -> Vec<u16> {
    registre()
        .read()
        .map(|r| {
            r.iter()
                .enumerate()
                .filter(|(_, slot)| slot.is_some())
                .filter_map(|(index, _)| u16::try_from(index).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// L'index d'une version à partir de son code de citation.
pub fn index_de_code(code: &str) -> Option<u16> {
    let code = code.trim().to_lowercase();
    registre().read().ok().and_then(|r| {
        r.iter()
            .enumerate()
            .find(|(_, slot)| {
                slot.as_ref()
                    .is_some_and(|version| version.version.id == code)
            })
            .and_then(|(index, _)| u16::try_from(index).ok())
    })
}

/// Les métadonnées d'une version par son index stable dans le registre.
///
/// Seuls les champs d'affichage sont copiés : le texte des chapitres reste
/// dans le registre et n'est jamais cloné pour un simple libellé.
pub fn nom(index: u16) -> Option<String> {
    avec_version(index, |version| version.version.nom.clone())
}

/// Le code de citation d'une version ajoutée.
pub fn code(index: u16) -> Option<String> {
    avec_version(index, |version| version.version.id.clone())
}

/// Le nom lisible d'un livre, sans son texte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivreResume {
    pub osis: String,
    pub nom: String,
    pub abreviation: String,
}

impl LivreResume {
    fn depuis(livre: &LivreVersion) -> Self {
        Self {
            osis: livre.osis.clone(),
            nom: livre.nom.clone(),
            abreviation: livre.abreviation.clone(),
        }
    }
}

/// Ce qu'un panneau a besoin de savoir d'une version, en une seule lecture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionResume {
    /// Le nombre de livres qu'elle contient.
    pub livres: usize,
    /// Le nombre total de chapitres.
    pub chapitres: usize,
    /// La provenance déclarée, éventuellement vide.
    pub source: String,
}

/// Lit une version du registre par son indice stable.
fn avec_version<T>(index: u16, lire: impl FnOnce(&VersionChargee) -> T) -> Option<T> {
    let registre = registre().read().ok()?;
    let version = registre.get(index as usize)?.as_ref()?;
    Some(lire(version))
}

/// Résout un nom ou une abréviation de livre dans les versions ajoutées.
///
/// La réponse n'est rendue que si elle est **unique** : deux versions qui
/// nomment différemment le même code ne doivent pas faire choisir au hasard.
/// Les livres embarqués sont résolus avant d'arriver ici.
pub fn code_pour_nom(nom: &str) -> Option<String> {
    use crate::langage::references::normaliser_nom;

    let cherche = normaliser_nom(nom);
    let mut codes = std::collections::HashSet::new();
    if let Ok(registre) = registre().read() {
        for version in registre.iter().flatten() {
            for livre in &version.version.livres {
                if normaliser_nom(&livre.osis) == cherche
                    || normaliser_nom(&livre.nom) == cherche
                    || normaliser_nom(&livre.abreviation) == cherche
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

/// Un modèle JSON minimal, à remplir par l'auteur.
///
/// Il est volontairement valide : les champs obligatoires sont présents, et
/// le texte d'exemple rappelle qu'il faut le remplacer par un texte dont on
/// détient les droits.
pub fn modele() -> String {
    let version = VersionBiblique {
        id: "ma-version".to_string(),
        nom: "Ma version".to_string(),
        langue: "fr".to_string(),
        source: "Texte dont je détiens les droits".to_string(),
        livres: vec![LivreVersion {
            osis: "Gen".to_string(),
            nom: "Genèse".to_string(),
            abreviation: "Gn".to_string(),
            chapitres: vec![vec![
                "Premier verset — remplacez ce texte.".to_string(),
                "Deuxième verset.".to_string(),
            ]],
        }],
    };
    serde_json::to_string_pretty(&version).unwrap_or_default()
}

/// Le chemin du fichier actif d'une version, s'il existe.
pub fn chemin_de(code: &str) -> Option<PathBuf> {
    let code = code.trim().to_lowercase();
    let direct = VersionBiblique::dossier().join(format!("{code}.json"));
    if std::fs::read(&direct)
        .ok()
        .and_then(|octets| VersionBiblique::depuis_octets(&octets).ok())
        .is_some_and(|version| version.id == code)
    {
        return Some(direct);
    }
    fichiers_de_versions()
        .ok()?
        .into_iter()
        .find(|(_, version)| version.id == code)
        .map(|(chemin, _)| chemin)
}

/// Copie le fichier d'une version vers une destination choisie.
///
/// C'est le geste de transport explicite : l'archive de projet ne contient
/// pas les versions bibliques, et l'application ne les copie jamais sans
/// que l'auteur l'ait demandé.
pub fn exporter(code: &str, cible: &Path) -> Result<(), String> {
    let Some(chemin) = chemin_de(code) else {
        return Err(format!("La version « {code} » n'est pas installée."));
    };
    if chemin == cible {
        return Ok(());
    }
    std::fs::copy(&chemin, cible)
        .map(|_| ())
        .map_err(|erreur| format!("Copie impossible : {erreur}"))
}

/// Les chiffres et la provenance d'une version, en une seule lecture du
/// registre : le panneau les affiche ensemble, il n'a pas à prendre trois
/// verrous pour une seule ligne.
pub fn resume(index: u16) -> Option<VersionResume> {
    avec_version(index, |version| VersionResume {
        livres: version.version.livres.len(),
        chapitres: version.chapitres.len(),
        source: version.version.source.clone(),
    })
}

/// Le livre d'une version ajoutée, par son code OSIS.
///
/// La version est désignée par son indice stable : deux versions peuvent
/// nommer le même code OSIS différemment, et c'est la version citée qui
/// décide de la forme affichée.
pub fn livre_pour_osis_dans(index: u16, osis: &str) -> Option<LivreResume> {
    avec_version(index, |version| {
        version
            .version
            .livres
            .iter()
            .find(|livre| livre.osis == osis)
            .map(LivreResume::depuis)
    })
    .flatten()
}

/// Le livre d'une version ajoutée, par son code OSIS.
pub fn livre_pour_osis(osis: &str) -> Option<LivreResume> {
    registre().read().ok().and_then(|registre| {
        registre.iter().flatten().find_map(|version| {
            version
                .version
                .livres
                .iter()
                .find(|livre| livre.osis == osis)
                .map(LivreResume::depuis)
        })
    })
}

/// Les livres d'une version, dans son ordre propre, sans leur texte.
pub fn livres(index: u16) -> Vec<LivreResume> {
    avec_version(index, |version| {
        version
            .version
            .livres
            .iter()
            .map(LivreResume::depuis)
            .collect()
    })
    .unwrap_or_default()
}

/// Les versets d'un chapitre, déjà numérotés à partir de 1.
pub fn chapitres(index: u16, livre: &str, chapitre: u32) -> Vec<(u32, String)> {
    avec_version(index, |version| {
        version
            .chapitres
            .get(&(livre.to_string(), chapitre))
            .cloned()
    })
    .flatten()
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
    installer_octets(&octets)
}

/// Installe une version déjà lue en mémoire — le chemin des archives
/// portables, qui n'ont pas de fichier source sur le disque.
pub fn installer_octets(octets: &[u8]) -> Result<VersionBiblique, String> {
    let version = VersionBiblique::depuis_octets(octets)?;
    installer_une(&version, octets)?;
    recharger();
    Ok(version)
}

/// Installe plusieurs versions déjà vérifiées, puis ne recharge le registre
/// qu'une fois. Les archives portables en joignent plusieurs : les relire et
/// les réinstaller une par une coûtait N lectures complètes du dossier.
pub fn installer_lot(versions: &[(VersionBiblique, Vec<u8>)]) -> Result<(), String> {
    for (version, octets) in versions {
        installer_une(version, octets)?;
    }
    recharger();
    Ok(())
}

/// Pose une version déjà validée sur le disque, sans toucher au registre.
fn installer_une(version: &VersionBiblique, octets: &[u8]) -> Result<(), String> {
    let dossier = VersionBiblique::dossier();
    std::fs::create_dir_all(&dossier)
        .map_err(|erreur| format!("Dossier des bibles impossible : {erreur}"))?;
    let destination = dossier.join(format!("{}.json", version.id));

    crate::services::stockage::ecrire_atomique_octets(&destination, octets)
        .map_err(|erreur| format!("Installation impossible : {erreur}"))?;

    // Le succès précède le nettoyage : un ancien fichier au même identifiant
    // ne disparaît qu'une fois la nouvelle version posée. Les fichiers
    // illisibles ou étrangers restent intacts.
    if let Ok(anciennes) = fichiers_de_versions() {
        for (chemin, ancienne) in anciennes {
            if chemin != destination && ancienne.id == version.id {
                let _ = std::fs::remove_file(&chemin);
            }
        }
    }
    Ok(())
}

/// Supprime une version ajoutée et tous ses fichiers homonymes.
///
/// Les fichiers partent à la corbeille : une suppression reste réversible.
/// Le retour dit combien de fichiers ont été déplacés.
pub fn supprimer(code: &str) -> Result<usize, String> {
    let code = code.trim().to_lowercase();
    if code.is_empty() {
        return Err("Aucune version à supprimer.".to_string());
    }
    let Ok(versions) = fichiers_de_versions() else {
        return Err("Le dossier des versions est introuvable.".to_string());
    };
    let mut supprimes = 0;
    for (chemin, version) in versions {
        if version.id == code {
            crate::services::stockage::vers_la_corbeille(&chemin)
                .map_err(|erreur| format!("Suppression impossible : {erreur}"))?;
            supprimes += 1;
        }
    }
    if supprimes == 0 {
        return Err(format!("La version « {code} » n'est pas installée."));
    }
    recharger();
    Ok(supprimes)
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
    fn le_modele_json_est_installable() {
        let octets = modele();
        let version = VersionBiblique::depuis_octets(octets.as_bytes()).expect("modèle valide");
        assert_eq!(version.id, "ma-version");
        assert!(!version.livres.is_empty());
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

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn les_indices_des_versions_ne_bougent_pas_apres_un_ajout() {
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();

        let premiere = std::env::temp_dir().join(format!(
            "version-biblique-indice-{}-a.json",
            std::process::id()
        ));
        std::fs::write(&premiere, serde_json::to_vec(&version("vulgate")).unwrap()).unwrap();
        installer(&premiere).expect("première installation");
        let _ = std::fs::remove_file(&premiere);
        let index_vulgate =
            crate::langage::references::Traduction::depuis_code("vulgate").expect("vulgate");
        assert_eq!(
            index_vulgate,
            crate::langage::references::Traduction::Externe(0)
        );

        let seconde = std::env::temp_dir().join(format!(
            "version-biblique-indice-{}-b.json",
            std::process::id()
        ));
        std::fs::write(&seconde, serde_json::to_vec(&version("aaa")).unwrap()).unwrap();
        installer(&seconde).expect("seconde installation");
        let _ = std::fs::remove_file(&seconde);

        assert_eq!(
            crate::langage::references::Traduction::depuis_code("vulgate"),
            Some(crate::langage::references::Traduction::Externe(0)),
            "l'index de la première version reste stable"
        );
        assert_eq!(
            crate::langage::references::Traduction::depuis_code("aaa"),
            Some(crate::langage::references::Traduction::Externe(1))
        );
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn une_version_se_supprime_vers_la_corbeille() {
        let mut v = version("vulgate");
        v.id = "supprimee".to_string();
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();
        let fichier = std::env::temp_dir().join(format!(
            "version-biblique-suppression-{}.json",
            std::process::id()
        ));
        std::fs::write(&fichier, serde_json::to_vec(&v).unwrap()).unwrap();
        installer(&fichier).expect("installation");
        let _ = std::fs::remove_file(&fichier);

        assert_eq!(supprimer("supprimee").expect("suppression"), 1);
        assert!(
            crate::langage::references::Traduction::depuis_code("supprimee").is_none(),
            "le registre ne la propose plus"
        );
        assert!(
            !dossier.join("supprimee.json").exists(),
            "le fichier a quitté le dossier actif"
        );
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn le_nom_du_livre_suit_la_version_citee() {
        let mut v = version("bereshit");
        v.livres[0].osis = "Gen".to_string();
        v.livres[0].nom = "Bereshit".to_string();
        v.livres[0].abreviation = "Ber".to_string();
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();
        let fichier =
            std::env::temp_dir().join(format!("version-biblique-nom-{}.json", std::process::id()));
        std::fs::write(&fichier, serde_json::to_vec(&v).unwrap()).unwrap();
        installer(&fichier).expect("installation");
        let _ = std::fs::remove_file(&fichier);

        let traduction =
            crate::langage::references::Traduction::depuis_code("bereshit").expect("version");
        assert_eq!(
            crate::langage::references::nom_du_livre_dans(
                "Gen",
                crate::langage::references::FormeLivre::FrancaiseLongue,
                Some(traduction),
            ),
            "Bereshit"
        );
        assert_eq!(
            crate::langage::references::nom_du_livre_dans(
                "Gen",
                crate::langage::references::FormeLivre::FrancaiseLongue,
                None,
            ),
            "Genèse",
            "sans version explicite, la table générale reste souveraine"
        );

        let mut reference = crate::langage::references::Reference::verset("Gen", 1, 1);
        reference.traduction = Some(traduction);
        let normes = crate::mise_en_page::modele::References {
            livre: crate::mise_en_page::modele::SystemeAbreviation::FrancaiseLongue,
            ..Default::default()
        };
        assert!(
            crate::langage::references::afficher(&reference, &normes).contains("Bereshit"),
            "la référence affiche le nom de sa version"
        );
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn une_version_s_exporte_pour_etre_transportee() {
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();
        let source = std::env::temp_dir().join(format!(
            "version-biblique-export-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &source,
            serde_json::to_vec(&version("transportee")).unwrap(),
        )
        .unwrap();
        installer(&source).expect("installation");
        let _ = std::fs::remove_file(&source);

        let cible = std::env::temp_dir().join(format!(
            "version-biblique-exportee-{}.json",
            std::process::id()
        ));
        exporter("transportee", &cible).expect("export");
        let octets = std::fs::read(&cible).expect("fichier exporté");
        let relue = VersionBiblique::depuis_octets(&octets).expect("JSON valide");
        assert_eq!(relue.id, "transportee");
        let _ = std::fs::remove_file(&cible);
    }

    #[test]
    #[ignore = "HOME jetable exigé : écrit une version dans les données"]
    fn une_version_biblique_voyage_dans_une_archive() {
        let dossier = VersionBiblique::dossier();
        std::fs::create_dir_all(&dossier).unwrap();
        let source = std::env::temp_dir().join(format!(
            "version-biblique-voyage-{}.json",
            std::process::id()
        ));
        std::fs::write(&source, serde_json::to_vec(&version("voyage")).unwrap()).unwrap();
        installer(&source).expect("installation");
        let _ = std::fs::remove_file(&source);

        let projet =
            std::env::temp_dir().join(format!("projet-version-voyage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&projet);
        std::fs::create_dir_all(&projet).unwrap();
        std::fs::write(projet.join("document.tw"), "# Titre\n").unwrap();
        let archive =
            std::env::temp_dir().join(format!("archive-version-voyage-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&archive);

        crate::project::portable::exporter_projet_avec_versions(
            &projet,
            &archive,
            &["voyage".to_string()],
        )
        .expect("export avec version");
        let manifeste = crate::project::portable::lire_manifeste(&archive).expect("manifeste");
        assert_eq!(manifeste.inventaire.bibles.len(), 1);
        assert_eq!(manifeste.inventaire.bibles[0].code, "voyage");

        supprimer("voyage").expect("retrait avant restauration");
        assert!(
            crate::langage::references::Traduction::depuis_code("voyage").is_none(),
            "la version a bien quitté le registre"
        );
        assert_eq!(
            crate::project::portable::restaurer_versions(&archive).expect("restauration"),
            1
        );
        assert!(
            crate::langage::references::Traduction::depuis_code("voyage").is_some(),
            "la version jointe est revenue"
        );

        let _ = std::fs::remove_file(&archive);
        let _ = std::fs::remove_dir_all(&projet);
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
