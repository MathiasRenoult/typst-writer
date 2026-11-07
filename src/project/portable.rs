//! L'export portable d'un projet, et l'import d'une sauvegarde.
//!
//! Ce qui sort d'ici doit entrer ailleurs — sur une autre machine : la
//! source, les normes, les styles, les bibliographies et les ressources,
//! avec l'inventaire de ce qui manquait déjà. Et jamais d'écrasement sans
//! choix : une collision se refuse d'abord, puis se résout par la corbeille
//! — réversible — si l'utilisateur l'a voulu.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Le manifeste placé dans chaque archive : l'inventaire du projet exporté.
pub const MANIFESTE: &str = "manifeste.json";

/// Ce que le projet porte en lui, catégorie par catégorie.
///
/// Les chemins sont relatifs à la racine du projet, avec des `/` partout :
/// une archive écrite sous Linux se lit sous Windows.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Inventaire {
    /// La source : document, chapitres, tout ce qui s'écrit.
    pub sources: Vec<String>,
    /// Les normes de mise en page du projet.
    pub normes: Vec<String>,
    /// Les styles bibliographiques du projet.
    pub styles: Vec<String>,
    /// Les bibliographies et leur registre Zotero.
    pub bibliographies: Vec<String>,
    /// Les ressources : images et autres fichiers du projet.
    pub ressources: Vec<String>,
    /// Référencés par la source mais absents du projet : dits, pas perdus.
    pub manquantes: Vec<String>,
}

impl Inventaire {
    /// Tout ce qui voyage, en un seul ordre stable.
    pub fn fichiers(&self) -> Vec<&str> {
        self.sources
            .iter()
            .chain(self.normes.iter())
            .chain(self.styles.iter())
            .chain(self.bibliographies.iter())
            .chain(self.ressources.iter())
            .map(String::as_str)
            .collect()
    }
}

/// Le manifeste de l'archive : qui a été exporté, par quelle version.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Manifeste {
    /// La version de l'application à l'export.
    pub version: String,
    /// Le nom du projet : c'est lui qui nomme la destination à l'import.
    pub nom: String,
    /// L'inventaire du projet exporté.
    pub inventaire: Inventaire,
}

/// Pourquoi un import n'est pas allé au bout.
#[derive(Debug)]
pub enum ErreurImport {
    /// Des fichiers existent déjà : sans choix, rien ne s'écrit.
    Collisions(Vec<String>),
    /// Tout le reste, dit en une phrase — et rien n'a bougé.
    Refus(String),
}

impl From<String> for ErreurImport {
    fn from(message: String) -> Self {
        ErreurImport::Refus(message)
    }
}

impl std::fmt::Display for ErreurImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurImport::Collisions(cles) => write!(
                f,
                "L'import est refusé sans rien écrire : {} existe(nt) déjà à la destination ({}).",
                cles.len(),
                cles.join(", ")
            ),
            ErreurImport::Refus(message) => write!(f, "{message}"),
        }
    }
}

/// Ce que l'import fait des fichiers déjà présents à la destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolitiqueCollision {
    /// Aucun écrasement : la moindre collision refuse l'import entier.
    Refuser,
    /// Choix explicite : ce qui est remplacé part à la corbeille, réversible.
    VersCorbeille,
}

/// Le compte rendu d'un import : ce qui est arrivé, où.
#[derive(Debug)]
pub struct RapportImport {
    /// Les fichiers écrits à la destination, en chemins relatifs.
    pub ecrits: Vec<String>,
    /// Les fichiers remplacés, partis à la corbeille (politique choisie).
    pub remplaces: Vec<(String, PathBuf)>,
}

/// Inventorie un projet : ce qui s'y trouve, et ce qui manque.
///
/// Les verrous, les sauvegardes antérieures et les temporaires ne voyagent
/// pas — ils appartiennent à la machine, pas au projet.
pub fn inventaire_projet(dir: &Path) -> Result<Inventaire, String> {
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un dossier de projet.", dir.display()));
    }

    let mut inventaire = Inventaire::default();
    let mut sources_texte = String::new();

    for (relatif, chemin) in fichiers_du_projet(dir)? {
        match classement(&relatif) {
            Classement::Source => {
                if let Ok(contenu) = std::fs::read_to_string(&chemin) {
                    sources_texte.push_str(&contenu);
                    sources_texte.push('\n');
                }
                inventaire.sources.push(relatif);
            }
            Classement::Normes => inventaire.normes.push(relatif),
            Classement::Style => inventaire.styles.push(relatif),
            Classement::Bibliographie => inventaire.bibliographies.push(relatif),
            Classement::Ressource => inventaire.ressources.push(relatif),
        }
    }

    // Les dépendances de la source : ce qu'elle appelle, trouvé ou non.
    for appel in appels_de_la_source(&sources_texte) {
        let chemin = dir.join(&appel);
        if chemin.is_file() {
            if !inventaire.fichiers().contains(&appel.as_str()) {
                inventaire.ressources.push(appel);
            }
        } else if !inventaire.manquantes.contains(&appel) {
            inventaire.manquantes.push(appel);
        }
    }

    Ok(inventaire)
}

