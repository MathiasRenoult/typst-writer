//! L'installation des paquets Typst intégrés à l'application.
//!
//! Le monde de rendu (voir [`crate::typst::render`]) résout les
//! `@local/…` **localement**, depuis [`crate::services::stockage::packages_dir`] :
//! jamais au réseau à l'exécution. Ce module y **pose** les fichiers des
//! paquets que l'application embarque — le glossaire glossarium en est le
//! premier — de façon idempotente et atomique :
//!
//! 1. si le paquet est déjà là, rien ne se passe ;
//! 2. sinon les fichiers s'écrivent dans un atelier voisin, unique ;
//! 3. la place finale se prend **atomiquement** (un dossier vide réservé),
//!    puis l'atelier la rejoint d'un renommage — deux installations
//!    simultanées ne se marchent jamais dessus.

use std::io::{ErrorKind, Result};
use std::path::{Path, PathBuf};

use crate::services::stockage::packages_dir;

/// L'identité Typst du glossaire embarqué.
pub const GLOSSARIUM: &str = "@local/glossarium:0.5.10";

/// L'identité Typst de primaviz, le moteur des graphiques.
pub const PRIMAVIZ: &str = "@local/primaviz:0.12.1";

/// L'identité Typst de showybox, le moteur des callouts.
pub const SHOWYBOX: &str = "@local/showybox:2.0.4";

/// Les fichiers de primaviz, vendus dans `vendor/primaviz/0.12.1/`.
pub(crate) const PRIMAVIZ_FICHIERS: &[(&str, &str)] =
    crate::typst::paquets_primaviz::PRIMAVIZ_FICHIERS;

/// Les fichiers de showybox, vendus dans `vendor/showybox/2.0.4/`.
pub(crate) const SHOWYBOX_FICHIERS: &[(&str, &str)] =
    crate::typst::paquets_showybox::SHOWYBOX_FICHIERS;

/// Les fichiers de glossarium 0.5.10, vendés dans `vendor/glossarium/`.
/// Le chemin du paquet est déduit de son identité : `<ns>/<nom>/<version>/`.
pub(crate) const GLOSSARIUM_FICHIERS: &[(&str, &str)] = &[
    (
        "glossarium.typ",
        include_str!("../../vendor/glossarium/0.5.10/glossarium.typ"),
    ),
    (
        "themes/default.typ",
        include_str!("../../vendor/glossarium/0.5.10/themes/default.typ"),
    ),
    (
        "typst.toml",
        include_str!("../../vendor/glossarium/0.5.10/typst.toml"),
    ),
    (
        "LICENSE",
        include_str!("../../vendor/glossarium/0.5.10/LICENSE"),
    ),
];

/// S'assure que glossarium est installé dans le dossier des paquets.
///
/// Idempotent : appelé à chaque assemblage, il ne rend rien à faire quand
/// le paquet est déjà en place.
pub fn assurer_glossarium() -> Result<PathBuf> {
    installer_dans(&packages_dir(), GLOSSARIUM, GLOSSARIUM_FICHIERS)
}

/// S'assure que primaviz est installé dans le dossier des paquets.
///
/// Idempotent : le point d'entrée du paquet fait foi.
pub fn assurer_primaviz() -> Result<PathBuf> {
    installer_dans_avec(&packages_dir(), PRIMAVIZ, PRIMAVIZ_FICHIERS, "src/lib.typ")
}

/// S'assure que showybox est installé dans le dossier des paquets.
///
/// Idempotent : le point d'entrée `showy.typ` fait foi.
pub fn assurer_showybox() -> Result<PathBuf> {
    installer_dans_avec(&packages_dir(), SHOWYBOX, SHOWYBOX_FICHIERS, "showy.typ")
}

/// Pose `fichiers` sous `racine/<ns>/<nom>/<version>/`, avec le point
/// d'entrée du paquet qui décide de sa présence.
///
/// Rend le chemin du paquet. Chaque appelant a son atelier (pid +
/// nanosecondes) : deux installations parallèles ne partagent rien.
pub(crate) fn installer_dans(
    racine: &Path,
    paquet: &str,
    fichiers: &[(&str, &str)],
) -> Result<PathBuf> {
    installer_dans_avec(racine, paquet, fichiers, "glossarium.typ")
}

/// La même installation, avec le fichier qui fait foi pour reconnaître un
/// paquet déjà posé.
pub(crate) fn installer_dans_avec(
    racine: &Path,
    paquet: &str,
    fichiers: &[(&str, &str)],
    point_entree: &str,
) -> Result<PathBuf> {
    let (ns, nom, version) = decouper_paquet(paquet)?;
    let destination = racine.join(ns).join(nom).join(version);

    // Déjà là : la présence du point d'entrée fait foi.
    if destination.join(point_entree).is_file() {
        return Ok(destination);
    }

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let atelier = racine
        .join(ns)
        .join(nom)
        .join(format!(".tw-paquet-{}-{unique}", std::process::id()));
    let _ = std::fs::remove_dir_all(&atelier);

    for (relatif, contenu) in fichiers {
        let chemin = atelier.join(relatif);
        if let Some(parent) = chemin.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&chemin, contenu)?;
    }

    // La place se prend atomiquement — un dossier vide réservé — puis
    // l'atelier la rejoint d'un renommage : le dossier vide cédera la
    // place, et deux courses parallèles ne choisissent jamais le même nom.
    std::fs::create_dir_all(destination.parent().unwrap_or(&destination))?;
    match std::fs::create_dir(&destination) {
        Ok(()) => {}
        Err(erreur) if erreur.kind() == ErrorKind::AlreadyExists => {
            // Un concurrent l'a posé (ou un reste d'une fois d'avant) : si
            // son point d'entrée est là, c'est gagné ; sinon la place se
            // reprend.
            let _ = std::fs::remove_dir_all(&atelier);
            if destination.join(point_entree).is_file() {
                return Ok(destination);
            }
            std::fs::remove_dir_all(&destination)?;
            std::fs::create_dir(&destination)?;
        }
        Err(erreur) => {
            let _ = std::fs::remove_dir_all(&atelier);
            return Err(erreur);
        }
    }

    if let Err(erreur) = std::fs::rename(&atelier, &destination) {
        let _ = std::fs::remove_dir(&destination);
        let _ = std::fs::remove_dir_all(&atelier);
        return Err(erreur);
    }

    Ok(destination)
}

