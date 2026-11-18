//! Les versions antérieures d'un document, telles que l'écran de
//! récupération les montre : date, taille, empreinte et aperçu.
//!
//! Le module ne décide rien : il lit `sauvegardes/` et présente. La
//! restauration elle-même passe par `State`, qui sauvegarde le texte courant
//! avant de le remplacer.

use std::path::{Path, PathBuf};

/// Une copie présente dans `sauvegardes/`.
#[derive(Debug, Clone)]
pub struct VersionSauvee {
    /// Le fichier complet.
    pub chemin: PathBuf,
    /// Son nom lisible.
    pub nom: String,
    /// La date de la copie, dans le fuseau local.
    pub date: String,
    /// Sa taille en octets.
    pub taille: u64,
    /// Son empreinte, en hexadécimal.
    pub empreinte: String,
    /// Le début du texte, pour reconnaître la version.
    pub apercu: String,
}

/// L'écran de restauration : la cible et les copies disponibles.
#[derive(Debug, Clone)]
pub struct Restauration {
    /// Le document qui serait remplacé.
    pub cible: PathBuf,
    /// Les copies, de la plus récente à la plus ancienne.
    pub versions: Vec<VersionSauvee>,
    /// La copie choisie.
    pub choisie: Option<usize>,
    /// Les copies qui n'ont pas pu être lues.
    pub illisibles: usize,
}

/// Lit les sauvegardes du document `cible`.
pub fn ouvrir(projet: &Path, cible: &Path) -> Result<Restauration, String> {
    let dossier = crate::project::sauvegardes_dir(projet);
    let nom = cible
        .file_name()
        .ok_or_else(|| "Le document n'a pas de nom de fichier.".to_string())?
        .to_string_lossy()
        .to_string();
    let prefixe = format!("{nom}.");

    let mut versions = Vec::new();
    let mut illisibles = 0;
    match std::fs::read_dir(&dossier) {
        Ok(entrees) => {
            for entree in entrees.flatten() {
                let chemin = entree.path();
                let Some(nom_copie) = chemin.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !nom_copie.starts_with(&prefixe) {
                    continue;
                }
                let nom_copie = nom_copie.to_string();
                let horodatage = nom_copie
                    .strip_prefix(&prefixe)
                    .and_then(|reste| reste.split('-').next())
                    .and_then(|secondes| secondes.parse::<u64>().ok());
                let Ok(metadata) = chemin.metadata() else {
                    illisibles += 1;
                    continue;
                };
                let Ok(octets) = std::fs::read(&chemin) else {
                    illisibles += 1;
                    continue;
                };
                let date = horodatage
                    .map(date_lisible)
                    .or_else(|| {
                        metadata
                            .modified()
                            .ok()
                            .and_then(|date| date.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|duree| date_lisible(duree.as_secs()))
                    })
                    .unwrap_or_else(|| "date inconnue".to_string());
                let texte = String::from_utf8_lossy(&octets[..octets.len().min(4096)]);
                let mut apercu = texte.split_whitespace().collect::<Vec<_>>().join(" ");
                if apercu.chars().count() > 220 {
                    apercu = format!("{}…", apercu.chars().take(220).collect::<String>());
                }
                versions.push(VersionSauvee {
                    chemin,
                    nom: nom_copie,
                    date,
                    taille: metadata.len(),
                    empreinte: format!("{:016x}", crate::project::empreinte_contenu(&octets)),
                    apercu,
                });
            }
        }
        Err(erreur) if erreur.kind() == std::io::ErrorKind::NotFound => {}
        Err(erreur) => {
            return Err(format!(
                "Le dossier des sauvegardes est illisible : {erreur}"
            ));
        }
    }
    versions.sort_by(|a, b| b.nom.cmp(&a.nom));

    Ok(Restauration {
        cible: cible.to_path_buf(),
        choisie: (!versions.is_empty()).then_some(0),
        versions,
        illisibles,
    })
}

/// Une date lisible : `JJ/MM/AAAA HH:MM`, dans le fuseau local.
///
/// Le nom de la copie porte l'heure en secondes ; la date affichée doit être
/// celle que l'auteur a vécue, pas celle de Greenwich.
pub(crate) fn date_lisible(secondes: u64) -> String {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    date_lisible_avec(secondes, offset)
}

/// La même date, avec un fuseau explicite — pour les épreuves.
fn date_lisible_avec(secondes: u64, offset: time::UtcOffset) -> String {
    let Ok(date) = time::OffsetDateTime::from_unix_timestamp(secondes as i64) else {
        return "date inconnue".to_string();
    };
    let date = date.to_offset(offset);
    format!(
        "{:02}/{:02}/{:04} {:02}:{:02}",
        date.day(),
        u8::from(date.month()),
        date.year(),
        date.hour(),
        date.minute()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_date_du_premier_jour_se_lit() {
        let utc = time::UtcOffset::UTC;
        assert_eq!(date_lisible_avec(0, utc), "01/01/1970 00:00");
        assert_eq!(
            date_lisible_avec(86_400 + 3_600 + 60, utc),
            "02/01/1970 01:01"
        );
        let plus_une_heure = time::UtcOffset::from_hms(1, 0, 0).unwrap();
        assert_eq!(
            date_lisible_avec(0, plus_une_heure),
            "01/01/1970 01:00",
            "la date suit le fuseau local"
        );
    }

    #[test]
    fn les_copies_du_document_se_lisent_avec_leur_premier_texte() {
        let projet =
            std::env::temp_dir().join(format!("typst-writer-restauration-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&projet);
        let sauvegardes = crate::project::sauvegardes_dir(&projet);
        std::fs::create_dir_all(&sauvegardes).unwrap();
        std::fs::write(sauvegardes.join("document.tw.100"), "version ancienne\n").unwrap();
        std::fs::write(sauvegardes.join("document.tw.200"), "version récente\n").unwrap();

        let restauration = ouvrir(&projet, &projet.join("document.tw")).unwrap();
        assert_eq!(restauration.versions.len(), 2);
        assert_eq!(restauration.versions[0].nom, "document.tw.200");
        assert!(restauration.versions[0].apercu.contains("récente"));
        assert_eq!(
            restauration.versions[0].taille,
            std::fs::metadata(sauvegardes.join("document.tw.200"))
                .unwrap()
                .len()
        );
        assert_eq!(restauration.versions[0].empreinte.len(), 16);
        assert_eq!(restauration.choisie, Some(0));
        assert_eq!(restauration.illisibles, 0);

        let _ = std::fs::remove_dir_all(&projet);
    }
}
