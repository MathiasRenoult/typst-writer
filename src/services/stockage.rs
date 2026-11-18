//! Le stockage : racine des données, écriture atomique, corbeille, noms sûrs.

use std::path::PathBuf;

/// Le dossier des paquets Typst installés : `<ns>/<nom>/<version>/`.
///
/// C'est le premier jalon de l'architecture paquets (bug-report-7,
/// point 4) : les `#import "@local/primaviz:…"` résolvent **localement**,
/// sans réseau. `TYPST_WRITER_PAQUETS` détourne la racine (tests,
/// installation partagée).
pub fn packages_dir() -> std::path::PathBuf {
    if let Ok(racine) = std::env::var("TYPST_WRITER_PAQUETS") {
        return std::path::PathBuf::from(racine);
    }
    dossier_donnees().join("paquets")
}

/// Dossier de travail de l'application : les projets y vivent.
///
/// Le dossier est créé s'il manque ; une création impossible est ignorée ici et
/// se manifestera à la première écriture. [`dossier_donnees`], lui, ne touche
/// pas au disque.
pub fn work_dir() -> PathBuf {
    let dir = dossier_donnees();

    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Où vivent les données, selon la plateforme.
///
/// Sous Unix — Linux comme macOS — c'est le chemin historique
/// `~/.typst-writer`, où vivent déjà les projets existants : rien ne
/// déménage sans qu'on l'ait décidé. Sous Windows, `%APPDATA%\typst-writer`,
/// avec le profil comme repli.
pub fn dossier_donnees() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("typst-writer");
        }
        if let Ok(profil) = std::env::var("USERPROFILE") {
            return PathBuf::from(profil).join(".typst-writer");
        }
        PathBuf::from(".typst-writer")
    }
    #[cfg(not(windows))]
    {
        match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home).join(".typst-writer"),
            Err(_) => PathBuf::from(".typst-writer"),
        }
    }
}

/// Le dossier des suppressions, à côté des projets.
///
/// Une suppression n'est pas une destruction : le fichier y va, et le
/// chemin est rendu pour se raviser. C'est l'annulation du quotidien,
/// celle qui ne demande pas de dialogue.
pub fn corbeille() -> std::path::PathBuf {
    let dossier = work_dir().join("corbeille");
    let _ = std::fs::create_dir_all(&dossier);

    dossier
}

/// Déplace un fichier vers la corbeille, et rend où il est allé.
///
/// Le nom y devient unique : deux « document.tw » supprimés ne se
/// marchent pas dessus. La place se prend **atomiquement** — deux courses
/// parallèles ne choisissent jamais le même nom.
pub fn vers_la_corbeille(chemin: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let nom = chemin
        .file_name()
        .and_then(|nom| nom.to_str())
        .unwrap_or("fichier")
        .to_string();
    // Le chemin d'origine est figé avant le déplacement : après, il n'existe
    // plus et `canonicalize` ne pourrait que revenir au chemin relatif.
    let origine = std::fs::canonicalize(chemin).unwrap_or_else(|_| chemin.to_path_buf());

    let mut numero = 1;
    loop {
        let nom_unique = if numero == 1 {
            nom.clone()
        } else {
            format!("{nom}-{numero}")
        };
        let destination = corbeille().join(&nom_unique);

        // La place se prend atomiquement — un fichier se marque d'un
        // fichier, un dossier d'un dossier vide : le renommage passe dans
        // les deux cas, et deux courses parallèles ne choisissent jamais
        // le même nom.
        let pris = if chemin.is_dir() {
            std::fs::create_dir(&destination).map(|_| ())
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map(drop)
        };

        match pris {
            Ok(()) => {
                if let Err(erreur) = std::fs::rename(chemin, &destination) {
                    let _ = std::fs::remove_file(&destination);
                    let _ = std::fs::remove_dir(&destination);
                    return Err(erreur);
                }
                // La corbeille ne perd pas l'adresse d'origine : l'écran de
                // récupération peut ainsi proposer un vrai retour, pas
                // seulement l'ouverture du dossier.
                let _ = std::fs::write(fiche_origine(&destination), origine.display().to_string());
                return Ok(destination);
            }
            Err(erreur) if erreur.kind() == std::io::ErrorKind::AlreadyExists => {
                numero += 1;
            }
            Err(erreur) => return Err(erreur),
        }
    }
}

/// La fiche d'origine d'une entrée de corbeille.
fn fiche_origine(entree: &std::path::Path) -> std::path::PathBuf {
    // Le point initial garde la fiche hors de la liste de l'écran ; le
    // suffixe la rend reconnaissable même dans un dossier ouvert à la main.
    let mut nom = std::ffi::OsString::from(".");
    if let Some(nom_entree) = entree.file_name() {
        nom.push(nom_entree);
    }
    nom.push(".origine");
    entree.with_file_name(nom)
}