/// `@ns/nom:version` → ses trois morceaux.
fn decouper_paquet(paquet: &str) -> Result<(&str, &str, &str)> {
    let sans_arobase = paquet
        .strip_prefix('@')
        .ok_or_else(|| erreur(paquet, "le paquet commence par @"))?;
    let (ns, suite) = sans_arobase
        .split_once('/')
        .ok_or_else(|| erreur(paquet, "le paquet porte un espace de noms"))?;
    let (nom, version) = suite
        .split_once(':')
        .ok_or_else(|| erreur(paquet, "le paquet porte une version"))?;

    if ns.is_empty() || nom.is_empty() || version.is_empty() {
        return Err(erreur(paquet, "le paquet est incomplet"));
    }

    Ok((ns, nom, version))
}

fn erreur(paquet: &str, raison: &str) -> std::io::Error {
    std::io::Error::new(ErrorKind::InvalidInput, format!("{paquet} : {raison}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un dossier de test, unique par test et par processus.
    fn racine(nom: &str) -> PathBuf {
        let chemin =
            std::env::temp_dir().join(format!("typst-writer-paquets-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&chemin);
        std::fs::create_dir_all(&chemin).expect("racine du test");
        chemin
    }

    /// Un paquet s'installe à sa place, entier — et deux fois de suite ne
    /// change rien.
    #[test]
    fn un_paquet_s_installe_puis_ne_bouge_plus() {
        let root = racine("installation");
        let chemin = installer_dans(&root, GLOSSARIUM, GLOSSARIUM_FICHIERS).expect("installation");

        assert_eq!(
            chemin,
            root.join("local/glossarium/0.5.10"),
            "le paquet rejoint sa place Typst"
        );
        for (relatif, contenu) in GLOSSARIUM_FICHIERS {
            let lu = std::fs::read_to_string(chemin.join(relatif)).expect("fichier du paquet");
            assert_eq!(&lu, contenu, "{relatif} est copié tel quel");
        }

        // La seconde fois ne réécrit pas : le point d'entrée fait foi.
        let marque = chemin.join("glossarium.typ");
        let avant = std::fs::read(&marque).expect("point d'entrée");
        installer_dans(&root, GLOSSARIUM, GLOSSARIUM_FICHIERS).expect("seconde installation");
        assert_eq!(std::fs::read(&marque).expect("relecture"), avant);

        // Aucun atelier ne traîne.
        let reste: Vec<_> = std::fs::read_dir(chemin.parent().expect("parent"))
            .expect("lecture")
            .filter_map(|entree| entree.ok())
            .map(|entree| entree.file_name().to_string_lossy().into_owned())
            .filter(|nom| nom.starts_with(".tw-paquet-"))
            .collect();
        assert!(reste.is_empty(), "des ateliers sont restés : {reste:?}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// primaviz se pose avec son point d'entrée, pas celui du glossaire.
    #[test]
    fn primaviz_s_installe_avec_son_point_entree() {
        let root = racine("primaviz");
        let chemin = installer_dans_avec(&root, PRIMAVIZ, PRIMAVIZ_FICHIERS, "src/lib.typ")
            .expect("installation de primaviz");

        assert_eq!(chemin, root.join("local/primaviz/0.12.1"));
        assert!(
            chemin.join("src/lib.typ").is_file(),
            "le point d'entrée est posé"
        );
        assert!(
            chemin.join("src/charts/pie.typ").is_file(),
            "les types sont posés"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// showybox se pose avec son point d'entrée et ses modules.
    #[test]
    fn showybox_s_installe_avec_son_point_entree() {
        let root = racine("showybox");
        let chemin = installer_dans_avec(&root, SHOWYBOX, SHOWYBOX_FICHIERS, "showy.typ")
            .expect("installation de showybox");

        assert_eq!(chemin, root.join("local/showybox/2.0.4"));
        assert!(
            chemin.join("showy.typ").is_file(),
            "le point d'entrée est posé"
        );
        assert!(
            chemin.join("lib/sections.typ").is_file(),
            "les modules sont posés"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Une identité mal formée se refuse, sans rien écrire.
    #[test]
    fn une_identite_mal_formee_se_refuse() {
        let root = racine("identite");
        for fautif in [
            "glossarium:0.5.10",
            "@local/glossarium",
            "@local:0.5.10",
            "",
        ] {
            assert!(
                installer_dans(&root, fautif, GLOSSARIUM_FICHIERS).is_err(),
                "{fautif} aurait dû être refusé"
            );
        }
        assert_eq!(
            std::fs::read_dir(&root).expect("lecture").count(),
            0,
            "rien n'a été écrit"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
