//! Le sérialiseur : Document structuré → texte (notre syntaxe).
//!
//! C'est l'écriture sur disque. Le texte produit doit être exactement celui que
//! le parseur relit — c'est la moitié « retour » du round-trip, l'autre étant
//! [`crate::langage::parse`].
//!
//! Le sérialiseur **échappe** ce qui ressemblerait à du balisage : un texte qui
//! contient une étoile, un arobase ou une accolade reste du texte. Sans cela,
//! le round-trip perdrait les documents qui parlent du langage lui-même — et
//! une note de bas de page commençant par `#` deviendrait un titre.

use super::schema::{Bloc, Document, Inline};

/// Caractères qui ouvrent un marqueur inline, et qu'un texte nu doit échapper.
///
/// L'ordre n'importe pas : chacun est échappé individuellement.
const MARQUEURS: [char; 7] = ['\\', '*', '`', '@', '[', '{', '^'];

/// Sérialise un document dans notre syntaxe.
///
/// Les blocs sont séparés par une ligne vide, et le texte se termine par un
/// saut de ligne. Un document vide donne une chaîne vide : c'est ce que le
/// parseur relit comme un document vide.
pub fn serialiser(document: &Document) -> String {
    let mut out = String::new();

    for (index, bloc) in document.blocs.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        match bloc {
            Bloc::Titre { niveau, contenu } => {
                out.push_str(&"#".repeat(*niveau as usize));
                out.push(' ');
                ecrire_inline(contenu, &mut out);
                out.push('\n');
            }
            Bloc::Paragraphe(contenu) => {
                ecrire_ligne(contenu, &mut out);
                out.push('\n');
            }
            Bloc::Citation(contenu) => {
                out.push_str("> ");
                ecrire_inline(contenu, &mut out);
                out.push('\n');
            }
            Bloc::ListePuce(items) => {
                for item in items {
                    out.push_str("- ");
                    ecrire_inline(item, &mut out);
                    out.push('\n');
                }
            }
            Bloc::ListeNumero(items) => {
                for (numero, item) in items.iter().enumerate() {
                    out.push_str(&format!("{}. ", numero + 1));
                    ecrire_inline(item, &mut out);
                    out.push('\n');
                }
            }
            Bloc::Brut(texte) => {
                let fence = fence_for_raw(texte);
                out.push_str(&fence);
                out.push_str("typst\n");
                out.push_str(texte);
                out.push('\n');
                out.push_str(&fence);
                out.push('\n');
            }
        }
    }

    out
}

/// Choisit un fence plus long que tout fence au début d'une ligne du contenu.
fn fence_for_raw(texte: &str) -> String {
    let longest = texte
        .lines()
        .map(|ligne| {
            ligne
                .trim_start()
                .chars()
                .take_while(|caractere| *caractere == '`')
                .count()
        })
        .max()
        .unwrap_or(0);

    "`".repeat(longest.max(2) + 1)
}

/// Écrit une ligne de texte, en échappant un marqueur de bloc en tête.
///
/// Un paragraphe qui commence par `- `, `>`, `# `, un numéro suivi de `. ` ou
/// une clôture de bloc brut serait relu comme un autre bloc. Le sérialiseur le
/// protège par une barre oblique, que le parseur sait retirer.
fn ecrire_ligne(contenu: &[Inline], out: &mut String) {
    let debut = out.len();

    ecrire_inline(contenu, out);

    if ouvre_un_bloc(&out[debut..]) {
        out.insert(debut, '\\');
    }
}

/// La même reconnaissance de marqueurs de blocs que le parseur.
fn ouvre_un_bloc(texte: &str) -> bool {
    let debut = texte.trim_start();

    if debut.starts_with('>') || debut.starts_with("- ") || debut.starts_with("```") {
        return true;
    }

    let dièses = debut
        .chars()
        .take_while(|caractere| *caractere == '#')
        .count();

    if (1..=6).contains(&dièses) && debut[dièses..].starts_with(' ') {
        return true;
    }

    let chiffres = debut
        .chars()
        .take_while(|caractere| caractere.is_ascii_digit())
        .count();

    chiffres > 0 && debut[chiffres..].starts_with(". ")
}

/// Écrit une suite d'éléments inline.
fn ecrire_inline(inline: &[Inline], out: &mut String) {
    for element in inline {
        match element {
            Inline::Texte(texte) => echapper_texte(texte, out),
            Inline::Gras(contenu) => {
                out.push_str("**");
                ecrire_inline(contenu, out);
                out.push_str("**");
            }
            Inline::Italique(contenu) => {
                out.push('*');
                ecrire_inline(contenu, out);
                out.push('*');
            }
            Inline::GrasItalique(contenu) => {
                out.push_str("***");
                ecrire_inline(contenu, out);
                out.push_str("***");
            }
            Inline::Code(texte) => {
                out.push('`');
                out.push_str(texte);
                out.push('`');
            }
            Inline::Couleur { couleur, contenu } => {
                out.push('{');
                out.push_str(couleur);
                out.push(':');
                ecrire_inline(contenu, out);
                out.push('}');
            }
            Inline::Citation(references) => {
                for (index, reference) in references.iter().enumerate() {
                    if index > 0 {
                        out.push_str("; ");
                    }

                    out.push('@');
                    out.push_str(&reference.cle);

                    if let Some(page) = &reference.page {
                        out.push('[');
                        echapper_page(page, out);
                        out.push(']');
                    }
                }
            }
            Inline::Note(contenu) => {
                out.push_str("^[");
                ecrire_inline(contenu, out);
                out.push(']');
            }
            Inline::Lien { texte, cible } => {
                out.push('[');
                ecrire_inline(texte, out);
                out.push_str("](");
                out.push_str(cible);
                out.push(')');
            }
        }
    }
}

