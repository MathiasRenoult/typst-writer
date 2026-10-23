//! Le parseur : texte (notre syntaxe) → Document structuré.
//!
//! La syntaxe est markdown-like, taillée pour la rédaction :
//!
//! ```text
//! # Titre de niveau 1
//! ## Titre de niveau 2
//!
//! Un paragraphe avec du **gras**, de l'*italique*, du `code`,
//! une citation @dupont2023, un [lien](https://exemple.fr) et
//! du texte {rouge:...}.
//!
//! > Une citation de bloc.
//!
//! - une puce
//! - une autre
//!
//! 1. un élément numéroté
//! 2. un autre
//!
//! ```typst
//! #figure(...)
//! ```
//! ```
//!
//! Le parseur est **déterministe** : le même texte donne toujours le même
//! Document. C'est la moitié du round-trip — l'autre est le sérialiseur, qui
//! doit produire exactement le texte que ce parseur lit.
//!
//! Trois règles rendent ce round-trip possible :
//!
//! - **L'échappement.** Une barre oblique rend littéral le caractère qui la
//!   suit : `\*` est une étoile, pas un italique. Le sérialiseur s'en sert pour
//!   tout texte qui ressemble à du balisage, et le parseur la retire.
//! - **Les délimiteurs d'emphase collent au texte.** `l'**Église**` est du gras
//!   — l'apostrophe n'a pas à être une espace. Seule une espace *intérieure*
//!   empêche l'emphase (`2 * 3` reste une multiplication).
//! - **Une ligne qui commence par une barre oblique est du texte**, jamais un
//!   bloc : c'est ainsi qu'un paragraphe peut commencer par `-` ou `#`.

use super::schema::{Bloc, Document, Inline};

/// Analyse un texte en Document structuré.
///
/// Ne renvoie jamais d'erreur : un texte malformé donne un Document qui
/// contient ce qui a pu être lu, et le reste en texte brut. C'est la même
/// tolérance que le reste de l'application — un fichier abîmé ne doit pas
/// empêcher d'ouvrir le projet.
pub fn parse(texte: &str) -> Document {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut blocs = Vec::new();
    let mut index = 0;

    while index < lignes.len() {
        let ligne = lignes[index];

        // Les lignes vides séparent les blocs, sans être elles-mêmes un bloc.
        // Ne pas avancer ici laisserait `lire_paragraphe` retourner le même
        // index, et la boucle accumulerait des paragraphes vides sans fin.
        if ligne.trim().is_empty() {
            index += 1;
            continue;
        }

        // Une ligne échappée est du texte, quel que soit ce qu'elle annonce.
        let echappee = ligne.trim_start().starts_with('\\');

        // Un bloc brut Typst : tout jusqu'au délimiteur fermant.
        if !echappee && ligne.trim_start().starts_with("```") {
            let (brut, suivant) = lire_brut(&lignes, index);
            blocs.push(Bloc::Brut(brut));
            index = suivant;
            continue;
        }

        // Un titre.
        if !echappee {
            if let Some(niveau) = niveau_titre(ligne) {
                let contenu = parse_inline(ligne.trim_start()[niveau as usize..].trim());
                blocs.push(Bloc::Titre { niveau, contenu });
                index += 1;
                continue;
            }
        }

        // Une citation de bloc.
        if !echappee && ligne.trim_start().starts_with('>') {
            let contenu = parse_inline(ligne.trim_start()[1..].trim());
            blocs.push(Bloc::Citation(contenu));
            index += 1;
            continue;
        }

        // Une liste à puces.
        if !echappee && ligne.trim_start().starts_with("- ") {
            let (items, suivant) = lire_liste(&lignes, index, "- ");
            blocs.push(Bloc::ListePuce(items));
            index = suivant;
            continue;
        }

        // Une liste numérotée.
        if !echappee && est_numero(ligne) {
            let (items, suivant) = lire_liste(&lignes, index, "");
            blocs.push(Bloc::ListeNumero(items));
            index = suivant;
            continue;
        }

        // Un paragraphe : tout ce qui n'est pas un autre bloc, jusqu'à une
        // ligne vide ou un nouveau bloc.
        let (paragraphe, suivant) = lire_paragraphe(&lignes, index);
        blocs.push(Bloc::Paragraphe(parse_inline(&paragraphe)));
        index = suivant;
    }

    Document { blocs }
}

