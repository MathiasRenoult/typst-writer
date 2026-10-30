//! Les marques de notes : des plages de texte fondues en un seul glyphe.
//!
//! Une citation groupée est **une** note — un seul appel de note réunit ses
//! références —, et une note libre `^[…]` en est une autre : toutes deux
//! s'écrivent d'un petit numéro en exposant, à la place du texte qu'elles
//! portent. Le texte, lui, reste le vrai : c'est la mise en page qui le tait.
//!
//! Une marque est une plage de colonnes de sa ligne logique. Sous elle, les
//! caractères ne tiennent plus : ils valent zéro de largeur, et le premier
//! d'entre eux porte la largeur du glyphe qui les remplace tous. Cette règle
//! unique réfracte l'habillage, la géométrie et les clics : partout où une
//! largeur se mesure, la marque est consultée, et la ligne se referme sur le
//! glyphe comme si le texte fondu n'avait jamais existé.

use crate::canvas_editor::{CodeEditor, measure_char_width};

/// Une plage de colonnes fondue en un glyphe.
///
/// Les plages d'une **même note** portent le même `groupe` : une note qui
/// court sur plusieurs lignes a une marque par rangon, mais c'est un seul
/// caractère — le curseur la saute, la suppression l'emporte, d'un bout à
/// l'autre du groupe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marque {
    /// Le groupe : la note entière, même sur plusieurs lignes.
    pub groupe: usize,
    /// La ligne logique où commence la plage.
    pub ligne: usize,
    /// La première colonne de la plage, en caractères de la ligne.
    pub debut: usize,
    /// La colonne qui suit la dernière : la plage est `[debut, fin)`.
    pub fin: usize,
    /// Le glyphe dessiné à la place du texte : le numéro de la note.
    pub label: String,
}

impl Marque {
    /// Une marque couvrant `[debut, fin)` de `ligne`, notée `label`.
    pub fn new(
        groupe: usize,
        ligne: usize,
        debut: usize,
        fin: usize,
        label: impl Into<String>,
    ) -> Self {
        Self {
            groupe,
            ligne,
            debut,
            fin,
            label: label.into(),
        }
    }

    /// La plage couvre-t-elle cette colonne ?
    pub fn couvre(&self, col: usize) -> bool {
        self.debut <= col && col < self.fin
    }
}

/// Les deux bords d'un groupe — la note entière, d'un bout à l'autre.
///
/// `((ligne, colonne) du début, (ligne, colonne) de la fin)`.
pub(crate) fn bords_du_groupe(
    marques: &[Marque],
    groupe: usize,
) -> Option<((usize, usize), (usize, usize))> {
    let mut bouts = marques.iter().filter(|m| m.groupe == groupe);
    let premier = bouts.next()?;

    let mut debut = (premier.ligne, premier.debut);
    let mut fin = (premier.ligne, premier.fin);

    for marque in bouts {
        if (marque.ligne, marque.debut) < debut {
            debut = (marque.ligne, marque.debut);
        }
        if (marque.ligne, marque.fin) > fin {
            fin = (marque.ligne, marque.fin);
        }
    }

    Some((debut, fin))
}

/// Les marques d'une ligne logique, prêtes à être consultées colonne par
/// colonne.
///
/// Les marches de largeur passent toutes par ici : on filtre une fois par
/// ligne, puis chaque caractère ne consulte que les marques de la sienne.
#[derive(Debug, Clone, Default)]
pub(crate) struct MarquesLigne<'a> {
    marques: Vec<&'a Marque>,
}

impl<'a> MarquesLigne<'a> {
    /// Les marques de `ligne`, dans l'ordre des colonnes.
    pub(crate) fn new(marques: &'a [Marque], ligne: usize) -> Self {
        Self {
            marques: marques.iter().filter(|m| m.ligne == ligne).collect(),
        }
    }

    /// Aucune marque.
    pub(crate) fn vide() -> Self {
        Self {
            marques: Vec::new(),
        }
    }

    /// La marque qui tient cette colonne, s'il y en a une.
    pub(crate) fn sous(&self, col: usize) -> Option<&'a Marque> {
        self.marques.iter().copied().find(|m| m.couvre(col))
    }

    /// Les marques de la ligne, dans l'ordre.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &'a Marque> {
        self.marques.iter().copied()
    }

    /// Une marque est-elle posée sur cette ligne ?
    #[cfg(test)]
    pub(crate) fn est_vide(&self) -> bool {
        self.marques.is_empty()
    }
}

/// La largeur d'un glyphe : la place du numéro en exposant.
///
/// Assez pour le chiffre — ou les chiffres, une note cent trente — et un
/// souffle autour, sans jamais reprendre toute la place du texte fondu.
pub(crate) fn largeur_marque(marque: &Marque, char_width: f32) -> f32 {
    let chiffres = marque.label.chars().count().max(1) as f32;

    char_width * (0.30 + 0.55 * chiffres)
}

/// La largeur d'un caractère de la ligne, marques fondues.
///
/// Sous une marque, le texte ne tient plus : ses caractères valent zéro, et
/// le premier porte la largeur du glyphe qui les remplace tous. Ailleurs,
/// c'est la mesure habituelle — y compris les onglets.
pub(crate) fn largeur_caractere(
    c: char,
    col: usize,
    marques: &MarquesLigne<'_>,
    full_char_width: f32,
    char_width: f32,
) -> f32 {
    match marques.sous(col) {
        Some(marque) if marque.debut == col => largeur_marque(marque, char_width),
        Some(_) => 0.0,
        None => measure_char_width(c, full_char_width, char_width),
    }
}

