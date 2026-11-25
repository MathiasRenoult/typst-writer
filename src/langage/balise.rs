//! Retrouver la balise sous un clic ou sous le curseur.
//!
//! L'éditeur ne connaît que la source : le panneau d'une balise a besoin de
//! sa plage exacte et de son nom pour se remplir. Ce module ne décide pas du
//! sens de la balise — il rend seulement le texte entre les délimiteurs.

use std::ops::Range;

/// Une balise `{{…}}` sous un décalage de la source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaliseSous {
    /// La plage complète, délimiteurs compris.
    pub plage: Range<usize>,
    /// Le nom de la balise : `c`, `d`, `b`, `chart`, `garde`…
    pub nom: String,
    /// Le contenu entre les délimiteurs, sans les espaces de bord.
    pub interieur: String,
}

/// La balise `{{…}}` qui contient `offset`, s'il y en a une.
///
/// Une balise de fermeture (`{{/garde}}`) n'a pas de panneau : elle n'est pas
/// rendue. Les grilles et les fences de code sont ignorées, comme dans le
/// formateur de tableaux.
pub fn balise_au(source: &str, offset: usize) -> Option<BaliseSous> {
    let offset = crate::base::texte::offset_sur_caractere(source, offset.min(source.len()));
    if dans_un_bloc_de_code(source, offset) {
        return None;
    }

    let fin_recherche = (offset + 2).min(source.len());
    let ouverture = source[..fin_recherche].rfind("{{")?;
    if ouverture > offset {
        return None;
    }
    let fin_rel = source[ouverture + 2..].find("}}")?;
    let fin = ouverture + 2 + fin_rel + 2;
    if offset > fin {
        return None;
    }

    let interieur = source[ouverture + 2..fin - 2].trim();
    if interieur.is_empty() || interieur.starts_with('/') {
        return None;
    }
    let nom = interieur
        .split(|c: char| c.is_whitespace() || c == '|' || c == '=')
        .next()
        .unwrap_or_default()
        .to_lowercase();
    if nom.is_empty() {
        return None;
    }

    Some(BaliseSous {
        plage: ouverture..fin,
        nom,
        interieur: interieur.to_string(),
    })
}

fn dans_un_bloc_de_code(source: &str, offset: usize) -> bool {
    let mut dans_code = false;
    let mut debut = 0;
    for ligne in source.split_inclusive('\n') {
        let fin = debut + ligne.len();
        let nu = ligne.trim_start();
        let fence = nu.starts_with("```") || nu.starts_with("~~~");
        if offset >= debut && offset < fin {
            return dans_code || fence;
        }
        if fence {
            dans_code = !dans_code;
        }
        debut = fin;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_balise_sous_le_curseur_donne_sa_plage_et_son_nom() {
        let source = "Avant {{c red|un mot}} après";
        let dedans = source.find("un mot").unwrap();
        let balise = balise_au(source, dedans).expect("balise");
        assert_eq!(balise.nom, "c");
        assert_eq!(&source[balise.plage.clone()], "{{c red|un mot}}");
        assert_eq!(balise.interieur, "c red|un mot");
    }

    #[test]
    fn la_balise_se_retrouve_sur_ses_bornes() {
        let source = "{{b Jn 3.16}}";
        assert!(balise_au(source, 0).is_some());
        assert!(balise_au(source, source.len()).is_some());
    }

    #[test]
    fn une_fermeture_ou_un_bloc_de_code_n_ouvre_rien() {
        assert!(balise_au("{{/garde}}", 4).is_none());
        let code = "```\n{{c red|mot}}\n```\n";
        assert!(balise_au(code, code.find("mot").unwrap()).is_none());
        assert!(balise_au("du texte {{c red|mot}}", 3).is_none());
    }
}
