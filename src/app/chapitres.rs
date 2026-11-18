//! Les chapitres d'un document : les têtes de niveau 1 du sommaire.
//!
//! Un chapitre n'est pas un fichier séparé : c'est une **tête de niveau 1**
//! dans `document.tw`, avec tout ce qui la suit jusqu'à la tête de niveau 1
//! suivante. La vue par chapitre ne réécrit jamais le document ; elle ne
//! montre qu'une tranche du rendu, et laisse la source entière dans l'éditeur.
//!
//! Les pages viennent du sommaire composé par Typst : une tranche de pages
//! est donc toujours celle du PDF réel, jamais une estimation.

use crate::typst::render::OutlineEntry;

/// Un chapitre du document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapitre {
    /// Son rang dans le document, à partir de zéro.
    pub index: usize,

    /// Le texte de la tête, sans son balisage.
    pub titre: String,

    /// La ligne de la tête dans `document.tw`, à partir de un.
    pub ligne: usize,

    /// La première ligne **après** le chapitre : la tête suivante, ou la fin
    /// du document. Sert à filtrer le sommaire sans dépendre des pages.
    pub ligne_fin: usize,

    /// La première page du chapitre, à partir de zéro.
    pub page: usize,

    /// La première page du chapitre suivant, ou le nombre total de pages.
    pub page_fin: usize,

    /// Les pages sont-elles connues ? Elles ne le sont qu'après la
    /// composition ; avant elle, la liste sert seulement à choisir un titre.
    pub pages_connues: bool,
}

impl Chapitre {
    /// Le nombre de pages affichées pour ce chapitre.
    pub fn pages(&self) -> usize {
        self.page_fin.saturating_sub(self.page).max(1)
    }

    /// La tête appartient-elle à ce chapitre, d'après sa ligne source ?
    pub fn contient_ligne(&self, ligne: usize) -> bool {
        ligne >= self.ligne && ligne < self.ligne_fin
    }
}

/// Construit les chapitres à partir du sommaire composé.
///
/// Seules les têtes de niveau 1 sont des chapitres. Le sommaire peut être
/// vide (document en cours de compilation), auquel cas il n'y a pas encore de
/// chapitres à proposer.
pub fn depuis_outline(outline: &[OutlineEntry], total_pages: usize) -> Vec<Chapitre> {
    let tetes: Vec<(usize, &OutlineEntry)> = outline
        .iter()
        .enumerate()
        .filter(|(_, entree)| entree.level == 1)
        .collect();

    tetes
        .iter()
        .enumerate()
        .map(|(rang, (_, entree))| {
            let page = entree.page.min(total_pages.saturating_sub(1));
            let page_fin = tetes
                .get(rang + 1)
                .map(|(_, suivante)| suivante.page)
                .unwrap_or(total_pages)
                .max(page + 1)
                .min(total_pages.max(1));
            let ligne_fin = tetes
                .get(rang + 1)
                .map(|(_, suivante)| suivante.line)
                .unwrap_or(usize::MAX);

            Chapitre {
                index: rang,
                titre: entree.text.trim().to_string(),
                ligne: entree.line,
                ligne_fin,
                page,
                page_fin,
                pages_connues: true,
            }
        })
        .collect()
}

/// Construit les chapitres depuis la source, avant la première compilation.
///
/// Les pages ne sont pas encore connues : elles seront remplacées par celles
/// du sommaire composé dès que le rendu sera prêt. La liste, elle, est déjà
/// utilisable pour filtrer le sommaire et choisir une tranche.
pub fn depuis_source(source: &str, document: &crate::langage::Document) -> Vec<Chapitre> {
    let mut tetes: Vec<(usize, String)> = crate::langage::parse::titres_source(source, document)
        .into_iter()
        .filter(|titre| titre.level == 1)
        .map(|titre| (titre.line, titre.text.trim().to_string()))
        .collect();

    tetes.sort_by_key(|(ligne, _)| *ligne);

    tetes
        .iter()
        .enumerate()
        .map(|(rang, (ligne, titre))| Chapitre {
            index: rang,
            titre: titre.clone(),
            ligne: *ligne,
            ligne_fin: tetes
                .get(rang + 1)
                .map(|(suivante, _)| *suivante)
                .unwrap_or(usize::MAX),
            // Sans composition, une tête ne peut pas encore désigner sa page.
            // Le sommaire composé corrigera ces deux valeurs au premier rendu.
            page: 0,
            page_fin: 1,
            pages_connues: false,
        })
        .collect()
}

