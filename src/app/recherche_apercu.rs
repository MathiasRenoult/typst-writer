//! Recherche dans la couche texte du document rendu.
//!
//! Le motif est littéral, insensible à la casse et aux accents français. On
//! cherche dans le **document composé** — notes, en-têtes et bibliographie
//! compris — parce que c'est ce que l'auteur voit dans le PDF.

use crate::typst::texte::{DocumentTexte, PageTexte, plier};

/// Une recherche ouverte dans l'aperçu.
#[derive(Debug, Clone, Default)]
pub struct RechercheApercu {
    /// Le motif saisi.
    pub motif: String,
    /// Les occurrences trouvées, dans l'ordre du document.
    pub resultats: Vec<ResultatApercu>,
    /// L'occurrence courante.
    pub courante: usize,
    /// La barre de recherche est-elle visible ?
    pub visible: bool,
}

/// Une occurrence dans une page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultatApercu {
    /// Page de l'occurrence.
    pub page: usize,
    /// Premier glyphe de l'occurrence.
    pub debut: usize,
    /// Dernier glyphe de l'occurrence, inclus.
    pub fin: usize,
}

impl RechercheApercu {
    /// Ouvre la recherche sur un motif vide.
    pub fn ouvrir(&mut self) {
        self.visible = true;
    }

    /// Ferme la barre sans oublier le motif.
    pub fn fermer(&mut self) {
        self.visible = false;
    }

    /// L'occurrence courante, si elle existe.
    pub fn courante(&self) -> Option<ResultatApercu> {
        self.resultats.get(self.courante).copied()
    }

    /// Passe à l'occurrence suivante ou précédente.
    pub fn avancer(&mut self, direction: i8) {
        if self.resultats.is_empty() {
            self.courante = 0;
            return;
        }

        let total = self.resultats.len();
        if direction > 0 {
            self.courante = (self.courante + 1) % total;
        } else if direction < 0 {
            self.courante = (self.courante + total - 1) % total;
        }
    }

    /// Le libellé du compteur `3/17`.
    pub fn compteur(&self) -> String {
        if self.resultats.is_empty() {
            "0/0".to_string()
        } else {
            format!("{}/{}", self.courante + 1, self.resultats.len())
        }
    }
}

/// Cherche un motif dans tout le document.
pub fn chercher(document: &DocumentTexte, motif: &str) -> Vec<ResultatApercu> {
    let motif: Vec<char> = motif
        .chars()
        .map(plier)
        .filter(|caractere| !caractere.is_whitespace())
        .collect();

    if motif.is_empty() {
        return Vec::new();
    }

    let mut resultats = Vec::new();

    for (page_index, page) in document.pages.iter().enumerate() {
        resultats.extend(chercher_dans_page(page_index, page, &motif));
    }

    resultats
}

fn chercher_dans_page(page_index: usize, page: &PageTexte, motif: &[char]) -> Vec<ResultatApercu> {
    let texte: Vec<char> = page
        .recherche
        .chars()
        .filter(|caractere| !caractere.is_whitespace())
        .collect();

    if texte.is_empty() || motif.len() > texte.len() {
        return Vec::new();
    }

    // Les ancres sont alignées sur `page.recherche`, qui peut contenir des
    // blancs que la recherche ignore. Reconstruire la même suite filtrée
    // garde la correspondance avec les glyphes.
    let ancres: Vec<Option<usize>> = page
        .recherche
        .chars()
        .zip(page.ancres.iter().copied())
        .filter(|(caractere, _)| !caractere.is_whitespace())
        .map(|(_, ancre)| ancre)
        .collect();

    // Une couche incohérente ne doit jamais faire tomber l'interface : sans
    // correspondance exacte entre le texte plié et ses ancres, on ne cherche
    // pas dans cette page.
    if ancres.len() != texte.len() {
        return Vec::new();
    }

    let mut resultats = Vec::new();
    let mut position = 0;

    while position + motif.len() <= texte.len() {
        if texte[position..position + motif.len()] == *motif {
            let debut = ancres[position..position + motif.len()]
                .iter()
                .flatten()
                .next()
                .copied();
            let fin = ancres[position..position + motif.len()]
                .iter()
                .rev()
                .flatten()
                .next()
                .copied();

            if let (Some(debut), Some(fin)) = (debut, fin) {
                resultats.push(ResultatApercu {
                    page: page_index,
                    debut,
                    fin,
                });
            }

            position += motif.len().max(1);
        } else {
            position += 1;
        }
    }

    resultats
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::layout::{Abs, Point, Rect, Size};
    use typst::syntax::Span;

    fn page(texte: &str) -> PageTexte {
        let mut page = PageTexte {
            taille: Size::new(Abs::pt(500.0), Abs::pt(700.0)),
            glyphes: Vec::new(),
            liens: Vec::new(),
            texte: String::new(),
            recherche: String::new(),
            ancres: Vec::new(),
        };
        let mut x = 0.0;

        for (rang, caractere) in texte.chars().enumerate() {
            let texte = caractere.to_string();
            page.glyphes.push(crate::typst::texte::GlyphePlace {
                texte: texte.clone().into(),
                rect: Rect::new(
                    Point::new(Abs::pt(x), Abs::pt(0.0)),
                    Point::new(Abs::pt(x + 6.0), Abs::pt(10.0)),
                ),
                ligne: 0,
                span: Span::detached(),
            });
            page.texte.push(caractere);
            page.recherche.push(crate::typst::texte::plier(caractere));
            page.ancres.push(Some(rang));
            x += 6.0;
        }

        page
    }

    fn document(texte: &str) -> DocumentTexte {
        DocumentTexte {
            pages: vec![page(texte)],
        }
    }

    #[test]
    fn trouve_les_occurrences_avec_casse_et_accents() {
        let document = document("Église et église, EGLISE.");
        let resultats = chercher(&document, "eglise");

        assert_eq!(resultats.len(), 3);
        assert_eq!(resultats[0].debut, 0);
        assert_eq!(resultats[1].debut, 10);
    }

    #[test]
    fn un_motif_vide_ne_trouve_rien() {
        let document = document("Un texte.");
        assert!(chercher(&document, "").is_empty());
        assert!(chercher(&document, "   ").is_empty());
    }

    #[test]
    fn plusieurs_pages_sont_parcourues_dans_l_ordre() {
        let document = DocumentTexte {
            pages: vec![page("Première page."), page("Seconde page.")],
        };
        let resultats = chercher(&document, "page");

        assert_eq!(resultats.len(), 2);
        assert_eq!(resultats[0].page, 0);
        assert_eq!(resultats[1].page, 1);
    }

    #[test]
    fn le_compteur_et_le_deplacement_restent_bornes() {
        let mut recherche = RechercheApercu {
            motif: "test".to_string(),
            resultats: vec![
                ResultatApercu {
                    page: 0,
                    debut: 0,
                    fin: 1,
                },
                ResultatApercu {
                    page: 0,
                    debut: 2,
                    fin: 3,
                },
            ],
            ..Default::default()
        };

        assert_eq!(recherche.compteur(), "1/2");
        recherche.avancer(1);
        assert_eq!(recherche.compteur(), "2/2");
        recherche.avancer(1);
        assert_eq!(recherche.compteur(), "1/2");
        recherche.avancer(-1);
        assert_eq!(recherche.compteur(), "2/2");
    }
}
