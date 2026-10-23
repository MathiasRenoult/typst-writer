//! Migration prudente des chapitres Typst vers le Document structuré.
//!
//! Seul le sous-ensemble dont la traduction est sûre est converti en blocs
//! riches. Toute commande ou syntaxe Typst non reconnue devient un unique bloc
//! brut : le document reste compilable sans perte, et l'utilisateur pourra le
//! convertir progressivement.

use super::parse;
use super::schema::{Bloc, Document};

/// Traduit un chapitre Typst simple en AST ; conserve le reste tel quel.
pub fn importer_typst(source: &str) -> Document {
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
            return brut(source);
        }

        if let Some((niveau, titre)) = titre_typst(lignes[index]) {
            let Some(titre) = convertir_inline(titre) else {
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

fn brut(source: &str) -> Document {
    Document {
        blocs: vec![Bloc::Brut(source.to_string())],
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