/// Exporte un projet vers une archive unique, manifeste compris.
pub fn exporter_projet(dir: &Path, cible: &Path) -> Result<Inventaire, String> {
    let inventaire = inventaire_projet(dir)?;
    if inventaire.sources.is_empty() {
        return Err("Un projet sans source ne s'exporte pas : il n'y a rien à porter.".into());
    }

    let fichier = std::fs::File::create(cible).map_err(|e| {
        format!(
            "L'archive {} ne peut pas être créée : {e}.",
            cible.display()
        )
    })?;
    let mut archive = zip::ZipWriter::new(fichier);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let manifeste = Manifeste {
        version: env!("CARGO_PKG_VERSION").into(),
        nom: dir
            .file_name()
            .map(|nom| nom.to_string_lossy().to_string())
            .unwrap_or_else(|| "projet".into()),
        inventaire: inventaire.clone(),
    };
    archive
        .start_file(MANIFESTE, options)
        .map_err(|e| format!("Le manifeste ne s'écrit pas : {e}."))?;
    let jeton = serde_json::to_vec_pretty(&manifeste)
        .map_err(|e| format!("Le manifeste ne se sérialise pas : {e}."))?;
    archive
        .write_all(&jeton)
        .map_err(|e| format!("Le manifeste ne s'écrit pas : {e}."))?;

    for relatif in inventaire.fichiers() {
        let chemin = dir.join(relatif);
        let octets = std::fs::read(&chemin)
            .map_err(|e| format!("{} ne se lit pas : {e}.", chemin.display()))?;
        archive
            .start_file(relatif, options)
            .map_err(|e| format!("{relatif} ne s'ouvre pas dans l'archive : {e}."))?;
        archive
            .write_all(&octets)
            .map_err(|e| format!("{relatif} ne s'écrit pas : {e}."))?;
    }

    archive
        .finish()
        .map_err(|e| format!("L'archive ne se termine pas : {e}."))?;
    Ok(inventaire)
}

/// Lit le manifeste d'une archive sans rien extraire.
/// Ce que l'import a à dire — produit hors du thread de l'interface,
/// porté par un message (feuille de route, phase 1, cartographie, geste 4).
#[derive(Debug, Clone)]
pub enum SortieImport {
    /// Le projet est écrit : sa destination, et la note à montrer.
    Fait(std::path::PathBuf, String),
    /// L'import a échoué : le mot à montrer.
    Erreur(String),
}
pub fn lire_manifeste(archive: &Path) -> Result<Manifeste, String> {
    let mut zip = ouvrir_archive(archive)?;
    let mut entree = zip.by_name(MANIFESTE).map_err(|_| {
        "Cette archive ne porte pas de manifeste : ce n'est pas une sauvegarde de projet."
            .to_string()
    })?;
    let mut jeton = Vec::new();
    entree
        .read_to_end(&mut jeton)
        .map_err(|e| format!("Le manifeste ne se lit pas : {e}."))?;
    serde_json::from_slice(&jeton).map_err(|e| format!("Le manifeste est illisible : {e}."))
}

/// Importe une sauvegarde à la destination, sans jamais écraser sans choix.
///
/// Tout se prépare d'abord, rien ne bouge avant : une collision refusée, ou
/// une cible qui n'est pas un fichier, laisse la destination exactement
/// comme elle était. Les remplacements choisis passent par la corbeille.
/// Le contenu lu et les collisions relevées — la préparation de l'import.
pub type Preparation = (Vec<(String, Vec<u8>)>, Vec<String>);

