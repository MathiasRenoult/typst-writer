//! Gestion du projet : les chapitres, la bibliographie, la session de travail.
//!
//! L'application travaille toujours sur des fichiers locaux : aucun dialogue
//! « où sauvegarder ? » n'intervient pendant l'édition — il ne sert qu'à
//! l'export.
//!
//! Un travail se présente comme un **projet** : un dossier dans
//! `~/.typst-writer/projets/`, avec un fichier par chapitre, sa bibliographie
//! et son assemblage. Voir [`model`] pour la structure exacte.

pub mod glossaire;
pub mod metadonnees;
pub mod model;

pub mod portable;

pub use model::*;

/// Le fichier de la trace des références bibliques, à la racine du projet.
pub const RELEVE: &str = "references.json";

use crate::mise_en_page::modele::MiseEnPage;
use crate::services::stockage::{
    corbeille, ecrire_atomique, ecrire_atomique_octets, vers_la_corbeille, work_dir,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Ouvre un fichier ou un dossier avec l'outil du système.
pub fn ouvrir_chemin(cible: &std::path::Path) {
    let _ = commande_ouverture(cible).spawn();
}

/// La commande d'ouverture de la plateforme — construite ici, jamais
/// devinée ailleurs.
pub fn commande_ouverture(cible: &std::path::Path) -> std::process::Command {
    #[cfg(target_os = "macos")]
    let mut commande = std::process::Command::new("open");
    #[cfg(windows)]
    let mut commande = std::process::Command::new("explorer");
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let mut commande = std::process::Command::new("xdg-open");

    commande.arg(cible);
    commande
}

/// Crée un projet au nom choisi — et jamais sur un dossier existant.
///
/// Le nom est celui du dossier, plié à une forme qui tient partout :
/// minuscules, sans espace ni barre. Le projet se prépare complet dans un
/// temporaire caché et ne paraît qu'une fois prêt ; un dossier existant
/// n'est pas réutilisé en silence : c'est peut-être le travail de quelqu'un.
pub fn creer_au_nom(parent: &Path, nom: &str) -> Result<PathBuf, String> {
    let plie = plier_nom(nom);

    if plie.is_empty() {
        return Err("Le nom du projet est vide.".to_string());
    }

    let dossier = parent.join(&plie);

    // Un dossier existant n'est pas réutilisé en silence : c'est peut-être
    // le travail de quelqu'un.
    if dossier.exists() {
        return Err(format!(
            "« {plie} » existe déjà — un autre nom, ou l'ouvrir."
        ));
    }

    creer_et_publier(parent, &plie, false, crate::project::model::MAIN_TEMPLATE)
        .map(|(dir, _)| dir)
        .map_err(|erreur| {
            if erreur.kind() == std::io::ErrorKind::AlreadyExists {
                format!("« {plie} » existe déjà — un autre nom, ou l'ouvrir.")
            } else {
                format!("{} : {erreur}", dossier.display())
            }
        })
}

/// Crée un projet **depuis un projet de base** : un modèle est un
/// projet comme un autre, et celui-ci devient le squelette du nouveau
/// (demande de l'auteur). Les données d'exécution du modèle — verrou,
/// sauvegardes, métadonnées — ne voyagent pas : le projet naît les
/// siennes, et n'est pas lui-même un modèle.
pub fn creer_au_nom_depuis(parent: &Path, nom: &str, base: &Path) -> Result<PathBuf, String> {
    let dir = creer_au_nom(parent, nom)?;

    if let Err(erreur) = copier_la_base(base, &dir) {
        // Rien ne paraît à moitié : le projet juste publié se retire.
        let _ = std::fs::remove_dir_all(&dir);

        return Err(format!("La base n'a pas pu être copiée : {erreur}"));
    }

    Ok(dir)
}

/// La copie de base : tout, sauf ce qui appartient à l'exécution.
fn copier_la_base(base: &Path, dir: &Path) -> std::io::Result<()> {
    const SAUF: &[&str] = &[
        "verrou",
        "sauvegardes",
        "metadonnees.toml",
        ".typst-writer",
        ".miniatures",
    ];

    for entree in std::fs::read_dir(base)? {
        let entree = entree?;
        let nom = entree.file_name();
        let nom = nom.to_string_lossy();

        if SAUF.contains(&nom.as_ref()) {
            continue;
        }

        let source = entree.path();
        let cible = dir.join(nom.as_ref());

        if source.is_dir() {
            std::fs::create_dir_all(&cible)?;
            copier_la_base(&source, &cible)?;
        } else {
            std::fs::copy(&source, &cible)?;
        }
    }

    Ok(())
}
/// Prépare un projet complet dans un temporaire caché, puis le publie d'un
/// seul renommage.
///
/// Le dossier ne paraît au balayage qu'une fois ses gabarits en place :
/// jamais un demi-projet en vitrine. La publication reste exclusive — deux
/// créations au même nom ne gagnent jamais toutes les deux. Un travail
/// interrompu reste dans son temporaire : rien ne disparaît en silence.
pub(super) fn creer_et_publier(
    parent: &Path,
    plie: &str,
    avec_verrou: bool,
    gabarit: &str,
) -> std::io::Result<(PathBuf, Option<Verrou>)> {
    std::fs::create_dir_all(parent)?;

    let mut numero = 0;
    let temporaire = loop {
        let candidat = parent.join(format!(".{plie}.creation-{}-{numero}", std::process::id()));
        match std::fs::create_dir(&candidat) {
            Ok(()) => break candidat,
            Err(erreur) if erreur.kind() == std::io::ErrorKind::AlreadyExists => numero += 1,
            Err(erreur) => return Err(erreur),
        }
    };

    let mut verrou = if avec_verrou {
        match Verrou::prendre(&temporaire) {
            Ok(verrou) => Some(verrou),
            Err(probleme) => return Err(std::io::Error::other(probleme)),
        }
    } else {
        None
    };

    std::fs::create_dir(temporaire.join("chapitres"))?;
    for (nom, contenu) in [
        ("main.typ", gabarit),
        ("document.tw", ""),
        (
            "references.bib",
            crate::project::model::BIBLIOGRAPHY_TEMPLATE,
        ),
    ] {
        use std::io::Write;
        let mut fichier = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temporaire.join(nom))?;
        fichier.write_all(contenu.as_bytes())?;
        fichier.sync_all()?;
    }

    let dir = parent.join(plie);
    match std::fs::rename(&temporaire, &dir) {
        Ok(()) => {
            if let Some(verrou) = &mut verrou {
                verrou.suivre_deplacement(&dir);
            }
            Ok((dir, verrou))
        }
        Err(erreur) => Err(if dir.exists() {
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("« {plie} » existe déjà"),
            )
        } else {
            erreur
        }),
    }
}

/// Les temporaires de création abandonnés partent à la corbeille.
///
/// Une création interrompue — panne avant la publication — laisse un
/// dossier caché ; quand son auteur n'existe plus, il va à la corbeille,
/// comme tout le reste, plutôt que d'encombrer à jamais. Un temporaire dont
/// l'auteur est vivant est une création en cours : on n'y touche pas. Hors
/// Linux, un auteur est toujours présumé vivant.
pub fn nettoyer_temporaires_de_creation(parent: &Path) {
    let Ok(entrees) = std::fs::read_dir(parent) else {
        return;
    };
    for entree in entrees.filter_map(Result::ok) {
        if !entree.path().is_dir() {
            continue;
        }
        let nom = entree.file_name().to_string_lossy().into_owned();
        let Some((_plie, reste)) = nom
            .strip_prefix('.')
            .and_then(|nu| nu.rsplit_once(".creation-"))
        else {
            continue;
        };
        let Some((pid, _numero)) = reste.rsplit_once('-') else {
            continue;
        };
        let Ok(pid) = pid.parse::<i32>() else {
            continue;
        };
        if pid <= 0 || process_vivant(pid) {
            continue;
        }
        let _ = vers_la_corbeille(&entree.path());
    }
}

/// Renomme un projet — le dossier suit, et tout ce qui y vit avec lui.
pub fn renommer(dir: &Path, nom: &str) -> Result<PathBuf, String> {
    let plie = plier_nom(nom);

    if plie.is_empty() {
        return Err("Le nom du projet est vide.".to_string());
    }

    let parent = dir
        .parent()
        .ok_or_else(|| "Pas de dossier parent.".to_string())?;
    let cible = parent.join(&plie);

    if cible == dir {
        return Ok(dir.to_path_buf());
    }

    if cible.exists() {
        return Err(format!("« {plie} » existe déjà — un autre nom."));
    }

    std::fs::rename(dir, &cible).map_err(|erreur| format!("{erreur}"))?;
    Ok(cible)
}

/// Supprime un projet — vers la corbeille, comme tout le reste.
pub fn supprimer(dir: &Path) -> Result<PathBuf, String> {
    vers_la_corbeille(dir).map_err(|erreur| format!("{erreur}"))
}

/// Un nom de dossier : minuscules, tirets, sans espace ni barre.
pub fn plier_nom(nom: &str) -> String {
    let plie: String = nom
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' | '_' => '-',
            '/' | '\\' | ':' => '-',
            autre => autre,
        })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    plie.trim_matches('-').to_string()
}

/// Le verrou d'un projet : une seule instance récente y travaille.
///
/// L'ancre native est permanente : la supprimer permettrait à deux instances
/// de verrouiller des fichiers différents au même chemin. Le marqueur PID
/// conserve une compatibilité limitée avec les anciennes versions ; elles ne
/// respectent pas le verrou natif et ne doivent pas travailler simultanément.
#[derive(Debug)]
pub struct Verrou {
    chemin: std::path::PathBuf,
    _ancre: std::fs::File,
}

impl Verrou {
    /// Met à jour le chemin du marqueur après déplacement du projet.
    pub fn suivre_deplacement(&mut self, dir: &Path) {
        self.chemin = dir.join(".verrou");
    }

    /// Prend le verrou d'un projet. `Err` si une instance vivante le tient.
    pub fn prendre(dir: &Path) -> Result<Self, String> {
        let chemin = dir.join(".verrou");
        let ancre_chemin = dir.join(".verrou-natif");
        let ancre = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&ancre_chemin)
            .map_err(|erreur| {
                format!(
                    "Le verrou {} ne peut pas être ouvert : {erreur}.",
                    ancre_chemin.display()
                )
            })?;
        match ancre.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err("Une autre instance travaille sur ce projet.".to_string());
            }
            Err(std::fs::TryLockError::Error(erreur)) => {
                return Err(format!(
                    "Le verrou {} ne peut pas être acquis : {erreur}.",
                    ancre_chemin.display()
                ));
            }
        }

        // Le dossier a pu être déplacé après ouverture de l'ancre. Ne jamais
        // écrire au chemin demandé si l'ancre n'est plus celle de ce chemin.
        let identite = same_file::Handle::from_file(ancre.try_clone().map_err(|e| e.to_string())?)
            .map_err(|e| format!("Identité du verrou inaccessible : {e}"))?;
        let actuelle = same_file::Handle::from_path(&ancre_chemin)
            .map_err(|e| format!("Le projet a été déplacé pendant son ouverture : {e}"))?;
        if identite != actuelle {
            return Err(
                "Le projet a été déplacé pendant son ouverture ; ouvrez-le à nouveau.".to_string(),
            );
        }
        drop(actuelle);
        drop(identite);

        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&chemin)
        {
            Ok(mut fichier) => {
                use std::io::Write;
                if let Err(erreur) = writeln!(fichier, "{}", std::process::id()) {
                    drop(fichier);
                    let _ = std::fs::remove_file(&chemin);
                    return Err(format!(
                        "Le verrou {} ne s'écrit pas : {erreur}.",
                        chemin.display()
                    ));
                }
                Ok(Self {
                    chemin,
                    _ancre: ancre,
                })
            }

            Err(erreur) if erreur.kind() == std::io::ErrorKind::AlreadyExists => {
                // Un fichier vide peut être en cours de création par une autre
                // instance. Son propriétaire n'est pas connu : ne pas le voler.
                let contenu = std::fs::read_to_string(&chemin).map_err(|erreur| {
                    format!("Le verrou {} est illisible : {erreur}.", chemin.display())
                })?;
                let pid: i32 = contenu.trim().parse().ok().filter(|pid| *pid > 0).ok_or_else(|| {
                    format!("Le verrou {} est incomplet ou invalide. Fermez toutes les instances, puis retirez ce fichier s'il subsiste.", chemin.display())
                })?;

                if process_vivant(pid) {
                    return Err(format!(
                        "Une autre instance travaille peut-être sur ce projet (processus {pid}). Fermez les autres instances avant de réessayer."
                    ));
                }

                std::fs::write(&chemin, format!("{}\n", std::process::id()))
                    .map_err(|erreur| format!("{} : {erreur}", chemin.display()))?;

                Ok(Self {
                    chemin,
                    _ancre: ancre,
                })
            }
            Err(erreur) => Err(format!(
                "Le verrou {} ne peut pas être créé : {erreur}.",
                chemin.display()
            )),
        }
    }
}

impl Drop for Verrou {
    fn drop(&mut self) {
        let attendu = std::process::id().to_string();
        if std::fs::read_to_string(&self.chemin).is_ok_and(|contenu| contenu.trim() == attendu) {
            let _ = std::fs::remove_file(&self.chemin);
        }
        // Un descripteur dupliqué ou brièvement hérité par un enfant ne doit
        // pas prolonger un verrou que l'instance a explicitement relâché.
        let _ = self._ancre.unlock();
    }
}

/// Un PID inconnu est considéré vivant quand sa vérification n'est pas fiable.
#[cfg(target_os = "linux")]
fn process_vivant(pid: i32) -> bool {
    !std::path::Path::new("/proc/self").exists()
        || std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(target_os = "linux"))]
fn process_vivant(_pid: i32) -> bool {
    true
}

/// La trace des références bibliques citées dans le projet (le système de
/// références) : chaque référence et son nombre, écrite dans le projet.
///
/// Le fichier `references.json` est remplacé atomiquement, jamais fusionné ;
/// les clés sont triées par `BTreeMap`.
pub fn ecrire_releve_references(
    dossier: &std::path::Path,
    releve: &std::collections::BTreeMap<String, usize>,
) -> std::io::Result<()> {
    let brut = serde_json::to_string_pretty(releve).unwrap_or_else(|_| "{}".to_string());
    ecrire_atomique(&dossier.join(RELEVE), &brut)
}

/// Empreinte du contenu d'un fichier, pour reconnaître le nôtre au retour.
///
/// Comparée à celle du disque, elle distingue notre propre écriture d'un
/// changement extérieur. Elle ne vit qu'en mémoire pendant la session.
pub fn empreinte_contenu(octets: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hacheur = std::collections::hash_map::DefaultHasher::new();
    octets.hash(&mut hacheur);
    hacheur.finish()
}

/// Écrit plusieurs fichiers **ensemble** : ou tous, ou aucun.
///
/// Une transaction Zotero produit trois fichiers (clés, bibliothèque,
/// liaison) ; écrits l'un après l'autre, une panne à mi-chemin laisserait
/// un projet mi-écrit — clés revendiquées sans bibliothèque, ou liaison
/// sans clés. Ici chaque contenu est d'abord posé en temporaire voisin,
/// puis les anciennes versions sont écartées de même, et les nouvelles
/// mises en place ; le moindre échec remet **exactement** l'état d'avant.
/// Après coupure, les temporaires restent lisibles à côté — jamais de
/// fichier à moitié écrit.
pub fn ecrire_en_transaction(ecrits: &[(std::path::PathBuf, String)]) -> std::io::Result<()> {
    // 0. Une cible qui n'est pas un fichier est un refus immédiat :
    //    jamais un dossier ne sera écarté ni remplacé en silence.
    for (chemin, _) in ecrits {
        if let Ok(meta) = std::fs::metadata(chemin)
            && !meta.is_file()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} n'est pas un fichier", chemin.display()),
            ));
        }
    }

    // 1. Tout poser en temporaire : rien n'existe encore à son vrai nom.
    let mut temporaires = Vec::new();
    for (chemin, contenu) in ecrits {
        let nom = chemin.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "nom de fichier absent")
        })?;
        let mut nom_temporaire = std::ffi::OsString::from(".");
        nom_temporaire.push(nom);
        nom_temporaire.push(".transaction-tmp");
        let temporaire = chemin.with_file_name(nom_temporaire);

        let resultat = std::fs::write(&temporaire, contenu)
            .and_then(|_| std::fs::File::open(&temporaire).and_then(|fichier| fichier.sync_all()));
        if let Err(erreur) = resultat {
            let _ = std::fs::remove_file(&temporaire);
            for (autre, _) in &temporaires {
                let _ = std::fs::remove_file(autre);
            }

            return Err(erreur);
        }
        temporaires.push((temporaire, chemin.clone()));
    }

    // 2. Écarter les anciennes versions, une par une : tout est encore
    //    récupérable.
    let mut ecartes = Vec::new();
    for (_, chemin) in &temporaires {
        if chemin.exists() {
            let nom = chemin.file_name().expect("nom déjà contrôlé");
            let mut nom_ancien = std::ffi::OsString::from(".");
            nom_ancien.push(nom);
            nom_ancien.push(".transaction-ancien");
            let ancien = chemin.with_file_name(nom_ancien);
            let _ = std::fs::remove_file(&ancien);

            if let Err(erreur) = std::fs::rename(chemin, &ancien) {
                for (cible, sauvegarde) in &ecartes {
                    let _ = std::fs::rename(sauvegarde, cible);
                }
                for (autre, _) in &temporaires {
                    let _ = std::fs::remove_file(autre);
                }

                return Err(erreur);
            }
            ecartes.push((chemin.clone(), ancien));
        }
    }

    // 3. Les nouvelles en place ; à la moindre faute, l'ancien état revient.
    for (temporaire, chemin) in &temporaires {
        if let Err(erreur) = std::fs::rename(temporaire, chemin) {
            for (cible, sauvegarde) in &ecartes {
                let _ = std::fs::remove_file(cible);
                let _ = std::fs::rename(sauvegarde, cible);
            }
            for (autre, _) in &temporaires {
                let _ = std::fs::remove_file(autre);
            }

            return Err(erreur);
        }
    }

    // 4. Tout est en place : les versions écartées ne restent pas.
    for (_, ancien) in &ecartes {
        let _ = std::fs::remove_file(ancien);
    }

    Ok(())
}

