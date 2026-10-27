//! Migration prudente des chapitres Typst vers le Document structuré.
//!
//! Seul le sous-ensemble dont la traduction est sûre est converti en blocs
//! riches. Toute commande ou syntaxe Typst non reconnue devient un unique bloc
//! brut : le document reste compilable sans perte, et l'utilisateur pourra le
//! convertir progressivement.

use super::parse;
use super::schema::{Bloc, Document};

/// Le rapport de migration : ce qui s'est converti, ce qui reste brut, et
/// pourquoi. La migration ne se fait pas en silence.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rapport {
    /// Les blocs convertis en syntaxe riche.
    pub elements_convertis: usize,

    /// Les blocs laissés en Typst brut.
    pub blocs_bruts: usize,

    /// Ce que la migration n'a pas traduit, et pourquoi.
    pub avertissements: Vec<String>,
}

/// Traduit un chapitre Typst simple en AST ; conserve le reste tel quel.
pub fn importer_typst(source: &str) -> Document {
    importer_typst_avec_rapport(source).0
}

/// La même migration, avec son rapport.
pub fn importer_typst_avec_rapport(source: &str) -> (Document, Rapport) {
    let mut avertissements = Vec::new();
    let document = importer(source, &mut avertissements);

    let blocs_bruts = document
        .blocs
        .iter()
        .filter(|bloc| matches!(bloc, Bloc::Brut(_)))
        .count();

    let rapport = Rapport {
        elements_convertis: document.blocs.len() - blocs_bruts,
        blocs_bruts,
        avertissements,
    };

    (document, rapport)
}

/// Traduit, en déclarant au passage ce qu'il n'a pas su traduire.
fn importer(source: &str, avertissements: &mut Vec<String>) -> Document {
    if source.trim().is_empty() {
        return Document::vide();
    }

    let lignes: Vec<&str> = source.lines().collect();
    let mut texte_tw = String::new();
    let mut index = 0;

    while index < lignes.len() {
        if lignes[index].trim().is_empty() {
            index += 1;
            continue;
        }

        if lignes[index].trim_start().starts_with("//") {
            avertissements.push("une ligne de commentaire Typst".to_string());
            return brut(source);
        }

        if let Some((niveau, titre)) = titre_typst(lignes[index]) {
            let Some(titre) = convertir_inline(titre) else {
                avertissements.push("un titre portant du balisage Typst".to_string());
                return brut(source);
            };

            separer_bloc(&mut texte_tw);
            texte_tw.push_str(&"#".repeat(niveau));
            texte_tw.push(' ');
            texte_tw.push_str(&titre);
            index += 1;
            continue;
        }

        if lignes[index].starts_with("- ") || lignes[index].starts_with("+ ") {
            let puces = lignes[index].starts_with("- ");
            separer_bloc(&mut texte_tw);
            let mut numero = 1;

            while index < lignes.len()
                && ((puces && lignes[index].starts_with("- "))
                    || (!puces && lignes[index].starts_with("+ ")))
            {
                let contenu = &lignes[index][2..];
                let Some(contenu) = convertir_inline(contenu) else {
                    avertissements.push("une liste portant du balisage Typst".to_string());
                    return brut(source);
                };

                if puces {
                    texte_tw.push_str("- ");
                } else {
                    texte_tw.push_str(&format!("{numero}. "));
                    numero += 1;
                }

                texte_tw.push_str(&contenu);
                texte_tw.push('\n');
                index += 1;
            }

            continue;
        }

        // Paragraphes : les lignes Typst consécutives deviennent un seul bloc.
        let debut = index;
        let mut lignes_paragraphe = Vec::new();

        while index < lignes.len()
            && !lignes[index].trim().is_empty()
            && titre_typst(lignes[index]).is_none()
            && !lignes[index].starts_with("- ")
            && !lignes[index].starts_with("+ ")
        {
            let Some(ligne) = convertir_inline(lignes[index].trim()) else {
                avertissements.push("une ligne portant du balisage Typst".to_string());
                return brut(source);
            };

            lignes_paragraphe.push(ligne);
            index += 1;
        }

        if lignes_paragraphe.is_empty() {
            // Une ligne qui ressemble à un titre mais n'a pas la syntaxe
            // reconnue ne doit ni boucler ni être altérée.
            if index == debut {
                return brut(source);
            }

            continue;
        }

        separer_bloc(&mut texte_tw);
        texte_tw.push_str(&lignes_paragraphe.join(" "));
    }

    parse(&texte_tw)
}