/// La préparation de l'import : l'archive se lit, les collisions se
/// relèvent — sans rien écrire. La question de confirmation s'appuie
/// dessus, synchrone ; les écritures, elles, partent hors du thread de
/// l'interface (feuille de route, phase 1, cartographie, geste 4).
pub fn preparer_import(archive: &Path, destination: &Path) -> Result<Preparation, ErreurImport> {
    let mut zip = ouvrir_archive(archive)?;
    let manifeste = {
        let mut entree = zip.by_name(MANIFESTE).map_err(|_| {
            "Cette archive ne porte pas de manifeste : ce n'est pas une sauvegarde de projet."
                .to_string()
        })?;
        let mut jeton = Vec::new();
        entree
            .read_to_end(&mut jeton)
            .map_err(|e| format!("Le manifeste ne se lit pas : {e}."))?;
        serde_json::from_slice::<Manifeste>(&jeton)
            .map_err(|e| format!("Le manifeste est illisible : {e}."))?
    };

    // Ce que l'archive porte, en mémoire d'abord — et aucun chemin ne sort
    // jamais de la destination : ni `..`, ni absolu, ni traversée par un
    // lien. Un nom qui tente le voyage refuse l'import entier.
    let mut contenu: Vec<(String, Vec<u8>)> = Vec::new();
    for relatif in manifeste.inventaire.fichiers() {
        if !chemin_confinable(relatif) {
            return Err(ErreurImport::Refus(format!(
                "{relatif} : un chemin d'archive ne peut pas sortir de la destination. L'import est refusé, rien n'a bougé."
            )));
        }
        let mut entree = zip
            .by_name(relatif)
            .map_err(|e| format!("{relatif} manque dans l'archive : {e}."))?;
        let mut octets = Vec::new();
        entree
            .read_to_end(&mut octets)
            .map_err(|e| format!("{relatif} ne se lit pas : {e}."))?;
        contenu.push((relatif.to_string(), octets));
    }

    // La préparation : collisions vues, cibles contrôlées — rien ne bouge
    // encore.
    let mut collisions = Vec::new();
    for (relatif, _) in &contenu {
        let cible = destination.join(relatif);
        if cible.exists() {
            if !cible.is_file() {
                return Err(ErreurImport::Refus(format!(
                    "{} existe déjà mais n'est pas un fichier : l'import est refusé, rien n'a bougé.",
                    cible.display()
                )));
            }
            collisions.push(relatif.clone());
        } else if let Some(parent) = cible.parent()
            && parent.exists()
            && !parent.is_dir()
        {
            return Err(ErreurImport::Refus(format!(
                "{} existe déjà mais n'est pas un dossier : l'import est refusé, rien n'a bougé.",
                parent.display()
            )));
        }
    }

    Ok((contenu, collisions))
}

/// Les collisions de l'import, relevées sans écrire.
pub fn collisions_import(archive: &Path, destination: &Path) -> Result<(), ErreurImport> {
    let (_, collisions) = preparer_import(archive, destination)?;
    if collisions.is_empty() {
        Ok(())
    } else {
        Err(ErreurImport::Collisions(collisions))
    }
}

