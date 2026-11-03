//! La recherche et le remplacement dans le document.
//!
//! Le motif est une **expression régulière** ou un **texte littéral**, et
//! les deux sont exacts : en régulier, les groupes se rappellent dans le
//! remplacement par `$1`, `$2`… ; en littéral, ni les métacaractères du
//! motif ni les `$` du remplacement ne signifient rien de plus que ce
//! qu'ils sont (R6, « littéral et regex exacts »). Un motif régulier
//! invalide se **dit**, jamais en silence.
//!
//! Tout ici est pur : la recherche ne touche ni à l'éditeur, ni à l'écran.

use regex::Regex;

/// Une recherche en cours : son motif, son remplacement, ses trouvailles.
#[derive(Debug, Default, Clone)]
pub struct Recherche {
    /// Le motif — expression régulière ou texte littéral.
    pub motif: String,

    /// Le mode : `true` = expression régulière, `false` = littéral.
    pub mode_regex: bool,

    /// Ce qui le remplace — `$1` rappelle un groupe, en mode régulier
    /// seulement.
    pub remplacement: String,

    /// Les occurrences, en décalages d'**octets** dans le texte cherché.
    pub occurrences: Vec<(usize, usize)>,

    /// L'aperçu des changements : ce que chaque occurrence devient, dans
    /// l'ordre du texte (R6, « aperçu des changements »).
    pub apercu: Vec<(String, String)>,

    /// Laquelle on regarde.
    pub courante: usize,

    /// Un motif invalide dit pourquoi.
    pub erreur: Option<String>,
}

/// Le motif tel qu'il s'exécute : la regex du motif, ou le texte littéral
/// échappé — sa traduction fidèle en regex.
pub fn motif_effectif(mode_regex: bool, motif: &str) -> String {
    if mode_regex {
        motif.to_string()
    } else {
        regex::escape(motif)
    }
}

/// Les occurrences d'un motif dans un texte, en décalages d'octets.
pub fn occurrences(
    mode_regex: bool,
    motif: &str,
    texte: &str,
) -> Result<Vec<(usize, usize)>, String> {
    if motif.is_empty() {
        return Ok(Vec::new());
    }

    let motif = Regex::new(&motif_effectif(mode_regex, motif))
        .map_err(|erreur| format!("motif : {erreur}"))?;

    Ok(motif
        .find_iter(texte)
        .map(|trouve| (trouve.start(), trouve.end()))
        .collect())
}

/// Ce qui remplace une occurrence.
///
/// En mode régulier, les groupes du motif y sont rappelés (`$1`) ; en mode
/// littéral, le remplacement est ce qu'il est — un `$` y reste un `$`.
pub fn remplacement(
    mode_regex: bool,
    motif: &str,
    remplacement: &str,
    texte: &str,
    debut: usize,
    fin: usize,
) -> Result<String, String> {
    if !mode_regex {
        return Ok(remplacement.to_string());
    }

    let motif = Regex::new(&motif_effectif(mode_regex, motif))
        .map_err(|erreur| format!("motif : {erreur}"))?;
    let plage = &texte[debut..fin];

    let captures = motif
        .captures(plage)
        .ok_or_else(|| "l'occurrence n'est plus là".to_string())?;

    let mut sorti = String::new();
    captures.expand(remplacement, &mut sorti);

    Ok(sorti)
}

/// L'aperçu des changements : ce que chaque occurrence devient — avant →
/// après, les groupes du motif déjà rappelés dans le remplacement. La
/// promesse de R6 : **voir avant de remplacer**.
pub fn apercu(
    mode_regex: bool,
    motif: &str,
    par: &str,
    texte: &str,
) -> Result<Vec<(String, String)>, String> {
    let mut apercu = Vec::new();

    for (debut, fin) in occurrences(mode_regex, motif, texte)? {
        let avant = texte[debut..fin].to_string();
        let apres = remplacement(mode_regex, motif, par, texte, debut, fin)?;
        apercu.push((avant, apres));
    }

    Ok(apercu)
}

