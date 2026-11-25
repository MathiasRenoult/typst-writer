//! La mise en liste, **ligne à ligne**.
//!
//! Le bouton « liste à puces » n'est pas une transformation de bloc : il
//! travaille sur la ligne du curseur, ou sur chaque ligne non vide de la
//! sélection. Un paragraphe de plusieurs lignes ne se transforme donc plus
//! en entier par surprise.

use std::ops::Range;

use crate::langage::blocs::Genre;

/// Le remplacement à poser dans le texte, et ce que devient le curseur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatageLignes {
    /// La plage en octets, sans le saut de ligne qui suit la dernière ligne.
    pub plage: Range<usize>,

    /// Le texte des lignes, séparé par des sauts de ligne.
    pub texte: String,

    /// La ligne du curseur, comptée à partir de `plage.start`.
    pub ligne_curseur: usize,

    /// La colonne du curseur après le remplacement, en caractères.
    pub colonne_curseur: usize,
}

/// Formate la ligne du curseur, ou toutes les lignes non vides de la
/// sélection, comme une liste du genre demandé.
///
/// `selection` est une paire de positions `(ligne, colonne)` normalisée dans
/// l'ordre du document. Les colonnes sont comptées en caractères, comme dans
/// l'éditeur. Une sélection qui s'arrête à la colonne zéro n'inclut pas cette
/// dernière ligne : c'est la convention habituelle des sélections de lignes.
///
/// Renvoie `None` quand il n'y a rien à changer : sélection composée
/// uniquement de lignes vides, ou ligne courante hors du texte.
pub fn formater_lignes(
    contenu: &str,
    selection: Option<((usize, usize), (usize, usize))>,
    curseur: (usize, usize),
    genre: Genre,
) -> Option<FormatageLignes> {
    if !matches!(genre, Genre::Puce | Genre::Numero) {
        return None;
    }

    let lignes: Vec<&str> = contenu.split('\n').collect();
    if lignes.is_empty() {
        return None;
    }

    let (premiere, derniere) = lignes_touchees(selection, curseur.0, lignes.len());

    let mut cibles: Vec<usize> = (premiere..=derniere)
        .filter(|index| !lignes[*index].trim().is_empty())
        .collect();

    // Sans sélection, une ligne vide reçoit tout de même sa marque : le
    // bouton écrit, il ne se tait jamais. Avec une sélection, les lignes
    // vides ne sont pas des cibles — « ni plus ».
    if cibles.is_empty() {
        if selection.is_some() {
            return None;
        }
        cibles.push(curseur.0.min(lignes.len() - 1));
    }

    let retirer = cibles
        .iter()
        .all(|index| a_la_marque(lignes[*index], genre));

    let mut resultat = Vec::with_capacity(derniere - premiere + 1);
    let mut delta_curseur = 0isize;
    let mut numero = 1usize;

    for (index, ligne) in lignes
        .iter()
        .enumerate()
        .skip(premiere)
        .take(derniere - premiere + 1)
    {
        let ligne = *ligne;
        let est_cible = cibles.contains(&index);

        let nouvelle = if est_cible {
            let (indentation, corps) = decouper_indentation(ligne);
            let corps = if retirer {
                retirer_la_marque(corps, genre)
            } else {
                sans_marque_de_liste(corps)
            };

            if retirer {
                format!("{indentation}{corps}")
            } else {
                let marque = match genre {
                    Genre::Puce => "- ".to_string(),
                    Genre::Numero => {
                        let marque = format!("{numero}. ");
                        numero += 1;
                        marque
                    }
                    _ => unreachable!("le genre est filtré en tête"),
                };
                format!("{indentation}{marque}{corps}")
            }
        } else {
            ligne.to_string()
        };

        if index == curseur.0 {
            delta_curseur = difference_de_colonnes(ligne, &nouvelle);
        }

        resultat.push(nouvelle);
    }

    let debuts = debuts_de_ligne(contenu);
    let debut = *debuts.get(premiere)?;
    let fin = *debuts.get(derniere)? + lignes[derniere].len();

    let colonne_curseur = adjuster_colonne(curseur.1, delta_curseur);

    Some(FormatageLignes {
        plage: debut..fin,
        texte: resultat.join("\n"),
        ligne_curseur: curseur.0.saturating_sub(premiere),
        colonne_curseur,
    })
}

/// Les lignes touchées par une sélection, bornes comprises.
///
/// Une sélection qui finit à la colonne zéro d'une ligne ne touche pas cette
/// ligne : elle s'arrête juste avant.
fn lignes_touchees(
    selection: Option<((usize, usize), (usize, usize))>,
    ligne_curseur: usize,
    nombre_de_lignes: usize,
) -> (usize, usize) {
    let (mut premiere, mut derniere) = match selection {
        Some(((debut_ligne, _), (fin_ligne, fin_colonne))) => {
            let derniere = if fin_ligne > debut_ligne && fin_colonne == 0 {
                fin_ligne - 1
            } else {
                fin_ligne
            };
            (debut_ligne, derniere)
        }
        None => (ligne_curseur, ligne_curseur),
    };

    premiere = premiere.min(nombre_de_lignes.saturating_sub(1));
    derniere = derniere
        .min(nombre_de_lignes.saturating_sub(1))
        .max(premiere);

    (premiere, derniere)
}

/// L'indentation de la ligne, puis son contenu.
fn decouper_indentation(ligne: &str) -> (&str, &str) {
    let corps = ligne.trim_start_matches([' ', '\t']);
    (&ligne[..ligne.len() - corps.len()], corps)
}