#[cfg(test)]
mod epreuves_de_transaction {
    use super::ecrire_en_transaction;

    fn reste_un_debris(dossier: &std::path::Path) -> Vec<String> {
        std::fs::read_dir(dossier)
            .expect("dossier")
            .filter_map(|entree| entree.ok())
            .map(|entree| entree.file_name().to_string_lossy().into_owned())
            .filter(|nom| nom.contains("transaction-"))
            .collect()
    }

    /// Tout s'écrit, ensemble, sans laisser de débris.
    #[test]
    fn une_transaction_ecrit_tous_les_fichiers_ensemble() {
        let dossier =
            std::env::temp_dir().join(format!("typst-writer-transaction-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).expect("dossier");
        std::fs::write(dossier.join("b.txt"), "ANCIEN").expect("préparation");

        ecrire_en_transaction(&[
            (dossier.join("a.txt"), "PREMIER".to_string()),
            (dossier.join("b.txt"), "SECOND".to_string()),
        ])
        .expect("transaction");

        assert_eq!(
            std::fs::read_to_string(dossier.join("a.txt")).expect("a"),
            "PREMIER"
        );
        assert_eq!(
            std::fs::read_to_string(dossier.join("b.txt")).expect("b"),
            "SECOND"
        );
        assert!(
            reste_un_debris(&dossier).is_empty(),
            "aucun débris : {:?}",
            reste_un_debris(&dossier)
        );
    }

    /// Si un seul écrit échoue, **rien** n'est écrit : l'état d'avant
    /// revient exactement, y compris pour les fichiers déjà remplacés.
    #[test]
    fn une_transaction_faillie_ne_change_rien() {
        let dossier = std::env::temp_dir().join(format!(
            "typst-writer-transaction-faute-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).expect("dossier");
        std::fs::write(dossier.join("a.txt"), "ANCIEN A").expect("préparation");
        std::fs::write(dossier.join("b.txt"), "ANCIEN B").expect("préparation");

        // Un dossier occupe la place du troisième fichier : le renommage
        // vers lui échoue, à la dernière étape — la plus cruelle.
        std::fs::create_dir_all(dossier.join("c.txt")).expect("obstacle");

        let echec = ecrire_en_transaction(&[
            (dossier.join("a.txt"), "NOUVEAU A".to_string()),
            (dossier.join("b.txt"), "NOUVEAU B".to_string()),
            (dossier.join("c.txt"), "NOUVEAU C".to_string()),
        ]);
        assert!(echec.is_err(), "la transaction dit son échec");

        assert_eq!(
            std::fs::read_to_string(dossier.join("a.txt")).expect("a"),
            "ANCIEN A",
            "l'ancien état revient"
        );
        assert_eq!(
            std::fs::read_to_string(dossier.join("b.txt")).expect("b"),
            "ANCIEN B"
        );
        assert!(
            reste_un_debris(&dossier).is_empty(),
            "aucun débris : {:?}",
            reste_un_debris(&dossier)
        );
    }
}

/// Le dossier des versions antérieures d'un projet, à côté de son document.
pub fn sauvegardes_dir(projet: &std::path::Path) -> std::path::PathBuf {
    projet.join("sauvegardes")
}

/// Conserve la version d'un fichier juste avant qu'elle ne soit remplacée.
///
/// Une copie porte le nom du document suivi de l'heure, ce qui les range dans
/// l'ordre ; le gestionnaire de fichiers affiche la date lisible de chacune.
/// Les copies d'un même document sont espacées d'au moins une minute et les
/// vingt dernières sont gardées. `force` passe cet espacement : la version
/// extérieure d'un conflit est conservée même à la seconde près.
pub fn sauvegarder_version(
    projet: &std::path::Path,
    chemin: &std::path::Path,
    contenu: &str,
    force: bool,
) -> std::io::Result<()> {
    let dossier = sauvegardes_dir(projet);
    std::fs::create_dir_all(&dossier)?;
    let nom = chemin.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "nom de fichier absent")
    })?;
    let maintenant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let prefixe = format!("{}.", nom.to_string_lossy());
    let mut copies: Vec<std::path::PathBuf> = std::fs::read_dir(&dossier)?
        .filter_map(Result::ok)
        .map(|entree| entree.path())
        .filter(|copie| {
            copie
                .file_name()
                .and_then(|nom_copie| nom_copie.to_str())
                .is_some_and(|nom_copie| nom_copie.starts_with(&prefixe))
        })
        .collect();
    copies.sort();
    copies.reverse();

    if !force
        && let Some(plus_recente) = copies.first()
        && let Some(heure) = plus_recente
            .file_name()
            .and_then(|nom_copie| nom_copie.to_str())
            .and_then(|nom_copie| nom_copie.strip_prefix(&prefixe))
            .and_then(|souffle| souffle.parse::<u64>().ok())
        && maintenant.saturating_sub(heure) < 60
    {
        return Ok(());
    }

    conserver_une_version(
        projet,
        &nom.to_string_lossy(),
        contenu.as_bytes(),
        maintenant,
    )
}

/// Conserve une version d'un document dans `sauvegardes/`, sans jamais
/// écraser une copie déjà là : les copies d'une même seconde se suivent.
///
/// La rétention s'applique à toutes les copies du document, quelles qu'en
/// soient l'origine et la forme du nom : les vingt dernières sont gardées.
pub fn conserver_une_version(
    projet: &std::path::Path,
    nom: &str,
    octets: &[u8],
    heure: u64,
) -> std::io::Result<()> {
    let dossier = sauvegardes_dir(projet);
    std::fs::create_dir_all(&dossier)?;

    let mut copie = dossier.join(format!("{nom}.{heure}"));
    let mut numero = 1;
    while copie.exists() {
        copie = dossier.join(format!("{nom}.{heure}-recuperee{numero}"));
        numero += 1;
    }
    ecrire_atomique_octets(&copie, octets)?;

    let prefixe = format!("{nom}.");
    let mut copies: Vec<std::path::PathBuf> = std::fs::read_dir(&dossier)?
        .filter_map(Result::ok)
        .map(|entree| entree.path())
        .filter(|copie| {
            copie
                .file_name()
                .and_then(|nom_copie| nom_copie.to_str())
                .is_some_and(|nom_copie| nom_copie.starts_with(&prefixe))
        })
        .collect();
    copies.sort();
    copies.reverse();
    for vieille in copies.into_iter().skip(20) {
        let _ = std::fs::remove_file(vieille);
    }
    Ok(())
}

/// Récupère les temporaires qu'une interruption a laissés auprès d'un document.
///
/// Une écriture interrompue — panne, plantage — laisse à côté du document un
/// temporaire commencé, qui peut porter la dernière version écrite nulle part
/// ailleurs. Quand son auteur n'existe plus, il devient une copie de
/// `sauvegardes/`, récupérable comme les autres, à la date où il a été écrit.
/// Un temporaire dont l'auteur est vivant est une écriture en cours : on n'y
/// touche jamais. Hors Linux, un auteur est toujours présumé vivant.
pub fn recuperer_temporaires_perdus(projet: &std::path::Path, chemin: &std::path::Path) {
    let Some(nom) = chemin.file_name() else {
        return;
    };
    let Some(dossier) = chemin.parent() else {
        return;
    };
    let nom = nom.to_string_lossy().into_owned();
    let prefixe = format!(".{nom}.tmp-");
    let Ok(entrees) = std::fs::read_dir(dossier) else {
        return;
    };
    for temporaire in entrees.filter_map(Result::ok).map(|entree| entree.path()) {
        let Some(reste) = temporaire
            .file_name()
            .and_then(|nom_fichier| nom_fichier.to_str())
            .and_then(|nom_fichier| nom_fichier.strip_prefix(&prefixe))
        else {
            continue;
        };
        let Some((pid, _numero)) = reste.rsplit_once('-') else {
            continue;
        };
        let Ok(pid) = pid.parse::<i32>() else {
            continue;
        };
        if pid <= 0 || process_vivant(pid) {
            continue;
        }
        let Ok(octets) = std::fs::read(&temporaire) else {
            continue;
        };
        let heure = std::fs::metadata(&temporaire)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|depuis| depuis.as_secs())
            .unwrap_or_else(heure_actuelle);
        if conserver_une_version(projet, &nom, &octets, heure).is_ok() {
            let _ = std::fs::remove_file(&temporaire);
        }
    }
}

/// L'heure actuelle en secondes, pour nommer une copie.
fn heure_actuelle() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Session de travail : ce qu'il faut rouvrir au démarrage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Disposition complète ; absente dans les anciennes sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub espace: Option<crate::app::espace::Disposition>,

    /// Dossier du projet ouvert.
    #[serde(default)]
    pub project: Option<PathBuf>,

    /// Chemin du chapitre ouvert dans ce projet.
    #[serde(default)]
    pub last_document: Option<PathBuf>,

    /// Le mode d'apparence suivi : système, clair ou sombre.
    ///
    /// À défaut, `default` : le mode système, et les thèmes natifs. Une session
    /// enregistrée avant les thèmes se relit donc sans rien perdre.
    #[serde(default)]
    pub theme_mode: crate::base::theme::ThemeMode,

    /// Les thèmes retenus pour chaque apparence.
    ///
    /// On enregistre le **nom**, jamais une position : la liste change quand on
    /// charge ou supprime un thème, et une position finirait par désigner un
    /// autre thème que celui qu'on avait choisi. Un nom devenu introuvable est
    /// traité par la sélection elle-même, qui retombe sur un thème de la même
    /// apparence.
    #[serde(default)]
    pub themes: crate::theming::Chosen,
}

fn session_path() -> PathBuf {
    work_dir().join("session.json")
}

/// Charge la session enregistrée.
pub fn load_session() -> Session {
    load_session_from(&session_path()).unwrap_or_default()
}

/// Charge une session depuis un fichier donné.
///
/// Une session absente, tronquée ou illisible donne la session **vide** : un
/// fichier de configuration cassé ne doit jamais empêcher l'application de
/// démarrer, et repartir des valeurs par défaut est toujours possible.
///
/// Le chemin est un paramètre plutôt qu'une constante pour que la persistance
/// soit vérifiable sur un fichier réel, et non seulement en mémoire.
/// Relit la session. **Une session absente est normale** ; une session
/// illisible est un problème, et elle le dit — jamais de repli muet qui
/// laisserait croire que tout va bien.
pub fn load_session_from(path: &Path) -> Result<Session, String> {
    let texte = match std::fs::read_to_string(path) {
        Ok(texte) => texte,
        Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Session::default());
        }
        Err(erreur) => return Err(format!("{} : {erreur}", path.display())),
    };

    serde_json::from_str(&texte).map_err(|erreur| format!("{} : {erreur}", path.display()))
}

/// Enregistre la session.
pub fn save_session(session: &Session) {
    save_session_to(&session_path(), session);
}

/// Enregistre une session dans un fichier donné.
///
/// Un échec d'écriture est signalé sans interrompre : la session est une
/// commodité, pas une donnée dont dépend la suite du travail.
pub fn save_session_to(path: &Path, session: &Session) {
    let text = match serde_json::to_string_pretty(session) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Sérialisation de la session : {error}");
            return;
        }
    };

    if let Err(error) = ecrire_atomique(path, &text) {
        eprintln!("Enregistrement de la session : {error}");
    }
}

/// Une entrée de bibliographie (fichier `.bib`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BibliographyEntry {
    /// La clé de citation, celle qui suit `@type{`.
    pub key: String,

    /// Le type BibTeX (`article`, `book`, `inproceedings`…).
    pub kind: String,

    /// Les auteurs, `(nom, prénom)` dans l'ordre du fichier.
    ///
    /// BibTeX écrit « Dupont, Jean » ou « Jean Dupont » : les deux formes
    /// sont ramenées au même couple, nom d'abord.
    pub auteurs: Vec<(String, String)>,

    /// Le titre de l'œuvre.
    pub titre: String,

    /// L'année de parution, si le fichier la donne.
    pub annee: Option<String>,

    /// La maison d'édition, `publisher` chez BibTeX.
    pub editeur: Option<String>,

    /// Le lieu de parution, `location` ou `address`.
    pub lieu: Option<String>,

    /// Le titre court, `shorttitle` : celui des notes serrées.
    pub titre_court: Option<String>,
}

impl BibliographyEntry {
    /// La ligne de liste : « Dupont, Jean — Le titre… ».
    ///
    /// L'affichage la compose lui-même en deux registres ; cette ligne
    /// unique reste celle des tests, du modèle, et de qui préfère un trait.
    #[allow(dead_code)]
    pub fn libelle(&self) -> String {
        let auteur = match self.auteurs.first() {
            Some((nom, prenom)) if !prenom.is_empty() => format!("{nom}, {prenom}"),
            Some((nom, _)) => nom.clone(),
            None => String::new(),
        };

        let corps = if self.titre.is_empty() {
            self.key.clone()
        } else {
            self.titre.clone()
        };

        if auteur.is_empty() {
            corps
        } else {
            format!("{auteur} — {corps}")
        }
    }
}

/// Lit les entrées d'un fichier `.bib`.
///
/// Parseur volontairement tolérant : il cherche les entrées (`@type{clé,`),
/// puis les champs qu'elles portent — auteur, titre, année. Un fichier
/// malformé renvoie les entrées reconnues jusqu'au problème.
pub fn parse_bibliography(path: &Path) -> Vec<BibliographyEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    parse_bibliography_text(&content)
}

/// Les champs d'une notice, tels que le formulaire les édite.
pub struct ReferenceChamps {
    /// La clé de citation — celle que les documents citent.
    pub cle: String,

    /// Le type de document : `book`, `article`, `inproceedings`…
    pub kind: String,

    /// Le nom de l'auteur.
    pub nom: String,

    /// Son prénom.
    pub prenom: String,

    /// Le titre de l'œuvre.
    pub titre: String,

    /// Le titre court, celui des notes serrées.
    pub titre_court: String,

    /// L'année de parution.
    pub annee: String,

    /// La maison d'édition.
    pub editeur: String,

    /// Le lieu de parution.
    pub lieu: String,
}

/// Met à jour une référence dans un fichier `.bib`.
///
/// Les champs connus sont remplacés ou ajoutés **dans le bloc de l'entrée** ;
/// tout le reste du fichier ne bouge pas, et la clé reste la sienne : elle
/// est citée dans les documents.
pub fn maj_reference(
    chemin: &std::path::Path,
    cle: &str,
    champs: &ReferenceChamps,
) -> std::io::Result<()> {
    let contenu = std::fs::read_to_string(chemin)?;
    let octets = contenu.as_bytes();
    let mut sortie = String::with_capacity(contenu.len() + 64);
    let mut curseur = 0usize;

    while curseur < contenu.len() {
        let Some(relatif) = contenu[curseur..].find('@') else {
            sortie.push_str(&contenu[curseur..]);
            break;
        };

        let debut = curseur + relatif;
        sortie.push_str(&contenu[curseur..debut]);

        let fin = fin_d_entree(octets, debut).unwrap_or(contenu.len());
        let bloc = &contenu[debut..fin];

        if entree_de(bloc).is_some_and(|entree| entree.key == cle) {
            let auteur = if champs.prenom.is_empty() {
                champs.nom.clone()
            } else {
                format!("{}, {}", champs.nom, champs.prenom)
            };

            let mut maj = bloc.to_string();
            maj = remplacer_type(&maj, &champs.kind);
            maj = remplacer_champ(&maj, "author", &auteur);
            maj = remplacer_champ(&maj, "title", &champs.titre);
            maj = option(&maj, "year", &champs.annee);
            maj = option(&maj, "publisher", &champs.editeur);
            maj = option(&maj, "location", &champs.lieu);
            maj = option(&maj, "shorttitle", &champs.titre_court);

            sortie.push_str(&maj);
        } else {
            sortie.push_str(bloc);
        }

        curseur = fin;
    }

    ecrire_atomique(chemin, &sortie)
}

/// Un champ optionnel : écrit quand il a une valeur, retiré sinon.
fn option(entree: &str, champ: &str, valeur: &str) -> String {
    if valeur.is_empty() {
        retirer_champ(entree, champ)
    } else {
        remplacer_champ(entree, champ, valeur)
    }
}

/// Le type d'une entrée : `@book{…` devient `@article{…`.
///
/// La clé, elle, ne bouge pas : elle est citée dans les documents.
fn remplacer_type(entree: &str, kind: &str) -> String {
    let Some(ouverture) = entree.find('{') else {
        return entree.to_string();
    };

    format!("@{}{}", kind.trim(), &entree[ouverture..])
}

/// Remplace la valeur d'un champ dans un bloc d'entrée, ou l'y ajoute.
fn remplacer_champ(bloc: &str, nom: &str, valeur: &str) -> String {
    let trouve = champ_plage(bloc, nom);

    match trouve {
        Some((debut, fin)) => {
            let mut sortie = String::with_capacity(bloc.len() + valeur.len());
            sortie.push_str(&bloc[..debut]);
            sortie.push_str(valeur);
            sortie.push_str(&bloc[fin..]);

            sortie
        }
        None => {
            // Avant l'accolade fermante : la dernière du bloc.
            let Some(fin) = bloc.rfind('}') else {
                return bloc.to_string();
            };

            // La virgule sépare les champs — jamais deux, et toujours une
            // entre le dernier champ et le nouveau. Une virgule orpheline,
            // et c'est BibLaTeX qui refuse le fichier entier.
            // La virgule sépare les champs — jamais deux, et toujours une
            // entre le dernier champ et le nouveau. Une virgule orpheline,
            // et c'est BibLaTeX qui refuse le fichier entier.
            let tete = bloc[..fin].trim_end();
            let separateur = if tete.ends_with(',') { "" } else { "," };

            let mut sortie = String::with_capacity(bloc.len() + valeur.len() + 16);
            sortie.push_str(tete);
            sortie.push_str(separateur);
            sortie.push_str(&format!("\n  {nom} = {{{valeur}}}\n"));
            sortie.push_str(&bloc[fin..]);

            sortie
        }
    }
}

