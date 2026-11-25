//! Le volume d'un texte : les mots qui appartiennent à l'auteur, les notes,
//! et la part des citations de bloc.
//!
//! Une note de bas de page est du texte de l'auteur, mais elle ne compte pas
//! dans le volume du corps : c'est la distinction que demande le compteur
//! d'écriture. Un bloc `>` est aussi du texte, mais il vient d'ailleurs ; il
//! reste dans le volume total et se compte à part, pour que l'auteur puisse
//! voir la place qu'il occupe.
//!
//! Le calcul part de l'AST, jamais du source balisé : les marqueurs, les clés
//! de citation et les délimiteurs ne sont pas des mots.

use super::schema::{Bloc, Document, Inline};

/// Les mots d'un document, séparés par leur provenance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Volume {
    /// Les mots du corps, notes exclues.
    pub mots: usize,

    /// Les mots des notes de bas de page, hors du volume du corps.
    pub mots_notes: usize,

    /// Les mots des citations de bloc (`>`), inclus dans `mots`.
    pub mots_citations: usize,
}

impl Volume {
    /// La part des citations de bloc dans le volume du corps, entre 0 et 1.
    pub fn part_des_citations(&self) -> f32 {
        if self.mots == 0 {
            0.0
        } else {
            self.mots_citations as f32 / self.mots as f32
        }
    }
}

/// Mesure un document déjà analysé.
pub fn mesurer(document: &Document) -> Volume {
    let mut volume = Volume::default();

    for bloc in &document.blocs {
        mesurer_bloc(bloc, &mut volume, false);
    }

    volume
}

/// Mesure un fragment de source — une sélection de l'éditeur, par exemple.
pub fn mesurer_texte(source: &str) -> Volume {
    mesurer(&super::parse::parse(source))
}

fn mesurer_bloc(bloc: &Bloc, volume: &mut Volume, dans_citation: bool) {
    match bloc {
        Bloc::Titre { contenu, .. } | Bloc::Paragraphe(contenu) => {
            mesurer_inline(contenu, volume, false, dans_citation)
        }
        Bloc::Citation(contenu) => mesurer_inline(contenu, volume, false, true),
        Bloc::ListePuce(items) | Bloc::ListeNumero(items) => {
            for item in items {
                mesurer_inline(&item.contenu, volume, false, dans_citation);
            }
        }
        Bloc::Tableau {
            entete,
            lignes,
            titre,
            ..
        } => {
            // Le titre est du texte visible, comme la légende d'une figure.
            ajouter(volume, titre, false, dans_citation);
            for cellule in entete.iter().chain(lignes.iter().flatten()) {
                mesurer_inline(cellule, volume, false, dans_citation);
            }
        }
        Bloc::Image { légende, .. } => mesurer_inline(légende, volume, false, dans_citation),
        Bloc::Décor { bloc, .. } | Bloc::Canonique(bloc) => {
            mesurer_bloc(bloc, volume, dans_citation)
        }
        Bloc::Garde(contenu) => {
            for bloc in contenu {
                mesurer_bloc(bloc, volume, dans_citation);
            }
        }
        // Un bloc brut, une formule ou une sortie de plugin n'est pas du texte
        // rédigé : il ne gonfle pas le compte de mots.
        Bloc::Brut(_)
        | Bloc::Formule(_)
        | Bloc::Graphique { .. }
        | Bloc::Chart(_)
        | Bloc::Glossaire { .. }
        | Bloc::Bibliographie
        | Bloc::IndexBiblique
        | Bloc::IndexPersonnes
        | Bloc::Sommaire
        | Bloc::Separateur
        | Bloc::Commentaire(_)
        | Bloc::SautDePage => {}
    }
}

fn mesurer_inline(inline: &[Inline], volume: &mut Volume, dans_note: bool, dans_citation: bool) {
    for element in inline {
        match element {
            Inline::Texte(texte) | Inline::Litteral(texte) => {
                ajouter(volume, texte, dans_note, dans_citation)
            }
            Inline::Bible(elements) => {
                let texte = super::references::afficher_elements(
                    elements,
                    &crate::mise_en_page::modele::References::default(),
                );
                ajouter(volume, &texte, dans_note, dans_citation);
            }
            Inline::BibleForme { elements, forme } => {
                let texte = super::references::afficher_elements_forme(
                    elements,
                    &crate::mise_en_page::modele::References::default(),
                    Some(*forme),
                );
                ajouter(volume, &texte, dans_note, dans_citation);
            }
            Inline::Gras(contenu)
            | Inline::Italique(contenu)
            | Inline::GrasItalique(contenu)
            | Inline::Souligne(contenu)
            | Inline::Barre(contenu)
            | Inline::Couleur { contenu, .. } => {
                mesurer_inline(contenu, volume, dans_note, dans_citation)
            }
            Inline::Personne { cle, contenu } => {
                if contenu.is_empty() {
                    ajouter(volume, cle, dans_note, dans_citation);
                } else {
                    mesurer_inline(contenu, volume, dans_note, dans_citation);
                }
            }
            Inline::Note(contenu) => mesurer_inline(contenu, volume, true, false),
            Inline::Code(texte) | Inline::Formule(texte) => {
                ajouter(volume, texte, dans_note, dans_citation)
            }
            Inline::Date(spec) => ajouter(volume, spec, dans_note, dans_citation),
            Inline::Meta(cle) | Inline::Glo { cle, .. } => {
                ajouter(volume, cle, dans_note, dans_citation)
            }
            Inline::Lien { texte, .. } | Inline::LienInterne { texte, .. } => {
                mesurer_inline(texte, volume, dans_note, dans_citation)
            }
            Inline::Canonique(interne) => mesurer_inline(
                std::slice::from_ref(interne.as_ref()),
                volume,
                dans_note,
                dans_citation,
            ),
            // Un appel bibliographique n'est pas un mot du texte : il se
            // compose en note, et son contenu appartient à la bibliographie.
            Inline::Citation(_) => {}
        }
    }
}

fn ajouter(volume: &mut Volume, texte: &str, dans_note: bool, dans_citation: bool) {
    let mots = crate::project::count_words(texte);

    if dans_note {
        volume.mots_notes += mots;
    } else {
        volume.mots += mots;
        if dans_citation {
            volume.mots_citations += mots;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_notes_sont_comptees_a_part_et_les_citations_aussi() {
        let document = super::super::parse::parse(
            "Un deux trois.\n\n> Quatre cinq six.\n\nUn mot^[une note de bas de page].\n",
        );

        let volume = mesurer(&document);

        assert_eq!(volume.mots, 8, "le corps sans la note");
        assert_eq!(volume.mots_notes, 6, "la note est mesurée à part");
        assert_eq!(volume.mots_citations, 3, "le bloc `>` est une citation");
        assert!((volume.part_des_citations() - 3.0 / 8.0).abs() < 1.0e-6);
    }

    #[test]
    fn une_balise_de_tableau_seule_ne_fait_pas_tomber_le_compte() {
        let volume = mesurer_texte("{{t \"Taux\"}}");
        assert_eq!(volume.mots, 0);
    }

    #[test]
    fn un_document_sans_mot_ne_divise_pas_par_zero() {
        assert_eq!(mesurer_texte("").part_des_citations(), 0.0);
    }
}