/// Écrit du texte nu, en échappant les caractères qui ouvrent un marqueur.
fn echapper_texte(texte: &str, out: &mut String) {
    for caractere in texte.chars() {
        if MARQUEURS.contains(&caractere) {
            out.push('\\');
        }

        out.push(caractere);
    }
}

/// Écrit une mention de page, en échappant ce qui romprait ses crochets.
fn echapper_page(page: &str, out: &mut String) {
    for caractere in page.chars() {
        if matches!(caractere, '\\' | ']') {
            out.push('\\');
        }

        out.push(caractere);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::langage::parse::parse;

    /// Le round-trip : parse(serialiser(doc)) == doc.
    ///
    /// C'est l'invariant du langage intermédiaire : le même texte donne toujours
    /// le même Document, et le même Document toujours le même texte.
    #[test]
    fn le_round_trip_conserve_le_document() {
        let texte = "# Titre\n\nUn paragraphe avec du **gras**, de l'*italique* et du `code`.\n\n> Une citation.\n\n- une puce\n- une autre\n\n1. un élément\n2. un autre\n\n```typst\n#figure(...)\n```\n";

        let document = parse(texte);
        let regenere = serialiser(&document);
        let relu = parse(&regenere);

        assert_eq!(relu, document, "le round-trip doit conserver le document");
        assert_eq!(
            regenere, texte,
            "le texte doit être reproduit à l'identique"
        );
    }

    /// Le round-trip conserve aussi les éléments inline riches.
    #[test]
    fn le_round_trip_conserve_les_inline_riches() {
        let texte = "Voir @dupont2023 et [ce lien](https://exemple.fr) et {rouge:du texte coloré} et ^[une précision].\n";

        let document = parse(texte);

        assert_eq!(parse(&serialiser(&document)), document);
    }

    /// Un appel groupé se réécrit tel quel — et reste **un** appel à la
    /// relecture. C'est la forme que le composeur de notes écrit.
    #[test]
    fn un_appel_groupe_se_recrit_tel_quel() {
        let texte = "Voir @dupont2023[p. 45]; @martin2020 et @bernard2021[pp. 12-15].\n";

        let document = parse(texte);
        let regenere = serialiser(&document);

        assert_eq!(regenere, texte);
        assert_eq!(parse(&regenere), document);
    }

    /// Une mention de page qui contient un crochet fermant revient entière au
    /// round-trip : le crochet s'échappe d'une barre oblique.
    #[test]
    fn une_page_qui_contient_un_crochet_revient_entiere() {
        let document = parse("Voir @dupont2023[p. 45, note 2\\]].\n");
        let regenere = serialiser(&document);

        assert_eq!(regenere, "Voir @dupont2023[p. 45, note 2\\]].\n");
        assert_eq!(parse(&regenere), document);
    }

    /// Un texte qui ressemble à du balisage reste du texte.
    ///
    /// C'est le cas qui casse un sérialiseur naïf : une astérisque dans une
    /// phrase, une arobase dans une adresse, une accolade dans une formule.
    #[test]
    fn un_texte_qui_ressemble_a_un_marqueur_est_echappe() {
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Texte(
                "Une *étoile*, un @grep et {accolades}.".to_string(),
            )])],
        };

        let texte = serialiser(&document);

        assert_eq!(texte, "Une \\*étoile\\*, un \\@grep et \\{accolades}.\n");
        assert_eq!(parse(&texte), document, "et le parseur le relit tel quel");
    }

    /// Un paragraphe qui commence par un marqueur de bloc reste un paragraphe.
    #[test]
    fn un_paragraphe_qui_commence_par_un_marqueur_reste_un_paragraphe() {
        let document = Document {
            blocs: vec![Bloc::Paragraphe(vec![Inline::Texte(
                "- pas une puce".to_string(),
            )])],
        };

        assert_eq!(parse(&serialiser(&document)), document);
    }

    /// Les blocs sont séparés par une ligne vide, sinon le parseur les
    /// recollerait en un seul paragraphe.
    #[test]
    fn les_blocs_sont_separes_par_une_ligne_vide() {
        let document = Document {
            blocs: vec![
                Bloc::Paragraphe(vec![Inline::Texte("Un".to_string())]),
                Bloc::Paragraphe(vec![Inline::Texte("Deux".to_string())]),
            ],
        };

        assert_eq!(serialiser(&document), "Un\n\nDeux\n");
    }

    /// Un bloc Typst qui contient un fence ne doit pas être tronqué au round-trip.
    #[test]
    fn un_bloc_brut_choisit_un_delimiteur_plus_long_que_son_contenu() {
        let document = Document {
            blocs: vec![Bloc::Brut("commande\n```\nreste".to_string())],
        };

        let source = serialiser(&document);

        assert!(source.starts_with("````typst\n"));
        assert_eq!(parse(&source), document);
    }
}