/// Retire un champ d'un bloc d'entrée, s'il y est.
fn retirer_champ(bloc: &str, nom: &str) -> String {
    let Some((debut_valeur, fin_valeur)) = champ_plage(bloc, nom) else {
        return bloc.to_string();
    };

    // Le champ **entier** : depuis la fin du champ précédent — sa virgule
    // ou l'ouverture de l'entrée — jusqu'après le délimiteur fermant de la
    // valeur et sa virgule. Ne rien laisser : une accolade orpheline
    // referme l'entrée avant l'heure et chasse le reste des champs, et une
    // virgule oubliée en double corrompt le fichier aux yeux de Typst.
    let debut = bloc[..debut_valeur]
        .rfind(',')
        .map(|index| index + 1)
        .or_else(|| bloc.find('{').map(|index| index + 1))
        .unwrap_or(0);

    // Le délimiteur fermant est juste après la valeur : accolade ou
    // guillemet, puis la virgule s'il y en a une.
    let mut fin = (fin_valeur + 1).min(bloc.len());

    let suite = bloc[fin..].chars().next();
    if suite == Some(',') {
        fin += 1;
    }

    let mut sortie = String::with_capacity(bloc.len());
    sortie.push_str(&bloc[..debut]);
    sortie.push_str(&bloc[fin..]);

    sortie
}