/// La largeur d'un tronçon de ligne, marques fondues.
///
/// `col_premier` est la colonne, dans la ligne logique, du premier caractère
/// de `texte` : les marques se lisent aux colonnes absolues.
pub(crate) fn largeur_troncon(
    texte: &str,
    col_premier: usize,
    marques: &MarquesLigne<'_>,
    full_char_width: f32,
    char_width: f32,
) -> f32 {
    texte
        .chars()
        .enumerate()
        .map(|(i, c)| {
            largeur_caractere(
                c,
                col_premier + i,
                marques,
                full_char_width,
                char_width,
            )
        })
        .sum()
}

/// Un bout de tronçon à dessiner : ce qui se lit, ou un glyphe.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Bout<'a> {
    /// Du texte qui se voit.
    Texte(&'a str),
    /// Un glyphe, à la place du texte fondu qu'il remplace.
    Glyphe(&'a Marque),
}

/// Découpe un tronçon à dessiner en bouts : le texte lisible, et les glyphes
/// qui prennent la place du texte fondu.
///
/// Le texte fondu sort entièrement du dessin — c'est le glyphe qui le tient —
/// et un glyphe ne se pose qu'une fois, là où sa marque commence, même si ses
/// caractères s'étirent sur plusieurs tronçons de couleur.
pub(crate) fn bouts_du_troncon<'a>(
    texte: &'a str,
    col_premier: usize,
    marques: &MarquesLigne<'a>,
) -> Vec<Bout<'a>> {
    let mut bouts = Vec::new();

    if marques.iter().next().is_none() {
        if !texte.is_empty() {
            bouts.push(Bout::Texte(texte));
        }

        return bouts;
    }

    let mut texte_debut = 0usize;
    let mut en_cours = false;
    for (col, (byte, _)) in texte.char_indices().enumerate() {
        let colonne = col_premier + col;

        match marques.sous(colonne) {
            Some(marque) if marque.debut == colonne => {
                if en_cours {
                    bouts.push(Bout::Texte(&texte[texte_debut..byte]));
                    en_cours = false;
                }

                bouts.push(Bout::Glyphe(marque));
            }
            Some(_) => {
                // La suite d'une marque : son texte fondu ne se dessine pas.
                if en_cours {
                    bouts.push(Bout::Texte(&texte[texte_debut..byte]));
                    en_cours = false;
                }
            }
            None => {
                if !en_cours {
                    texte_debut = byte;
                    en_cours = true;
                }
            }
        }
    }

    if en_cours {
        bouts.push(Bout::Texte(&texte[texte_debut..]));
    }

    bouts
}

impl CodeEditor {
    /// Pose les marques de notes : les plages que la ligne fond en un glyphe.
    ///
    /// L'appelant dit où sont les notes — lui seul lit la langue —, et le
    /// rendu ne reçoit qu'une règle de largeur. La révision monte : les
    /// caches d'habillage savent alors que la ligne a changé de forme.
    pub fn set_marques(&mut self, marques: Vec<Marque>) {
        self.marques = marques;
        self.marques_revision = self.marques_revision.wrapping_add(1);
    }

    /// Les marques posées, pour que l'hôte les compare aux siennes et ne
    /// dérange pas le cache d'habillage quand rien n'a bougé.
    pub fn marques(&self) -> &[Marque] {
        &self.marques
    }

    /// Le nombre de glyphes posés — pour l'hôte, qui doit pouvoir vérifier
    /// qu'il n'y a jamais de texte à nu là où un glyphe devrait être.
    pub fn nombre_de_marques(&self) -> usize {
        self.marques.len()
    }

    /// La marque touchée par le dernier clic, s'il a visé un glyphe :
    /// (ligne, début, fin) de la plage, ou `None`.
    pub fn dernier_clic_sur_marque(&self) -> Option<(usize, usize, usize)> {
        self.dernier_clic_marque
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: f32 = 16.0;
    const NARROW: f32 = 8.0;

    /// Une ligne de marque : `@dupont2023[p. 45]`, colonnes 0 à 18.
    fn marque() -> Marque {
        Marque::new(0, 0, 0, 18, "2")
    }

    /// Sous la marque, le texte ne tient plus : zéro par caractère, et le
    /// premier porte le glyphe.
    #[test]
    fn le_texte_fondu_ne_tient_plus() {
        let liste = [marque()];
        let marques = MarquesLigne::new(&liste, 0);

        assert_eq!(
            largeur_caractere('@', 0, &marques, FULL, NARROW),
            largeur_marque(&marque(), NARROW),
            "le premier caractère porte le glyphe"
        );
        assert_eq!(
            largeur_caractere('d', 1, &marques, FULL, NARROW),
            0.0,
            "le reste du texte fondu ne prend pas de place"
        );
    }

    /// Hors de la marque, la mesure est la mesure habituelle.
    #[test]
    fn hors_de_la_marque_rien_ne_change() {
        let liste = [marque()];
        let marques = MarquesLigne::new(&liste, 0);

        assert_eq!(
            largeur_caractere('x', 40, &marques, FULL, NARROW),
            NARROW,
            "la colonne 40 n'est dans aucune marque"
        );
    }

    /// La largeur du tronçon tient compte de la marque : toute la ligne
    /// `@dupont2023[p. 45]` ne vaut qu'un glyphe.
    #[test]
    fn un_troncon_fondu_ne_vaut_qu_un_glyphe() {
        let liste = [marque()];
        let marques = MarquesLigne::new(&liste, 0);

        assert_eq!(
            largeur_troncon("@dupont2023[p. 45]", 0, &marques, FULL, NARROW),
            largeur_marque(&marque(), NARROW)
        );
    }
}
