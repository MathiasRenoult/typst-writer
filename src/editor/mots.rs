//! Le mot sous le curseur, pour les raccourcis de mise en forme.
//!
//! Un `Ctrl+B` sans sélection ne doit pas choisir une plage au hasard : il
//! prend le mot que l'auteur voit. Le curseur peut être dans le mot, juste
//! après lui, ou dans le blanc qui le précède.

use std::ops::Range;

/// Les caractères qui font partie d'un mot pour la mise en forme.
///
/// L'apostrophe et le trait d'union restent dans le mot : « l'Église » et
/// « porte-parole » se mettent en gras d'un seul geste, et les accents sont
/// des caractères alphanumériques comme les autres.
fn est_un_caractere_de_mot(caractere: char) -> bool {
    caractere.is_alphanumeric() || matches!(caractere, '_' | '\'' | '’' | '-')
}

fn debut_du_mot(texte: &str, offset: usize) -> usize {
    texte[..offset]
        .char_indices()
        .rev()
        .take_while(|(_, caractere)| est_un_caractere_de_mot(*caractere))
        .last()
        .map_or(offset, |(index, _)| index)
}

fn fin_du_mot(texte: &str, offset: usize) -> usize {
    texte[offset..]
        .char_indices()
        .take_while(|(_, caractere)| est_un_caractere_de_mot(*caractere))
        .last()
        .map_or(offset, |(index, caractere)| {
            offset + index + caractere.len_utf8()
        })
}

/// La plage du mot sous, avant ou après le curseur.
///
/// Le mot sous le curseur passe en premier ; à défaut, le mot qui précède,
/// puis celui qui suit. Un décalage dans un caractère accentué est ramené à
/// sa frontière avant tout calcul.
pub fn mot_autour(texte: &str, offset: usize) -> Option<Range<usize>> {
    let offset = crate::base::texte::offset_sur_caractere(texte, offset.min(texte.len()));
    let dans_le_mot = texte[offset..]
        .chars()
        .next()
        .is_some_and(est_un_caractere_de_mot)
        || texte[..offset]
            .chars()
            .next_back()
            .is_some_and(est_un_caractere_de_mot);

    if dans_le_mot {
        let debut = debut_du_mot(texte, offset);
        let fin = fin_du_mot(texte, offset);
        if debut < fin {
            return Some(debut..fin);
        }
    }

    // Le curseur est dans un blanc : le mot avant est celui qu'on vient de
    // taper, et celui qui suit est celui qu'on s'apprête à qualifier.
    let avant = debut_du_mot(texte, offset);
    if avant < offset {
        return Some(avant..offset);
    }

    // Sauter les blancs qui séparent le curseur du mot suivant.
    let mut debut_apres = offset;
    while debut_apres < texte.len() {
        let Some(caractere) = texte[debut_apres..].chars().next() else {
            break;
        };
        if est_un_caractere_de_mot(caractere) {
            break;
        }
        debut_apres += caractere.len_utf8();
    }
    let apres = fin_du_mot(texte, debut_apres);
    (debut_apres < apres).then_some(debut_apres..apres)
}

/// Le début du mot précédent, pour `Ctrl+flèche gauche`.
///
/// Le curseur peut être au milieu d'un mot, à son début ou dans un blanc :
/// dans un mot, on recule jusqu'à son début ; dans un blanc, jusqu'au début
/// du mot précédent. Les accents et les apostrophes restent dans le mot,
/// comme pour la mise en forme.
pub fn debut_mot_precedent(texte: &str, offset: usize) -> usize {
    let offset = crate::base::texte::offset_sur_caractere(texte, offset.min(texte.len()));
    let mut index = offset;

    // Si le curseur est dans un mot, son début est la cible. Sinon, on
    // traverse le blanc qui le précède, puis on recule dans le mot précédent.
    let dans_le_mot = index > 0
        && texte[..index]
            .chars()
            .next_back()
            .is_some_and(est_un_caractere_de_mot);

    if !dans_le_mot {
        while index > 0 {
            let Some(caractere) = texte[..index].chars().next_back() else {
                break;
            };
            if est_un_caractere_de_mot(caractere) {
                break;
            }
            index -= caractere.len_utf8();
        }
    }

    while index > 0 {
        let Some(caractere) = texte[..index].chars().next_back() else {
            break;
        };
        if !est_un_caractere_de_mot(caractere) {
            break;
        }
        index -= caractere.len_utf8();
    }

    index
}

