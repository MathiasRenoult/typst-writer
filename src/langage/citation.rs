//! La grammaire d'une clé de citation, partagée par tous les lecteurs.
//!
//! Le document `.tw`, le composeur, le correcteur de compilation et
//! l'éditeur doivent s'arrêter exactement au même endroit. Une clé
//! bibliographique peut contenir des lettres, des chiffres, des tirets, des
//! soulignements, des points et des deux-points — les clés BibTeX réelles ne
//! se limitent pas aux identifiants Rust.
//!
//! Dans un fichier Typst brut, un `@fig:une` peut aussi être l'étiquette
//! d'une figure : le correcteur de compilation décide, selon le dialecte et
//! les étiquettes connues, s'il s'agit d'un renvoi. La grammaire de la clé,
//! elle, reste la même partout.
//!
//! Une adresse électronique n'est **pas** une citation : le `@` intérieur à un
//! mot (`user@example.com`) est précédé d'une lettre, donc il n'ouvre aucune
//! clé. C'est la frontière qui tranche, pas l'apparence de la clé.

/// Un caractère qui peut appartenir à une clé de citation.
pub fn est_caractere_de_cle(caractere: char) -> bool {
    caractere.is_alphanumeric() || matches!(caractere, '_' | '-' | '.' | ':')
}

/// La clé est-elle non vide et composée uniquement de caractères autorisés ?
pub fn cle_valide(cle: &str) -> bool {
    !cle.is_empty()
        && !cle.starts_with(['.', ':'])
        && !cle.ends_with(['.', ':'])
        && cle.chars().all(est_caractere_de_cle)
}

/// Le `@` à cette position ouvre-t-il une citation ?
///
/// Il ne doit pas être collé à la fin d'un mot : `user@example.com` reste une
/// adresse, `@dupont2023` après une espace ou une ponctuation est une
/// citation. Un `@` déjà échappé ne passe pas par cette fonction.
pub fn frontiere(avant: &str) -> bool {
    !avant.chars().next_back().is_some_and(|caractere| {
        caractere.is_alphanumeric() || caractere == '_' || caractere == '@'
    })
}

/// La citation en cours d'écriture sous le curseur, même incomplète.
///
/// Sert au geste `Tab` : `@exemple` n'est pas encore une citation complète
/// pour le parseur, mais l'auteur est en train de l'écrire. On rend la plage
/// du `@` jusqu'à la fin de la clé, sans jamais la modifier ici.
pub fn appel_incomplet_au_curseur(texte: &str, offset: usize) -> Option<(usize, usize)> {
    let offset = offset.min(texte.len());
    let avant = &texte[..offset];
    let arobase = avant.rfind('@')?;

    if !frontiere(&texte[..arobase]) || est_echappe(texte, arobase) {
        return None;
    }

    let mut fin = arobase + 1;
    for caractere in texte[arobase + 1..].chars() {
        if !est_caractere_de_cle(caractere) {
            break;
        }
        fin += caractere.len_utf8();
    }

    (offset <= fin).then_some((arobase, fin))
}

/// Le curseur est-il dans une bande de code, où `@` n'est jamais une citation ?
///
/// Les blocs clôturés par des triples accents graves et le code inline sont
/// des bandes opaques, comme dans le parseur. Cette lecture ne remplace pas
/// la grammaire : elle évite seulement qu'un `Tab` ouvre le composeur sur du
/// code.
pub fn dans_une_bande_opaque(texte: &str, offset: usize) -> bool {
    let offset = offset.min(texte.len());
    let avant = &texte[..offset];

    let clôtures = avant
        .lines()
        .filter(|ligne| ligne.trim_start().starts_with("```"))
        .count();
    if clôtures % 2 == 1 {
        return true;
    }

    let debut_ligne = avant.rfind('\n').map_or(0, |index| index + 1);
    let ligne = &avant[debut_ligne..];

    let mut accents = 0usize;
    let mut echappe = false;
    for caractere in ligne.chars() {
        if echappe {
            echappe = false;
            continue;
        }
        match caractere {
            '\\' => echappe = true,
            '`' => accents += 1,
            _ => {}
        }
    }

    accents % 2 == 1
}

fn est_echappe(texte: &str, position: usize) -> bool {
    let mut barres = 0usize;
    for caractere in texte[..position].chars().rev() {
        if caractere == '\\' {
            barres += 1;
        } else {
            break;
        }
    }

    barres % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_cle_admet_les_points_et_les_deux_points() {
        assert!(cle_valide("dupont2023"));
        assert!(cle_valide("smith.2020"));
        assert!(cle_valide("fig:une"));
        assert!(!cle_valide(""));
        assert!(!cle_valide("deux mots"));
        assert!(!cle_valide("clé/ouverte"));
    }

    #[test]
    fn une_adresse_electronique_n_est_pas_une_frontiere() {
        assert!(!frontiere("Contactez user"));
        assert!(!frontiere("user_"));
        assert!(frontiere("Voir "));
        assert!(frontiere("(Voir "));
        assert!(frontiere(""));
    }

    #[test]
    fn un_appel_incomplet_donne_sa_plage_sans_la_modifier() {
        let texte = "Voir @exemple";

        assert_eq!(
            appel_incomplet_au_curseur(texte, texte.len()),
            Some((5, 13))
        );
        assert_eq!(appel_incomplet_au_curseur("@", 1), Some((0, 1)));
        assert_eq!(appel_incomplet_au_curseur("user@example.com", 16), None);
        assert_eq!(
            appel_incomplet_au_curseur("Voir @exemple ailleurs", 9),
            Some((5, 13))
        );
    }

    #[test]
    fn une_bande_de_code_n_ouvre_pas_de_citation() {
        assert!(dans_une_bande_opaque("```\n@exemple\n```", 9));
        assert!(dans_une_bande_opaque("`@exemple`", 4));
        assert!(!dans_une_bande_opaque("Voir @exemple", 6));
        assert!(!dans_une_bande_opaque("`code` puis @exemple", 14));
    }
}