/// Position source d'un titre du document intermédiaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceHeading {
    /// Niveau du titre, de 1 à 6.
    pub level: u8,
    /// Texte visible, sans balisage inline.
    pub text: String,
    /// Numéro de ligne à partir de un.
    pub line: usize,
}

/// Associe les titres de l'AST aux lignes où ils sont écrits.
///
/// Le balayage exclut les blocs Typst bruts et les titres échappés, car ils ne
/// sont pas des blocs `Titre` du document structuré.
pub fn titres_source(texte: &str, document: &Document) -> Vec<SourceHeading> {
    let mut positions = Vec::new();
    let mut fence_ouverte = None;

    for (index, ligne) in texte.lines().enumerate() {
        if let Some(longueur) = fence_ouverte {
            if ferme_fence(ligne, longueur) {
                fence_ouverte = None;
            }
            continue;
        }

        let longueur = longueur_fence(ligne);
        if longueur >= 3 {
            fence_ouverte = Some(longueur);
            continue;
        }

        if ligne.trim_start().starts_with('\\') {
            continue;
        }

        if let Some(level) = niveau_titre(ligne) {
            positions.push((level, index + 1));
        }
    }

    document
        .blocs
        .iter()
        .filter_map(|bloc| match bloc {
            Bloc::Titre { niveau, contenu } => Some((*niveau, contenu)),
            _ => None,
        })
        .zip(positions)
        .filter_map(|((level, contenu), (source_level, line))| {
            (level == source_level).then(|| SourceHeading {
                level,
                text: super::schema::inline_texte_nu(contenu),
                line,
            })
        })
        .collect()
}

/// La ligne commence-t-elle un bloc, dans le corps d'un paragraphe ?
fn ouvre_un_bloc(ligne: &str) -> bool {
    let debut = ligne.trim_start();

    // Une ligne échappée est du texte : elle n'interrompt pas le paragraphe.
    !debut.starts_with('\\')
        && (niveau_titre(ligne).is_some()
            || debut.starts_with('>')
            || debut.starts_with("- ")
            || est_numero(ligne)
            || debut.starts_with("```"))
}

/// Le niveau d'un titre, si la ligne en est un.
///
/// `#` à `######` : un à six dièses, suivis d'une espace. Un `#` sans espace
/// n'est pas un titre — c'est du texte.
fn niveau_titre(ligne: &str) -> Option<u8> {
    let debut = ligne.trim_start();
    let dièses = debut.chars().take_while(|c| *c == '#').count();

    if dièses == 0 || dièses > 6 {
        return None;
    }

    // Le dièse doit être suivi d'une espace, sinon ce n'est pas un titre.
    debut[dièses..].starts_with(' ').then_some(dièses as u8)
}

/// Une ligne est-elle le début d'une liste numérotée ?
fn est_numero(ligne: &str) -> bool {
    let debut = ligne.trim_start();
    let chiffres = debut.chars().take_while(|c| c.is_ascii_digit()).count();

    chiffres > 0 && debut[chiffres..].starts_with(". ")
}

/// Nombre de backticks consécutifs au début de la ligne, après indentation.
fn longueur_fence(ligne: &str) -> usize {
    ligne
        .trim_start()
        .chars()
        .take_while(|caractere| *caractere == '`')
        .count()
}

/// Une clôture doit avoir au moins la longueur d'ouverture, sans suffixe.
fn ferme_fence(ligne: &str, longueur_ouverture: usize) -> bool {
    let debut = ligne.trim_start();
    let longueur = longueur_fence(ligne);

    longueur >= longueur_ouverture && debut[longueur..].trim().is_empty()
}

/// Lit un bloc brut Typst, jusqu'au délimiteur fermant de longueur compatible.
///
/// Les fences internes plus courtes restent du contenu ; le sérialiseur choisit
/// une ouverture plus longue que toute séquence présente dans le texte brut.
/// Renvoie le contenu brut et l'index de la ligne suivante.
fn lire_brut(lignes: &[&str], index: usize) -> (String, usize) {
    let longueur_ouverture = longueur_fence(lignes[index]).max(3);
    let mut contenu = Vec::new();
    let mut suivant = index + 1;

    while suivant < lignes.len() {
        if ferme_fence(lignes[suivant], longueur_ouverture) {
            return (contenu.join("\n"), suivant + 1);
        }

        contenu.push(lignes[suivant]);
        suivant += 1;
    }

    (contenu.join("\n"), suivant)
}