pub fn importer_projet(
    archive: &Path,
    destination: &Path,
    politique: PolitiqueCollision,
) -> Result<RapportImport, ErreurImport> {
    let (contenu, collisions) = preparer_import(archive, destination)?;
    if !collisions.is_empty() && politique == PolitiqueCollision::Refuser {
        return Err(ErreurImport::Collisions(collisions));
    }

    // Le choix est fait : ce qui est remplacé part à la corbeille, où il se
    // retrouve. Puis tout se pose dans un atelier voisin — et rien ne
    // rejoint la destination qu'une fois tout prêt : **un import interrompu
    // laisse la destination exactement comme avant**.
    let mut remplaces = Vec::new();
    for relatif in &collisions {
        let cible = destination.join(relatif);
        let copie = crate::project::vers_la_corbeille(&cible)
            .map_err(|e| format!("{} ne part pas à la corbeille : {e}.", cible.display()))?;
        remplaces.push((relatif.clone(), copie));
    }

    let atelier = {
        // L'atelier est unique **par import**, pas par processus : deux
        // imports parallèles (les épreuves, la vie réelle) ne se
        // videraient pas l'un l'autre.
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        destination
            .parent()
            .unwrap_or(destination)
            .join(format!(".tw-import-{}-{unique}", std::process::id()))
    };
    let _ = std::fs::remove_dir_all(&atelier);

    let pose = (|| -> Result<(), String> {
        // Tout se pose d'abord, à côté.
        for (relatif, octets) in &contenu {
            let posee = atelier.join(relatif);
            if let Some(parent) = posee.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{} ne se crée pas : {e}.", parent.display()))?;
            }
            crate::project::ecrire_atomique_octets(&posee, octets)
                .map_err(|e| format!("{} ne se pose pas : {e}.", posee.display()))?;
        }
        // Puis chaque fichier rejoint la destination, d'un renommage.
        for (relatif, _) in &contenu {
            let cible = destination.join(relatif);
            if let Some(parent) = cible.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{} ne se crée pas : {e}.", parent.display()))?;
            }
            std::fs::rename(atelier.join(relatif), &cible)
                .map_err(|e| format!("{} ne rejoint pas la destination : {e}.", cible.display()))?;
        }
        Ok(())
    })();

    if let Err(panne) = pose {
        let _ = std::fs::remove_dir_all(&atelier);
        // La remise exacte de l'état d'avant : rien de neuf ne reste, ce
        // qui était là revient — et la panne est dite pour ce qu'elle est.
        for (relatif, _) in &contenu {
            let _ = std::fs::remove_file(destination.join(relatif));
        }
        for (relatif, copie) in remplaces.iter().rev() {
            let _ = std::fs::rename(copie, destination.join(relatif));
        }
        return Err(ErreurImport::Refus(format!(
            "{panne} L'import est interrompu : la destination est revenue à son état d'avant, rien n'a été perdu."
        )));
    }

    let _ = std::fs::remove_dir_all(&atelier);
    Ok(RapportImport {
        ecrits: contenu.into_iter().map(|(relatif, _)| relatif).collect(),
        remplaces,
    })
}

/// Un chemin d'archive reste-t-il sous la destination ?
///
/// C'est le même trou que les advisories des archives : un nom qui monte
/// vers `..`, qui part d'une racine, ou qui porte un lecteur, écrirait
/// hors du projet.
fn chemin_confinable(relatif: &str) -> bool {
    !relatif.is_empty()
        && !relatif.starts_with('/')
        && !relatif.starts_with('\\')
        && !relatif.contains("..")
        && !relatif.contains(':')
        && !relatif.contains('\\')
}

/// Ouvre une archive, ou dit pourquoi elle ne s'ouvre pas.
fn ouvrir_archive(archive: &Path) -> Result<zip::ZipArchive<std::fs::File>, String> {
    let fichier = std::fs::File::open(archive)
        .map_err(|e| format!("{} ne s'ouvre pas : {e}.", archive.display()))?;
    zip::ZipArchive::new(fichier)
        .map_err(|e| format!("{} n'est pas une archive lisible : {e}.", archive.display()))
}

/// Ce que la source appelle : images, inclusions, bibliographies.
///
/// Les URL lointaines et les ancres ne sont pas des fichiers du projet.
fn appels_de_la_source(texte: &str) -> Vec<String> {
    let mut appels = Vec::new();
    let mut collecter = |brut: &str| {
        let chemin = brut.trim().trim_matches('"').trim_matches('\'');
        if chemin.is_empty()
            || chemin.contains("://")
            || chemin.starts_with('#')
            || chemin.starts_with('/')
        {
            return;
        }
        let chemin = chemin.strip_prefix("./").unwrap_or(chemin).to_string();
        if !appels.contains(&chemin) {
            appels.push(chemin);
        }
    };

    // La forme markdown-esque du dialecte .tw : `![…](chemin)`.
    let mut reste = texte;
    while let Some(fin) = reste.find("](") {
        reste = &reste[fin + 2..];
        if let Some(parenthese) = reste.find(')') {
            collecter(&reste[..parenthese]);
            reste = &reste[parenthese..];
        }
    }

    // Les formes Typst des sources brutes : `#image("…")` et voisines.
    for motif in ["#image(", "#include(", "#bibliography("] {
        let mut reste = texte;
        while let Some(fin) = reste.find(motif) {
            reste = &reste[fin + motif.len()..];
            if let Some(fin) = reste.find(')') {
                collecter(&reste[..fin]);
                reste = &reste[fin..];
            }
        }
    }

    appels
}

