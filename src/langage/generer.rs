//! Le générateur : Document structuré → Typst.
//!
//! C'est la traduction qui alimente le moteur de rendu existant
//! (`crate::typst::render`), qu'on réutilise tel quel. Le résultat est le
//! **corps d'un chapitre** : l'assemblage (`main.typ`) garde l'en-tête de mise
//! en page et la bibliographie, et `#include` ce texte. Un chapitre n'a donc
//! pas à réimporter les normes — le faire les appliquerait deux fois.
//!
//! Le Typst reste le maître du rendu final : tout ce que le langage ne sait
//! pas exprimer passe par un bloc brut, inséré tel quel.
//!
//! Le texte est **échappé** : une astérisque dans une phrase reste une
//! astérisque, elle n'ouvre pas de gras. C'est la contrepartie du texte nu que
//! le sérialiseur produit.

use super::schema::{Bloc, Document, Inline};

/// Génère le corps Typst d'un document.
pub fn generer(document: &Document) -> String {
    let mut out = String::new();

    for bloc in &document.blocs {
        generer_bloc(bloc, &mut out);
    }

    out
}

/// Génère un bloc, suivi d'une ligne vide — la séparation de blocs de Typst.
fn generer_bloc(bloc: &Bloc, out: &mut String) {
    match bloc {
        Bloc::Titre { niveau, contenu } => {
            out.push_str(&"=".repeat(*niveau as usize));
            out.push(' ');
            generer_inline(contenu, out, false);
            out.push_str("\n\n");
        }
        Bloc::Paragraphe(contenu) => {
            generer_inline(contenu, out, true);
            out.push_str("\n\n");
        }
        Bloc::Citation(contenu) => {
            out.push_str("#quote(block: true)[");
            generer_inline(contenu, out, false);
            out.push_str("]\n\n");
        }
        Bloc::ListePuce(items) => {
            for item in items {
                out.push_str("- ");
                generer_inline(item, out, false);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::ListeNumero(items) => {
            // Typst numérote avec `+`, et non avec un chiffre : il suit l'ordre
            // autant que l'usure des listes réordonnées.
            for item in items {
                out.push_str("+ ");
                generer_inline(item, out, false);
                out.push('\n');
            }

            out.push('\n');
        }
        Bloc::Brut(texte) => {
            out.push_str(texte);
            out.push_str("\n\n");
        }
    }
}

/// Génère une suite d'éléments inline.
///
/// `debut_de_ligne` dit que le premier caractère écrit tombe en tête de ligne :
/// une `-`, un `+`, un `=` ou un `/` y ouvrirait une liste ou un titre, et doit
/// donc être échappé.
fn generer_inline(inline: &[Inline], out: &mut String, debut_de_ligne: bool) {
    let mut en_tete = debut_de_ligne;

    for element in inline {
        match element {
            Inline::Texte(texte) => {
                echapper_texte(texte, out, en_tete);
                en_tete = false;
            }
            Inline::Gras(contenu) => {
                out.push('*');
                generer_inline(contenu, out, false);
                out.push('*');
                en_tete = false;
            }
            Inline::Italique(contenu) => {
                out.push('_');
                generer_inline(contenu, out, false);
                out.push('_');
                en_tete = false;
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("*_");
                generer_inline(contenu, out, false);
                out.push_str("_*");
                en_tete = false;
            }
            Inline::Code(texte) => {
                out.push('`');
                out.push_str(texte);
                out.push('`');
                en_tete = false;
            }
            Inline::Couleur { couleur, contenu } => {
                out.push_str("#text(fill: ");
                out.push_str(couleur);
                out.push_str(")[");
                generer_inline(contenu, out, false);
                out.push(']');
                en_tete = false;
            }
            Inline::Citation(cle) => {
                out.push('@');
                out.push_str(cle);
                en_tete = false;
            }
            Inline::Note(contenu) => {
                out.push_str("#footnote[");
                generer_inline(contenu, out, false);
                out.push(']');
                en_tete = false;
            }
            Inline::Lien { texte, cible } => {
                out.push_str("#link(\"");
                out.push_str(cible);
                out.push_str("\")[");
                generer_inline(texte, out, false);
                out.push(']');
                en_tete = false;
            }
        }
    }
}

/// Écrit du texte en échappant ce que Typst lirait comme du balisage.
fn echapper_texte(texte: &str, out: &mut String, debut_de_ligne: bool) {
    for (index, caractere) in texte.chars().enumerate() {
        let echappe = match caractere {
            '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '^' | '<' | '>' | '[' | ']' | '~' => true,
            // En tête de ligne seulement : ailleurs, ces signes sont du texte.
            '-' | '+' | '=' | '/' => index == 0 && debut_de_ligne,
            _ => false,
        };

        if echappe {
            out.push('\\');
        }

        out.push(caractere);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langage::parse::parse;
    use std::path::PathBuf;

    /// Compile un texte généré par le vrai moteur Typst.
    ///
    /// C'est la seule preuve qui vaille pour un générateur : le texte peut être
    /// bien formé et refusé, ou l'inverse. On compile l'assemblage minimal —
    /// un `#include` du chapitre — avec le texte en mémoire.
    fn compiler(typst: &str) -> crate::typst::render::CompileOutcome {
        let root = PathBuf::from("/projet");
        let entry = "#include \"chapitre.typ\"\n".to_string();
        let overrides = vec![(root.join("chapitre.typ"), typst.to_string())];

        crate::typst::render::compile(&root, &entry, &overrides)
    }

    /// Le document généré compile sans erreur.
    #[test]
    fn le_texte_genere_compile() {
        let texte = "# Titre\n\n## Sous-titre\n\nUn paragraphe avec du **gras**, de l'*italique*, du `code`, du ***gras italique***, un [lien](https://exemple.fr) et du {red:texte rouge} et une note ^[une précision].\n\n> Une citation de bloc.\n\n- une puce\n- une autre\n\n1. un élément\n2. un autre\n\n```typst\n#let repère = 1\n```\n";

        let document = parse(texte);
        let outcome = compiler(&generer(&document));

        assert!(
            outcome.errors.is_empty(),
            "le Typst généré doit compiler : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Le texte qui ressemble à du balisage ne devient pas du balisage.
    ///
    /// Sans échappement, « 2 * 3 » ouvrirait un italique jamais fermé, et
    /// « # » en tête de paragraphe ferait une erreur de compilation.
    #[test]
    fn un_texte_qui_ressemble_a_un_marqueur_reste_du_texte() {
        let texte = "Un calcul : 2 * 3 * 4 = 24, et un # en tête.\n\n# Un titre\n";

        let document = parse(texte);
        let outcome = compiler(&generer(&document));

        assert!(
            outcome.errors.is_empty(),
            "le texte nu doit être échappé : {:?}",
            outcome
                .errors
                .iter()
                .map(|erreur| erreur.message.clone())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            outcome
                .document
                .as_deref()
                .map(crate::typst::render::page_count),
            Some(1)
        );
    }

    /// Les blocs deviennent le Typst attendu.
    #[test]
    fn les_blocs_deviennent_le_typst_attendu() {
        let document = parse(
            "# Titre\n\nUn **gras** et un *italique*.\n\n> Une citation.\n\n- une puce\n\n1. un\n",
        );

        let typst = generer(&document);

        assert!(typst.contains("= Titre\n\n"), "{typst}");
        assert!(typst.contains("*gras*"), "{typst}");
        assert!(typst.contains("_italique_"), "{typst}");
        assert!(
            typst.contains("#quote(block: true)[Une citation.]"),
            "{typst}"
        );
        assert!(typst.contains("- une puce"), "{typst}");
        assert!(typst.contains("+ un\n"), "{typst}");
    }

    /// Une citation bibliographique se génère en référence Typst.
    #[test]
    fn une_citation_bibliographique_devient_une_reference() {
        let document = parse("Voir @dupont2023.\n");

        assert!(generer(&document).contains("@dupont2023"));
    }

    /// Le corps généré n'importe pas l'en-tête : l'assemblage s'en charge, et
    /// l'importer deux fois appliquerait les normes deux fois.
    #[test]
    fn le_corps_genere_n_importe_pas_l_entete() {
        let typst = generer(&parse("Bonjour.\n"));

        assert!(!typst.contains("#import"), "{typst}");
        assert!(!typst.contains("#show"), "{typst}");
        assert!(!typst.contains("#bibliography"), "{typst}");
    }
}