/// Lit une liste (à puces ou numérotée), jusqu'à une ligne qui n'en est plus.
///
/// Renvoie les éléments et l'index de la ligne suivante.
fn lire_liste(lignes: &[&str], index: usize, prefixe: &str) -> (Vec<Vec<Inline>>, usize) {
    let mut items = Vec::new();
    let mut suivant = index;

    while suivant < lignes.len() {
        let ligne = lignes[suivant];

        let contenu = if ligne.trim_start().starts_with('\\') {
            None
        } else if prefixe.is_empty() {
            // Liste numérotée : `1. texte`.
            if est_numero(ligne) {
                let debut = ligne.trim_start();
                let chiffres = debut.chars().take_while(|c| c.is_ascii_digit()).count();
                Some(&debut[chiffres + 2..])
            } else {
                None
            }
        } else if ligne.trim_start().starts_with(prefixe) {
            Some(&ligne.trim_start()[prefixe.len()..])
        } else {
            None
        };

        match contenu {
            Some(texte) => {
                items.push(parse_inline(texte.trim()));
                suivant += 1;
            }
            None => break,
        }
    }

    (items, suivant)
}

/// Lit un paragraphe : des lignes consécutives, jusqu'à une ligne vide ou un
/// nouveau bloc.
///
/// Renvoie le texte du paragraphe (lignes jointes par une espace) et l'index de
/// la ligne suivante.
fn lire_paragraphe(lignes: &[&str], index: usize) -> (String, usize) {
    let mut lignes_du_paragraphe = Vec::new();
    let mut suivant = index;

    while suivant < lignes.len() {
        let ligne = lignes[suivant];

        // Une ligne vide termine le paragraphe.
        if ligne.trim().is_empty() {
            break;
        }

        // Un nouveau bloc termine aussi le paragraphe.
        if ouvre_un_bloc(ligne) {
            break;
        }

        lignes_du_paragraphe.push(ligne.trim());
        suivant += 1;
    }

    (lignes_du_paragraphe.join(" "), suivant)
}