/// La fin du mot suivant, pour `Ctrl+flèche droite`.
///
/// Partir du curseur, traverser les blancs qui le suivent, puis finir le mot
/// trouvé. Un curseur déjà dans un mot finit ce mot avant de traverser les
/// blancs ; la cible est ainsi la fin visible du mot courant. Un curseur
/// placé après un mot va donc à la fin du mot suivant.
pub fn fin_mot_suivant(texte: &str, offset: usize) -> usize {
    let offset = crate::base::texte::offset_sur_caractere(texte, offset.min(texte.len()));
    let mut index = offset;

    while index < texte.len() {
        let Some(caractere) = texte[index..].chars().next() else {
            break;
        };
        if est_un_caractere_de_mot(caractere) {
            break;
        }
        index += caractere.len_utf8();
    }

    while index < texte.len() {
        let Some(caractere) = texte[index..].chars().next() else {
            break;
        };
        if !est_un_caractere_de_mot(caractere) {
            break;
        }
        index += caractere.len_utf8();
    }

    index
}

/// La plage d'un mot à supprimer avec `Ctrl+Retour arrière` ou `Ctrl+Suppr`.
pub fn plage_mot(texte: &str, offset: usize, vers_la_gauche: bool) -> Range<usize> {
    if vers_la_gauche {
        debut_mot_precedent(texte, offset)..offset
    } else {
        offset..fin_mot_suivant(texte, offset)
    }
}

/// La fermeture qui correspond à un ouvrant tapé sur une sélection.
pub fn fermeture_de(ouvrant: char) -> Option<char> {
    match ouvrant {
        '"' => Some('"'),
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_mots_se_sautent_entiers() {
        let texte = "Une Église vivante";
        assert_eq!(debut_mot_precedent(texte, texte.len()), 12);
        assert_eq!(debut_mot_precedent(texte, 12), 4);
        assert_eq!(fin_mot_suivant(texte, 0), 3);
        assert_eq!(fin_mot_suivant(texte, 3), 11);
        assert_eq!(fin_mot_suivant(texte, texte.len()), texte.len());
    }

    #[test]
    fn une_plage_de_mot_ne_coupe_pas_un_accent() {
        let texte = "Une Église vivante";
        let fin = texte.find("Église").unwrap() + "Église".len();
        let plage = plage_mot(texte, fin, true);
        assert_eq!(&texte[plage], "Église");
    }

    #[test]
    fn le_mot_sous_le_curseur_est_entier() {
        let texte = "Une Église vivante";
        let offset = texte.find("Église").unwrap() + "Égli".len();
        assert_eq!(mot_autour(texte, offset), Some(4..11));
    }

    #[test]
    fn le_mot_avant_puis_apres_le_curseur_sert_de_repli() {
        let texte = "porte-parole et l'Église";
        let avant = texte.find(" et").unwrap();
        assert_eq!(mot_autour(texte, avant), Some(0..12));
        let apres = texte.find("et l").unwrap();
        assert_eq!(mot_autour(texte, apres), Some(13..15));
    }

    #[test]
    fn un_blanc_isole_ne_donne_pas_de_mot() {
        assert_eq!(mot_autour("   ", 1), None);
        assert_eq!(mot_autour("", 0), None);
    }

    #[test]
    fn les_quatre_ouvrants_ont_leur_fermant() {
        assert_eq!(fermeture_de('"'), Some('"'));
        assert_eq!(fermeture_de('('), Some(')'));
        assert_eq!(fermeture_de('['), Some(']'));
        assert_eq!(fermeture_de('{'), Some('}'));
        assert_eq!(fermeture_de('<'), None);
    }
}