/// Les entrées du sommaire qui appartiennent à une tranche de chapitre.
///
/// L'index retourné est celui du sommaire complet : un clic sur une tête
/// garde donc le même message et la même cible dans les deux vues.
pub fn filtrer_outline<'a>(
    outline: &'a [OutlineEntry],
    chapitre: Option<&Chapitre>,
) -> Vec<(usize, &'a OutlineEntry)> {
    outline
        .iter()
        .enumerate()
        .filter(|(_, entree)| chapitre.is_none_or(|chapitre| chapitre.contient_ligne(entree.line)))
        .collect()
}

/// La plage de pages affichée pour un chapitre.
///
/// `None` signifie le document entier. Une plage vide est ramenée à la
/// dernière page du document, pour ne jamais demander une page inexistante.
pub fn plage_pages(chapitre: Option<&Chapitre>, total_pages: usize) -> std::ops::Range<usize> {
    if total_pages == 0 {
        return 0..0;
    }

    let Some(chapitre) = chapitre else {
        return 0..total_pages;
    };

    let debut = chapitre.page.min(total_pages - 1);
    let fin = chapitre.page_fin.clamp(debut + 1, total_pages);
    debut..fin
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entree(level: usize, text: &str, line: usize, page: usize) -> OutlineEntry {
        OutlineEntry {
            level,
            text: text.to_string(),
            file: None,
            line,
            page,
        }
    }

    #[test]
    fn les_tetes_de_niveau_un_sont_les_chapitres() {
        let outline = vec![
            entree(1, "Ouverture", 1, 0),
            entree(2, "Une section", 3, 0),
            entree(1, "Deuxième", 10, 3),
            entree(2, "Une autre", 12, 4),
        ];

        let chapitres = depuis_outline(&outline, 7);

        assert_eq!(chapitres.len(), 2);
        assert_eq!(chapitres[0].titre, "Ouverture");
        assert_eq!(chapitres[0].page, 0);
        assert_eq!(chapitres[0].page_fin, 3);
        assert_eq!(chapitres[1].titre, "Deuxième");
        assert_eq!(chapitres[1].page, 3);
        assert_eq!(chapitres[1].page_fin, 7);
        assert_eq!(chapitres[1].pages(), 4);
    }

    #[test]
    fn une_tete_de_niveau_deux_n_est_pas_un_chapitre() {
        let outline = vec![entree(2, "Section", 1, 0), entree(3, "Détail", 4, 1)];

        assert!(depuis_outline(&outline, 3).is_empty());
    }

    #[test]
    fn le_sommaire_d_un_chapitre_ne_garde_que_ses_tetes() {
        let outline = vec![
            entree(1, "Ouverture", 1, 0),
            entree(2, "Section", 3, 1),
            entree(1, "Suite", 10, 4),
        ];
        let chapitres = depuis_outline(&outline, 8);

        let vues = filtrer_outline(&outline, Some(&chapitres[0]));

        assert_eq!(vues.len(), 2);
        assert_eq!(vues[0].0, 0);
        assert_eq!(vues[1].0, 1);
        assert_eq!(vues[0].1.text, "Ouverture");
        assert_eq!(vues[1].1.text, "Section");
    }

    #[test]
    fn une_plage_de_chapitre_est_bornee_au_document() {
        let outline = vec![entree(1, "Un", 1, 2), entree(1, "Deux", 10, 8)];
        let chapitres = depuis_outline(&outline, 8);

        assert_eq!(plage_pages(None, 8), 0..8);
        assert_eq!(plage_pages(Some(&chapitres[0]), 8), 2..8);
        assert_eq!(plage_pages(Some(&chapitres[1]), 8), 7..8);
        assert_eq!(plage_pages(Some(&chapitres[0]), 0), 0..0);
    }

    #[test]
    fn les_chapitres_se_lisent_dans_la_source() {
        let source = "# Ouverture\n\nUn texte.\n\n## Section\n\n# Suite\n";
        let document = crate::langage::parse(source);

        let chapitres = depuis_source(source, &document);

        assert_eq!(chapitres.len(), 2);
        assert_eq!(chapitres[0].ligne, 1);
        assert_eq!(chapitres[0].ligne_fin, 7);
        assert_eq!(chapitres[1].ligne, 7);
    }
}