/// Analyse une suite d'éléments inline.
///
/// Le balisage est reconnu : `**gras**`, `*italique*` ou `_italique_`,
/// `***gras+italique***`, `` `code` ``, `@clé`, `[texte](cible)`,
/// `{couleur:texte}`. La barre oblique échappe le caractère qui la suit.
fn parse_inline(texte: &str) -> Vec<Inline> {
    let mut elements = Vec::new();
    let mut tampon = String::new();
    let mut reste = texte;

    // L'échappement et le texte nu s'accumulent dans `tampon` : un paragraphe
    // sans balisage ne doit pas produire un élément par caractère.
    macro_rules! vider {
        () => {
            if !tampon.is_empty() {
                elements.push(Inline::Texte(std::mem::take(&mut tampon)));
            }
        };
    }

    while let Some(caractere) = reste.chars().next() {
        // Une barre oblique rend littéral le caractère suivant.
        if caractere == '\\' {
            let mut suite = reste[1..].chars();

            match suite.next() {
                Some(litteral) => {
                    tampon.push(litteral);
                    reste = suite.as_str();
                }
                None => {
                    tampon.push('\\');
                    reste = "";
                }
            }

            continue;
        }

        // Gras + italique : `***texte***`.
        if let Some((contenu, apres)) = emphasis(reste, "***") {
            vider!();
            elements.push(Inline::GrasItalique(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Gras : `**texte**`.
        if let Some((contenu, apres)) = emphasis(reste, "**") {
            vider!();
            elements.push(Inline::Gras(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Italique : `*texte*` ou `_texte_` (la seconde forme sert à
        // l'imbrication : `**gras _et italique_**`).
        if let Some((contenu, apres)) = emphasis(reste, "*").or_else(|| emphasis(reste, "_")) {
            vider!();
            elements.push(Inline::Italique(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Code : `` `texte` ``.
        if let Some((contenu, apres)) = delimit(reste, "`") {
            vider!();
            elements.push(Inline::Code(contenu.to_string()));
            reste = apres;
            continue;
        }

        // Note : `^[texte]`.
        if let Some((contenu, apres)) = note(reste) {
            vider!();
            elements.push(Inline::Note(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Citation : `@clé`.
        if let Some((cle, apres)) = citation(reste) {
            vider!();
            elements.push(Inline::Citation(cle.to_string()));
            reste = apres;
            continue;
        }

        // Lien : `[texte](cible)`.
        if let Some((texte_lien, cible, apres)) = lien(reste) {
            vider!();
            elements.push(Inline::Lien {
                texte: parse_inline(texte_lien),
                cible: cible.to_string(),
            });
            reste = apres;
            continue;
        }

        // Couleur : `{couleur:texte}`.
        if let Some((couleur, contenu, apres)) = couleur(reste) {
            vider!();
            elements.push(Inline::Couleur {
                couleur: couleur.to_string(),
                contenu: parse_inline(contenu),
            });
            reste = apres;
            continue;
        }

        tampon.push(caractere);
        reste = &reste[caractere.len_utf8()..];
    }

    vider!();

    elements
}

/// Lit une emphase ouverte au début de `texte`.
///
/// Le délimiteur ouvrant doit précéder un caractère non blanc — `**gras**`
/// colle au texte, comme dans `l'**Église**`. Le délimiteur fermant doit
/// suivre un caractère non blanc : `2 * 3` reste une multiplication.
fn emphasis<'a>(texte: &'a str, delim: &str) -> Option<(&'a str, &'a str)> {
    let reste = texte.strip_prefix(delim)?;

    // Un `*` seul ne doit pas être pris pour l'ouverture de `**`.
    if reste.starts_with('*') || reste.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }

    let mut offset = 0;

    while let Some(trouve) = reste[offset..].find(delim) {
        let index = offset + trouve;
        let avant = reste[..index].chars().next_back();

        if index > 0 && avant.is_some_and(|c| !c.is_whitespace() && c != '\\') {
            return Some((&reste[..index], &reste[index + delim.len()..]));
        }

        offset = index + delim.len();
    }

    None
}

/// Trouve un texte entre deux délimiteurs identiques, sans contrainte de blanc.
///
/// Sert au code inline et au contenu des liens, où l'espacement n'a pas de
/// valeur de frontière.
fn delimit<'a>(texte: &'a str, delim: &str) -> Option<(&'a str, &'a str)> {
    let reste = texte.strip_prefix(delim)?;
    let ferme = reste.find(delim)?;

    Some((&reste[..ferme], &reste[ferme + delim.len()..]))
}

/// Une note : `^[texte]`.
fn note(texte: &str) -> Option<(&str, &str)> {
    let apres_ouverture = texte.strip_prefix("^[")?;
    let fin = apres_ouverture.find(']')?;

    Some((&apres_ouverture[..fin], &apres_ouverture[fin + 1..]))
}

/// Une citation bibliographique : `@clé`.
///
/// La clé est une suite de caractères alphanumériques, tirets et underscores.
fn citation(texte: &str) -> Option<(&str, &str)> {
    let apres = texte.strip_prefix('@')?;

    let fin = apres
        .char_indices()
        .find(|(_, c)| !(c.is_alphanumeric() || *c == '-' || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(apres.len());

    if fin == 0 {
        return None;
    }

    Some((&apres[..fin], &apres[fin..]))
}

/// Un lien : `[texte](cible)`.
fn lien(texte: &str) -> Option<(&str, &str, &str)> {
    let apres_ouvert = texte.strip_prefix('[')?;

    let ferme = apres_ouvert.find(']')?;
    let texte_lien = &apres_ouvert[..ferme];

    let apres_ferme = &apres_ouvert[ferme + 1..];
    let apres_parenthèse = apres_ferme.strip_prefix('(')?;
    let fin = apres_parenthèse.find(')')?;
    let cible = &apres_parenthèse[..fin];

    Some((texte_lien, cible, &apres_parenthèse[fin + 1..]))
}

/// Une couleur : `{couleur:texte}`.
fn couleur(texte: &str) -> Option<(&str, &str, &str)> {
    let apres_ouvert = texte.strip_prefix('{')?;

    let deux_points = apres_ouvert.find(':')?;
    let couleur = &apres_ouvert[..deux_points];

    let apres_deux_points = &apres_ouvert[deux_points + 1..];
    let ferme = apres_deux_points.find('}')?;
    let contenu = &apres_deux_points[..ferme];

    Some((couleur, contenu, &apres_deux_points[ferme + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un document vide donne un Document vide.
    #[test]
    fn un_texte_vide_donne_un_document_vide() {
        assert_eq!(parse(""), Document::vide());
    }

    /// Les lignes vides séparent les blocs, mais ne créent pas de blocs vides
    /// et ne bloquent pas le parseur.
    #[test]
    fn les_lignes_vides_separent_les_blocs_sans_boucle_infinie() {
        let document = parse("\n# Titre\n\nTexte\n\n\n- puce\n\n");

        assert_eq!(
            document.blocs,
            vec![
                Bloc::Titre {
                    niveau: 1,
                    contenu: vec![Inline::Texte("Titre".to_string())],
                },
                Bloc::Paragraphe(vec![Inline::Texte("Texte".to_string())]),
                Bloc::ListePuce(vec![vec![Inline::Texte("puce".to_string())]]),
            ]
        );
    }

    /// Un titre se lit avec son niveau.
    #[test]
    fn un_titre_se_lit_avec_son_niveau() {
        let doc = parse("# Titre\n\n## Sous-titre\n");

        assert_eq!(
            doc.blocs,
            vec![
                Bloc::Titre {
                    niveau: 1,
                    contenu: vec![Inline::Texte("Titre".to_string())],
                },
                Bloc::Titre {
                    niveau: 2,
                    contenu: vec![Inline::Texte("Sous-titre".to_string())],
                },
            ]
        );
    }

    /// Un `#` sans espace n'est pas un titre.
    #[test]
    fn un_diese_sans_espace_n_est_pas_un_titre() {
        let doc = parse("#pas-un-titre\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Texte(
                "#pas-un-titre".to_string()
            )])]
        );
    }

    /// Un paragraphe se lit, et les lignes se joignent par une espace.
    #[test]
    fn un_paragraphe_se_lit() {
        let doc = parse("Première ligne\nseconde ligne\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Texte(
                "Première ligne seconde ligne".to_string()
            )])]
        );
    }

    /// Le gras, l'italique et le gras+italique se lisent.
    #[test]
    fn le_balisage_de_texte_se_lit() {
        let doc = parse("Du **gras**, de l'*italique* et du ***les deux***.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Du ".to_string()),
                Inline::Gras(vec![Inline::Texte("gras".to_string())]),
                Inline::Texte(", de l'".to_string()),
                Inline::Italique(vec![Inline::Texte("italique".to_string())]),
                Inline::Texte(" et du ".to_string()),
                Inline::GrasItalique(vec![Inline::Texte("les deux".to_string())]),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Un balisage collé à une apostrophe est bien du balisage.
    ///
    /// C'est le cas courant en français : `l'**Église**`. Exiger une espace
    /// avant l'ouverture laisserait les étoiles visibles dans le texte.
    #[test]
    fn un_balisage_colle_a_un_mot_se_lit() {
        let doc = parse("l'**Église** et *l'État*.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("l'".to_string()),
                Inline::Gras(vec![Inline::Texte("Église".to_string())]),
                Inline::Texte(" et ".to_string()),
                Inline::Italique(vec![Inline::Texte("l'État".to_string())]),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Une multiplication reste du texte : une étoile isolée par des espaces
    /// n'ouvre pas d'italique.
    #[test]
    fn une_etoile_isolee_reste_du_texte() {
        let doc = parse("2 * 3 font six.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Texte(
                "2 * 3 font six.".to_string()
            )])]
        );
    }

    /// L'italique bas s'écrit aussi `_`, pour l'imbrication.
    #[test]
    fn l_italique_bas_se_lit() {
        let doc = parse("**gras _et italique_**\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Gras(vec![
                Inline::Texte("gras ".to_string()),
                Inline::Italique(vec![Inline::Texte("et italique".to_string())]),
            ])])]
        );
    }

    /// La barre oblique rend littéral le caractère suivant.
    #[test]
    fn l_echappement_rend_le_caractere_litteral() {
        let doc = parse("Une \\*étoile\\* et du \\#texte.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Texte(
                "Une *étoile* et du #texte.".to_string()
            )])]
        );
    }

    /// Une ligne échappée n'ouvre pas de bloc : elle reste du paragraphe.
    #[test]
    fn une_ligne_echappee_reste_du_paragraphe() {
        let doc = parse("Un paragraphe\n\\# pas un titre\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![Inline::Texte(
                "Un paragraphe # pas un titre".to_string()
            )])]
        );
    }

    /// Le code inline se lit.
    #[test]
    fn le_code_inline_se_lit() {
        let doc = parse("Du `code` ici.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Du ".to_string()),
                Inline::Code("code".to_string()),
                Inline::Texte(" ici.".to_string()),
            ])]
        );
    }

    /// Une citation bibliographique se lit.
    #[test]
    fn une_citation_se_lit() {
        let doc = parse("Voir @dupont2023.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Voir ".to_string()),
                Inline::Citation("dupont2023".to_string()),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Un lien se lit avec son texte et sa cible.
    #[test]
    fn un_lien_se_lit() {
        let doc = parse("Un [lien](https://exemple.fr) ici.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Un ".to_string()),
                Inline::Lien {
                    texte: vec![Inline::Texte("lien".to_string())],
                    cible: "https://exemple.fr".to_string(),
                },
                Inline::Texte(" ici.".to_string()),
            ])]
        );
    }

    /// Une couleur se lit avec sa valeur et son contenu.
    #[test]
    fn une_couleur_se_lit() {
        let doc = parse("Du {rouge:texte coloré}.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Du ".to_string()),
                Inline::Couleur {
                    couleur: "rouge".to_string(),
                    contenu: vec![Inline::Texte("texte coloré".to_string())],
                },
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Une note inline se lit comme un élément distinct.
    #[test]
    fn une_note_inline_se_lit() {
        let doc = parse("Une note ^[texte précis] ici.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Une note ".to_string()),
                Inline::Note(vec![Inline::Texte("texte précis".to_string())]),
                Inline::Texte(" ici.".to_string()),
            ])]
        );
    }

    /// Une citation de bloc se lit.
    #[test]
    fn une_citation_de_bloc_se_lit() {
        let doc = parse("> Une citation.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Citation(vec![Inline::Texte(
                "Une citation.".to_string()
            )])]
        );
    }

    /// Une liste à puces se lit.
    #[test]
    fn une_liste_a_puces_se_lit() {
        let doc = parse("- un\n- deux\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::ListePuce(vec![
                vec![Inline::Texte("un".to_string())],
                vec![Inline::Texte("deux".to_string())],
            ])]
        );
    }

    /// Une liste numérotée se lit.
    #[test]
    fn une_liste_numerotee_se_lit() {
        let doc = parse("1. un\n2. deux\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::ListeNumero(vec![
                vec![Inline::Texte("un".to_string())],
                vec![Inline::Texte("deux".to_string())],
            ])]
        );
    }

    /// Un bloc brut Typst se lit, jusqu'au délimiteur fermant.
    #[test]
    fn un_bloc_brut_se_lit() {
        let doc = parse("```typst\n#figure(...)\n```\n");

        assert_eq!(doc.blocs, vec![Bloc::Brut("#figure(...)".to_string())]);
    }

    /// Un bloc brut non fermé se lit jusqu'à la fin.
    #[test]
    fn un_bloc_brut_non_ferme_se_lit_jusqu_a_la_fin() {
        let doc = parse("```typst\n#figure(...)\n");

        assert_eq!(doc.blocs, vec![Bloc::Brut("#figure(...)".to_string())]);
    }

    /// Les titres gardent leur ligne source pour la navigation de l'outline,
    /// sans prendre les faux titres d'un bloc Typst ou d'une ligne échappée.
    #[test]
    fn titres_source_ignorent_les_blocs_bruts_et_les_lignes_echappees() {
        let source = "# Introduction\n\n## Un **titre**\n\n````typst\n```\n# faux\n```\n````\n\\# échappé\n### Suite\n";
        let document = parse(source);

        let titres = titres_source(source, &document);

        assert_eq!(
            titres,
            vec![
                SourceHeading {
                    level: 1,
                    text: "Introduction".to_string(),
                    line: 1,
                },
                SourceHeading {
                    level: 2,
                    text: "Un titre".to_string(),
                    line: 3,
                },
                SourceHeading {
                    level: 3,
                    text: "Suite".to_string(),
                    line: 11,
                },
            ]
        );
    }
}