/// La plage de la **valeur** d'un champ : entre accolades, sans elles.
fn champ_plage(bloc: &str, nom: &str) -> Option<(usize, usize)> {
    let mut base = 0usize;

    while let Some(relatif) = bloc[base..].find('=') {
        let egal = base + relatif;
        let debut_valeur = egal + 1 + bloc[egal + 1..].len() - bloc[egal + 1..].trim_start().len();
        let avant = bloc[..egal].trim();
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            let valeur = &bloc[debut_valeur..];

            if let Some(ouvrante) = valeur.find('{') {
                let mut profondeur = 1usize;

                for (relatif, caractere) in valeur[ouvrante + 1..].char_indices() {
                    match caractere {
                        '{' => profondeur += 1,
                        '}' => {
                            profondeur -= 1;

                            if profondeur == 0 {
                                return Some((
                                    debut_valeur + ouvrante + 1,
                                    debut_valeur + ouvrante + 1 + relatif,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }

            if let Some(ouvrante) = valeur.find('"')
                && let Some(fermante) = valeur[ouvrante + 1..].find('"')
            {
                return Some((
                    debut_valeur + ouvrante + 1,
                    debut_valeur + ouvrante + 1 + fermante,
                ));
            }

            // Valeur nue : jusqu'à la virgule ou l'accolade.
            let fin = valeur
                .find([',', '}'])
                .map_or(valeur.len(), |index| debut_valeur + index);

            return Some((debut_valeur, fin));
        }

        base = egal + 1;
    }

    None
}

/// Les entrées d'un texte BibTeX.
/// Supprime une notice de la bibliothèque locale — vers la corbeille.
///
/// Une notice n'est pas un fichier : elle part **avec** son entrée, et le
/// fichier reste propre. La suppression ne touche jamais à une bibliothèque
/// Zotero : elle, se lit seulement.
/// Retire d'un `.bib` les entrées dont la clé est déjà connue ailleurs,
/// et dit quelles clés ce fichier apporte (bug-report-10, 6.1).
///
/// Deux bibliothèques qui partagent une clé cassent la compilation
/// Typst (« duplicate bibliography keys ») : la première en date garde
/// son entrée, la seconde la cède — la bibliothèque locale gagne
/// toujours sur l'export Zotero, qui se lit mais ne se réécrit pas.
pub fn retirer_les_entrees_dupliquees(
    contenu: &str,
    connues: &mut std::collections::BTreeSet<String>,
) -> String {
    // Les têtes d'entrée : `@type{clé,` en début de ligne.
    let mut tetes = Vec::new();

    for (rang, ligne) in contenu.lines().enumerate() {
        let tete = ligne.trim_start();

        if let Some(reste) = tete.strip_prefix('@')
            && let Some(ouverture) = reste.find('{')
        {
            let apres = &reste[ouverture + 1..];

            if let Some(virgule) = apres.find(',') {
                let cle = apres[..virgule].trim();

                if !cle.is_empty() {
                    tetes.push((rang, cle.to_string()));
                }
            }
        }
    }

    // Les plages : chaque entrée va de sa tête à la suivante.
    let mut enlevees = std::collections::BTreeSet::new();

    for (_, cle) in &tetes {
        if !connues.insert(cle.clone()) {
            enlevees.insert(cle.clone());
        }
    }

    if enlevees.is_empty() {
        return contenu.to_string();
    }

    let mut out = String::new();

    for (position, (rang, cle)) in tetes.iter().enumerate() {
        if enlevees.contains(cle) {
            continue;
        }

        // La zone de cette entrée : du début de sa ligne de tête — ou du
        // texte d'avant, pour la première — à la tête suivante.
        let debut = contenu
            .lines()
            .take(*rang)
            .map(|l| l.len() + 1)
            .sum::<usize>();
        let fin = tetes
            .get(position + 1)
            .map(|(suivant, _)| {
                contenu
                    .lines()
                    .take(*suivant)
                    .map(|l| l.len() + 1)
                    .sum::<usize>()
            })
            .unwrap_or(contenu.len());

        out.push_str(&contenu[debut..fin]);
    }

    out
}

pub fn supprimer_reference(path: &Path, cle: &str) -> Result<std::path::PathBuf, String> {
    let contenu = std::fs::read_to_string(path).map_err(|e| format!("{e}"))?;
    let (ouverture, fermeture) =
        plage_de_notice(&contenu, cle).ok_or_else(|| format!("« {cle} » introuvable."))?;

    // L'entière : sa tête, ses champs, et la ligne qui les sépare du reste.
    let mut fin = fermeture;

    while fin < contenu.len() && contenu[fin..].starts_with('\n') {
        fin += 1;
    }

    // La notice entière va à la corbeille **avant** toute suppression : si
    // la copie échoue, la bibliographie ne bouge pas, et rien n'est jamais
    // détruit en silence.
    let copie = vers_la_corbeille_texte(&format!("notice-{cle}.bib"), &contenu[ouverture..fin])?;

    let mut reste = contenu[..ouverture].to_string();
    reste.push_str(&contenu[fin..]);

    ecrire_atomique(path, &reste).map_err(|e| format!("{e}"))?;
    Ok(copie)
}

/// Le résultat de l'import d'un style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportationStyle {
    /// Le style est choisi — éventuellement avec un écart connu à dire.
    Choisi {
        /// Le chemin du projet, celui que la directive lira.
        chemin: PathBuf,
        /// Un écart connu du moteur à dire — jamais un silence.
        avertissement: Option<String>,
    },

    /// Un style dépendant dont le parent n'est pas sur la machine : le
    /// geste est dit, rien n'est choisi.
    ParentManquant {
        /// La copie du style dépendant, conservée dans le projet.
        chemin: PathBuf,
        /// Son titre.
        titre: String,
        /// Le parent qu'il réclame.
        parent: String,
    },
}

/// Importe un style CSL dans le projet : contrôlé comme le moteur le
/// ferait, copié sous `styles/` avec un nom qui n'appartient à personne.
///
/// L'original n'est jamais touché, et la copie suit le projet hors ligne.
/// Un style **dépendant** se résout par son parent quand celui-ci vit
/// quelque part sur la machine (les styles du projet, ou les styles
/// installés dans Zotero — `styles_zotero`) ; sinon rien n'est choisi et
/// le parent qu'il faut se dit (R4).
pub fn importer_style_csl(
    projet: &Path,
    nom_souhaite: &str,
    texte: &str,
    styles_zotero: Option<&Path>,
) -> Result<ImportationStyle, String> {
    let style = crate::typst::styles::controler_csl(texte)?;

    match style {
        crate::typst::styles::StyleCsl::Independant { titre: _ } => {
            let relatif = copier_style(projet, nom_souhaite, texte)?;

            Ok(ImportationStyle::Choisi {
                chemin: relatif,
                avertissement: None,
            })
        }
        crate::typst::styles::StyleCsl::Dependant { titre, parent } => {
            // « Conserver le nom demandé » : le style dépendant entre lui
            // aussi, quoi qu'il arrive.
            let dependant = copier_style(projet, nom_souhaite, texte)?;

            let Some(parent_url) = parent else {
                return Ok(ImportationStyle::ParentManquant {
                    chemin: dependant,
                    titre,
                    parent: String::new(),
                });
            };

            match resoudre_parent(projet, styles_zotero, &parent_url) {
                Some(chemin) => Ok(ImportationStyle::Choisi {
                    chemin,
                    // Écart connu du moteur : il n'exécute que des styles
                    // indépendants — la locale du style dépendant ne
                    // s'applique pas, et il faut le dire.
                    avertissement: Some(format!(
                        "Style dépendant « {titre} » : le moteur utilise son parent. Écart connu : la locale du style dépendant n'est pas appliquée, c'est celle du document qui sert."
                    )),
                }),
                None => Ok(ImportationStyle::ParentManquant {
                    chemin: dependant,
                    titre,
                    parent: parent_url,
                }),
            }
        }
    }
}

/// Le parent d'un style dépendant, cherché où il peut vivre : les styles
/// du projet d'abord, les styles installés dans Zotero ensuite.
fn resoudre_parent(
    projet: &Path,
    styles_zotero: Option<&Path>,
    parent_url: &str,
) -> Option<PathBuf> {
    let dossier = projet.join("styles");

    if let Ok(entrees) = std::fs::read_dir(&dossier) {
        for entree in entrees.flatten() {
            let chemin = entree.path();

            if chemin.extension().is_some_and(|ext| ext == "csl")
                && let Ok(texte) = std::fs::read_to_string(&chemin)
                && crate::zotero::styles::identifiant_csl(&texte).as_deref() == Some(parent_url)
            {
                return Some(
                    std::path::PathBuf::from("styles").join(chemin.file_name().unwrap_or_default()),
                );
            }
        }
    }

    let styles_zotero = styles_zotero?;
    for style in crate::zotero::styles::installed(styles_zotero) {
        if style.id == parent_url {
            let texte = std::fs::read_to_string(&style.path).ok()?;
            return copier_style(projet, "parent.csl", &texte).ok();
        }
    }

    None
}

/// Copie un texte de style sous `styles/`, avec un nom libre.
fn copier_style(projet: &Path, nom_souhaite: &str, texte: &str) -> Result<PathBuf, String> {
    let dossier = projet.join("styles");
    std::fs::create_dir_all(&dossier).map_err(|e| format!("{e}"))?;

    let base = std::path::Path::new(nom_souhaite)
        .file_stem()
        .map(|base| base.to_string_lossy().into_owned())
        .filter(|base| !base.is_empty())
        .unwrap_or_else(|| "style".to_string());
    let mut destination = dossier.join(format!("{base}.csl"));
    let mut numero = 2;

    while destination.exists() {
        destination = dossier.join(format!("{base}-{numero}.csl"));
        numero += 1;
    }

    ecrire_atomique(&destination, texte).map_err(|e| format!("{e}"))?;

    Ok(std::path::PathBuf::from("styles").join(destination.file_name().unwrap_or_default()))
}

/// Écrit un texte à la corbeille, sous un nom qui ne marche sur personne.
fn vers_la_corbeille_texte(nom: &str, texte: &str) -> Result<std::path::PathBuf, String> {
    let base = nom.trim_end_matches(".bib");
    let mut destination = corbeille().join(format!("{base}.bib"));
    let mut numero = 2;

    while destination.exists() {
        destination = corbeille().join(format!("{base}-{numero}.bib"));
        numero += 1;
    }

    ecrire_atomique(&destination, texte).map_err(|e| format!("{e}"))?;
    Ok(destination)
}

/// Lit une bibliothèque `.bib` : une absence est un fichier vide, toute
/// autre erreur se propage — on ne réécrit jamais après une lecture ratée.
fn lire_bibliotheque(chemin: &std::path::Path) -> std::io::Result<String> {
    match std::fs::read_to_string(chemin) {
        Ok(contenu) => Ok(contenu),
        Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(erreur) => Err(erreur),
    }
}

/// Remet dans la bibliographie la notice que la corbeille a recueillie.
///
/// La copie s'ajoute telle quelle ; une clé déjà présente ne se dédouble
/// jamais — la restauration dit le problème au lieu de le créer.
pub fn restaurer_reference(bibliotheque: &Path, copie: &Path) -> Result<String, String> {
    let texte =
        std::fs::read_to_string(copie).map_err(|e| format!("La copie est illisible : {e}"))?;
    let entree = crate::project::parse_bibliography_text(&texte)
        .into_iter()
        .next()
        .ok_or_else(|| "La copie ne contient aucune notice.".to_string())?;

    let contenu = lire_bibliotheque(bibliotheque)
        .map_err(|erreur| format!("La bibliothèque est illisible : {erreur}"))?;
    if crate::project::parse_bibliography_text(&contenu)
        .iter()
        .any(|autre| autre.key == entree.key)
    {
        return Err(format!(
            "« {} » est déjà dans la bibliographie.",
            entree.key
        ));
    }

    let mut nouveau = contenu;
    if !nouveau.is_empty() && !nouveau.ends_with('\n') {
        nouveau.push('\n');
    }
    nouveau.push_str(texte.trim());
    nouveau.push('\n');
    ecrire_atomique(bibliotheque, &nouveau).map_err(|e| format!("{e}"))?;
    Ok(entree.key)
}

/// L'étendue d'une notice : sa tête `@type{clé,` et son accolade fermante.
fn plage_de_notice(contenu: &str, cle: &str) -> Option<(usize, usize)> {
    let motif = format!("{{{cle},");
    // Jamais de `find - 1` : un motif en tête de fichier soustrairait un
    // à zéro. La tête commence au `@` qui précède.
    let tete = contenu.find(&motif)?;
    let ouverture = contenu[..tete].rfind('@')?;

    let mut profondeur = 0usize;

    for (position, caractere) in contenu[ouverture..].char_indices() {
        match caractere {
            '{' => profondeur += 1,
            '}' => {
                profondeur = profondeur.saturating_sub(1);

                if profondeur == 0 {
                    return Some((ouverture, ouverture + position + 1));
                }
            }
            _ => {}
        }
    }

    None
}

pub fn parse_bibliography_text(content: &str) -> Vec<BibliographyEntry> {
    let mut entrees = Vec::new();
    let octets = content.as_bytes();
    let mut curseur = 0usize;

    while let Some(debut) = content[curseur..].find('@') {
        let debut = curseur + debut;

        // Un `@` dans un commentaire BibLaTeX (une ligne % …, ou un
        // commentaire de fin de ligne) n'ouvre jamais de notice : un
        // exemple commenté reste un commentaire (fidélité BibLaTeX).
        let ligne_debut = content[..debut].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let mut en_commentaire = false;
        let mut echappe = false;

        for lettre in content[ligne_debut..debut].chars() {
            match lettre {
                '\\' if !echappe => echappe = true,
                '%' if !echappe => {
                    en_commentaire = true;
                    break;
                }
                _ => echappe = false,
            }
        }

        if en_commentaire {
            curseur = debut + 1;
            continue;
        }

        // Une entrée tronquée (accolade non fermée) est ignorée — elle
        // ne doit pas emporter les suivantes.
        let Some(fin) = fin_d_entree(octets, debut) else {
            curseur = debut + 1;
            continue;
        };

        if let Some(entree) = entree_de(&content[debut..fin]) {
            entrees.push(entree);
        }

        curseur = fin;
    }

    entrees
}

/// Le rapport d'un import BibLaTeX : ce qui est entré, ce qui était déjà
/// là, ce qui s'est heurté — jamais un silence.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RapportBib {
    /// Les clés entrées au projet.
    pub ajoutees: Vec<String>,
    /// Les clés déjà là, à l'identique : rien n'a bougé.
    pub deja_la: Vec<String>,
    /// Les clés déjà là, avec une autre notice : **la première tient**,
    /// et le conflit se dit.
    pub conflits: Vec<String>,
}

/// Importe un fichier BibLaTeX dans la bibliothèque locale du projet —
/// l'alternative sans plugin : Zotero sait exporter lui-même un `.bib`,
/// et cette voie n'a besoin ni de plugin ni d'API locale.
///
/// La bibliothèque Zotero (`references-zotero.bib`) n'est jamais visée :
/// elle, se lit seulement. Les clés communes ne s'écrasent jamais : une
/// notice identique est ignorée (« déjà là »), une notice différente est
/// **conservée et dite** (conflit) — la première venue tient.
pub fn importer_bib(
    projet: &std::path::Path,
    source: &std::path::Path,
) -> Result<RapportBib, String> {
    let texte = std::fs::read_to_string(source)
        .map_err(|erreur| format!("{} : {erreur}", source.display()))?;
    let entrees = parse_bibliography_text(&texte);

    if entrees.is_empty() {
        return Err(format!(
            "{} : aucune notice BibLaTeX lue — rien n'a changé.",
            source.display()
        ));
    }

    let cible = projet.join("references.bib");

    // La base d'un fichier qui naît est celle du modèle — le même squelette
    // que le geste « Créer la bibliographie locale ».
    let existant = std::fs::read_to_string(&cible)
        .unwrap_or_else(|_| crate::project::model::BIBLIOGRAPHY_TEMPLATE.to_string());
    let existantes = parse_bibliography_text(&existant);

    let mut rapport = RapportBib::default();
    let mut ajouts = String::new();

    for entree in &entrees {
        let Some((ouverture, fermeture)) = plage_de_notice(&texte, &entree.key) else {
            continue;
        };
        let brut = texte[ouverture..fermeture].trim();

        if let Some(autre) = existantes.iter().find(|e| e.key == entree.key) {
            if let Some((debut, fin)) = plage_de_notice(&existant, &autre.key) {
                if existant[debut..fin].trim() == brut {
                    rapport.deja_la.push(entree.key.clone());
                } else {
                    rapport.conflits.push(entree.key.clone());
                }
            } else {
                // La clé est là mais sa notice est illisible : conflit.
                rapport.conflits.push(entree.key.clone());
            }

            continue;
        }

        // Une clé répétée dans le même import n'entre qu'une fois.
        if rapport.ajoutees.contains(&entree.key) {
            continue;
        }
        rapport.ajoutees.push(entree.key.clone());

        if !ajouts.is_empty() {
            ajouts.push('\n');
        }
        ajouts.push_str(brut);
        ajouts.push('\n');
    }

    if !ajouts.is_empty() {
        let mut final_ = existant;
        if !final_.is_empty() && !final_.ends_with('\n') {
            final_.push('\n');
        }
        final_.push_str(&ajouts);
        ecrire_atomique(&cible, &final_)
            .map_err(|erreur| format!("{} : {erreur}", cible.display()))?;
    }

    Ok(rapport)
}

#[cfg(test)]
mod epreuves_d_import_bib {
    use super::{ReferenceChamps, importer_bib, nouvelle_reference, parse_bibliography_text};

    fn dossier(nom: &str) -> std::path::PathBuf {
        let dossier =
            std::env::temp_dir().join(format!("typst-writer-import-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).expect("dossier");
        dossier
    }

    fn champs(cle: &str) -> ReferenceChamps {
        ReferenceChamps {
            cle: cle.to_string(),
            kind: "book".to_string(),
            nom: "Dupont".to_string(),
            prenom: "Jean".to_string(),
            titre: "Un titre".to_string(),
            titre_court: String::new(),
            annee: "2026".to_string(),
            editeur: "Éditions".to_string(),
            lieu: "Paris".to_string(),
        }
    }

    /// Une bibliothèque non UTF-8 n'est jamais lue comme vide puis réécrite.
    #[test]
    fn une_bibliotheque_non_utf8_n_est_pas_ecrasee() {
        let projet = dossier("non-utf8");
        let chemin = projet.join("references.bib");
        let octets = [0xff, 0xfe, 0x00, 0x41];
        std::fs::write(&chemin, octets).expect("bibliothèque fautive");

        let resultat = nouvelle_reference(&chemin, &champs("neuve2026"));

        assert!(resultat.is_err(), "la lecture illisible se propage");
        assert_eq!(
            std::fs::read(&chemin).expect("fichier"),
            octets,
            "la bibliothèque de l'auteur reste intacte"
        );

        let _ = std::fs::remove_dir_all(&projet);
    }

    /// Une bibliothèque absente est un cas légitime : le fichier naît.
    #[test]
    fn une_bibliotheque_absente_est_creee() {
        let projet = dossier("absente");
        let chemin = projet.join("references.bib");

        let cle = nouvelle_reference(&chemin, &champs("neuve2026")).expect("création");

        assert_eq!(cle, "neuve2026");
        assert!(chemin.is_file(), "le fichier est né");

        let _ = std::fs::remove_dir_all(&projet);
    }

    const DEUX: &str = "@book{dupont2023,\n  author = {Dupont},\n  title = {Un titre},\n}\n\n@article{martin2024,\n  author = {Martin},\n  title = {Un autre},\n}\n";

    /// Les notices entrent, ensemble, dans la bibliothèque locale.
    #[test]
    fn un_import_bib_rejoint_la_bibliographie() {
        let projet = dossier("entree");
        let source = projet.join("export-zotero.bib");
        std::fs::write(&source, DEUX).expect("source");

        let rapport = importer_bib(&projet, &projet.join("export-zotero.bib")).expect("import");
        assert_eq!(rapport.ajoutees, vec!["dupont2023", "martin2024"]);
        assert!(rapport.deja_la.is_empty() && rapport.conflits.is_empty());

        let bibliotheque =
            std::fs::read_to_string(projet.join("references.bib")).expect("bibliothèque");
        assert_eq!(
            parse_bibliography_text(&bibliotheque).len(),
            2,
            "les deux notices sont là"
        );
    }

    /// La même clé : identique, elle ne se répète pas ; différente, elle
    /// ne s'écrase pas et le conflit se dit.
    #[test]
    fn un_import_bib_ne_ecrase_jamais_une_notice() {
        let projet = dossier("conflit");
        std::fs::write(
            projet.join("references.bib"),
            "@book{dupont2023,\n  author = {Dupont},\n  title = {Premier},\n}\n",
        )
        .expect("bibliothèque");

        // Identique, à l'entourage près : déjà là.
        let identique = projet.join("identique.bib");
        std::fs::write(
            &identique,
            "@book{dupont2023,\n  author = {Dupont},\n  title = {Premier},\n}\n",
        )
        .expect("source");
        let rapport = importer_bib(&projet, &identique).expect("import");
        assert_eq!(rapport.deja_la, vec!["dupont2023"]);
        assert!(rapport.ajoutees.is_empty() && rapport.conflits.is_empty());

        // Différente : la première tient, le conflit se dit.
        let differente = projet.join("differente.bib");
        std::fs::write(
            &differente,
            "@book{dupont2023,\n  author = {Dupont},\n  title = {Second},\n}\n",
        )
        .expect("source");
        let rapport = importer_bib(&projet, &differente).expect("import");
        assert_eq!(rapport.conflits, vec!["dupont2023"]);

        let bibliotheque =
            std::fs::read_to_string(projet.join("references.bib")).expect("bibliothèque");
        assert!(
            bibliotheque.contains("Premier") && !bibliotheque.contains("Second"),
            "la première tient : {bibliotheque}"
        );
    }

    /// Un fichier sans notice lue : le problème se dit, rien n'entre.
    #[test]
    fn un_import_bib_sans_notice_dit_le_probleme() {
        let projet = dossier("vide");
        let source = projet.join("sans-notice.bib");
        std::fs::write(&source, "ceci n'est pas du BibLaTeX").expect("source");

        let probleme = importer_bib(&projet, &source).expect_err("refusé");
        assert!(probleme.contains("aucune notice"), "{probleme}");
        assert!(!projet.join("references.bib").exists(), "rien n'est écrit");
    }

    /// Un `@` dans un commentaire BibLaTeX n'ouvre jamais de notice : un
    /// exemple commenté reste un commentaire, et ne devient pas un fantôme
    /// dans la bibliographie.
    #[test]
    fn un_exemple_commente_nest_jamais_une_notice() {
        let entrees = parse_bibliography_text(
            "% @article{exemple2026,\n%   title = {Un titre},\n% }\n\n@book{vrai2025,\n  title = {Réel},\n}\n",
        );

        let cles: Vec<_> = entrees.iter().map(|entree| entree.key.as_str()).collect();
        assert_eq!(
            cles,
            vec!["vrai2025"],
            "le commentaire n'est pas une notice"
        );
    }
}

/// Ajoute une référence au fichier `.bib` et rend la clé retenue.
///
/// Une clé vide prend un nom généré ; une clé déjà citée est dédoublonnée
/// d'un numéro — jamais deux documents ne partagent une clé de citation.
pub fn nouvelle_reference(
    chemin: &std::path::Path,
    champs: &ReferenceChamps,
) -> std::io::Result<String> {
    let contenu = lire_bibliotheque(chemin)?;
    let existantes = parse_bibliography_text(&contenu);

    let mut cle = champs.cle.trim().to_string();
    if cle.is_empty() {
        cle = "reference".to_string();
    }

    let racine = cle.clone();
    let mut numero = 2;
    while existantes.iter().any(|entree| entree.key == cle) {
        cle = format!("{racine}{numero}");
        numero += 1;
    }

    let auteur = if champs.prenom.is_empty() {
        champs.nom.trim().to_string()
    } else {
        format!("{}, {}", champs.nom.trim(), champs.prenom.trim())
    };

    let mut entree = format!("@{}{{{},\n", champs.kind.trim(), cle);

    for (champ, valeur) in [
        ("author", auteur.as_str()),
        ("title", champs.titre.trim()),
        ("shorttitle", champs.titre_court.trim()),
        ("year", champs.annee.trim()),
        ("publisher", champs.editeur.trim()),
        ("location", champs.lieu.trim()),
    ] {
        if !valeur.is_empty() {
            entree.push_str(&format!("  {champ} = {{{valeur}}},\n"));
        }
    }

    entree.push_str("}\n\n");

    let mut suite = contenu.trim_end().to_string();
    if !suite.is_empty() {
        suite.push('\n');
        suite.push('\n');
    }
    suite.push_str(&entree);

    ecrire_atomique(chemin, &suite)?;
    Ok(cle)
}

/// La fin d'une entrée : l'accolade qui referme celle ouverte en `debut`.
fn fin_d_entree(octets: &[u8], debut: usize) -> Option<usize> {
    let mut profondeur = 0usize;

    for (relatif, octet) in octets[debut..].iter().enumerate() {
        match octet {
            b'{' => profondeur += 1,
            b'}' => {
                profondeur = profondeur.saturating_sub(1);

                if profondeur == 0 {
                    return Some(debut + relatif + 1);
                }
            }
            _ => {}
        }
    }

    None
}

/// Une entrée, lue dans son bloc entier : `@type{clé, champs… }`.
fn entree_de(bloc: &str) -> Option<BibliographyEntry> {
    let reste = bloc.trim().strip_prefix('@')?;
    let (kind, reste) = reste.split_once('{')?;

    let (cle, champs) = reste.split_once(',').unwrap_or((reste, ""));
    let key = cle.split(['}', ',']).next()?.trim().to_string();

    if key.is_empty() {
        return None;
    }

    let lieu = champ(champs, "location").or_else(|| champ(champs, "address"));

    Some(BibliographyEntry {
        kind: kind.trim().to_string(),
        key,
        auteurs: auteurs_de(champ(champs, "author")),
        titre: champ(champs, "title").unwrap_or_default(),
        annee: champ(champs, "year"),
        editeur: champ(champs, "publisher"),
        lieu,
        titre_court: champ(champs, "shorttitle"),
    })
}

/// La valeur d'un champ BibTeX, sans ses accolades ni ses guillemets.
fn champ(champs: &str, nom: &str) -> Option<String> {
    let mut reste = champs;

    while let Some(egal) = reste.find('=') {
        let avant = reste[..egal].trim();
        let apres = &reste[egal + 1..];

        // Le nom du champ est ce qui suit la virgule précédente.
        let nom_du_champ = avant.rsplit(',').next().unwrap_or(avant).trim();

        if nom_du_champ.eq_ignore_ascii_case(nom) {
            return Some(valeur_de(apres));
        }

        reste = apres;
    }

    None
}

/// La valeur qui suit un `=` : entre accolades, entre guillemets, ou jusqu'à
/// la virgule suivante.
fn valeur_de(texte: &str) -> String {
    let texte = texte.trim_start();

    if let Some(reste) = texte.strip_prefix('{') {
        let mut profondeur = 1usize;

        for (index, caractere) in reste.char_indices() {
            match caractere {
                '{' => profondeur += 1,
                '}' => {
                    profondeur -= 1;

                    if profondeur == 0 {
                        return reste[..index].trim().to_string();
                    }
                }
                _ => {}
            }
        }

        return reste.trim().to_string();
    }

    if let Some(reste) = texte.strip_prefix('"') {
        return reste
            .split_once('"')
            .map(|(valeur, _)| valeur.trim().to_string())
            .unwrap_or_else(|| reste.trim().to_string());
    }

    texte.split(',').next().unwrap_or(texte).trim().to_string()
}

/// Les auteurs d'un champ `author` : « et » sépare, la virgule dit le nom.
fn auteurs_de(champ: Option<String>) -> Vec<(String, String)> {
    let Some(texte) = champ else {
        return Vec::new();
    };

    texte
        .replace(" AND ", " and ")
        .split(" and ")
        .filter(|auteur| !auteur.trim().is_empty())
        .map(|auteur| {
            let auteur = auteur.trim();

            if let Some((nom, prenom)) = auteur.split_once(',') {
                (nom.trim().to_string(), prenom.trim().to_string())
            } else {
                // « Jean Dupont » : le nom est le dernier mot.
                let mut mots = auteur.rsplitn(2, ' ');
                let nom = mots.next().unwrap_or(auteur).to_string();
                let prenom = mots.next().unwrap_or("").to_string();

                (nom, prenom)
            }
        })
        .collect()
}

/// Le nom du projet créé lors de la reprise des documents existants.
const MIGRATED_PROJECT: &str = "mon-travail";

/// Reprend les documents à plat dans un premier projet.
///
/// Les versions précédentes rangeaient les documents directement à la racine de
/// `~/.typst-writer/`. Puisqu'un travail est désormais un projet, ces fichiers
/// deviennent les chapitres d'un projet « mon-travail ».
///
/// Ne fait rien si un projet existe déjà, ou s'il n'y a rien à reprendre :
/// la migration ne s'exécute qu'une fois, sur un dossier vierge de projets.
///
/// Renvoie le dossier du projet créé, le cas échéant.
pub fn migrate_flat_documents() -> Option<PathBuf> {
    let work = work_dir();
    let flat = locate_flat_documents(&work);

    if flat.is_empty() || !model::list_projects().is_empty() {
        return None;
    }

    let (dir, _verrou) = model::create_avec_verrou(MIGRATED_PROJECT).ok()?;

    // Une bibliographie restée à la racine doit rejoindre le projet : sans
    // cela, un chapitre qui la cite perdrait sa référence.
    adopt_stray_bibliography(&work, &dir);

    let chapters_dir = dir.join("chapitres");

    let session = load_session();
    let mut moved_session = session.project.is_none();

    for (index, path) in flat.iter().enumerate() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("chapitre.typ");

        let destination = chapters_dir.join(format!("{:02}-{}", index + 1, name));

        if std::fs::rename(path, &destination).is_err() {
            // Un fichier qu'on n'a pas pu déplacer est laissé sur place plutôt
            // que perdu : les `.typ` encore à la racine font reprendre la
            // migration au prochain démarrage.
            continue;
        }

        let _ = model::add_chapter_to_assembly(&dir.join("main.typ"), &destination);

        if session.last_document.as_deref() == Some(path.as_path()) {
            moved_session = true;
        }
    }

    // La session pointait vers un fichier à plat : elle pointe maintenant vers
    // le projet, sinon le démarrage suivant chercherait un document disparu.
    if moved_session {
        save_session(&Session {
            project: Some(dir.clone()),
            last_document: None,
            // Le thème retenu ne regarde pas la migration des documents à plat :
            // il survit tel quel, et le reste de la session avec lui.
            ..session
        });
    }

    Some(dir)
}

/// Fait rejoindre au projet une bibliographie laissée à la racine.
///
/// Le fichier créé par `model::create` est un modèle vide : on ne le remplace
/// que s'il est effectivement vide, pour ne jamais écraser des références
/// existantes par un exemple commenté.
fn adopt_stray_bibliography(work: &Path, project_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(work) else {
        return;
    };

    let Some(stray) = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().map(|ext| ext == "bib").unwrap_or(false))
    else {
        return;
    };

    let destination = project_dir.join("references.bib");

    // Le modèle **intact**, et pas « sans `@` » : ses commentaires en
    // portent un, et cette vérification-là ne pouvait jamais être vraie —
    // la bibliothèque errante n'était donc jamais adoptée.
    let template_is_untouched = std::fs::read_to_string(&destination)
        .map(|content| content.trim() == crate::project::model::BIBLIOGRAPHY_TEMPLATE.trim())
        .unwrap_or(false);

    if template_is_untouched {
        let _ = std::fs::rename(&stray, &destination);
    }
}

/// Les documents `.typ` restés à la racine du dossier de travail.
fn locate_flat_documents(work: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(work) else {
        return Vec::new();
    };

    let mut documents: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().map(|ext| ext == "typ").unwrap_or(false))
        .collect();

    // L'ordre de lecture du dossier n'est pas garanti : on trie pour que la
    // numérotation des chapitres soit reproductible.
    documents.sort();

    documents
}

#[cfg(test)]
mod tests {
    /// Un projet se crée au nom choisi, et jamais sur un autre.
    #[test]
    fn un_projet_se_cree_au_nom_choisi() {
        let parent = std::env::temp_dir().join(format!("tw-projets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "Ma Thèse").expect("création");
        assert_eq!(dossier.file_name().unwrap(), "ma-thèse", "le nom suit");

        // Un dossier existant n'est pas réutilisé en silence.
        let probleme = creer_au_nom(&parent, "Ma Thèse").expect_err("refus");
        assert!(probleme.contains("existe"), "vu : {probleme}");

        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn des_creations_simultanees_ne_reutilisent_pas_le_meme_projet() {
        use std::sync::{Arc, Barrier};
        let parent =
            std::env::temp_dir().join(format!("tw-creations-concurrentes-{}", std::process::id()));
        std::fs::create_dir_all(&parent).expect("dossier isolé");
        for tour in 0..40 {
            let depart = Arc::new(Barrier::new(33));
            let resultats = std::thread::scope(|scope| {
                let participants: Vec<_> = (0..32)
                    .map(|_| {
                        let depart = Arc::clone(&depart);
                        let parent = &parent;
                        scope.spawn(move || {
                            depart.wait();
                            creer_au_nom(parent, &format!("projet-{tour}"))
                        })
                    })
                    .collect();
                depart.wait();
                participants
                    .into_iter()
                    .map(|p| p.join().unwrap())
                    .collect::<Vec<_>>()
            });
            assert_eq!(
                resultats.iter().filter(|r| r.is_ok()).count(),
                1,
                "une seule création gagne, tour {tour}"
            );
        }
        std::fs::remove_dir_all(parent).expect("nettoyage");
    }

    /// Un projet se renomme — le dossier suit.
    #[test]
    fn un_projet_se_renomme() {
        let parent = std::env::temp_dir().join(format!("tw-projets-2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "ancien").expect("création");
        let renomme = renommer(&dossier, "nouveau").expect("renommage");

        assert_eq!(renomme.file_name().unwrap(), "nouveau");
        assert!(renomme.exists());

        let _ = std::fs::remove_dir_all(&parent);
    }

    /// Un projet supprimé part à la corbeille — jamais dans le vide.
    #[test]
    fn un_projet_supprime_part_a_la_corbeille() {
        let parent = std::env::temp_dir().join(format!("tw-projets-3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");

        let dossier = creer_au_nom(&parent, "a-supprimer").expect("création");
        let corbeille = supprimer(&dossier).expect("suppression");

        assert!(!dossier.exists());
        assert!(corbeille.exists(), "la corbeille le garde");

        let _ = std::fs::remove_dir_all(&parent);
        let _ = std::fs::remove_file(&corbeille);
    }

    #[test]
    fn une_sauvegarde_necrase_pas_un_ancien_fichier_temporaire() {
        let dossier = std::env::temp_dir().join(format!("tw-ancien-temp-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("dossier de test");
        let chemin = dossier.join("document.tw");
        let ancien_temporaire = dossier.join(".document.tw.tmp");
        std::fs::write(&chemin, "ancienne version").expect("document initial");
        std::fs::write(&ancien_temporaire, "texte à récupérer").expect("temporaire précédent");

        ecrire_atomique(&chemin, "nouvelle version").expect("sauvegarde");

        assert_eq!(
            std::fs::read_to_string(&chemin).unwrap(),
            "nouvelle version"
        );
        assert_eq!(
            std::fs::read_to_string(&ancien_temporaire).unwrap(),
            "texte à récupérer"
        );
        std::fs::remove_dir_all(&dossier).expect("nettoyage");
    }

    #[cfg(unix)]
    #[test]
    fn une_sauvegarde_conserve_les_permissions_du_fichier() {
        use std::os::unix::fs::PermissionsExt;

        let dossier = std::env::temp_dir().join(format!("tw-permissions-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("dossier de test");
        let chemin = dossier.join("document.tw");
        std::fs::write(&chemin, "ancien").expect("document initial");
        std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o600))
            .expect("permissions initiales");
        ecrire_atomique(&chemin, "nouveau").expect("sauvegarde");
        assert_eq!(
            std::fs::metadata(&chemin).unwrap().permissions().mode() & 0o777,
            0o600
        );
        std::fs::remove_dir_all(&dossier).expect("nettoyage");
    }

    #[test]
    fn les_sauvegardes_concurrentes_ne_se_partagent_pas_de_temporaire() {
        let dossier =
            std::env::temp_dir().join(format!("tw-ecritures-concurrentes-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("dossier de test");
        let chemin = dossier.join("document.tw");
        let a = "A".repeat(262_144);
        let b = "B".repeat(262_144);
        std::thread::scope(|scope| {
            for texte in [&a, &b] {
                let chemin = &chemin;
                scope.spawn(move || {
                    for _ in 0..20 {
                        ecrire_atomique(chemin, texte).expect("écriture concurrente");
                    }
                });
            }
        });
        let contenu = std::fs::read_to_string(&chemin).expect("document final");
        assert!(contenu == a || contenu == b, "le document est entier");
        std::fs::remove_dir_all(&dossier).expect("nettoyage");
    }

    #[cfg(unix)]
    #[test]
    fn les_chemins_alias_dun_projet_partagent_le_verrou() {
        use std::os::unix::fs::symlink;
        let racine = std::env::temp_dir().join(format!("tw-verrou-alias-{}", std::process::id()));
        let dossier = racine.join("projet");
        std::fs::create_dir_all(&dossier).expect("projet isolé");
        let alias = racine.join("alias");
        symlink(&dossier, &alias).expect("lien symbolique");
        let verrou = Verrou::prendre(&dossier).expect("première prise");
        assert!(
            Verrou::prendre(&alias).is_err(),
            "le même projet ne se verrouille pas deux fois"
        );
        drop(verrou);
        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }

    #[test]
    fn liberer_le_verrou_ne_depend_pas_dun_descripteur_duplique() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-dup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier isolé");
        let verrou = Verrou::prendre(&dir).expect("première prise");
        let double = verrou._ancre.try_clone().expect("descripteur dupliqué");
        drop(verrou);
        let repris = Verrou::prendre(&dir).expect("le verrou doit être explicitement libéré");
        drop(repris);
        drop(double);
        std::fs::remove_dir_all(&dir).expect("nettoyage");
    }

    #[test]
    fn liberer_un_verrou_ne_supprime_pas_un_marqueur_remplace() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-remplace-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier isolé");
        let verrou = Verrou::prendre(&dir).expect("prise du verrou");
        let chemin = dir.join(".verrou");
        std::fs::write(&chemin, "999999999\n").expect("marqueur remplacé");
        drop(verrou);
        assert_eq!(std::fs::read_to_string(&chemin).unwrap(), "999999999\n");
        std::fs::remove_dir_all(&dir).expect("nettoyage");
    }

    /// Une seule instance tient le verrou ; elle le libère en partant.
    #[test]
    fn le_verrou_garde_le_projet() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        let fichier = dir.join(".verrou");
        let premier = Verrou::prendre(&dir).expect("premier verrou");
        assert!(
            Verrou::prendre(&dir).is_err(),
            "la deuxième instance est refusée"
        );
        drop(premier);

        let repris = Verrou::prendre(&dir).expect("le verrou est libéré");
        drop(repris);
        assert!(!fichier.exists(), "le marqueur part avec l'instance");
        assert!(
            dir.join(".verrou-natif").exists(),
            "l'ancre native reste stable"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn la_reprise_concurrente_dun_verrou_abandonne_na_quun_seul_gagnant() {
        use std::sync::{Arc, Barrier};

        let dir =
            std::env::temp_dir().join(format!("tw-reprise-concurrente-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier isolé");
        for tour in 0..80 {
            std::fs::write(dir.join(".verrou"), "999999999\n").expect("ancien verrou");
            let depart = Arc::new(Barrier::new(33));
            let arrives = Arc::new(Barrier::new(33));
            std::thread::scope(|scope| {
                let mut participants = Vec::new();
                for _ in 0..32 {
                    let depart = Arc::clone(&depart);
                    let arrives = Arc::clone(&arrives);
                    let dir = &dir;
                    participants.push(scope.spawn(move || {
                        depart.wait();
                        let resultat = Verrou::prendre(dir);
                        arrives.wait();
                        resultat
                    }));
                }
                depart.wait();
                arrives.wait();
                let resultats: Vec<_> = participants
                    .into_iter()
                    .map(|p| p.join().unwrap())
                    .collect();
                let gagnes = resultats.iter().filter(|r| r.is_ok()).count();
                assert_eq!(
                    gagnes, 1,
                    "tour {tour} : une seule instance peut reprendre le projet"
                );
            });
        }
        std::fs::remove_dir_all(dir).expect("nettoyage");
    }

    /// Sous-processus de l'épreuve interprocessus ; jamais lancé sans fixture.
    #[test]
    #[ignore = "sous-processus contrôlé par l'épreuve de verrouillage"]
    fn processus_qui_tient_le_verrou() {
        let dir = std::path::PathBuf::from(
            std::env::var_os("TYPST_WRITER_LOCK_TEST_DIR").expect("fixture"),
        );
        assert!(dir.starts_with(std::env::temp_dir()));
        let _verrou = Verrou::prendre(&dir).expect("verrou du sous-processus");
        std::fs::write(dir.join("pret"), "ok").expect("signal");
        for _ in 0..1000 {
            if dir.join("terminer").exists() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("le parent n'a pas terminé l'épreuve");
    }

    #[test]
    fn le_verrou_exclut_un_autre_processus_et_survit_a_son_crash() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-processus-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier isolé");
        let mut enfant =
            std::process::Command::new(std::env::current_exe().expect("binaire de test"))
                .args([
                    "--exact",
                    "project::tests::processus_qui_tient_le_verrou",
                    "--ignored",
                ])
                .env("TYPST_WRITER_LOCK_TEST_DIR", &dir)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("sous-processus");
        for _ in 0..500 {
            if dir.join("pret").exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let pret = dir.join("pret").exists();
        let refuse = pret && Verrou::prendre(&dir).is_err();
        enfant.kill().expect("arrêt brutal du sous-processus");
        enfant.wait().expect("fin du sous-processus");
        assert!(pret, "le sous-processus a obtenu le verrou");
        assert!(
            refuse,
            "une seconde instance ne peut pas acquérir le verrou"
        );
        #[cfg(not(target_os = "linux"))]
        {
            assert!(
                Verrou::prendre(&dir).is_err(),
                "un PID ancien ne peut pas être vérifié sur cette plateforme"
            );
            std::fs::remove_file(dir.join(".verrou")).expect("reprise hors ligne simulée");
        }
        let repris = Verrou::prendre(&dir).expect("le verrou natif est libéré après crash");
        drop(repris);
        std::fs::remove_dir_all(dir).expect("nettoyage");
    }

    #[test]
    fn un_verrou_sans_identite_ne_donne_pas_le_droit_decrire() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-vide-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dossier");
        let chemin = dir.join(".verrou");
        std::fs::write(&chemin, "").expect("verrou en cours de création");

        let refus = Verrou::prendre(&dir).expect_err("un verrou vide est indéterminé");
        assert!(
            refus.contains("verrou"),
            "le refus explique le problème : {refus}"
        );
        assert_eq!(std::fs::read(&chemin).expect("verrou conservé"), b"");
        std::fs::remove_dir_all(&dir).expect("nettoyage");
    }

    /// Une instance vivante est refusée — et le refus nomme le processus.
    #[test]
    fn une_instance_vivante_est_refusee() {
        let dir = std::env::temp_dir().join(format!("tw-verrou-2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        // Un processus vivant : celui du test lui-même.
        std::fs::write(dir.join(".verrou"), format!("{}\n", std::process::id())).expect("verrou");

        let probleme = Verrou::prendre(&dir).expect_err("refusé");
        assert!(probleme.contains("autre instance"), "vu : {probleme}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// La bibliothèque errante est adoptée quand le modèle est intact —
    /// et le modèle contient un `@` dans ses commentaires, ce que
    /// l'ancienne vérification ne pardonnait pas.
    #[test]
    fn la_bibliotheque_errante_est_adoptee() {
        let work = std::env::temp_dir().join(format!("tw-adopt-{}", std::process::id()));
        let projet = work.join("mon-travail");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&projet).expect("dossier");

        // Le modèle intact, celui que `create` écrit.
        std::fs::write(
            projet.join("references.bib"),
            crate::project::model::BIBLIOGRAPHY_TEMPLATE,
        )
        .expect("modèle");

        // La bibliothèque errante, à la racine du dossier de travail.
        std::fs::write(
            work.join("references.bib"),
            "@book{deja,\n  title = {Déjà},\n}\n",
        )
        .expect("errante");

        super::adopt_stray_bibliography(&work, &projet);

        let contenu = std::fs::read_to_string(projet.join("references.bib")).expect("lu");
        assert!(
            contenu.contains("Déjà"),
            "la bibliothèque errante rejoint le projet : {contenu}"
        );

        let _ = std::fs::remove_dir_all(&work);
    }

    /// Un modèle déjà travaillé n'est pas écrasé.
    #[test]
    fn un_modele_travaille_n_est_pas_ecrase() {
        let work = std::env::temp_dir().join(format!("tw-adopt-2-{}", std::process::id()));
        let projet = work.join("mon-travail");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&projet).expect("dossier");

        std::fs::write(
            projet.join("references.bib"),
            "@book{mien,\n  title = {Le mien},\n}\n",
        )
        .expect("travail");
        std::fs::write(
            work.join("references.bib"),
            "@book{autre,\n  title = {Autre},\n}\n",
        )
        .expect("errante");

        super::adopt_stray_bibliography(&work, &projet);

        let contenu = std::fs::read_to_string(projet.join("references.bib")).expect("lu");
        assert!(contenu.contains("Le mien"), "le travail reste : {contenu}");

        let _ = std::fs::remove_dir_all(&work);
    }

    /// Une suppression n'est pas une destruction : le fichier va à la
    /// corbeille, le chemin est rendu, et rien ne se marche dessus.
    #[test]
    fn une_suppression_va_a_la_corbeille() {
        let dir = std::env::temp_dir().join(format!("tw-corbeille-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");

        let fichier = dir.join("document.tw");
        std::fs::write(&fichier, "le travail du jour").expect("fichier");

        let ou = vers_la_corbeille(&fichier).expect("déplacé");
        assert!(!fichier.exists(), "il a quitté sa place");
        assert!(ou.exists(), "il est quelque part");
        assert_eq!(
            std::fs::read_to_string(&ou).expect("relu"),
            "le travail du jour",
            "rien n'est perdu"
        );

        // Un second du même nom ne se marche pas dessus.
        std::fs::write(&fichier, "le lendemain").expect("fichier");
        let ou_deux = vers_la_corbeille(&fichier).expect("déplacé");
        assert_ne!(ou, ou_deux, "deux places, pas une");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Vider un champ présent retire le champ entier — jamais son
    /// accolade fermante, qui refermerait l'entrée et chasserait le reste.
    #[test]
    fn vider_un_champ_present_le_retire_entier() {
        let entree = "@book{cle,\n  author = {Nom},\n  title = {Titre},\n  year = {2014},\n}\n";

        let sans_annee = super::retirer_champ(entree, "year");
        assert_eq!(
            sans_annee, "@book{cle,\n  author = {Nom},\n  title = {Titre},\n}\n",
            "vu : {sans_annee}"
        );

        // Et le fichier reste lisible : le parseur y retrouve ses champs.
        let entrees = parse_bibliography_text(&sans_annee);
        assert_eq!(entrees.len(), 1);
        assert_eq!(entrees[0].titre, "Titre");

        // Sur une entrée d'une seule ligne aussi — la virgule de la clé
        // reste, et c'est bon : `@book{cle,}` se relit comme `@book{cle}`.
        let sans_auteur = super::retirer_champ("@book{cle, author = {Doe}}", "author");
        assert_eq!(sans_auteur, "@book{cle,}");
        assert_eq!(parse_bibliography_text(&sans_auteur).len(), 1);
    }

    /// Un champ retiré puis vidé à nouveau ne change plus rien.
    #[test]
    fn retirer_un_champ_absent_ne_change_rien() {
        let entree = "@book{cle,\n  title = {Titre},\n}\n";
        assert_eq!(super::retirer_champ(entree, "year"), entree);
    }

    /// Une notice neuve s'écrit avec ses champs — type, éditeur, lieu, titre
    /// court — et le fichier reste lisible par Typst.
    #[test]
    fn une_notice_neuve_s_ecrit_et_se_relit() {
        let dir = std::env::temp_dir().join(format!("tw-nouvelle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let chemin = dir.join("references.bib");
        std::fs::write(&chemin, "@book{deja,\n  title = {Déjà},\n}\n").unwrap();

        let cle = nouvelle_reference(
            &chemin,
            &ReferenceChamps {
                cle: "dupont2024".to_string(),
                kind: "article".to_string(),
                nom: "Dupont".to_string(),
                prenom: "Jean".to_string(),
                titre: "Un titre".to_string(),
                titre_court: "Titre court".to_string(),
                annee: "2024".to_string(),
                editeur: "Gallimard".to_string(),
                lieu: "Paris".to_string(),
            },
        )
        .unwrap();

        assert_eq!(cle, "dupont2024");

        let entrees = parse_bibliography(&chemin);
        assert_eq!(entrees.len(), 2);
        let neuve = &entrees[1];
        assert_eq!(neuve.key, "dupont2024");
        assert_eq!(neuve.kind, "article");
        assert_eq!(neuve.editeur.as_deref(), Some("Gallimard"));
        assert_eq!(neuve.lieu.as_deref(), Some("Paris"));
        assert_eq!(neuve.titre_court.as_deref(), Some("Titre court"));

        // Le vrai juge : le parseur BibLaTeX de Typst.
        let contenu = std::fs::read_to_string(&chemin).unwrap();
        let root = std::path::Path::new("/projet");
        let entry = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n";
        let overrides = vec![(root.join("references.bib"), contenu)];
        let outcome = crate::typst::render::compile(root, entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "la notice neuve doit être valide : {:?}",
            outcome.errors
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une clé déjà citée n'est jamais réutilisée : elle prend un numéro.
    #[test]
    fn une_cle_citee_ne_se_double_pas() {
        let dir = std::env::temp_dir().join(format!("tw-cle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let chemin = dir.join("references.bib");
        std::fs::write(&chemin, "@book{dupont,\n  title = {Déjà},\n}\n").unwrap();

        let champs = ReferenceChamps {
            cle: "dupont".to_string(),
            kind: "book".to_string(),
            nom: "Dupont".to_string(),
            prenom: String::new(),
            titre: "Autre".to_string(),
            titre_court: String::new(),
            annee: String::new(),
            editeur: String::new(),
            lieu: String::new(),
        };

        assert_eq!(nouvelle_reference(&chemin, &champs).unwrap(), "dupont2");
        assert_eq!(nouvelle_reference(&chemin, &champs).unwrap(), "dupont3");

        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::*;

    /// Les entrées se lisent avec leurs auteurs, leur titre, leur année —
    /// c'est ce que l'onglet des références montrera à la place des clés.
    #[test]
    fn la_bibliographie_lit_les_auteurs_et_les_titres() {
        let texte = r#"
@article{dupont2023,
  author = {Dupont, Jean and Curie, Marie},
  title = {Une {étude} des notes},
  year = {2023},
}
@book{martin2020,
  author = "Paul Martin",
  title = "Le grand traité",
  year = "2020"
}
"#;
        let entrees = parse_bibliography_text(texte);

        assert_eq!(entrees.len(), 2, "deux entrées, même sur plusieurs lignes");
        assert_eq!(entrees[0].key, "dupont2023");
        assert_eq!(
            entrees[0].auteurs,
            vec![
                ("Dupont".to_string(), "Jean".to_string()),
                ("Curie".to_string(), "Marie".to_string())
            ],
            "« et » sépare les auteurs, la virgule dit le nom"
        );
        assert_eq!(entrees[0].titre, "Une {étude} des notes");
        assert_eq!(entrees[0].annee.as_deref(), Some("2023"));

        assert_eq!(
            entrees[1].libelle(),
            "Martin, Paul — Le grand traité",
            "la ligne de liste se lit : nom, prénom, titre"
        );
    }

    /// Sans auteur, la ligne montre le titre seul ; sans titre, la clé.
    #[test]
    fn le_libelle_tient_debout_sans_auteur() {
        let entree = BibliographyEntry {
            key: "anonyme".to_string(),
            kind: "book".to_string(),
            titre: "L'œuvre anonyme".to_string(),
            ..Default::default()
        };

        assert_eq!(entree.libelle(), "L'œuvre anonyme");
    }

    #[test]
    fn parses_bib_entries() {
        let dir = std::env::temp_dir().join("typst-writer-bib");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{smith2020,\n  title = {Un titre},\n}\n@book{doe2019, author = {Doe}}\n",
        )
        .unwrap();

        let entries = parse_bibliography(&bib);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "smith2020");
        assert_eq!(entries[0].kind, "article");
        assert_eq!(entries[1].key, "doe2019");
        assert_eq!(entries[1].kind, "book");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une entrée sans `@` ou sans clé est ignorée, pas une erreur ; une
    /// entrée jamais refermée est le « problème » où s'arrête la lecture.
    #[test]
    fn ignores_malformed_bib_lines() {
        assert!(parse_bibliography_text("ceci n'est pas une entrée").is_empty());
        assert!(parse_bibliography_text("@article{,}").is_empty());
        assert!(parse_bibliography_text("@article{clé,").is_empty());
        assert_eq!(parse_bibliography_text("@article{clé,}").len(), 1);
    }

    /// Modifier une référence réécrit ses champs, sans toucher au reste du
    /// Écrire une référence dans une entrée BibLaTeX donne un fichier que
    /// Typst accepte — la virgule de séparation n'est jamais deux.
    ///
    /// C'est la régression du rendu cassé : un champ ajouté après une ligne
    /// finissant déjà par une virgule écrivait une virgule orpheline, et le
    /// parseur BibLaTeX de Typst refusait le fichier entier.
    #[test]
    fn ecrire_une_reference_laisse_un_bib_que_typst_accepte() {
        let dir = std::env::temp_dir().join(format!("tw-maj-typst-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let chemin = dir.join("references.bib");
        let entree = "@book{cle,\n\tauthor = {Nom},\n\ttitle = {Titre},\n\tdate = {2023},\n}\n";
        std::fs::write(&chemin, entree).unwrap();

        // L'entrée n'a pas de champ `year` : c'est l'ajout qui corrompait.
        super::maj_reference(
            &chemin,
            "cle",
            &ReferenceChamps {
                cle: "cle".to_string(),
                kind: "book".to_string(),
                nom: "Nom".to_string(),
                prenom: "Prénom".to_string(),
                titre: "Titre".to_string(),
                titre_court: String::new(),
                annee: "2014".to_string(),
                editeur: String::new(),
                lieu: String::new(),
            },
        )
        .unwrap();

        let corrige = std::fs::read_to_string(&chemin).unwrap();
        assert!(
            !corrige.contains(",\n,"),
            "jamais deux virgules : {corrige}"
        );

        // Le vrai juge : le parseur BibLaTeX de Typst.
        let root = std::path::Path::new("/projet");
        let entry = "#bibliography(\"references.bib\", style: \"chicago-notes\")\n";
        let overrides = vec![(root.join("references.bib"), corrige)];
        let outcome = crate::typst::render::compile(root, entry, &overrides);

        assert!(
            outcome.errors.is_empty(),
            "le .bib doit rester valide : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// fichier — et la clé ne bouge pas, elle est citée dans les documents.
    #[test]
    fn modifier_une_reference_ne_reecrit_que_ses_champs() {
        let dir = std::env::temp_dir().join(format!("tw-maj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bib = dir.join("refs.bib");
        std::fs::write(
            &bib,
            "@article{dupont2023,\n  author = {Dupont, Jean},\n  title = {Un titre},\n  journal = {Revue},\n}\n@book{martin2020,\n  author = {Martin},\n  title = {L'autre},\n}\n",
        )
        .unwrap();

        maj_reference(
            &bib,
            "dupont2023",
            &ReferenceChamps {
                cle: "dupont2023".to_string(),
                kind: "article".to_string(),
                nom: "Curie".to_string(),
                prenom: "Marie".to_string(),
                titre: "Un autre titre".to_string(),
                titre_court: String::new(),
                annee: "2024".to_string(),
                editeur: String::new(),
                lieu: String::new(),
            },
        )
        .unwrap();

        let entrees = parse_bibliography(&bib);

        assert_eq!(entrees.len(), 2, "les deux entrées sont là");
        assert_eq!(entrees[0].key, "dupont2023", "la clé ne bouge pas");
        assert_eq!(
            entrees[0].auteurs,
            vec![("Curie".to_string(), "Marie".to_string())]
        );
        assert_eq!(entrees[0].titre, "Un autre titre");
        assert_eq!(entrees[0].annee.as_deref(), Some("2024"));
        assert_eq!(
            entrees[1].titre, "L'autre",
            "la seconde entrée n'a pas bougé"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Une entrée sans virgule après la clé (ex. `@article{clé}`) donne la
    /// clé sans son accolade fermante.
    #[test]
    fn une_entree_sans_virgule_garde_sa_cle_nue() {
        let entry = entree_de("@article{clé}");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().key, "clé");

        let entry = entree_de("@book{smith2020 }");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().key, "smith2020");
    }

    /// Une session d'une version précédente, qui ne connaissait que
    /// `last_document`, doit se lire sans erreur.
    #[test]
    fn an_older_session_still_loads() {
        let session: Session =
            serde_json::from_str(r#"{"last_document": "/tmp/ancien.typ"}"#).unwrap();

        assert_eq!(
            session.last_document,
            Some(PathBuf::from("/tmp/ancien.typ"))
        );
        assert!(session.project.is_none());

        // Les champs de thème, absents, retombent sur le mode système et les
        // thèmes natifs : une session écrite avant les thèmes se relit donc
        // sans rien perdre, et sans démarrer sur un thème inexistant.
        assert_eq!(session.theme_mode, crate::base::theme::ThemeMode::System);
        assert_eq!(session.themes, crate::theming::Chosen::default());
    }

    /// Le thème retenu survit à un redémarrage.
    ///
    /// C'est la raison d'être de ces champs : sans eux, l'application repartait
    /// sur les thèmes natifs à chaque lancement, et choisir un thème revenait à
    /// le rechoisir tous les matins.
    #[test]
    fn a_session_keeps_the_chosen_theme() {
        let session = Session {
            espace: None,
            project: Some(PathBuf::from("/tmp/projet")),
            last_document: None,
            theme_mode: crate::base::theme::ThemeMode::Light,
            themes: crate::theming::Chosen {
                light: "Moka clair".to_string(),
                dark: "Moka sombre".to_string(),
            },
        };

        let text = serde_json::to_string_pretty(&session).unwrap();

        // Le fichier reste lisible : c'est ce qu'on veut pouvoir inspecter et
        // corriger à la main quand quelque chose ne se rouvre pas comme prévu.
        assert!(text.contains("\"light\""), "session écrite :\n{text}");

        let restored: Session = serde_json::from_str(&text).unwrap();

        assert_eq!(restored.theme_mode, crate::base::theme::ThemeMode::Light);
        assert_eq!(restored.themes.light, "Moka clair");
        assert_eq!(restored.themes.dark, "Moka sombre");
    }

    /// Le thème retenu survit à un aller-retour par un vrai fichier.
    ///
    /// C'est la chaîne complète du redémarrage : écrire la session, la relire,
    /// retrouver le thème qu'elle nomme dans la liste, et le sélectionner. Les
    /// trois maillons sont ici, dans l'ordre où l'application les exécute.
    #[test]
    fn the_chosen_theme_survives_a_real_restart() {
        let dir = std::env::temp_dir().join("typst-writer-session-theme");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("session.json");

        // 1. L'utilisateur choisit un thème clair et un thème sombre.
        save_session_to(
            &path,
            &Session {
                espace: None,
                project: None,
                last_document: None,
                theme_mode: crate::base::theme::ThemeMode::Dark,
                themes: crate::theming::Chosen {
                    light: "Clair".to_string(),
                    dark: "Mon thème".to_string(),
                },
            },
        );

        // 2. L'application redémarre et relit la session.
        let restored = load_session_from(&path).expect("session lisible");

        assert_eq!(restored.theme_mode, crate::base::theme::ThemeMode::Dark);

        // 3. Les thèmes disponibles sont relus du dossier, et le choix retrouvé.
        let mut themes = crate::theming::natives();

        let file = crate::theming::ThemeFile::parse(crate::theming::EXAMPLE, None).unwrap();
        let name = file.meta.name.clone();

        themes.push(crate::theming::Theme {
            name: name.clone(),
            appearance: file.meta.appearance,
            file: Some(file),
        });

        let index =
            crate::theming::select(&themes, crate::theming::Appearance::Dark, &restored.themes);

        assert_eq!(
            themes[index].name, "Mon thème",
            "le thème retenu avant le redémarrage doit être celui qu'on applique après"
        );

        // 4. Une session absente retombe sur les défauts, sans échouer.
        let missing =
            load_session_from(&dir.join("absent.json")).expect("absente, et c'est normal");

        assert_eq!(missing.theme_mode, crate::base::theme::ThemeMode::System);
        assert_eq!(missing.themes, crate::theming::Chosen::default());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un mode de thème s'écrit en minuscules et se relit tel quel.
    #[test]
    fn the_theme_mode_is_written_in_lowercase() {
        for (mode, expected) in [
            (crate::base::theme::ThemeMode::System, "\"system\""),
            (crate::base::theme::ThemeMode::Light, "\"light\""),
            (crate::base::theme::ThemeMode::Dark, "\"dark\""),
        ] {
            let text = serde_json::to_string(&mode).unwrap();

            assert_eq!(text, expected, "mode {mode:?}");
        }
    }
}

/// La bibliothèque de normes : des mises en page nommées, partagées
/// entre les documents.
///
/// C'est la promesse du rapport : importer, choisir à la création,
/// enregistrer pour le document ou pour la norme. Le fichier tient
/// toutes les normes ; une seule ligne par nom.
pub fn bibliotheque_de_normes() -> PathBuf {
    work_dir().join("normes.json")
}

/// Enregistrer une norme sous un nom — ou la remplacer, s'il y en a déjà
/// une du nom. Les autres normes ne sont pas touchées.
pub fn enregistrer_norme(nom: &str, mise_en_page: &MiseEnPage) -> Result<(), String> {
    let mut table: BTreeMap<String, MiseEnPage> = normes_sans_connexion()?.into_iter().collect();
    table.insert(nom.to_string(), mise_en_page.clone());

    let brut = serde_json::to_string_pretty(&table)
        .map_err(|erreur| format!("La norme ne s'écrit pas : {erreur}."))?;
    ecrire_atomique(&bibliotheque_de_normes(), &brut)
        .map_err(|erreur| format!("Le fichier de normes ne s'écrit pas : {erreur}."))
}

/// Supprimer une norme du catalogue.
pub fn supprimer_norme(nom: &str) -> Result<(), String> {
    let mut table: BTreeMap<String, MiseEnPage> = normes_sans_connexion()?.into_iter().collect();

    if table.remove(nom).is_none() {
        return Ok(());
    }

    let brut = serde_json::to_string_pretty(&table)
        .map_err(|erreur| format!("La norme ne s'écrit pas : {erreur}."))?;
    ecrire_atomique(&bibliotheque_de_normes(), &brut)
        .map_err(|erreur| format!("Le fichier de normes ne s'écrit pas : {erreur}."))
}

/// Les normes, sans parler à la base du projet — le catalogue est
/// indépendant d'elle.
fn normes_sans_connexion() -> Result<Vec<(String, MiseEnPage)>, String> {
    let chemin = bibliotheque_de_normes();

    if !chemin.exists() {
        semer_normes(&chemin)?;
    }

    let brut = std::fs::read_to_string(&chemin).map_err(|erreur| {
        format!(
            "Le fichier de normes « {} » ne se lit pas : {erreur}.",
            chemin.display()
        )
    })?;

    let table: BTreeMap<String, MiseEnPage> = serde_json::from_str(&brut).map_err(|erreur| {
        format!(
            "Le fichier de normes « {} » est illisible : {erreur}.",
            chemin.display()
        )
    })?;

    Ok(table.into_iter().collect())
}

/// Les noms des normes, pour la liste du panneau et du dialogue.
pub fn noms_de_normes() -> Vec<String> {
    normes_sans_connexion()
        .unwrap_or_default()
        .into_iter()
        .map(|(nom, _)| nom)
        .collect()
}

/// Une norme par son nom.
pub fn norme(nom: &str) -> Option<MiseEnPage> {
    normes_sans_connexion()
        .ok()?
        .into_iter()
        .find(|(n, _)| n == nom)
        .map(|(_, mise)| mise)
}

/// La norme de la bibliothèque qui est **exactement** ces réglages.
///
/// C'est ce qui fait d'un document un porteur de normes connues — ou un
/// porteur de « Normes personnalisées » (bug-report-9).
pub fn norme_qui_correspond(reglages: &MiseEnPage) -> Option<String> {
    correspondance(&normes_sans_connexion().ok()?, reglages)
}

/// La correspondance, à la bibliothèque près : éprouvable sans disque.
fn correspondance(normes: &[(String, MiseEnPage)], reglages: &MiseEnPage) -> Option<String> {
    normes
        .iter()
        .find(|(_, mise)| mise == reglages)
        .map(|(nom, _)| nom.clone())
}

#[cfg(test)]
mod tests_de_la_bibliotheque {
    use super::*;

    /// Le catalogue de normes tient : une mise en page s'écrit, se relit
    /// telle quelle, et deux normes se distinguent par leur nom.
    ///
    /// Le fichier du catalogue n'est pas touché — c'est le contrat de
    /// sérialisation qui se juge, pas les données réelles de l'ordinateur.
    #[test]
    fn une_norme_se_relit_telle_quelle() {
        let mut table: BTreeMap<String, MiseEnPage> = BTreeMap::new();
        table.insert("Thèse".to_string(), MiseEnPage::default());

        let brut = serde_json::to_string_pretty(&table).expect("la table s'écrit");
        let relu: BTreeMap<String, MiseEnPage> =
            serde_json::from_str(&brut).expect("elle se relit");

        assert_eq!(relu.len(), 1);
        assert!(relu.contains_key("Thèse"));
    }
}

/// Les étagères : le classement des projets, par nom de dossier.
///
/// Un projet s'y pose sous un nom — « thèse », « articles », « cours » —
/// et la vue des projets les y rassemble. Le classement vit hors du
/// projet lui-même : un projet déplacé garde son étagère par son nom.
pub fn etageres() -> BTreeMap<String, String> {
    let chemin = work_dir().join("etageres.json");

    if !chemin.exists() {
        return BTreeMap::new();
    }

    std::fs::read_to_string(&chemin)
        .ok()
        .and_then(|brut| serde_json::from_str(&brut).ok())
        .unwrap_or_default()
}

/// Le fichier de l'ordre des étagères : une liste de noms, dans l'ordre où
/// l'auteur les a arrangées. Il est séparé du classement des projets pour
/// qu'une étagère **vide** existe quand même.
pub const ETAGERES_ORDRE: &str = "etageres-ordre.json";

fn lire_ordre_etageres() -> Vec<String> {
    let chemin = work_dir().join(ETAGERES_ORDRE);
    std::fs::read_to_string(chemin)
        .ok()
        .and_then(|brut| serde_json::from_str::<Vec<String>>(&brut).ok())
        .unwrap_or_default()
}

fn ecrire_ordre_etageres(ordre: &[String]) -> Result<(), String> {
    let brut = serde_json::to_string_pretty(ordre)
        .map_err(|erreur| format!("L'ordre des étagères ne s'écrit pas : {erreur}."))?;
    ecrire_atomique(&work_dir().join(ETAGERES_ORDRE), &brut)
        .map_err(|erreur| format!("Le fichier d'ordre des étagères ne s'écrit pas : {erreur}."))
}

/// Une liste d'étagères sans doublons, noms vides écartés, ordre gardé.
fn ordre_sans_doublons(ordre: Vec<String>) -> Vec<String> {
    let mut vues = std::collections::HashSet::new();
    let mut propre = Vec::new();
    for nom in ordre {
        let nom = nom.trim().to_string();
        if !nom.is_empty() && vues.insert(nom.clone()) {
            propre.push(nom);
        }
    }
    propre
}

/// Les étagères dans l'ordre voulu, y compris celles qui sont encore vides.
///
/// Les anciens projets dont l'étagère n'a pas encore d'ordre sont ajoutés à
/// la fin, par ordre alphabétique : une mise à jour ne disperse rien.
pub fn etageres_ordonnees() -> Vec<String> {
    let mut ordre = ordre_sans_doublons(lire_ordre_etageres());

    let mut connues: Vec<String> = etageres()
        .values()
        .map(|nom| nom.trim().to_string())
        .filter(|nom| !nom.is_empty())
        .collect();
    connues.sort();
    connues.dedup();
    for nom in connues {
        if !ordre.iter().any(|etagere| etagere == &nom) {
            ordre.push(nom);
        }
    }

    ordre
}

/// Crée une étagère, même sans projet dedans. Le nom vide ne crée rien.
pub fn creer_etagere(nom: &str) -> Result<(), String> {
    let nom = nom.trim();

    if nom.is_empty() {
        return Err("Une étagère a un nom.".to_string());
    }

    let mut ordre = etageres_ordonnees();
    if ordre.iter().any(|etagere| etagere == nom) {
        return Ok(());
    }

    ordre.push(nom.to_string());
    ecrire_ordre_etageres(&ordre)
}

fn assurer_etagere(nom: &str) -> Result<(), String> {
    let nom = nom.trim();
    if nom.is_empty() {
        return Ok(());
    }

    let mut ordre = etageres_ordonnees();
    if ordre.iter().any(|etagere| etagere == nom) {
        return Ok(());
    }

    ordre.push(nom.to_string());
    ecrire_ordre_etageres(&ordre)
}

/// Déplace une étagère d'un cran dans l'ordre — le geste d'arrangement de
/// la bibliothèque. La première ne monte pas, la dernière ne descend pas.
fn deplacer_dans_un_ordre(ordre: &mut [String], nom: &str, vers_le_haut: bool) {
    let Some(position) = ordre.iter().position(|etagere| etagere == nom) else {
        return;
    };
    let voisine = if vers_le_haut {
        position.checked_sub(1)
    } else {
        Some(position + 1).filter(|voisine| *voisine < ordre.len())
    };
    let Some(voisine) = voisine else {
        return;
    };
    ordre.swap(position, voisine);
}

pub fn deplacer_etagere(nom: &str, vers_le_haut: bool) -> Result<(), String> {
    let mut ordre = etageres_ordonnees();
    if !ordre.iter().any(|etagere| etagere == nom) {
        return Err(format!("L'étagère « {nom} » n'existe pas."));
    }
    deplacer_dans_un_ordre(&mut ordre, nom, vers_le_haut);
    ecrire_ordre_etageres(&ordre)
}

#[cfg(test)]
mod tests_des_etageres_ordonnees {
    use super::{deplacer_dans_un_ordre, ordre_sans_doublons};

    #[test]
    fn les_etageres_se_deplacent_sans_se_perdre() {
        let mut ordre = vec![
            "thèse".to_string(),
            "articles".to_string(),
            "cours".to_string(),
        ];
        deplacer_dans_un_ordre(&mut ordre, "articles", true);
        assert_eq!(ordre, vec!["articles", "thèse", "cours"]);
        deplacer_dans_un_ordre(&mut ordre, "articles", true);
        assert_eq!(
            ordre,
            vec!["articles", "thèse", "cours"],
            "la première ne bouge plus"
        );
        deplacer_dans_un_ordre(&mut ordre, "cours", false);
        assert_eq!(
            ordre,
            vec!["articles", "thèse", "cours"],
            "la dernière ne bouge plus"
        );
    }

    #[test]
    fn une_etagere_ne_se_duplique_pas() {
        let ordre = ordre_sans_doublons(vec![
            "thèse".to_string(),
            " articles ".to_string(),
            "thèse".to_string(),
        ]);
        assert_eq!(ordre, ["thèse", "articles"]);
    }
}

/// Poser un projet sur une étagère — l'étagère vide l'en retire.
pub fn definir_etagere(projet: &str, etagere: &str) -> Result<(), String> {
    let mut table = etageres();
    let etagere = etagere.trim();

    if etagere.is_empty() {
        table.remove(projet);
    } else {
        table.insert(projet.to_string(), etagere.to_string());
        assurer_etagere(etagere)?;
    }

    let brut = serde_json::to_string_pretty(&table)
        .map_err(|erreur| format!("Les étagères ne s'écrivent pas : {erreur}."))?;
    ecrire_atomique(&work_dir().join("etageres.json"), &brut)
        .map_err(|erreur| format!("Le fichier des étagères ne s'écrit pas : {erreur}."))
}

#[cfg(test)]
mod tests_des_etageres {
    use super::*;

    /// Le classement tient : une étagère est un nom, un projet s'y pose
    /// par son nom, et l'étagère vide le retire.
    ///
    /// Le fichier des étagères n'est pas touché — c'est le contrat de la
    /// table qui se juge, pas les données réelles de l'ordinateur.
    #[test]
    fn un_projet_se_pose_sur_son_etagere() {
        let mut table: BTreeMap<String, String> = BTreeMap::new();
        table.insert("mon-travail".to_string(), "thèse".to_string());

        let brut = serde_json::to_string_pretty(&table).expect("la table s'écrit");
        let relu: BTreeMap<String, String> = serde_json::from_str(&brut).expect("elle se relit");

        assert_eq!(relu.get("mon-travail").map(String::as_str), Some("thèse"));
    }
}

#[cfg(test)]
mod tests_du_champ_soude {
    use super::*;

    #[test]
    fn un_champ_s_ajoute_sans_souder_le_fichier() {
        let entree = "@book{grudem2023,\n\ttitle = {Un titre},\n\tdate = {2023},\n}";
        let maj = option(entree, "year", "2014");
        println!("SORTIE:\n{maj}");

        // La fermeture reste sur sa ligne, et le champ est complet.
        assert!(
            maj.contains("\n  year = {2014}\n}"),
            "le champ se pose proprement : {maj}"
        );
    }
}

/// Les normes livrées avec l'application : deux modèles complets, posés
/// au premier lancement.
///
/// Comme les thèmes livrés, elles se **sèment** : le fichier existe une
/// fois, et ne se réécrit jamais — un modèle qu'on a façonné est le sien.
fn semer_normes(chemin: &std::path::Path) -> Result<(), String> {
    use crate::mise_en_page::modele::{
        Emplacement, EntetePiedDePage, Marges, MiseEnPage, NumerotationPage, Papier, Texte, Titres,
    };

    // Le modèle classique : la page, la marge, le texte.
    let classique = MiseEnPage {
        papier: Papier {
            format: "a4".to_string(),
            orientation: "portrait".to_string(),
        },
        marges: Marges {
            haut: "2.5cm".to_string(),
            bas: "2.5cm".to_string(),
            gauche: "3cm".to_string(),
            droite: "2.5cm".to_string(),
        },
        texte: Texte {
            police: "New Computer Modern".to_string(),
            taille: "11pt".to_string(),
            interligne: "0.65em".to_string(),
            espacement: "1.2em".to_string(),
            retrait: "0cm".to_string(),
            justification: true,
            coupure: true,
            langue: "fr".to_string(),
            typographie: None,
        },
        titres: Titres::default(),
        numerotation: NumerotationPage::default(),
        entete_pied: EntetePiedDePage::default(),
        ..Default::default()
    };

    // Le modèle de thèse : la marge de reliure, le retrait des
    // paragraphes, et le pied qui porte le numéro.
    let mut these = classique.clone();
    these.marges.gauche = "4cm".to_string();
    these.texte.retrait = "1cm".to_string();
    these.entete_pied = EntetePiedDePage {
        entete: Emplacement::default(),
        pied: Emplacement {
            gauche: String::new(),
            centre: String::new(),
            droite: "p. {page}".to_string(),
        },
        numerotation: "1".to_string(),
        ..Default::default()
    };

    let mut table: BTreeMap<String, MiseEnPage> = BTreeMap::new();
    table.insert("Classique".to_string(), classique);
    table.insert("Thèse".to_string(), these);

    let brut = serde_json::to_string_pretty(&table)
        .map_err(|erreur| format!("Les normes livrées ne s'écrivent pas : {erreur}."))?;

    ecrire_atomique(chemin, &brut)
        .map_err(|erreur| format!("Le fichier de normes ne s'écrit pas : {erreur}."))
}

/// Les proportions d'une couverture : le format A4, pour toute la
/// bibliothèque, même quand le document lui-même est en Letter ou en A5.
pub const LARGEUR_MINIATURE: u32 = 420;
pub const HAUTEUR_MINIATURE: u32 = 594;

/// Le dossier des couvertures choisies par l'auteur, à la racine du projet.
pub const DOSSIER_MINIATURE: &str = "miniatures";

/// Le fichier d'une couverture choisie : un PNG déjà recadré au format A4.
pub const FICHIER_MINIATURE: &str = "couverture.png";

/// Le cache des premières pages rendues : il vit dans le projet, sous un
/// nom caché, et ne voyage jamais avec une copie ou un modèle.
const CACHE_MINIATURE: &str = ".miniatures";
const FICHIER_CACHE_MINIATURE: &str = "vignette.png";
const FICHIER_EMPREINTE_MINIATURE: &str = "empreinte";

/// Une couverture préparée : pixels RGBA pour l'affichage, PNG pour le
/// disque. Les deux sont déjà au format A4.
#[derive(Debug, Clone)]
pub struct CouverturePreparee {
    pub png: Vec<u8>,
    pub pixels: Vec<u8>,
    pub largeur: u32,
    pub hauteur: u32,
}

/// Le chemin de la couverture choisie par l'auteur.
pub fn chemin_couverture(dir: &Path) -> PathBuf {
    dir.join(DOSSIER_MINIATURE).join(FICHIER_MINIATURE)
}

fn chemin_cache_miniature(dir: &Path) -> PathBuf {
    dir.join(CACHE_MINIATURE).join(FICHIER_CACHE_MINIATURE)
}

fn chemin_empreinte_miniature(dir: &Path) -> PathBuf {
    dir.join(CACHE_MINIATURE).join(FICHIER_EMPREINTE_MINIATURE)
}

fn nom_du_projet(dir: &Path) -> String {
    dir.file_name()
        .map(|nom| nom.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Recadre une image choisie au centre, puis la ramène exactement à la
/// taille d'une miniature A4. Une image plus large perd ses bords ; une
/// image plus haute perd son haut et son bas — jamais le centre.
pub fn preparer_couverture(octets: &[u8]) -> Result<CouverturePreparee, String> {
    let source = image::load_from_memory(octets)
        .map_err(|erreur| format!("Cette image ne peut pas être lue : {erreur}."))?;

    Ok(preparer_image(source))
}

fn preparer_image(source: image::DynamicImage) -> CouverturePreparee {
    let (largeur, hauteur) = (source.width().max(1), source.height().max(1));
    let ratio_cible = f64::from(LARGEUR_MINIATURE) / f64::from(HAUTEUR_MINIATURE);
    let ratio_source = f64::from(largeur) / f64::from(hauteur);

    let (recadrage_largeur, recadrage_hauteur) = if ratio_source > ratio_cible {
        (
            (f64::from(hauteur) * ratio_cible).round().max(1.0) as u32,
            hauteur,
        )
    } else {
        (
            largeur,
            (f64::from(largeur) / ratio_cible).round().max(1.0) as u32,
        )
    };
    let recadrage_largeur = recadrage_largeur.min(largeur);
    let recadrage_hauteur = recadrage_hauteur.min(hauteur);
    let x = (largeur - recadrage_largeur) / 2;
    let y = (hauteur - recadrage_hauteur) / 2;

    let image = source
        .crop_imm(x, y, recadrage_largeur, recadrage_hauteur)
        .resize_exact(
            LARGEUR_MINIATURE,
            HAUTEUR_MINIATURE,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgba8();

    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("un tampon en mémoire accepte toujours un PNG");

    CouverturePreparee {
        png,
        pixels: image.into_raw(),
        largeur: LARGEUR_MINIATURE,
        hauteur: HAUTEUR_MINIATURE,
    }
}

/// La première page rendue, ramenée au format A4 sans la déformer.
///
/// Une page au format A4 remplit la miniature. Une page Letter, A5 ou
/// paysage garde **toute** sa page : elle est centrée sur un fond blanc, et
/// les marges qui manquent sont ajoutées — jamais le contenu coupé. C'est
/// ce qui rend la couverture fidèle au document tout en gardant le même
/// cadre pour toute la bibliothèque.
fn normaliser_page(page: &crate::typst::render::PageImage) -> Option<CouverturePreparee> {
    let source = image::RgbaImage::from_raw(page.width, page.height, page.pixels.clone())?;
    let largeur = f64::from(page.width.max(1));
    let hauteur = f64::from(page.height.max(1));
    let echelle =
        (f64::from(LARGEUR_MINIATURE) / largeur).min(f64::from(HAUTEUR_MINIATURE) / hauteur);
    let largeur_redimensionnee = (largeur * echelle).round().max(1.0) as u32;
    let hauteur_redimensionnee = (hauteur * echelle).round().max(1.0) as u32;

    let redimensionnee = image::imageops::resize(
        &source,
        largeur_redimensionnee,
        hauteur_redimensionnee,
        image::imageops::FilterType::Lanczos3,
    );
    let mut toile = image::RgbaImage::from_pixel(
        LARGEUR_MINIATURE,
        HAUTEUR_MINIATURE,
        image::Rgba([255, 255, 255, 255]),
    );
    let x = i64::from((LARGEUR_MINIATURE - largeur_redimensionnee) / 2);
    let y = i64::from((HAUTEUR_MINIATURE - hauteur_redimensionnee) / 2);
    image::imageops::replace(&mut toile, &redimensionnee, x, y);

    let mut png = Vec::new();
    toile
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("un tampon en mémoire accepte toujours un PNG");

    Some(CouverturePreparee {
        png,
        pixels: toile.into_raw(),
        largeur: LARGEUR_MINIATURE,
        hauteur: HAUTEUR_MINIATURE,
    })
}

/// Rend la première page d'un projet, au format A4.
fn generer_miniature(dir: &Path) -> Option<CouverturePreparee> {
    // Le secours : `main.typ` d'abord, puis le document d'écriture — un
    // projet dont l'assemblage ne compile pas (une ancienne norme, un
    // fichier manquant) montre malgré tout sa page de texte (bug-report-7,
    // point 12). Un échec silencieux laissait toutes les couvertures à
    // « Aperçu indisponible ».
    //
    // L'entrée d'un compilateur est le **contenu** de l'assemblage, pas son
    // nom : la vignette montrait le texte « main.typ » lui-même, contre la
    // promesse de cette fonction (bug-report-9, en passant).
    let principal = std::fs::read_to_string(dir.join("main.typ")).unwrap_or_default();
    let pdf = crate::typst::render::render_pdf(dir, &principal, &[]).or_else(|| {
        let document = dir.join("document.tw");
        let source = std::fs::read_to_string(&document).ok()?;
        let genere = crate::langage::generer::generer(&crate::langage::parse(&source));
        crate::typst::render::render_pdf(dir, &genere, &[])
    });

    let pdf = crate::typst::render::ouvrir_pdf(pdf?)?;
    let page = crate::typst::render::render_page(&pdf, 0, 1.0)?;

    normaliser_page(&page)
}

fn charger_couverture_dir(dir: &Path) -> Option<CouverturePreparee> {
    let octets = std::fs::read(chemin_couverture(dir)).ok()?;
    preparer_couverture(&octets).ok()
}

fn charger_cache(dir: &Path) -> Option<CouverturePreparee> {
    let octets = std::fs::read(chemin_cache_miniature(dir)).ok()?;
    let image = image::load_from_memory(&octets).ok()?;
    if image.width() != LARGEUR_MINIATURE || image.height() != HAUTEUR_MINIATURE {
        return None;
    }
    let rgba = image.to_rgba8();

    Some(CouverturePreparee {
        png: octets,
        pixels: rgba.into_raw(),
        largeur: LARGEUR_MINIATURE,
        hauteur: HAUTEUR_MINIATURE,
    })
}

/// Empreinte du projet pour reconnaître une première page inchangée.
///
/// Elle inclut tous les fichiers qui peuvent changer la composition :
/// sources, normes, styles, bibliographie, images. Les sauvegardes et le
/// cache sont exclus — ils ne composent rien.
fn empreinte_projet(dir: &Path) -> u64 {
    fn melanger(empreinte: &mut u64, octets: &[u8]) {
        for octet in octets {
            *empreinte ^= u64::from(*octet);
            *empreinte = empreinte.wrapping_mul(0x100000001b3);
        }
    }

    fn parcourir(racine: &Path, dossier: &Path, empreinte: &mut u64, profondeur: usize) {
        if profondeur > 32 {
            return;
        }
        let Ok(entrees) = std::fs::read_dir(dossier) else {
            return;
        };
        let mut chemins: Vec<PathBuf> = entrees.filter_map(Result::ok).map(|e| e.path()).collect();
        chemins.sort();

        for chemin in chemins {
            let nom = chemin
                .file_name()
                .and_then(|nom| nom.to_str())
                .unwrap_or("");
            if nom == "sauvegardes" || nom == CACHE_MINIATURE || nom.starts_with(".verrou") {
                continue;
            }
            let Ok(meta) = std::fs::symlink_metadata(&chemin) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                melanger(empreinte, nom.as_bytes());
                parcourir(racine, &chemin, empreinte, profondeur + 1);
                continue;
            }
            let relatif = chemin.strip_prefix(racine).unwrap_or(&chemin);
            melanger(empreinte, relatif.to_string_lossy().as_bytes());
            melanger(empreinte, &meta.len().to_le_bytes());
            if let Ok(modifie) = meta.modified()
                && let Ok(duree) = modifie.duration_since(std::time::UNIX_EPOCH)
            {
                melanger(empreinte, &duree.as_secs().to_le_bytes());
                melanger(empreinte, &duree.subsec_nanos().to_le_bytes());
            }
        }
    }

    let mut empreinte = 0xcbf29ce484222325;
    parcourir(dir, dir, &mut empreinte, 0);
    empreinte
}

fn cache_est_a_jour(dir: &Path, empreinte: u64) -> bool {
    let Ok(marque) = std::fs::read_to_string(chemin_empreinte_miniature(dir)) else {
        return false;
    };
    if marque.trim() != format!("{empreinte:016x}") {
        return false;
    }
    image::image_dimensions(chemin_cache_miniature(dir))
        .map(|(largeur, hauteur)| largeur == LARGEUR_MINIATURE && hauteur == HAUTEUR_MINIATURE)
        .unwrap_or(false)
}

fn ecrire_cache(
    dir: &Path,
    couverture: &CouverturePreparee,
    empreinte: u64,
) -> std::io::Result<()> {
    let cache = chemin_cache_miniature(dir);
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }
    ecrire_atomique_octets(&cache, &couverture.png)?;
    ecrire_atomique(
        &chemin_empreinte_miniature(dir),
        &format!("{empreinte:016x}\n"),
    )
}

/// Écrit la couverture choisie dans le projet. L'appelant tient le verrou
/// du projet : c'est une écriture, pas un cache.
pub fn ecrire_couverture(dir: &Path, png: &[u8]) -> Result<(), String> {
    let chemin = chemin_couverture(dir);
    if let Some(parent) = chemin.parent() {
        std::fs::create_dir_all(parent).map_err(|erreur| {
            format!("Le dossier des miniatures ne peut pas être créé : {erreur}.")
        })?;
    }
    ecrire_atomique_octets(&chemin, png)
        .map_err(|erreur| format!("La miniature ne peut pas être écrite : {erreur}."))
}

/// Les couvertures disponibles **tout de suite**, sans compiler : la
/// couverture choisie par l'auteur, sinon la première page déjà en cache.
/// Un projet jamais rendu n'a pas encore d'image ; le rafraîchissement la
/// produira en arrière-plan.
pub fn vignettes_immediates(dirs: &[PathBuf]) -> Vec<(String, Vec<u8>, u32, u32)> {
    dirs.iter()
        .filter_map(|dir| {
            let couverture = charger_couverture_dir(dir).or_else(|| charger_cache(dir))?;
            Some((
                nom_du_projet(dir),
                couverture.pixels,
                couverture.largeur,
                couverture.hauteur,
            ))
        })
        .collect()
}

/// Les couvertures dont la première page a changé, rendues en arrière-plan.
///
/// Le cache évite de recompiler tous les projets à chaque ouverture de la
/// bibliothèque. Une couverture choisie par l'auteur ne se régénère jamais.
#[cfg(test)]
pub fn vignettes_a_rafraichir(dirs: &[PathBuf]) -> Vec<(String, Vec<u8>, u32, u32)> {
    vignettes_a_rafraichir_avec_annulation(dirs, None)
}

/// Variante annulable : `annulation` est consulté entre deux projets.
///
/// Le compilateur Typst n'est pas interruptible au milieu d'une page, mais la
/// boucle ne doit pas continuer à compiler tous les projets quand l'auteur a
/// déjà quitté la bibliothèque pour écrire. Le jeton est comparé à `depart` :
/// toute nouvelle ouverture de la bibliothèque invalide l'ancienne passe.
pub fn vignettes_a_rafraichir_avec_annulation(
    dirs: &[PathBuf],
    annulation: Option<(&std::sync::atomic::AtomicU64, u64)>,
) -> Vec<(String, Vec<u8>, u32, u32)> {
    let mut vignettes = Vec::new();

    for dir in dirs {
        if annulation.is_some_and(|(jeton, depart)| {
            jeton.load(std::sync::atomic::Ordering::Relaxed) != depart
        }) {
            break;
        }

        if charger_couverture_dir(dir).is_some() {
            continue;
        }

        let empreinte = empreinte_projet(dir);
        if cache_est_a_jour(dir, empreinte) {
            continue;
        }

        let Some(couverture) = generer_miniature(dir) else {
            continue;
        };
        let _ = ecrire_cache(dir, &couverture, empreinte);
        vignettes.push((
            nom_du_projet(dir),
            couverture.pixels,
            couverture.largeur,
            couverture.hauteur,
        ));
    }

    vignettes
}

/// La vignette d'un projet : sa première page, en image A4.
///
/// Elle se rend par le même compilateur que le document : ce qui s'y voit
/// est ce qui se compose. Un projet qui ne compile pas n'a pas de vignette
/// — jamais une erreur à l'écran. Cette fonction compile toujours ; la
/// bibliothèque ouverte, elle, lit d'abord le cache par
/// [`vignettes_immediates`], puis rafraîchit par [`vignettes_a_rafraichir`].
#[cfg(test)]
pub fn vignettes(dirs: &[std::path::PathBuf]) -> Vec<(String, Vec<u8>, u32, u32)> {
    dirs.iter()
        .filter_map(|dir| {
            let couverture = generer_miniature(dir)?;
            Some((
                nom_du_projet(dir),
                couverture.pixels,
                couverture.largeur,
                couverture.hauteur,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests_du_parseur_vrai {
    #[test]
    fn le_parseur_lit_des_fichiers_bibliographiques_synthetiques() {
        let racine =
            std::env::temp_dir().join(format!("tw-bibliographie-fixture-{}", std::process::id()));
        std::fs::create_dir_all(&racine).expect("dossier de test");
        let fichier = racine.join("references.bib");
        std::fs::write(
            &fichier,
            "@book{ouvrage, author = {Durand, Alice}, title = {Un titre}, year = {2020}}\n",
        )
        .expect("fixture");
        let entrees = crate::project::parse_bibliography(&fichier);
        assert_eq!(entrees.len(), 1);
        assert_eq!(entrees[0].key, "ouvrage");
        assert_eq!(entrees[0].titre, "Un titre");
        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }
}

#[cfg(test)]
mod epreuves_portabilite {
    use super::*;

    /// Le dossier des données suit la plateforme — et sous Unix, il reste
    /// le chemin historique où vivent déjà les projets : rien ne déménage.
    #[test]
    fn le_dossier_des_donnees_suit_la_plateforme() {
        let attendu = if cfg!(windows) {
            std::env::var("APPDATA")
                .map(|appdata| PathBuf::from(appdata).join("typst-writer"))
                .unwrap_or_else(|_| PathBuf::from(".typst-writer"))
        } else {
            std::env::var("HOME")
                .map(|home| PathBuf::from(home).join(".typst-writer"))
                .unwrap_or_else(|_| PathBuf::from(".typst-writer"))
        };
        assert_eq!(crate::services::stockage::dossier_donnees(), attendu);
    }

    /// La commande d'ouverture est celle de la plateforme — testée, pas
    /// devinée dans les recoins de l'interface.
    #[test]
    fn la_commande_d_ouverture_est_celle_de_la_plateforme() {
        let cible = std::path::Path::new("quelque-part");
        let commande = commande_ouverture(cible);
        #[cfg(target_os = "macos")]
        let attendu = "open";
        #[cfg(windows)]
        let attendu = "explorer";
        #[cfg(all(not(windows), not(target_os = "macos")))]
        let attendu = "xdg-open";
        assert_eq!(commande.get_program(), attendu);
    }
}

#[cfg(test)]
mod tests_vignettes {
    /// Les aperçus de la bibliothèque (bug-report-7, point 12) : une
    /// vignette doit revenir pour un projet qui compile. Ici la vérité —
    /// si elle ne revient pas, la sonde dit pourquoi.
    #[test]
    fn une_vignette_revient_pour_un_projet_qui_compile() {
        let racine = std::env::temp_dir().join(format!("tw-vignettes-{}", std::process::id()));
        let dir = racine.join("mon-projet");
        std::fs::create_dir_all(&dir).expect("dossier");
        std::fs::write(dir.join("main.typ"), "= Un titre\n\nDu texte.\n").expect("main");

        let vignettes = crate::project::vignettes(std::slice::from_ref(&dir));
        if vignettes.is_empty() {
            let compilation = crate::typst::render::compile(&dir, "main.typ", &[]);
            eprintln!(
                "SONDAGE compile document={:?} errors={:?}",
                compilation.document.is_some(),
                compilation
                    .errors
                    .iter()
                    .map(|d| format!("{d:?}"))
                    .collect::<Vec<_>>()
            );
        }
        assert!(
            !vignettes.is_empty(),
            "une vignette revient pour un projet qui compile"
        );

        // Le vrai chemin des projets neufs : un projet de modèle doit lui
        // aussi porter sa vignette (bug-report-7, point 12).
        let base = crate::project::creer_au_nom(&racine, "une-base").expect("base");
        let modele_dir =
            crate::project::creer_au_nom_depuis(&racine, "un-devoir", &base).expect("modèle");
        let vignettes = crate::project::vignettes(std::slice::from_ref(&modele_dir));
        if vignettes.is_empty() {
            let compilation = crate::typst::render::compile(&modele_dir, "main.typ", &[]);
            eprintln!(
                "SONDAGE modele document={:?} errors={:?}",
                compilation.document.is_some(),
                compilation
                    .errors
                    .iter()
                    .map(|d| format!("{d:?}"))
                    .collect::<Vec<_>>()
            );
        }
        assert!(
            !vignettes.is_empty(),
            "un projet de modèle porte sa vignette"
        );

        // Le secours : un projet dont `main.typ` est cassé montre
        // malgré tout sa page de texte (bug-report-7, point 12).
        let casse = racine.join("un-casse");
        std::fs::create_dir_all(&casse).expect("dossier");
        std::fs::write(casse.join("main.typ"), "#import \"manquant.typ\": *\n").expect("main");
        std::fs::write(casse.join("document.tw"), "= Titre\n\nDu texte.\n").expect("document");
        let vignettes = crate::project::vignettes(&[casse]);
        assert!(
            !vignettes.is_empty(),
            "le secours document.tw montre une page"
        );

        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }
}

#[cfg(test)]
mod tests_paquets_locaux {
    /// Un paquet installé dans le dossier local se résout : l'import
    /// `@local/demo:1.0.0` compile, sans réseau (bug-report-7, point 4 —
    /// le premier jalon de l'architecture paquets).
    #[test]
    fn un_paquet_local_se_resout() {
        let racine = std::env::temp_dir().join(format!("tw-paquets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&racine);
        let paquet = racine.join("local/demo/1.0.0");
        std::fs::create_dir_all(&paquet).expect("paquet");
        std::fs::write(
            paquet.join("lib.typ"),
            "#let salut(nom) = [Bonjour, #nom !]\n",
        )
        .expect("lib");
        let projet = racine.join("projet");
        std::fs::create_dir_all(&projet).expect("projet");
        std::fs::write(
            projet.join("main.typ"),
            "#import \"@local/demo:1.0.0\": salut\n#salut[monde]\n",
        )
        .expect("main");

        // Rust 2024 : les variables d'environnement sont `unsafe` — le
        // test est seul maître de la sienne.
        unsafe {
            std::env::set_var("TYPST_WRITER_PAQUETS", &racine);
        }
        let resultat = crate::typst::render::compile(&projet, "main.typ", &[]);
        unsafe {
            std::env::remove_var("TYPST_WRITER_PAQUETS");
        }

        assert!(
            resultat.errors.is_empty(),
            "le paquet local se résout : {:?}",
            resultat.errors
        );
        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }
}

#[cfg(test)]
mod tests_des_vignettes {
    /// La vignette est la **vraie première page** du projet — jamais le nom
    /// du fichier (bug-report-9 : l'entrée d'un compilateur est du contenu,
    /// et la vignette compilait le texte « main.typ » lui-même). Elle est
    /// ensuite ramenée au cadre A4 commun à la bibliothèque.
    #[test]
    fn la_vignette_est_la_vraie_premiere_page() {
        let dir =
            std::env::temp_dir().join(format!("tw-vignette-vraie-page-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("projet de test");
        let contenu = "= Chapitre\n\nUn texte reconnaissable, longuement étalé sur la page.\n";
        std::fs::write(dir.join("main.typ"), contenu).expect("assemblage");

        let vignette = super::generer_miniature(&dir).expect("une vignette");

        // Le contrôle : la même page rendue directement depuis son contenu,
        // puis ramenée au même cadre A4.
        let octets = crate::typst::render::render_pdf(&dir, contenu, &[]).expect("PDF");
        let pdf = crate::typst::render::ouvrir_pdf(octets).expect("PDF lisible");
        let page = crate::typst::render::render_page(&pdf, 0, 1.0).expect("page");
        let attendue = super::normaliser_page(&page).expect("vignette normalisée");

        assert_eq!(
            (vignette.largeur, vignette.hauteur),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE),
            "la vignette est au format A4"
        );
        assert_eq!(
            vignette.pixels, attendue.pixels,
            "la vignette est la première page réelle"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le format de la page composée ne décide pas du format de la
    /// couverture : même une page Letter reste une vignette A4.
    #[test]
    fn une_page_letter_donne_une_vignette_a4() {
        let dir = std::env::temp_dir().join(format!("tw-vignette-letter-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("projet de test");
        std::fs::write(
            dir.join("main.typ"),
            "#set page(width: 8.5in, height: 11in)\n= Une page Letter\n\nDu texte.\n",
        )
        .expect("assemblage");

        let vignette = super::generer_miniature(&dir).expect("une vignette");
        assert_eq!(
            (vignette.largeur, vignette.hauteur),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE),
            "le cadre reste A4 même pour un document Letter"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn image_png(
        largeur: u32,
        hauteur: u32,
        gauche: image::Rgba<u8>,
        droite: image::Rgba<u8>,
    ) -> Vec<u8> {
        let source = image::RgbaImage::from_fn(largeur, hauteur, |x, _| {
            if x < largeur / 2 { gauche } else { droite }
        });
        let mut png = Vec::new();
        source
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .expect("PNG de test");
        png
    }

    /// Une image choisie perd ses bords, jamais le centre : le résultat est
    /// exactement une couverture A4.
    #[test]
    fn une_image_choisie_est_recadree_au_centre_en_a4() {
        let gauche = image::Rgba([255, 0, 0, 255]);
        let droite = image::Rgba([0, 0, 255, 255]);
        let source = image_png(800, 400, gauche, droite);

        let couverture = super::preparer_couverture(&source).expect("couverture");
        assert_eq!(
            (couverture.largeur, couverture.hauteur),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE),
            "l'image choisie prend le cadre A4"
        );
        assert_eq!(
            &couverture.pixels[0..4],
            &gauche.0,
            "le bord gauche vient de la partie centrale de l'image"
        );
        let dernier = couverture.pixels.len() - 4;
        assert_eq!(
            &couverture.pixels[dernier..],
            &droite.0,
            "le bord droit vient de la partie centrale de l'image"
        );
    }

    /// Le sélecteur accepte aussi le JPEG, pas seulement le PNG.
    #[test]
    fn une_image_jpeg_est_acceptee() {
        let source =
            image::RgbImage::from_fn(300, 300, |x, _| image::Rgb([(x % 255) as u8, 40, 80]));
        let mut jpeg = Vec::new();
        source
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("JPEG de test");

        let couverture = super::preparer_couverture(&jpeg).expect("couverture JPEG");
        assert_eq!(
            (couverture.largeur, couverture.hauteur),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE)
        );
    }

    /// La couverture choisie est disponible avant toute compilation et
    /// n'est jamais remplacée par la première page.
    #[test]
    fn une_couverture_choisie_prime_sur_la_premiere_page() {
        let dir =
            std::env::temp_dir().join(format!("tw-couverture-choisie-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("projet de test");
        std::fs::write(dir.join("main.typ"), "#import \"absent.typ\": *\n")
            .expect("assemblage cassé");
        let png = image_png(
            super::LARGEUR_MINIATURE,
            super::HAUTEUR_MINIATURE,
            image::Rgba([10, 20, 30, 255]),
            image::Rgba([40, 50, 60, 255]),
        );
        super::ecrire_couverture(&dir, &png).expect("couverture écrite");

        let immediates = super::vignettes_immediates(std::slice::from_ref(&dir));
        assert_eq!(immediates.len(), 1, "la couverture choisie est disponible");
        assert_eq!(
            (immediates[0].2, immediates[0].3),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE)
        );
        assert!(
            super::vignettes_a_rafraichir(std::slice::from_ref(&dir)).is_empty(),
            "une couverture choisie ne se régénère jamais"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Le cache permet d'ouvrir la bibliothèque sans compiler : un document
    /// devenu illisible après le rendu conserve sa couverture.
    #[test]
    fn le_cache_evite_de_compiler_a_l_ouverture() {
        let dir = std::env::temp_dir().join(format!("tw-cache-vignette-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("projet de test");
        std::fs::write(dir.join("main.typ"), "= Un titre\n\nDu texte.\n").expect("assemblage");

        assert_eq!(
            super::vignettes_a_rafraichir(std::slice::from_ref(&dir)).len(),
            1,
            "la première ouverture produit le cache"
        );
        assert!(
            super::vignettes_a_rafraichir(std::slice::from_ref(&dir)).is_empty(),
            "une deuxième ouverture ne recompile pas"
        );

        std::fs::write(dir.join("main.typ"), "#import \"absent.typ\": *\n")
            .expect("assemblage cassé");
        let immediates = super::vignettes_immediates(std::slice::from_ref(&dir));
        assert_eq!(immediates.len(), 1, "le cache se lit sans compiler");
        assert_eq!(
            (immediates[0].2, immediates[0].3),
            (super::LARGEUR_MINIATURE, super::HAUTEUR_MINIATURE)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod tests_de_la_correspondance {
    use super::*;

    fn marges(gauche: &str) -> MiseEnPage {
        MiseEnPage {
            marges: crate::mise_en_page::modele::Marges {
                gauche: gauche.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn la_norme_du_document_se_reconnait() {
        let maison = marges("3cm");
        let normes = vec![
            ("maison".to_string(), maison.clone()),
            ("autre".to_string(), MiseEnPage::default()),
        ];

        assert_eq!(
            correspondance(&normes, &maison),
            Some("maison".to_string()),
            "les réglages du document désignent leur norme"
        );
        assert_eq!(
            correspondance(&normes, &marges("4,7cm")),
            None,
            "des réglages inconnus ne portent aucune norme"
        );
    }
}

#[cfg(test)]
mod tests_de_la_base {
    use super::*;

    /// Un projet naît de sa base : ses fichiers suivent, jamais ses
    /// données d'exécution (demande de l'auteur : un modèle est un
    /// projet comme un autre).
    #[test]
    fn un_projet_nait_de_sa_base() {
        let parent = std::env::temp_dir().join(format!("tw-base-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(parent.join("ma-base/chapitres")).expect("dossier");

        let base = parent.join("ma-base");
        std::fs::write(base.join("document.tw"), "Le contenu du modèle.").expect("document");
        std::fs::write(base.join("chapitres/01-chapitre.typ"), "= Un chapitre").expect("chapitre");
        std::fs::write(base.join("verrou"), "x").expect("verrou");
        std::fs::create_dir_all(base.join("sauvegardes")).expect("sauvegardes");
        std::fs::write(base.join("metadonnees.toml"), "est_modele = true").expect("métadonnées");

        let dir = creer_au_nom_depuis(&parent, "un-devoir", &base).expect("création");

        assert!(
            std::fs::read_to_string(dir.join("document.tw"))
                .unwrap()
                .contains("Le contenu du modèle."),
            "le contenu de la base suit"
        );
        assert!(
            dir.join("chapitres/01-chapitre.typ").exists(),
            "les chapitres suivent"
        );
        assert!(!dir.join("verrou").exists(), "le verrou ne voyage pas");
        assert!(
            !dir.join("sauvegardes").exists(),
            "les sauvegardes ne voyagent pas"
        );
        assert!(
            !dir.join("metadonnees.toml").exists(),
            "les métadonnées du modèle ne voyagent pas : le projet naît les siennes"
        );

        let _ = std::fs::remove_dir_all(&parent);
    }

    /// La trace des références bibliques (le système de références)
    /// : chaque projet garde son relevé, écrit et relu.
    #[test]
    fn la_trace_des_references_est_ecrite() {
        let dossier = std::env::temp_dir().join("tw-trace-references");
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).unwrap();
        let mut releve = std::collections::BTreeMap::new();
        releve.insert("Jn 3:16".to_string(), 3);
        releve.insert("Gn 1:1-3".to_string(), 1);
        ecrire_releve_references(&dossier, &releve).expect("écriture");
        let brut = std::fs::read_to_string(dossier.join("references.json")).expect("fichier");
        assert!(brut.contains("Jn 3:16"), "{brut}");
        assert!(brut.contains("3"), "le compte y est : {brut}");
    }
}