/// La ligne porte-t-elle la marque de ce genre de liste ?
fn a_la_marque(ligne: &str, genre: Genre) -> bool {
    let (_, corps) = decouper_indentation(ligne);

    match genre {
        Genre::Puce => corps.starts_with("- "),
        Genre::Numero => crate::langage::parse::est_numero(corps),
        _ => false,
    }
}

/// Retire la marque du genre demandé, et elle seule.
fn retirer_la_marque(corps: &str, genre: Genre) -> &str {
    match genre {
        Genre::Puce => corps.strip_prefix("- ").unwrap_or(corps),
        Genre::Numero => {
            let chiffres = corps.chars().take_while(|c| c.is_ascii_digit()).count();
            if chiffres > 0 && corps[chiffres..].starts_with(". ") {
                &corps[chiffres + 2..]
            } else {
                corps
            }
        }
        _ => corps,
    }
}

/// Retire une marque de liste, puce ou numéro, pour qu'une conversion ne
/// laisse pas `- 1. texte`.
fn sans_marque_de_liste(corps: &str) -> &str {
    let sans_puce = retirer_la_marque(corps, Genre::Puce);

    if sans_puce != corps {
        sans_puce
    } else {
        retirer_la_marque(corps, Genre::Numero)
    }
}

/// Le nombre de caractères ajoutés ou retirés devant la ligne.
fn difference_de_colonnes(ancienne: &str, nouvelle: &str) -> isize {
    nouvelle.chars().count() as isize - ancienne.chars().count() as isize
}

fn adjuster_colonne(colonne: usize, delta: isize) -> usize {
    if delta < 0 {
        colonne.saturating_sub(delta.unsigned_abs())
    } else {
        colonne.saturating_add(delta as usize)
    }
}

/// Les décalages de début de chaque ligne.
fn debuts_de_ligne(contenu: &str) -> Vec<usize> {
    let mut debuts = vec![0];

    for (index, caractere) in contenu.char_indices() {
        if caractere == '\n' {
            debuts.push(index + 1);
        }
    }

    debuts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appliquer(contenu: &str, formatage: &FormatageLignes) -> String {
        format!(
            "{}{}{}",
            &contenu[..formatage.plage.start],
            formatage.texte,
            &contenu[formatage.plage.end..]
        )
    }

    #[test]
    fn sans_selection_une_seule_ligne_est_touchee() {
        let formatage = formater_lignes(
            "une phrase\ndeux phrases.\n\nLa suite.",
            None,
            (1, 3),
            Genre::Puce,
        )
        .expect("formatage");

        assert_eq!(
            appliquer("une phrase\ndeux phrases.\n\nLa suite.", &formatage),
            "une phrase\n- deux phrases.\n\nLa suite."
        );
        assert_eq!(formatage.ligne_curseur, 0);
        assert_eq!(formatage.colonne_curseur, 5);
    }

    #[test]
    fn la_selection_ne_touche_que_ses_lignes_non_vides() {
        let formatage = formater_lignes(
            "une phrase\n\ndeux phrases.\n\nLa suite.",
            Some(((0, 0), (3, 0))),
            (3, 0),
            Genre::Puce,
        )
        .expect("formatage");

        assert_eq!(
            appliquer("une phrase\n\ndeux phrases.\n\nLa suite.", &formatage),
            "- une phrase\n\n- deux phrases.\n\nLa suite."
        );
        assert!(!formatage.texte.contains("- La suite"));
    }

    #[test]
    fn une_selection_finissant_a_la_colonne_zero_n_inclut_pas_sa_derniere_ligne() {
        let formatage = formater_lignes(
            "une phrase\ndeux phrases.\n\nLa suite.",
            Some(((0, 0), (2, 0))),
            (2, 0),
            Genre::Puce,
        )
        .expect("formatage");

        assert_eq!(
            appliquer("une phrase\ndeux phrases.\n\nLa suite.", &formatage),
            "- une phrase\n- deux phrases.\n\nLa suite."
        );
    }

    #[test]
    fn redemander_la_marque_la_retire_sur_les_lignes_choisies() {
        let formatage = formater_lignes(
            "- une\n- deux\n\n- trois",
            Some(((0, 0), (2, 0))),
            (2, 0),
            Genre::Puce,
        )
        .expect("formatage");

        assert_eq!(
            appliquer("- une\n- deux\n\n- trois", &formatage),
            "une\ndeux\n\n- trois"
        );
    }

    #[test]
    fn une_ligne_vide_sans_selection_recoit_sa_marque() {
        let formatage = formater_lignes("", None, (0, 0), Genre::Puce).expect("formatage");

        assert_eq!(formatage.texte, "- ");
        assert_eq!(formatage.colonne_curseur, 2);
    }

    #[test]
    fn une_selection_de_lignes_vides_ne_fait_rien() {
        assert!(formater_lignes("\n\n", Some(((0, 0), (1, 0))), (1, 0), Genre::Puce).is_none());
    }

    #[test]
    fn une_puce_devient_un_numero_sans_garder_les_deux_marques() {
        let formatage =
            formater_lignes("- une\n- deux", None, (0, 0), Genre::Numero).expect("formatage");

        assert_eq!(appliquer("- une\n- deux", &formatage), "1. une\n- deux");
    }

    #[test]
    fn les_lignes_numerotees_se_suivent() {
        let formatage = formater_lignes("une\ndeux", Some(((0, 0), (2, 0))), (2, 0), Genre::Numero)
            .expect("formatage");

        assert_eq!(appliquer("une\ndeux", &formatage), "1. une\n2. deux");
    }
}