/// Les fichiers du projet, en chemins relatifs normauxés (`/` partout).
///
/// Les verrous, les sauvegardes antérieures et les temporaires sont de la
/// machine, pas du projet.
fn fichiers_du_projet(dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let mut trouves = Vec::new();
    let mut dossiers = vec![(String::new(), dir.to_path_buf())];

    while let Some((relatif_parent, parent)) = dossiers.pop() {
        let entrees = std::fs::read_dir(&parent)
            .map_err(|e| format!("{} ne se parcourt pas : {e}.", parent.display()))?;
        for entree in entrees {
            let entree = entree
                .map_err(|e| format!("Une entrée de {} se lit mal : {e}.", parent.display()))?;
            let nom = entree.file_name().to_string_lossy().to_string();
            let relatif = if relatif_parent.is_empty() {
                nom.clone()
            } else {
                format!("{relatif_parent}/{nom}")
            };

            if nom == "sauvegardes"
                || nom.starts_with('.')
                || nom.ends_with('~')
                || nom.ends_with(".tmp")
            {
                continue;
            }

            let chemin = entree.path();
            if chemin.is_dir() {
                dossiers.push((relatif, chemin));
            } else {
                trouves.push((relatif, chemin));
            }
        }
    }

    trouves.sort();
    Ok(trouves)
}

enum Classement {
    Source,
    Normes,
    Style,
    Bibliographie,
    Ressource,
}