/// Le chemin d'origine inscrit à côté d'une entrée de corbeille.
pub fn origine_de(entree: &std::path::Path) -> Option<std::path::PathBuf> {
    let fiche = fiche_origine(entree);
    let texte = std::fs::read_to_string(fiche).ok()?;
    let chemin = texte.trim();
    (!chemin.is_empty()).then(|| std::path::PathBuf::from(chemin))
}

/// Une entrée de corbeille est-elle la fiche d'origine d'une autre ?
pub fn est_une_fiche(nom: &str) -> bool {
    nom.ends_with(".origine")
}

/// Remet une entrée de corbeille à son emplacement d'origine.
///
/// Rien n'est écrasé : si l'emplacement est déjà occupé, la corbeille garde
/// l'entrée et le dit. La fiche d'origine ne survit pas au retour réussi.
pub fn restaurer_depuis_la_corbeille(
    entree: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let Some(origine) = origine_de(entree) else {
        return Err("Cette entrée ne dit pas d'où elle vient.".to_string());
    };
    if !entree.exists() {
        return Err("Cette entrée n'existe plus dans la corbeille.".to_string());
    }
    if origine.exists() {
        return Err(format!(
            "{} existe déjà : rien n'a été restauré.",
            origine.display()
        ));
    }
    if let Some(parent) = origine.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|erreur| format!("{} ne se crée pas : {erreur}.", parent.display()))?;
    }
    std::fs::rename(entree, &origine)
        .map_err(|erreur| format!("Restauration impossible : {erreur}."))?;
    let _ = std::fs::remove_file(fiche_origine(entree));
    Ok(origine)
}

/// Écrit un texte par la voie atomique : temporaire voisin puis renommage.
///
/// puis renommé. Les permissions de la cible existante sont conservées ; sous
/// Unix, le dossier parent est lui aussi synchronisé pour que le renommage
/// survive à une coupure. Une erreur supprime le temporaire et laisse la cible
/// intacte. Le dossier parent doit déjà exister.
pub fn ecrire_atomique(chemin: &std::path::Path, contenu: &str) -> std::io::Result<()> {
    ecrire_atomique_octets(chemin, contenu.as_bytes())
}

/// La même voie, pour des octets qui ne sont pas du texte (un import).
pub fn ecrire_atomique_octets(chemin: &std::path::Path, octets: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static PROCHAIN_TEMPORAIRE: AtomicU64 = AtomicU64::new(0);
    let nom = chemin.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "nom de fichier absent")
    })?;
    let (temporaire, mut fichier) = loop {
        let numero = PROCHAIN_TEMPORAIRE.fetch_add(1, Ordering::Relaxed);
        let mut nom_temporaire = std::ffi::OsString::from(".");
        nom_temporaire.push(nom);
        nom_temporaire.push(format!(".tmp-{}-{numero}", std::process::id()));
        let temporaire = chemin.with_file_name(nom_temporaire);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporaire)
        {
            Ok(fichier) => break (temporaire, fichier),
            Err(erreur) if erreur.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(erreur) => return Err(erreur),
        }
    };

    let resultat = (|| {
        match std::fs::metadata(chemin) {
            Ok(metadata) => fichier.set_permissions(metadata.permissions())?,
            Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {}
            Err(erreur) => return Err(erreur),
        }
        fichier.write_all(octets)?;
        fichier.sync_all()?;
        drop(fichier);
        std::fs::rename(&temporaire, chemin)?;
        // Le renommage lui-même doit survivre à une coupure : sans la
        // synchronisation du dossier, la panne peut le perdre.
        #[cfg(unix)]
        if let Some(parent) = chemin.parent()
            && let Ok(dossier) = std::fs::File::open(parent)
        {
            let _ = dossier.sync_all();
        }
        Ok(())
    })();
    if resultat.is_err() {
        let _ = std::fs::remove_file(&temporaire);
    }
    resultat
}

/// Transforme un titre en nom de fichier : minuscules, tirets, sans accent.
pub fn slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());

    let mut pending_dash = false;

    for character in title.chars() {
        // La casse est traitée avant le repli : sinon « État » perdrait sa
        // majuscule accentuée, qui n'est ni alphanumérique ASCII ni repliée.
        let folded = character
            .to_lowercase()
            .next()
            .map(fold_accent)
            .unwrap_or(character);

        if folded.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }

            slug.push(folded.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }

    if slug.is_empty() {
        slug.push_str("chapitre");
    }

    slug
}

/// Réduit les lettres accentuées à leur équivalent sans accent.
fn fold_accent(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        'œ' => 'o',
        other => other,
    }
}