/// Tous les remplacements d'un coup, du dernier au premier.
///
/// Du dernier au premier : les décalages des précédents ne bougent pas,
/// et le texte reste juste.
pub fn remplacer_tout(
    mode_regex: bool,
    motif: &str,
    par: &str,
    texte: &str,
) -> Result<String, String> {
    let occurrences = occurrences(mode_regex, motif, texte)?;

    let mut sortie = texte.to_string();

    for (debut, fin) in occurrences.into_iter().rev() {
        let neuf = remplacement(mode_regex, motif, par, &sortie, debut, fin)?;
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
            occurrences(true, r"\d+", "1 22 333").expect("recherche"),
            vec![(0, 1), (2, 4), (5, 8)]
        );

        // Les occurrences ne se chevauchent pas : la recherche régulière
        // reprend après chaque trouvaille.
        assert_eq!(
            occurrences(true, "aba", "ababa").expect("recherche"),
            vec![(0, 3)]
        );

        assert_eq!(occurrences(true, "", "du texte").expect("vide"), vec![]);
    }

    /// Un motif invalide dit pourquoi, sans emporter le reste.
    #[test]
    fn un_motif_invalide_se_dit() {
        let erreur = occurrences(true, "(", "du texte").expect_err("invalide");
        assert!(erreur.contains("motif"), "vu : {erreur}");
    }

    /// Les groupes se rappellent dans le remplacement.
    #[test]
    fn les_groupes_se_rappellent() {
        let texte = "Dupont, Jean";
        let neuf = remplacement(true, r"(\w+), (\w+)", "$2 $1", texte, 0, texte.len())
            .expect("remplacement");
        assert_eq!(neuf, "Jean Dupont");
    }

    /// Tout remplacer va du dernier au premier : rien ne se décale.
    #[test]
    fn remplacer_tout_ne_decale_rien() {
        assert_eq!(
            remplacer_tout(true, r"\d+", "N", "1 22 333").expect("remplacement"),
            "N N N"
        );

        // Et les groupes y sont rappelés.
        assert_eq!(
            remplacer_tout(true, r"(\w+)@(\w+)", "$2@$1", "a@b c@d").expect("remplacement"),
            "b@a d@c"
        );
    }

    /// Le mode littéral est **littéral** : les métacaractères du motif ne
    /// sont pas des métacaractères (R6, « littéral et regex exacts »).
    #[test]
    fn le_litteral_ne_lit_pas_les_metacaracteres() {
        assert_eq!(
            occurrences(false, "a.b", "a.b axb").expect("recherche"),
            vec![(0, 3)],
            "le point littéral n'est pas « n'importe quel caractère »"
        );

        assert_eq!(
            occurrences(false, "(note)", "un (note) ici").expect("recherche"),
            vec![(3, 9)],
            "une parenthèse littérale n'est pas un groupe"
        );
    }

    /// L'aperçu montre ce que chaque occurrence devient — les groupes déjà
    /// rappelés, pour voir avant de remplacer.
    #[test]
    fn l_apercu_montre_ce_que_chaque_occurrence_devient() {
        let apercu = apercu(true, r"(\w+)@(\w+)", "$2@$1", "a@b et c@d").expect("aperçu");

        assert_eq!(
            apercu,
            vec![
                ("a@b".to_string(), "b@a".to_string()),
                ("c@d".to_string(), "d@c".to_string()),
            ]
        );
    }

    /// Le remplacement littéral garde ses `$` : `$1` reste `$1`, il ne
    /// devient jamais un groupe — et le motif littéral reste ce qu'il est.
    #[test]
    fn le_remplacement_litteral_garde_ses_dollars() {
        let texte = "prix : 10";
        let neuf = remplacement(false, "10", "$1", texte, 8, 10).expect("remplacement");
        assert_eq!(neuf, "$1", "le dollar littéral reste un dollar");

        // Et tout remplacer suit la même règle, sur un motif à points.
        assert_eq!(
            remplacer_tout(false, "a.b", "[$0]", "a.b axb").expect("remplacement"),
            "[$0] axb",
            "seule la forme littérale est remplacée"
        );
    }
}