fn classement(relatif: &str) -> Classement {
    if relatif.ends_with(".tw") || relatif.ends_with(".typ") {
        Classement::Source
    } else if relatif.ends_with("mise-en-page.toml") {
        Classement::Normes
    } else if relatif.starts_with("styles/") {
        Classement::Style
    } else if relatif.ends_with(".bib") || relatif == "zotero.json" {
        Classement::Bibliographie
    } else {
        Classement::Ressource
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un dossier de test neuf, propre à ce test seul — jamais partagé.
    fn dossier(nom: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "typst-writer-portable-{nom}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("chapitres")).expect("dossier");
        dir
    }

    /// Un projet de voyage : source, chapitre, normes, style, bibliographie,
    /// registre et une image appelée par la source.
    fn projet_pret(nom: &str) -> PathBuf {
        let dir = dossier(nom);
        std::fs::write(
            dir.join("document.tw"),
            "# Titre\n\n![la coupe](images/coupe.png)\n",
        )
        .expect("document");
        std::fs::write(dir.join("chapitres/ch1.typ"), "Du texte.").expect("chapitre");
        std::fs::write(dir.join("mise-en-page.toml"), "langue = 'fr'\n").expect("normes");
        std::fs::create_dir_all(dir.join("styles")).expect("styles");
        std::fs::write(dir.join("styles/local.csl"), "<style/>").expect("style");
        std::fs::write(dir.join("references.bib"), "@book{a, title = {T}}\n")
            .expect("bibliographie");
        std::fs::write(dir.join("zotero.json"), "{}").expect("registre");
        std::fs::create_dir_all(dir.join("images")).expect("images");
        std::fs::write(dir.join("images/coupe.png"), b"\x89PNG").expect("image");
        dir
    }

    #[test]
    fn un_export_reimporte_le_projet_au_complet() {
        let dir = projet_pret("rond");
        let archive =
            std::env::temp_dir().join(format!("tw-portable-rond-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&archive);

        let inventaire = exporter_projet(&dir, &archive).expect("export");
        assert!(inventaire.sources.contains(&"document.tw".to_string()));
        assert!(inventaire.normes.contains(&"mise-en-page.toml".to_string()));
        assert!(inventaire.styles.contains(&"styles/local.csl".to_string()));
        assert!(
            inventaire
                .bibliographies
                .contains(&"references.bib".to_string())
        );
        assert!(
            inventaire
                .ressources
                .contains(&"images/coupe.png".to_string())
        );
        assert!(
            inventaire.manquantes.is_empty(),
            "{:?}",
            inventaire.manquantes
        );

        let destination = dossier("rond-arrivee");
        let rapport =
            importer_projet(&archive, &destination, PolitiqueCollision::Refuser).expect("import");
        assert!(rapport.remplaces.is_empty());

        // Tout y est, octet pour octet.
        for relatif in inventaire.fichiers() {
            assert_eq!(
                std::fs::read(dir.join(relatif)).expect("source"),
                std::fs::read(destination.join(relatif)).expect("arrivée"),
                "{relatif} ne voyage pas à l'identique"
            );
        }

        // Et le manifeste dit ce que l'archive portait.
        let manifeste = lire_manifeste(&archive).expect("manifeste");
        assert_eq!(manifeste.inventaire.sources, inventaire.sources);
        assert!(!manifeste.version.is_empty());
    }

    #[test]
    fn un_export_dit_ce_qui_manque() {
        let dir = dossier("manques");
        std::fs::write(dir.join("document.tw"), "![perdue](images/perdue.png)\n")
            .expect("document");

        let inventaire = inventaire_projet(&dir).expect("inventaire");
        assert_eq!(
            inventaire.manquantes,
            vec!["images/perdue.png".to_string()],
            "la dépendance absente est dite, pas passée sous silence"
        );
    }

    #[test]
    fn un_import_refuse_sans_choix_et_ne_ecrase_rien() {
        let dir = projet_pret("refus");
        let archive =
            std::env::temp_dir().join(format!("tw-portable-refus-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&archive);
        exporter_projet(&dir, &archive).expect("export");

        // La destination a déjà le document, avec un autre contenu.
        let destination = dossier("refus-arrivee");
        std::fs::write(destination.join("document.tw"), "DÉJÀ À MOI").expect("existant");

        let refus = importer_projet(&archive, &destination, PolitiqueCollision::Refuser);
        assert!(
            matches!(refus, Err(ErreurImport::Collisions(_))),
            "sans choix, pas d'écrasement : {refus:?}"
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("document.tw")).expect("document"),
            "DÉJÀ À MOI",
            "la collision refuse, mais ne touche pas"
        );
        assert!(
            !destination.join("chapitres/ch1.typ").exists(),
            "l'import refusé n'écrit rien du tout, même ailleurs"
        );
    }

    #[test]
    fn un_import_choisi_passe_par_la_corbeille() {
        let dir = projet_pret("corbeille");
        let archive =
            std::env::temp_dir().join(format!("tw-portable-corbeille-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&archive);
        exporter_projet(&dir, &archive).expect("export");

        let destination = dossier("corbeille-arrivee");
        std::fs::write(destination.join("document.tw"), "CE QUE J'AVAIS").expect("existant");

        let rapport = importer_projet(&archive, &destination, PolitiqueCollision::VersCorbeille)
            .expect("import");

        // Le nouveau est là, l'ancien est à la corbeille — retrouvable.
        assert_eq!(
            std::fs::read_to_string(destination.join("document.tw")).expect("document"),
            std::fs::read_to_string(dir.join("document.tw")).expect("source")
        );
        assert_eq!(rapport.remplaces.len(), 1);
        let (_, copie) = &rapport.remplaces[0];
        assert_eq!(
            std::fs::read_to_string(copie).expect("copie"),
            "CE QUE J'AVAIS",
            "ce qui est remplacé se retrouve dans la corbeille"
        );
    }

    #[test]
    fn un_import_refuse_une_cible_qui_n_est_pas_un_fichier() {
        let dir = projet_pret("piege");
        let archive =
            std::env::temp_dir().join(format!("tw-portable-piege-{}.zip", std::process::id()));
        let _ = std::fs::remove_file(&archive);
        exporter_projet(&dir, &archive).expect("export");

        // Le piège : un dossier porte le nom d'un fichier de l'archive.
        let destination = dossier("piege-arrivee");
        std::fs::create_dir_all(destination.join("document.tw")).expect("piège");

        let refus = importer_projet(&archive, &destination, PolitiqueCollision::VersCorbeille);
        assert!(
            refus.is_err(),
            "un dossier sous un nom de fichier se refuse"
        );
        assert!(
            !destination.join("chapitres/ch1.typ").exists(),
            "le refus vient AVANT tout déplacement : rien n'a bougé"
        );
    }
}

#[cfg(test)]
mod tests_confinement {
    use super::*;
    use std::io::Write;

    /// Une archive forgée dont le manifeste appelle un chemin qui monte :
    /// l'import se refuse entier, et rien ne fuit hors de la destination.
    #[test]
    fn un_import_refuse_un_chemin_qui_sort_du_projet() {
        let dir = std::env::temp_dir().join(format!("tw-portable-fuite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier");
        let archive = dir.join("fuite.zip");

        let fichier = std::fs::File::create(&archive).expect("archive");
        let mut zip = zip::ZipWriter::new(fichier);
        let options = zip::write::SimpleFileOptions::default();
        let manifeste = Manifeste {
            version: "0".into(),
            nom: "fuite".into(),
            inventaire: Inventaire {
                ressources: vec!["../fuie.txt".into()],
                ..Inventaire::default()
            },
        };
        zip.start_file(MANIFESTE, options).expect("manifeste");
        zip.write_all(&serde_json::to_vec(&manifeste).expect("json"))
            .expect("jeton");
        zip.start_file("../fuie.txt", options).expect("entrée");
        zip.write_all(b"FUITE").expect("contenu");
        zip.finish().expect("fin");

        let destination = dir.join("arrivee");
        std::fs::create_dir_all(&destination).expect("destination");
        let refus = importer_projet(&archive, &destination, PolitiqueCollision::VersCorbeille);
        assert!(refus.is_err(), "le chemin qui sort se refuse : {refus:?}");
        assert!(
            !dir.join("fuie.txt").exists(),
            "rien n'a fui hors de la destination"
        );
    }
}

#[cfg(test)]
mod tests_interruption {
    use super::*;
    use std::io::Write;

    /// Une archive forgée : une notice qui remplace, et un fichier
    /// impossible à créer — l'import casse au milieu.
    fn archive_interrompue(dir: &std::path::Path) -> std::path::PathBuf {
        let archive = dir.join("interrompu.zip");
        let fichier = std::fs::File::create(&archive).expect("archive");
        let mut zip = zip::ZipWriter::new(fichier);
        let options = zip::write::SimpleFileOptions::default();

        let impossible = format!("{}.txt", "x".repeat(300));
        let manifeste = Manifeste {
            version: "0".into(),
            nom: "interrompu".into(),
            inventaire: Inventaire {
                sources: vec!["document.txt".into(), impossible.clone()],
                ..Inventaire::default()
            },
        };
        zip.start_file(MANIFESTE, options).expect("manifeste");
        zip.write_all(&serde_json::to_vec(&manifeste).expect("json"))
            .expect("jeton");
        zip.start_file("document.txt", options).expect("entrée");
        zip.write_all(b"CE QUE L'ARCHIVE PORTE").expect("contenu");
        zip.start_file(impossible.as_str(), options)
            .expect("entrée");
        zip.write_all(b"JAMAIS CREE").expect("contenu");
        zip.finish().expect("fin");
        archive
    }

    /// L'import interrompu est récupérable : la destination est exactement
    /// comme avant — le fichier remplacé est remis, rien d'autre n'est
    /// arrivé, et le compte rendu dit que rien n'a été perdu.
    #[test]
    fn un_import_interrompu_reste_recuperable() {
        let dir =
            std::env::temp_dir().join(format!("tw-portable-interruption-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let destination = dir.join("arrivee");
        std::fs::create_dir_all(&destination).expect("destination");
        std::fs::write(destination.join("document.txt"), "CE QUE J'AVAIS").expect("document");
        std::fs::write(destination.join("intact.txt"), "INTACT").expect("intact");

        let archive = archive_interrompue(&dir);
        let echec = importer_projet(&archive, &destination, PolitiqueCollision::VersCorbeille);
        assert!(echec.is_err(), "l'import casse au milieu : {echec:?}");

        // Récupération : tout est comme avant, octet pour octet.
        assert_eq!(
            std::fs::read_to_string(destination.join("document.txt")).expect("document"),
            "CE QUE J'AVAIS",
            "le fichier remplacé est remis"
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("intact.txt")).expect("intact"),
            "INTACT"
        );
        let entrees: Vec<_> = std::fs::read_dir(&destination)
            .expect("destination")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entrees.len(), 2, "rien d'autre n'est arrivé : {entrees:?}");
    }
}

// ---------- Le `.tw` unique (jalon 2 du format transportable) ----------

/// Le projet entier dans un seul `.tw` : le document, et ses pièces
/// embarquées — c'est le transfert en un fichier (bug-report-7, point 3).
pub fn exporter_tw(dir: &Path) -> Result<String, String> {
    let fichier = crate::langage::portable::FichierPortable {
        document: std::fs::read_to_string(dir.join("document.tw"))
            .or_else(|_| std::fs::read_to_string(dir.join("main.typ")))
            .unwrap_or_default(),
        meta: crate::project::metadonnees::Metadonnees::charger(dir).paires,
        normes: std::fs::read_to_string(dir.join(crate::mise_en_page::REGLAGES))
            .unwrap_or_default(),
        bibliographie: std::fs::read_to_string(dir.join("references.bib")).unwrap_or_default(),
        glossaire: std::fs::read_to_string(crate::project::glossaire::Glossaire::chemin(dir))
            .unwrap_or_default(),
        ..Default::default()
    };

    Ok(fichier.ecrire())
}

/// Reconstruit un projet depuis un seul `.tw` : les pièces à leur place,
/// l'accroche posée, `main.typ` prêt à composer. Un fichier du futur se
/// lit mais ne s'importe pas — ce qu'on ne voit pas ne se perd pas.
pub fn importer_tw(texte: &str, dir: &Path) -> Result<(), String> {
    let fichier = crate::langage::portable::FichierPortable::lire(texte);
    if fichier.version_inconnue() {
        return Err(
            "Ce fichier vient d\'une version plus récente : je ne le reconstruis pas à moitié."
                .to_string(),
        );
    }

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("document.tw"), &fichier.document).map_err(|e| e.to_string())?;
    if !fichier.meta.is_empty() {
        let meta = crate::project::metadonnees::Metadonnees {
            paires: fichier.meta.clone(),
            ..Default::default()
        };
        meta.sauver(dir)?;
    }
    if !fichier.normes.trim().is_empty() {
        std::fs::write(dir.join(crate::mise_en_page::REGLAGES), &fichier.normes)
            .map_err(|e| e.to_string())?;
    }
    if !fichier.bibliographie.trim().is_empty() {
        std::fs::write(dir.join("references.bib"), &fichier.bibliographie)
            .map_err(|e| e.to_string())?;
    }
    if !fichier.glossaire.trim().is_empty() {
        std::fs::write(
            crate::project::glossaire::Glossaire::chemin(dir),
            &fichier.glossaire,
        )
        .map_err(|e| e.to_string())?;
    }

    // L'assemblage, et son accroche : le projet se compose seul.
    let main = dir.join("main.typ");
    if !main.exists() {
        std::fs::write(&main, crate::project::model::MAIN_TEMPLATE).map_err(|e| e.to_string())?;
    }
    let normes = crate::mise_en_page::charger(dir);
    let _ = crate::mise_en_page::appliquer(dir, &normes, true).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests_tw_unique {
    use super::{exporter_tw, importer_tw};

    /// Le transfert en un fichier : ce qui sort du projet y revient,
    /// pièce pour pièce.
    #[test]
    fn un_projet_va_et_vient_dans_un_seul_tw() {
        let racine = std::env::temp_dir().join(format!("tw-unique-{}", std::process::id()));
        let depart = racine.join("depart");
        let arrivee = racine.join("arrivee");
        let _ = std::fs::remove_dir_all(&racine);
        std::fs::create_dir_all(&depart).expect("dossier");

        std::fs::write(depart.join("document.tw"), "= Mon document\n\nDu texte.\n")
            .expect("document");
        let mut meta = crate::project::metadonnees::Metadonnees::base();
        meta.definir("", "titre".to_string(), "Ma thèse".to_string());
        meta.sauver(&depart).expect("métadonnées");
        std::fs::write(
            depart.join(crate::mise_en_page::REGLAGES),
            "page_de_titre = true\n",
        )
        .expect("normes");
        std::fs::write(
            depart.join("references.bib"),
            "@book{ouvrage,\n  title = {Un},\n}\n",
        )
        .expect("bibliographie");
        let mut glossaire = crate::project::glossaire::Glossaire::default();
        glossaire.definir(crate::project::glossaire::EntreeGlossaire::nouvelle("sdo"));
        glossaire.sauver(&depart).expect("glossaire");

        let texte = exporter_tw(&depart).expect("export");
        importer_tw(&texte, &arrivee).expect("import");

        assert_eq!(
            std::fs::read_to_string(arrivee.join("document.tw")).expect("document"),
            "= Mon document\n\nDu texte.\n",
            "le document revient"
        );
        let meta = crate::project::metadonnees::Metadonnees::charger(&arrivee);
        assert_eq!(
            meta.valeur("titre"),
            Some("Ma thèse"),
            "les métadonnées reviennent"
        );
        let normes = crate::mise_en_page::charger(&arrivee);
        assert!(normes.page_de_titre, "les normes reviennent");
        assert!(
            std::fs::read_to_string(arrivee.join("references.bib"))
                .expect("bibliographie")
                .contains("ouvrage"),
            "la bibliographie revient"
        );
        assert!(
            arrivee.join("main.typ").exists(),
            "l'assemblage est prêt à composer"
        );
        let glossaire = crate::project::glossaire::Glossaire::charger(&arrivee);
        assert_eq!(glossaire.entrees.len(), 1, "le glossaire revient");
        assert_eq!(glossaire.entrees[0].key, "sdo");

        std::fs::remove_dir_all(&racine).expect("nettoyage");
    }
}
