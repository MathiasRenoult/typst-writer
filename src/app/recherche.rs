//! La recherche et le remplacement dans le document.
//!
//! Le motif est une **expression régulière** : les groupes se rappellent
//! dans le remplacement par `$1`, `$2`… C'est la puissance qu'on attend
//! d'un outil de long travail, et elle se paie d'une règle : un motif
//! invalide se **dit**, jamais en silence.
//!
//! Tout ici est pur : la recherche ne touche ni au éditeur, ni à l'écran.

use regex::Regex;

/// Une recherche en cours : son motif, son remplacement, ses trouvailles.
#[derive(Debug, Default, Clone)]
pub struct Recherche {
    /// Le motif, expression régulière.
    pub motif: String,

    /// Ce qui le remplace — `$1` rappelle un groupe.
    pub remplacement: String,

    /// Les occurrences, en décalages d'**octets** dans le texte cherché.
    pub occurrences: Vec<(usize, usize)>,

    /// Laquelle on regarde.
    pub courante: usize,

    /// Un motif invalide dit pourquoi.
    pub erreur: Option<String>,
}

/// Les occurrences d'un motif dans un texte, en décalages d'octets.
pub fn occurrences(motif: &str, texte: &str) -> Result<Vec<(usize, usize)>, String> {
    if motif.is_empty() {
        return Ok(Vec::new());
    }

    let motif = Regex::new(motif).map_err(|erreur| format!("motif : {erreur}"))?;

    Ok(motif
        .find_iter(texte)
        .map(|trouve| (trouve.start(), trouve.end()))
        .collect())
}

/// Ce qui remplace une occurrence — les groupes du motif y sont rappelés.
pub fn remplacement(
    motif: &str,
    remplacement: &str,
    texte: &str,
    debut: usize,
    fin: usize,
) -> Result<String, String> {
    let motif = Regex::new(motif).map_err(|erreur| format!("motif : {erreur}"))?;
    let plage = &texte[debut..fin];

    let captures = motif
        .captures(plage)
        .ok_or_else(|| "l'occurrence n'est plus là".to_string())?;

    let mut sorti = String::new();
    captures.expand(remplacement, &mut sorti);

    Ok(sorti)
}

/// Toutes les remplacements d'un coup, du dernier au premier.
///
/// Du dernier au premier : les décalages des précédents ne bougent pas,
/// et le texte reste juste.
pub fn remplacer_tout(motif: &str, par: &str, texte: &str) -> Result<String, String> {
    let occurrences = occurrences(motif, texte)?;

    let mut sortie = texte.to_string();

    for (debut, fin) in occurrences.into_iter().rev() {
        let neuf = remplacement(motif, par, &sortie, debut, fin)?;
        sortie.replace_range(debut..fin, &neuf);
    }

    Ok(sortie)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les occurrences se trouvent, en octets — la monnaie de l'éditeur.
    #[test]
    fn les_occurrences_se_trouvent() {
        assert_eq!(
            occurrences("mot", "un mot, deux mots").expect("motif"),
            vec![(3, 6), (13, 16)]
        );

        assert_eq!(occurrences("", "du texte").expect("vide"), vec![]);
    }

    /// Un motif invalide dit pourquoi, sans emporter le reste.
    #[test]
    fn un_motif_invalide_se_dit() {
        let erreur = occurrences("(", "du texte").expect_err("invalide");
        assert!(erreur.contains("motif"), "vu : {erreur}");
    }

    /// Les groupes se rappellent dans le remplacement.
    #[test]
    fn les_groupes_se_rappellent() {
        let texte = "Dupont, Jean";
        let neuf =
            remplacement(r"(\w+), (\w+)", "$2 $1", texte, 0, texte.len()).expect("remplacement");
        assert_eq!(neuf, "Jean Dupont");
    }

    /// Tout remplacer va du dernier au premier : rien ne se décale.
    #[test]
    fn remplacer_tout_ne_decale_rien() {
        assert_eq!(
            remplacer_tout(r"\d+", "N", "1 22 333").expect("remplacement"),
            "N N N"
        );

        // Et les groupes y sont rappelés.
        assert_eq!(
            remplacer_tout(r"(\w+)@(\w+)", "$2@$1", "a@b c@d").expect("remplacement"),
            "b@a d@c"
        );
    }
}