/// Le chapitre entier en bloc brut. Les fins de ligne s'y rognent : elles
/// n'ont pas de sens en Typst, et les garderait grossir d'un cycle à l'autre
/// — la migration doit être idempotente.
fn brut(source: &str) -> Document {
    Document {
        blocs: vec![Bloc::Brut(source.trim_end().to_string())],
    }
}

fn separer_bloc(texte: &mut String) {
    if !texte.is_empty() && !texte.ends_with("\n\n") {
        if !texte.ends_with('\n') {
            texte.push('\n');
        }
        texte.push('\n');
    }
}

fn titre_typst(ligne: &str) -> Option<(usize, &str)> {
    let niveau = ligne
        .chars()
        .take_while(|caractere| *caractere == '=')
        .count();

    if !(1..=6).contains(&niveau) || !ligne[niveau..].starts_with(' ') {
        return None;
    }

    Some((niveau, ligne[niveau..].trim()))
}

/// Convertit les emphases Typst simples et échappe les marqueurs littéraux.
///
/// L'imbrication complexe, les commandes, les liens et les expressions sont
/// volontairement refusés ici : l'appelant conservera alors le chapitre entier
/// dans un bloc brut plutôt que d'en produire une approximation silencieuse.
fn convertir_inline(texte: &str) -> Option<String> {
    let mut resultat = String::new();
    let mut reste = texte;

    while !reste.is_empty() {
        let caractere = reste.chars().next()?;

        // Ces marqueurs ont une structure Typst dont cette migration ne prétend
        // pas connaître le sens.
        if matches!(
            caractere,
            '#' | '[' | ']' | '{' | '}' | '<' | '>' | '\\' | '$' | '^'
        ) {
            return None;
        }

        if caractere == '*' || caractere == '_' {
            let suite = &reste[caractere.len_utf8()..];
            let fermeture = suite.find(caractere);

            if let Some(fin) = fermeture {
                let contenu = &suite[..fin];
                let commence_blanc = contenu.chars().next().is_some_and(char::is_whitespace);
                let finit_blanc = contenu.chars().next_back().is_some_and(char::is_whitespace);
                let contient_emphase = contenu.contains(['*', '_']);

                if !contenu.is_empty() && !commence_blanc && !finit_blanc && !contient_emphase {
                    if caractere == '*' {
                        resultat.push_str("**");
                        resultat.push_str(contenu);
                        resultat.push_str("**");
                    } else {
                        resultat.push('*');
                        resultat.push_str(contenu);
                        resultat.push('*');
                    }

                    reste = &suite[fin + caractere.len_utf8()..];
                    continue;
                }
            }

            // Une étoile de multiplication ou un soulignement littéral reste
            // du texte, pas une emphase ouverte.
            resultat.push('\\');
            resultat.push(caractere);
            reste = &reste[caractere.len_utf8()..];
            continue;
        }

        if caractere == '`' {
            let suite = &reste[1..];
            let fin = suite.find('`')?;
            let code = &suite[..fin];

            if code.is_empty() || code.contains('`') {
                return None;
            }

            resultat.push('`');
            resultat.push_str(code);
            resultat.push('`');
            reste = &suite[fin + 1..];
            continue;
        }

        if caractere == '@' {
            let suite = &reste[1..];
            let longueur = suite
                .char_indices()
                .find(|(_, caractere)| {
                    !(caractere.is_alphanumeric() || *caractere == '-' || *caractere == '_')
                })
                .map(|(index, _)| index)
                .unwrap_or(suite.len());

            if longueur == 0 {
                resultat.push_str("\\@");
                reste = suite;
            } else {
                resultat.push('@');
                resultat.push_str(&suite[..longueur]);
                reste = &suite[longueur..];
            }

            continue;
        }

        resultat.push(caractere);
        reste = &reste[caractere.len_utf8()..];
    }

    Some(resultat)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le rapport dit ce qui s'est passé : converti ou brut, et pourquoi.
    #[test]
    fn le_rapport_dit_ce_qui_s_est_passe() {
        let (document, rapport) = importer_typst_avec_rapport("= Titre\n\nUn paragraphe simple.\n");
        assert_eq!(rapport.blocs_bruts, 0, "{rapport:?}");
        assert_eq!(rapport.elements_convertis, 2, "{rapport:?}");
        assert!(rapport.avertissements.is_empty(), "{rapport:?}");
        assert!(document.est_canonique(), "la migration produit du canonique");

        let (document, rapport) = importer_typst_avec_rapport("#figure(image(\"a.png\"))\n");
        assert_eq!(rapport.blocs_bruts, 1, "{rapport:?}");
        assert_eq!(rapport.elements_convertis, 0, "{rapport:?}");
        assert!(
            rapport.avertissements.iter().any(|motif| motif.contains("Typst")),
            "le motif est dit : {rapport:?}"
        );
        assert_eq!(document.blocs.len(), 1);
    }

    /// La migration est idempotente : migrer, régénérer, remigrer revient au
    /// même document. C'est le sens de « non destructive » — la source
    /// Typst survit dans le brut, et le converti se reconvert tel quel.
    #[test]
    fn la_migration_est_idempotente() {
        // Une source Typst convertible (le gras Typst est `*texte*`).
        let source = "= Titre\n\nUn *gras* et un paragraphe.\n\n- puce\n";

        let une_fois = importer_typst(source);
        let regenere = super::super::generer::generer(&une_fois);
        let deux_fois = importer_typst(&regenere);

        assert_eq!(deux_fois, une_fois, "migrer du migré reformate");

        // Et une source non convertible : le brut revient au brut.
        let source = "#figure(image(\"a.png\"))\n";

        let une_fois = importer_typst(source);
        let regenere = super::super::generer::generer(&une_fois);
        let deux_fois = importer_typst(&regenere);

        assert_eq!(deux_fois, une_fois, "le brut revient au brut");
    }

    /// Les dépendances survivent : images, inclusions, macros et renvois
    /// restent dans le corps généré, à leur place.
    #[test]
    fn la_migration_preserve_les_dependances() {
        let source = "#figure(image(\"images/graphe.png\", width: 80%), caption: [Le graphe])\n\n#let rappel = 2\n\nVoir @analyse.\n";

        let document = importer_typst(source);
        let typst = super::super::generer::generer(&document);

        assert!(typst.contains("images/graphe.png"), "{typst}");
        assert!(typst.contains("#let rappel = 2"), "{typst}");
        assert!(typst.contains("@analyse"), "{typst}");
    }

    /// La fidélité du rendu : ce qui compile avant compile après, sur autant
    /// de pages, et le texte se retrouve.
    #[test]
    fn le_rendu_est_fidele_avant_et_apres() {
        use crate::typst::render::{CompileOutcome, compile};

        let source = "= Le titre\n\nUn paragraphe avec du *gras* et des mots reconnaissables.\n\n- une puce ici\n";

        let avant = compile(std::path::Path::new("/projet"), source, &[]);
        let typst = super::super::generer::generer(&importer_typst(source));
        let apres = compile(std::path::Path::new("/projet"), &typst, &[]);

        let pages = |outcome: &CompileOutcome| {
            outcome
                .document
                .as_ref()
                .map(|document| crate::typst::render::page_count(document))
        };

        assert!(avant.errors.is_empty(), "{:?}", avant.errors);
        assert!(apres.errors.is_empty(), "{:?}", apres.errors);
        assert_eq!(pages(&apres), pages(&avant), "le nombre de pages tient");
        assert!(typst.contains("des mots reconnaissables"), "{typst}");
    }

    use crate::langage::schema::{Bloc, Document, Inline};

    #[test]
    fn importe_les_titres_paragraphes_gras_italiques_et_listes_simples() {
        let document = super::importer_typst(
            "= Introduction\n\nUne phrase avec *gras* et _italique_.\n\n- une puce\n+ un élément numéroté\n",
        );

        assert_eq!(
            document,
            Document {
                blocs: vec![
                    Bloc::Titre {
                        niveau: 1,
                        contenu: vec![Inline::Texte("Introduction".to_string())],
                    },
                    Bloc::Paragraphe(vec![
                        Inline::Texte("Une phrase avec ".to_string()),
                        Inline::Gras(vec![Inline::Texte("gras".to_string())]),
                        Inline::Texte(" et ".to_string()),
                        Inline::Italique(vec![Inline::Texte("italique".to_string())]),
                        Inline::Texte(".".to_string()),
                    ]),
                    Bloc::ListePuce(vec![vec![Inline::Texte("une puce".to_string())]]),
                    Bloc::ListeNumero(vec![vec![
                        Inline::Texte("un élément numéroté".to_string(),)
                    ]]),
                ],
            }
        );
    }

    #[test]
    fn conserve_en_brut_un_chapitre_avec_des_commandes_typst() {
        let source = "// commentaire conservé\n#set page(margin: 2cm)\n= Titre";

        assert_eq!(
            super::importer_typst(source),
            Document {
                blocs: vec![Bloc::Brut(source.to_string())],
            }
        );
    }
}
