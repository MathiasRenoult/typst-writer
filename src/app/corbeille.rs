//! La corbeille : ce qui a été supprimé, et d'où cela venait.
//!
//! Le stockage écrit une fiche `.origine` à côté de chaque entrée. L'écran
//! peut donc proposer un vrai retour à la place d'origine, sans demander à
//! l'auteur de retrouver le chemin à la main.

use std::path::PathBuf;

/// Une entrée de la corbeille.
#[derive(Debug, Clone)]
pub struct EntreeCorbeille {
    /// Le fichier ou dossier dans la corbeille.
    pub chemin: PathBuf,
    /// Son nom lisible.
    pub nom: String,
    /// La date de la suppression.
    pub date: String,
    /// Sa taille, ou zéro pour un dossier.
    pub taille: u64,
    /// Un dossier se restaure comme un dossier.
    pub est_dossier: bool,
    /// L'emplacement d'origine, quand la fiche existe.
    pub origine: Option<PathBuf>,
}

/// L'écran de la corbeille.
#[derive(Debug, Clone)]
pub struct Corbeille {
    /// Les entrées, de la plus récente à la plus ancienne.
    pub entrees: Vec<EntreeCorbeille>,
    /// L'entrée choisie.
    pub choisie: Option<usize>,
    /// Les entrées illisibles, comptées plutôt que tues.
    pub illisibles: usize,
}

/// Lit la corbeille sans rien déplacer.
pub fn ouvrir() -> Result<Corbeille, String> {
    let dossier = crate::services::stockage::corbeille();
    let entrees = match std::fs::read_dir(&dossier) {
        Ok(entrees) => entrees,
        Err(erreur) => {
            return Err(format!("La corbeille est illisible : {erreur}"));
        }
    };

    let mut trouvees = Vec::new();
    let mut illisibles = 0;
    for entree in entrees.flatten() {
        let chemin = entree.path();
        let Some(nom) = chemin.file_name().and_then(|nom| nom.to_str()) else {
            continue;
        };
        if crate::services::stockage::est_une_fiche(nom) {
            continue;
        }
        let nom = nom.to_string();
        let Ok(metadata) = chemin.metadata() else {
            illisibles += 1;
            continue;
        };
        let date = metadata
            .modified()
            .ok()
            .and_then(|date| date.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duree| crate::app::restauration::date_lisible(duree.as_secs()))
            .unwrap_or_else(|| "date inconnue".to_string());
        let est_dossier = metadata.is_dir();
        trouvees.push(EntreeCorbeille {
            chemin: chemin.clone(),
            nom,
            date,
            taille: if est_dossier { 0 } else { metadata.len() },
            est_dossier,
            origine: crate::services::stockage::origine_de(&chemin),
        });
    }
    trouvees.sort_by(|a, b| b.nom.cmp(&a.nom));

    Ok(Corbeille {
        choisie: (!trouvees.is_empty()).then_some(0),
        entrees: trouvees,
        illisibles,
    })
}
