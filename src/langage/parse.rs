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

use super::schema::{Bloc, Document, ElementListe, Inline, Reference};

/// Analyse un texte en Document structuré.
///
/// Ne renvoie jamais d'erreur : un texte malformé donne un Document qui
/// contient ce qui a pu être lu, et le reste en texte brut. C'est la même
/// tolérance que le reste de l'application — un fichier abîmé ne doit pas
/// empêcher d'ouvrir le projet.
pub fn parse(texte: &str) -> Document {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut blocs = Vec::new();
    let mut en_attente = Vec::new();
    let mut index = 0;

    while index < lignes.len() {
        let ligne = lignes[index];

        // Trois lignes vides consécutives font un saut de page (EDITOR-04).
        if ligne.trim().is_empty()
            && index + 2 < lignes.len()
            && lignes[index + 1].trim().is_empty()
            && lignes[index + 2].trim().is_empty()
        {
            blocs.push(habiller(Bloc::SautDePage, &mut en_attente));
            index += 3;
            continue;
        }

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
            blocs.push(habiller(Bloc::Brut(brut), &mut en_attente));
            index = suivant;
            continue;
        }

        // Un titre.
        if !echappee && let Some(niveau) = niveau_titre(ligne) {
            let contenu = parse_inline(ligne.trim_start()[niveau as usize..].trim());
            blocs.push(habiller(Bloc::Titre { niveau, contenu }, &mut en_attente));
            index += 1;
            continue;
        }

        // Une citation de bloc.
        if !echappee && ligne.trim_start().starts_with('>') {
            let contenu = parse_inline(ligne.trim_start()[1..].trim());
            blocs.push(habiller(Bloc::Citation(contenu), &mut en_attente));
            index += 1;
            continue;
        }

        // Une liste à puces.
        if !echappee && ligne.trim_start().starts_with("- ") {
            let (items, suivant) = lire_liste(&lignes, index, "- ");
            blocs.push(habiller(Bloc::ListePuce(items), &mut en_attente));
            index = suivant;
            continue;
        }

        // Une liste numérotée.
        if !echappee && est_numero(ligne) {
            let (items, suivant) = lire_liste(&lignes, index, "");
            blocs.push(habiller(Bloc::ListeNumero(items), &mut en_attente));
            index = suivant;
            continue;
        }

        // Un graphique : `{graphique type=barre}` puis des lignes `label | valeur`.
        if !echappee
            && ligne.trim_start().starts_with("{graphique")
            && let Some((graphique, suivant)) = lire_graphique(&lignes, index)
        {
            blocs.push(habiller(graphique, &mut en_attente));
            index = suivant;
            continue;
        }

        // La liste du glossaire : `{glossaire}`, ou `{glossaire: tout}`
        // pour y montrer aussi les termes jamais référencés.
        if !echappee && let Some(tout) = glossaire_ligne(ligne) {
            blocs.push(habiller(Bloc::Glossaire { tout }, &mut en_attente));
            index += 1;
            continue;
        }

        // La section bibliographie : `{bibliographie}` (bug-report-9).
        if !echappee && bibliographie_ligne(ligne) {
            blocs.push(habiller(Bloc::Bibliographie, &mut en_attente));
            index += 1;
            continue;
        }

        // La table des matières : `[[TOC]]` (bug-report-10, 1.3).
        if !echappee && sommaire_ligne(ligne) {
            blocs.push(habiller(Bloc::Sommaire, &mut en_attente));
            index += 1;
            continue;
        }

        // Une ligne d'attributs habille le bloc qui suit. Un doute, et ce
        // n'en est pas une : la ligne redevient du texte.
        if !echappee
            && let Some(proprietes) = lire_proprietes_ligne(ligne)
            && un_bloc_suit(&lignes, index)
        {
            en_attente.push(proprietes);
            index += 1;
            continue;
        }

        // Une formule à part : `$$` sur sa ligne, ou en une seule.
        if !echappee && ligne.trim_start().starts_with("$$") {
            let (formule, suivant) = lire_formule(&lignes, index);
            blocs.push(habiller(Bloc::Formule(formule), &mut en_attente));
            index = suivant;
            continue;
        }

        // Un séparateur horizontal : `---` seul sur sa ligne.
        if !echappee && ligne.trim() == "---" {
            blocs.push(habiller(Bloc::Separateur, &mut en_attente));
            index += 1;
            continue;
        }

        // Un commentaire : `// texte`. Conservé dans le .tw, ignoré au rendu.
        if !echappee && ligne.trim_start().starts_with("//") {
            let texte = ligne
                .trim_start()
                .trim_start_matches("//")
                .trim()
                .to_string();
            blocs.push(habiller(Bloc::Commentaire(texte), &mut en_attente));
            index += 1;
            continue;
        }

        // Un tableau : son en-tête, sa ligne de séparation, ses lignes.
        // Sans séparation, ce n'en est pas un — la ligne reste du texte.
        if !echappee
            && ligne.trim_start().starts_with('|')
            && let Some((tableau, suivant)) = lire_tableau(&lignes, index)
        {
            blocs.push(habiller(tableau, &mut en_attente));
            index = suivant;
            continue;
        }

        // Une image : `![légende](chemin)`.
        if !echappee
            && ligne.trim_start().starts_with("![")
            && let Some((légende, cible)) = lire_image(ligne)
        {
            blocs.push(habiller(Bloc::Image { légende, cible }, &mut en_attente));
            index += 1;
            continue;
        }

        // Un paragraphe : tout ce qui n'est pas un autre bloc, jusqu'à une
        // ligne vide ou un nouveau bloc.
        let (paragraphe, suivant) = lire_paragraphe(&lignes, index);
        // Un paragraphe de blancs n'en est pas un : la forme canonique
        // n'admet pas un paragraphe vide (feuille de route, point 2.2).
        if !paragraphe.trim().is_empty() {
            blocs.push(habiller(
                Bloc::Paragraphe(parse_inline(&paragraphe)),
                &mut en_attente,
            ));
        }
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
        .filter(|((level, _), (source_level, _))| level == source_level)
        .map(|((level, contenu), (_, line))| SourceHeading {
            level,
            text: super::schema::inline_texte_nu(contenu),
            line,
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
            || debut.starts_with("```")
            // L'image est un bloc de ligne : `![…](…)`, formé ou pas.
            || lire_image(ligne).is_some()
            // Une ligne de tableau — mais sans séparation, ce n'en est
            // pas une : le parseur sait le reste.
            || (debut.starts_with('|') && cellules(ligne).is_some())
            // Une formule à part.
            || debut.starts_with("$$")
            // Une ligne d'attributs trouvée — mais elle flotte jusqu'à son
            // bloc, et le parseur sait le reste.
            || crate::langage::parse::lire_proprietes_ligne(ligne).is_some())
}

/// Le niveau d'un titre, si la ligne en est un.
///
/// `#` à `######` : un à six dièses, suivis d'une espace. Un `#` sans espace
/// n'est pas un titre — c'est du texte.
pub fn niveau_titre(ligne: &str) -> Option<u8> {
    let debut = ligne.trim_start();
    let dièses = debut.chars().take_while(|c| *c == '#').count();

    if dièses == 0 || dièses > 6 {
        return None;
    }

    // Le dièse doit être suivi d'une espace, sinon ce n'est pas un titre.
    debut[dièses..].starts_with(' ').then_some(dièses as u8)
}

/// Une ligne est-elle le début d'une liste numérotée ?
pub fn est_numero(ligne: &str) -> bool {
    let debut = ligne.trim_start();
    let chiffres = debut.chars().take_while(|c| c.is_ascii_digit()).count();

    chiffres > 0 && debut[chiffres..].starts_with(". ")
}

/// Nombre de backticks consécutifs au début de la ligne, après indentation.
pub fn longueur_fence(ligne: &str) -> usize {
    ligne
        .trim_start()
        .chars()
        .take_while(|caractere| *caractere == '`')
        .count()
}

/// Une clôture doit avoir au moins la longueur d'ouverture, sans suffixe.
pub fn ferme_fence(ligne: &str, longueur_ouverture: usize) -> bool {
    let debut = ligne.trim_start();
    let longueur = longueur_fence(ligne);

    longueur >= longueur_ouverture && debut[longueur..].trim().is_empty()
}

/// Habille un bloc des attributs en attente — les manteaux les plus
/// éloignés d'abord, pour que la lecture donne la lecture.
fn habiller(bloc: Bloc, en_attente: &mut Vec<crate::langage::schema::Proprietes>) -> Bloc {
    let mut habille = bloc;

    while let Some(proprietes) = en_attente.pop() {
        habille = Bloc::Décor {
            proprietes,
            bloc: Box::new(habille),
        };
    }

    habille
}

/// Un bloc suit-il, pour qu'un attribut trouve à s'appliquer ?
///
/// Les lignes vides ne comptent pas : un attribut flotte jusqu'à son bloc.
/// Une autre ligne d'attributs n'est pas un bloc — elle se fond avec lui.
fn un_bloc_suit(lignes: &[&str], index: usize) -> bool {
    lignes[index + 1..]
        .iter()
        .any(|ligne| !ligne.trim().is_empty() && lire_proprietes_ligne(ligne).is_none())
}

/// Lit une ligne d'attributs : `{clé=valeur clé2={valeur à espace}}`.
///
/// `None` si la ligne n'est pas une ligne d'attributs — c'est la grammaire
/// des propriétés qui tranche, jamais l'apparence : `{rouge:texte}` n'en
/// est pas une, et reste ce qu'il est.
pub fn lire_proprietes_ligne(ligne: &str) -> Option<crate::langage::schema::Proprietes> {
    let corps = ligne.trim();
    let corps = corps.strip_prefix('{')?.strip_suffix('}')?;

    let mut proprietes = crate::langage::schema::Proprietes::vide();
    let mut reste = corps.trim();

    while !reste.is_empty() {
        // Un nom de style, sans égal : une intention, pas une mesure.
        // C'est la forme courante — `{encadré}` — et c'est elle qui garde
        // la source au markdown.
        let egal = match reste.find('=') {
            Some(egal) if !reste[..egal].contains(' ') => egal,
            _ => {
                // Le nom va jusqu'à la prochaine espace.
                let fin = reste.find(' ').unwrap_or(reste.len());
                let nom = reste[..fin].trim();

                if nom.is_empty()
                    || !nom
                        .chars()
                        .all(|c| c.is_alphabetic() || c == '-' || c == '_')
                {
                    return None;
                }

                proprietes.noms.push(nom.to_string());
                reste = reste[fin..].trim_start();
                continue;
            }
        };

        let cle = reste[..egal].trim();

        if cle.is_empty()
            || !cle
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }

        reste = reste[egal + 1..].trim_start();

        let valeur;
        if reste.starts_with('{') {
            let fin = reste.find('}')?;
            valeur = reste[1..fin].to_string();
            reste = reste[fin + 1..].trim_start();
        } else {
            let fin = reste.find(' ').unwrap_or(reste.len());
            valeur = reste[..fin].to_string();
            reste = reste[fin..].trim_start();
        }

        if valeur.is_empty() || valeur != valeur.trim() {
            return None;
        }

        proprietes.poser(cle, valeur);
    }

    (!proprietes.est_vide()).then_some(proprietes)
}

/// Lit une formule à part : `$$ … $$`, en une ou plusieurs lignes.
///
/// Comme un bloc brut : jamais fermée, elle va jusqu'à la fin du
/// document. Rien n'est jamais une erreur — ce qui ne se lit pas comme
/// une construction se lit comme du texte.
fn lire_formule(lignes: &[&str], index: usize) -> (String, usize) {
    let ligne = lignes[index].trim();

    // En une seule ligne : `$$x^2$$`.
    if ligne.len() > 4 && ligne.ends_with("$$") {
        return (ligne[2..ligne.len() - 2].to_string(), index + 1);
    }

    // Sur ses lignes : le contenu va jusqu'au `$$` fermant.
    let mut contenu = Vec::new();
    let mut suivant = index + 1;

    while suivant < lignes.len() {
        let ligne = lignes[suivant].trim();

        if ligne == "$$" {
            suivant += 1;
            break;
        }

        contenu.push(lignes[suivant]);
        suivant += 1;
    }

    (contenu.join("\n"), suivant)
}

/// Lit un tableau : en-tête, ligne de séparation, lignes.
///
/// La forme est celle du markdown — `| A | B |`, `| --- | :-: |` — et
/// sans la ligne de séparation, ce n'est rien : la ligne est du texte,
/// comme tout ce qui ne se lit pas.
fn lire_tableau(lignes: &[&str], index: usize) -> Option<(crate::langage::schema::Bloc, usize)> {
    let entete = cellules(lignes[index])?;

    if entete.is_empty() {
        return None;
    }

    // La ligne de séparation est la suivante, sans ligne vide entre deux.
    let alignements = alignements(lignes.get(index + 1)?)?;

    if alignements.len() != entete.len() {
        return None;
    }

    let mut corps = Vec::new();
    let mut suivant = index + 2;
    let mut nb_colonnes = entete.len();

    while let Some(ligne) = lignes.get(suivant) {
        if !ligne.trim_start().starts_with('|') {
            break;
        }

        let Some(cellules) = cellules(ligne) else {
            break;
        };

        // EDITOR-05 : les lignes du corps ont le même nombre de colonnes.
        // Une cellule en plus **élargit** le tableau — jamais de troncature :
        // comble les trous, garde tout le reste.
        nb_colonnes = nb_colonnes.max(cellules.len());

        corps.push(cellules);
        suivant += 1;
    }

    // L'en-tête et les alignements suivent la plus large des lignes.
    let mut entete = entete;
    entete.resize(nb_colonnes, "");
    let mut alignements = alignements;
    alignements.resize(nb_colonnes, crate::langage::schema::Alignement::Gauche);
    for ligne in &mut corps {
        ligne.resize(nb_colonnes, "");
    }

    let entete = entete.into_iter().map(parse_inline).collect();
    let lignes = corps
        .into_iter()
        .map(|ligne| ligne.into_iter().map(parse_inline).collect())
        .collect();

    Some((
        crate::langage::schema::Bloc::Tableau {
            entete,
            lignes,
            alignements,
        },
        suivant,
    ))
}

/// Les cellules d'une ligne de tableau : entre les barres, les barres
/// échappées restent du contenu.
fn cellules(ligne: &str) -> Option<Vec<&str>> {
    let corps = ligne.trim();

    // La ligne commence et finit par sa barre : c'est elle qui borne.
    let corps = corps.strip_prefix('|')?.strip_suffix('|')?;

    let mut cellules = Vec::new();
    let mut courant = 0usize;
    let mut caracteres = corps.char_indices();

    while let Some((position, caractere)) = caracteres.next() {
        match caractere {
            '\\' => {
                // Le caractère échappé ne fait jamais signe.
                caracteres.next();
            }
            '|' => {
                cellules.push(corps[courant..position].trim());
                courant = position + 1;
            }
            _ => {}
        }
    }

    cellules.push(corps[courant..].trim());
    Some(cellules)
}

/// Les alignements de la ligne de séparation : `---`, `:-:`, `---:`.
fn alignements(ligne: &str) -> Option<Vec<crate::langage::schema::Alignement>> {
    use crate::langage::schema::Alignement;

    let cellules = cellules(ligne)?;
    let mut alignements = Vec::new();

    for cellule in &cellules {
        let trait_ = cellule.trim();

        if trait_.is_empty() {
            return None;
        }

        let gauche = trait_.starts_with(':');
        let droite = trait_.ends_with(':');
        let tirets = trait_.trim_matches(':');

        if tirets.is_empty() || !tirets.chars().all(|c| c == '-') {
            return None;
        }

        alignements.push(match (gauche, droite) {
            (true, true) => Alignement::Centre,
            (false, true) => Alignement::Droite,
            _ => Alignement::Gauche,
        });
    }

    Some(alignements)
}

/// Lit une image : `![légende](chemin)`. `None` si la forme n'y est pas.
///
/// La forme est celle du lien, à l'`!` près : elle hérite donc de tout ce
/// que le lecteur de lien sait déjà — l'imbrication des crochets, les
/// échappements, l'opacité du code, et les cibles qui portent des
/// parenthèses échappées. Une légende qui contient un lien se lit juste.
/// Lit un bloc graphique : `{graphique type=barre titre="Titre"}` puis des
/// lignes `label | valeur` jusqu'à une ligne vide ou non-conforme.
pub fn lire_graphique(
    lignes: &[&str],
    index: usize,
) -> Option<(crate::langage::schema::Bloc, usize)> {
    let ligne = lignes.get(index)?;
    let corps = ligne.trim().strip_prefix("{graphique")?.trim();
    let corps = corps.trim_end_matches('}').trim();

    // Les propriétés inline : `type=barre titre="Évolution"`.
    let mut type_ = String::from("barre");
    let mut titre = String::new();
    for mot in corps.split_whitespace() {
        if let Some(v) = mot.strip_prefix("type=") {
            type_ = v.trim_matches('"').to_string();
        } else if let Some(v) = mot.strip_prefix("titre=") {
            titre = v.trim_matches('"').to_string();
        }
    }

    // Les lignes de données : `label | valeur`.
    let mut donnees = Vec::new();
    let mut suivant = index + 1;
    while let Some(ligne) = lignes.get(suivant) {
        let ligne = ligne.trim();
        if ligne.is_empty() {
            break;
        }
        let Some((label, valeur)) = ligne.split_once('|') else {
            break;
        };
        let Ok(v) = valeur.trim().replace(',', ".").parse::<f64>() else {
            break;
        };
        donnees.push((label.trim().to_string(), v));
        suivant += 1;
    }

    if donnees.is_empty() {
        return None;
    }

    Some((
        crate::langage::schema::Bloc::Graphique {
            type_,
            titre,
            donnees,
        },
        suivant,
    ))
}

pub fn lire_image(ligne: &str) -> Option<(Vec<crate::langage::schema::Inline>, String)> {
    let corps = ligne.trim().strip_prefix('!')?;
    let (texte, cible, reste) = lien(corps)?;

    // Rien après la forme : une image est une ligne, pas une phrase.
    if !reste.trim().is_empty() {
        return None;
    }

    // La légende peut se trouver dans le lien : `![alt](chemin | légende)`.
    // Le `|` sépare le chemin de la légende — la légende l'emporte sur
    // le texte alternatif, qui reste pour l'accessibilité.
    if let Some((chemin, legende)) = cible.split_once('|') {
        let legende = legende.trim();
        return Some((parse_inline(legende), chemin.trim().to_string()));
    }

    Some((parse_inline(&texte), cible))
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
fn lire_liste(lignes: &[&str], index: usize, prefixe: &str) -> (Vec<ElementListe>, usize) {
    let mut items = Vec::new();
    let mut suivant = index;

    while suivant < lignes.len() {
        let ligne = lignes[suivant];

        // Le niveau d'imbrication : deux espaces de retrait par niveau
        // (bug-report-10, 3.1).
        let niveau = (ligne.len() - ligne.trim_start().len()) / 2;

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
                items.push(ElementListe {
                    niveau,
                    contenu: parse_inline(texte.trim()),
                });
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

        // Un nouveau bloc termine aussi le paragraphe — mais la première
        // ligne est toujours à lui : c'est elle qui l'a fait commencer, et
        // un bloc qui n'a pas été reconnu comme tel reste du texte.
        if suivant > index && ouvre_un_bloc(ligne) {
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
        // Une barre oblique ouvre un **littéral** : le caractère qui suit ne
        // reçoit aucune interprétation — ni syntaxique, ni typographique.
        // Les échappements qui se suivent forment un seul littéral.
        if caractere == '\\' {
            vider!();

            let mut litteral = String::new();
            let mut position = 1;

            loop {
                // Le caractère que la barre porte.
                let Some(litterel) = reste[position..].chars().next() else {
                    // Une barre oblique en fin de texte reste elle-même.
                    litteral.push('\\');
                    break;
                };

                litteral.push(litterel);
                position += litterel.len_utf8();

                // Une autre barre enchaîne l'échappement ; sinon la main est
                // rendue au parseur.
                if reste[position..].starts_with('\\') {
                    position += 1;
                } else {
                    break;
                }
            }

            if !litteral.is_empty() {
                elements.push(Inline::Litteral(litteral));
            }

            reste = &reste[position..];
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

        // Souligné : `__texte__`.
        if let Some((contenu, apres)) = emphasis(reste, "__") {
            vider!();
            elements.push(Inline::Souligne(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Barré : `~~texte~~`.
        if let Some((contenu, apres)) = emphasis(reste, "~~") {
            vider!();
            elements.push(Inline::Barre(parse_inline(contenu)));
            reste = apres;
            continue;
        }

        // Italique : `*texte*` ou `_texte_` (la seconde forme sert à
        // l'imbrication : `**gras _et italique_**`). Le `_`, lui, n'ouvre
        // qu'à une **frontière de mot** : `snake_case` est du texte
        // technique, pas deux mots dont le second serait en italique.
        let frontiere = {
            let avant = &texte[..texte.len() - reste.len()];
            !avant.chars().last().is_some_and(|c| c.is_alphanumeric())
        };

        if let Some((contenu, apres)) =
            emphasis(reste, "*").or_else(|| frontiere.then(|| emphasis(reste, "_")).flatten())
        {
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

        // Formule en ligne : `$x^2$` — une bande opaque comme le code.
        if let Some((contenu, apres)) = delimit(reste, "$") {
            vider!();
            elements.push(Inline::Formule(contenu.to_string()));
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

        // Citation : `@clé`, `@clé[p. 45]`, ou un appel groupé
        // `@clé[p. 45]; @autre` qui réunit ses références dans une note.
        if let Some((references, apres)) = appel(reste) {
            vider!();
            elements.push(Inline::Citation(references));
            reste = apres;
            continue;
        }

        // Lien : `[texte](cible)`.
        if let Some((texte_lien, cible, apres)) = lien(reste) {
            vider!();
            elements.push(Inline::Lien {
                texte: parse_inline(&texte_lien),
                cible,
            });
            reste = apres;
            continue;
        }

        // Terme du glossaire : `{glo:clé}`, `{glo:clé pluriel}`… — avant
        // la couleur, qui prendrait `glo` pour un nom de teinte.
        if let Some((cle, majuscule, pluriel, apres)) = glo(reste) {
            vider!();
            elements.push(Inline::Glo {
                cle,
                majuscule,
                pluriel,
            });
            reste = apres;
            continue;
        }

        // Date : `{date:...}` — avant la couleur, qui prendrait `date`
        // pour un nom de teinte.
        if let Some((spec, apres)) = date(reste) {
            vider!();
            elements.push(Inline::Date(spec.to_string()));
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

    let mut curseur = 0;
    let mut dernier: Option<char> = None;

    while curseur < reste.len() {
        let caractere = reste[curseur..].chars().next()?;

        // Une barre oblique et le caractère qu'elle porte font un tout.
        if caractere == '\\' {
            if let Some(litteral) = reste[curseur + 1..].chars().next() {
                dernier = Some(litteral);
                curseur += 1 + litteral.len_utf8();
                continue;
            }

            dernier = Some('\\');
            curseur += 1;
            continue;
        }

        // Un code est opaque : ce qui s'y trouve ne ferme pas l'emphase.
        if caractere == '`'
            && let Some(fin) = fin_du_code(reste, curseur)
        {
            dernier = Some('`');
            curseur = fin;
            continue;
        }

        if reste[curseur..].starts_with(delim)
            && curseur > 0
            && dernier.is_some_and(|d| !d.is_whitespace())
        {
            return Some((&reste[..curseur], &reste[curseur + delim.len()..]));
        }

        dernier = Some(caractere);
        curseur += caractere.len_utf8();
    }

    None
}

/// Trouve un texte entre deux délimiteurs identiques, sans contrainte de blanc.
///
/// Sert au code inline, dont le contenu est **littéral** : ni échappement ni
/// balisage n'y sont reconnus. Sa seule règle est de ne pas contenir le
/// délimiteur lui-même — un code ne contient pas de backtick.
fn delimit<'a>(texte: &'a str, delim: &str) -> Option<(&'a str, &'a str)> {
    let reste = texte.strip_prefix(delim)?;
    let ferme = reste.find(delim)?;

    Some((&reste[..ferme], &reste[ferme + delim.len()..]))
}

/// Une note : `^[texte]`.
///
/// Le crochet fermant se cherche à l'épreuve des échappements **et de
/// l'imbrication** : une note peut contenir un lien `[x](y)` ou une autre
/// note, dont les crochets se comptent par paire. Un crochet échappé n'en
/// ferme aucun.
fn note(texte: &str) -> Option<(&str, &str)> {
    let apres_ouverture = texte.strip_prefix("^[")?;
    let (contenu, reste) = jusqu_au_crochet_fermant(apres_ouverture)?;

    Some((contenu, reste))
}

/// La fin d'un code inline ouvert par le backtick en `position`, s'il existe.
///
/// Un code est une **bande opaque** pour les balayages qui l'entourent : ce
/// qui s'y trouve ne ferme ni une emphase, ni une note, ni une couleur. Un
/// backtick jamais fermé n'ouvre pas de code — il reste du texte.
fn fin_du_code(texte: &str, position: usize) -> Option<usize> {
    let ferme = texte[position + 1..].find('`')?;
    Some(position + 1 + ferme + 1)
}

/// Le contenu d'un `[` ouvert, jusqu'au `]` qui le ferme.
///
/// Les crochets se comptent par paire et les échappements par unité :
/// `[un [deux] trois]` tient entier, comme `[un \] caché]`.
fn jusqu_au_crochet_fermant(texte: &str) -> Option<(&str, &str)> {
    let mut profondeur = 1;
    let mut curseur = 0;

    while curseur < texte.len() {
        let caractere = texte[curseur..].chars().next()?;

        if caractere == '\\' {
            if let Some(litteral) = texte[curseur + 1..].chars().next() {
                curseur += 1 + litteral.len_utf8();
                continue;
            }

            curseur += 1;
            continue;
        }

        // Un code est opaque : ce qui s'y trouve ne compte pas.
        if caractere == '`'
            && let Some(fin) = fin_du_code(texte, curseur)
        {
            curseur = fin;
            continue;
        }

        match caractere {
            '[' => profondeur += 1,
            ']' => {
                profondeur -= 1;

                if profondeur == 0 {
                    return Some((&texte[..curseur], &texte[curseur + 1..]));
                }
            }
            _ => {}
        }

        curseur += caractere.len_utf8();
    }

    None
}

/// Le contenu d'une `(` ouverte, jusqu'à la `)` qui la ferme, **déséchappé**.
///
/// Les parenthèses d'une cible de lien sont échappées à l'écriture ; la
/// lecture les rend littérales. Une barre oblique y échappe ce qui suit.
fn jusqu_au_parenthese_fermante(texte: &str) -> Option<(String, &str)> {
    let mut contenu = String::new();
    let mut caracteres = texte.chars();

    while let Some(caractere) = caracteres.next() {
        match caractere {
            '\\' => contenu.push(caracteres.next()?),
            ')' => return Some((contenu, caracteres.as_str())),
            _ => contenu.push(caractere),
        }
    }

    None
}

/// Un appel de citation : `@clé`, `@clé[p. 45]`, ou plusieurs références.
///
/// Un appel est **une** note de bas de page : ses références se joignent par
/// `; ` — `@dupont2023[p. 45]; @martin2020` — et s'y retrouvent toutes. Un
/// mot entre deux rompt le groupe : `@a; puis @b` reste deux appels, donc
/// deux notes.
fn appel(texte: &str) -> Option<(Vec<Reference>, &str)> {
    let (premiere, mut reste) = reference(texte)?;
    let mut references = vec![premiere];

    while let Some((autre, apres)) = suite_appel(reste) {
        references.push(autre);
        reste = apres;
    }

    Some((references, reste))
}

/// La suite d'un appel : `; ` puis une autre référence, ou rien du tout.
fn suite_appel(texte: &str) -> Option<(Reference, &str)> {
    let apres = texte.trim_start().strip_prefix(';')?;
    reference(apres.trim_start())
}

/// Une référence : `@clé`, suivie de sa mention de page entre crochets.
///
/// La clé est une suite de caractères alphanumériques, tirets et underscores.
/// La page s'écrit telle qu'elle apparaîtra — `p. 45`, `pp. 12-15`,
/// `chap. 3` — et une barre oblique y rend littéral le caractère suivant.
fn reference(texte: &str) -> Option<(Reference, &str)> {
    let apres = texte.strip_prefix('@')?;

    let fin = apres
        .char_indices()
        .find(|(_, c)| !(c.is_alphanumeric() || *c == '-' || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(apres.len());

    if fin == 0 {
        return None;
    }

    let cle = &apres[..fin];
    let reste = &apres[fin..];

    let Some(page_brute) = reste.strip_prefix('[') else {
        return Some((Reference::new(cle), reste));
    };

    match lire_page(page_brute) {
        // Une page vide n'est pas de page : `@clé[]` est `@clé`.
        Some((page, apres)) if page.is_empty() => Some((Reference::new(cle), apres)),
        Some((page, apres)) => Some((Reference::new(cle).avec_page(page), apres)),
        // Un crochet jamais fermé n'ouvre pas de page : la référence reste
        // nue, et les crochets qui suivent redeviennent du texte.
        None => Some((Reference::new(cle), reste)),
    }
}

/// L'appel de citation qui contient une position, et sa plage dans le texte.
///
/// C'est la porte du composeur sur une citation déjà écrite : un appel peut
/// réunir plusieurs références, et c'est son texte **entier** qu'il faut
/// connaître pour l'ouvrir dessus et le réécrire à la fin. On remonte aux `@`
/// précédents — un appel commence toujours par l'un d'eux, et un `@` échappé
/// n'en ouvre pas —, et le plus proche dont la plage contient la position est
/// le bon.
pub fn appel_autour(texte: &str, offset: usize) -> Option<(Vec<Reference>, usize, usize)> {
    let offset = offset.min(texte.len());
    let mut candidats = Vec::new();

    for (start, _) in texte.match_indices('@') {
        if start > offset {
            break;
        }

        // Un `@` échappé est du texte, pas un appel.
        if texte[..start].ends_with('\\') {
            continue;
        }

        // Un `@` collé à un mot est du texte : une adresse électronique
        // n'ouvre jamais le composeur.
        if texte[..start]
            .chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }

        candidats.push(start);
    }

    for start in candidats {
        let Some((references, reste)) = appel(&texte[start..]) else {
            continue;
        };

        let fin = texte.len() - reste.len();

        // Le premier `@` dont la plage contient la position ouvre l'appel du
        // plus **ancien** : c'est lui qui englobe un groupe entier, là où le
        // plus proche ne donnerait que la dernière référence.
        if offset <= fin {
            return Some((references, start, fin));
        }
    }

    None
}

/// Lit une mention de page, jusqu'au crochet fermant.
fn lire_page(texte: &str) -> Option<(String, &str)> {
    let mut page = String::new();
    let mut caracteres = texte.chars();

    while let Some(caractere) = caracteres.next() {
        match caractere {
            // Une barre oblique rend littéral le caractère suivant.
            '\\' => page.push(caracteres.next()?),
            ']' => return Some((page.trim().to_string(), caracteres.as_str())),
            _ => page.push(caractere),
        }
    }

    None
}

/// Un lien : `[texte](cible)`.
///
/// Le texte se lit jusqu'au crochet fermant à l'épreuve de l'imbrication —
/// il peut contenir une note ou un autre lien —, et la cible jusqu'à la
/// parenthèse fermante, déséchappée.
fn lien(texte: &str) -> Option<(String, String, &str)> {
    let apres_ouvert = texte.strip_prefix('[')?;

    let (texte_lien, apres_ferme) = jusqu_au_crochet_fermant(apres_ouvert)?;

    let apres_parenthese = apres_ferme.strip_prefix('(')?;
    let (cible, reste) = jusqu_au_parenthese_fermante(apres_parenthese)?;

    Some((texte_lien.to_string(), cible, reste))
}

/// Une date : `{date:...}` — la spécification jusqu'à la `}` qui ferme.
fn date(texte: &str) -> Option<(&str, &str)> {
    let reste = texte.strip_prefix("{date:")?;
    let (spec, reste) = jusqu_a_l_accolade_fermante(reste)?;
    Some((spec, reste))
}

/// La ligne `{bibliographie}` — et rien d'autre.
fn bibliographie_ligne(ligne: &str) -> bool {
    ligne.trim() == "{bibliographie}"
}

/// La ligne `[[TOC]]` — et rien d'autre.
fn sommaire_ligne(ligne: &str) -> bool {
    ligne.trim() == "[[TOC]]"
}

/// La ligne `{glossaire}` ou `{glossaire: tout}` — et rien d'autre.
fn glossaire_ligne(ligne: &str) -> Option<bool> {
    match ligne.trim() {
        "{glossaire}" => Some(false),
        "{glossaire: tout}" => Some(true),
        _ => None,
    }
}

/// Un terme du glossaire : `{glo:clé}` avec ses mots-clés facultatifs
/// `majuscule` et `pluriel`.
///
/// La clé est un identifiant ([`crate::project::glossaire::EntreeGlossaire::cle_valide`]) :
/// ce qui ne l'est pas n'est pas un terme — la couleur s'en occupera,
/// comme avant.
fn glo(texte: &str) -> Option<(String, bool, bool, &str)> {
    let reste = texte.strip_prefix("{glo:")?;
    let (spec, reste) = jusqu_a_l_accolade_fermante(reste)?;

    let mut mots = spec.split_whitespace();
    let cle = mots.next()?;
    if !crate::project::glossaire::EntreeGlossaire::cle_valide(cle) {
        return None;
    }

    let mut majuscule = false;
    let mut pluriel = false;
    for mot in mots {
        match mot {
            "majuscule" => majuscule = true,
            "pluriel" => pluriel = true,
            _ => return None,
        }
    }

    Some((cle.to_string(), majuscule, pluriel, reste))
}

/// Une couleur : `{couleur:texte}`.
///
/// Le nom va jusqu'au premier deux-points — il n'en contient pas —, et le
/// contenu jusqu'à l'accolade fermante, à l'épreuve de l'imbrication : une
/// couleur peut en contenir une autre.
fn couleur(texte: &str) -> Option<(&str, &str, &str)> {
    let apres_ouvert = texte.strip_prefix('{')?;

    let deux_points = apres_ouvert.find(':')?;
    let couleur = &apres_ouvert[..deux_points];

    let apres_deux_points = &apres_ouvert[deux_points + 1..];
    let (contenu, reste) = jusqu_a_l_accolade_fermante(apres_deux_points)?;

    Some((couleur, contenu, reste))
}

/// Le contenu d'une `{` ouverte, jusqu'à la `}` qui la ferme.
///
/// Les accolades se comptent par paire et les échappements par unité :
/// `{red:{blue:x}}` tient entier.
fn jusqu_a_l_accolade_fermante(texte: &str) -> Option<(&str, &str)> {
    let mut profondeur = 1;
    let mut curseur = 0;

    while curseur < texte.len() {
        let caractere = texte[curseur..].chars().next()?;

        if caractere == '\\' {
            if let Some(litteral) = texte[curseur + 1..].chars().next() {
                curseur += 1 + litteral.len_utf8();
                continue;
            }

            curseur += 1;
            continue;
        }

        // Un code est opaque : ce qui s'y trouve ne compte pas.
        if caractere == '`'
            && let Some(fin) = fin_du_code(texte, curseur)
        {
            curseur = fin;
            continue;
        }

        match caractere {
            '{' => profondeur += 1,
            '}' => {
                profondeur -= 1;

                if profondeur == 0 {
                    return Some((&texte[..curseur], &texte[curseur + 1..]));
                }
            }
            _ => {}
        }

        curseur += caractere.len_utf8();
    }

    None
}

#[cfg(test)]
mod tests {

    /// Une cellule en plus n'est jamais tronquée : le tableau s'élargit, et
    /// tout se relit. La seule transformation admise est la mise à largeur —
    /// comble les trous, garde tout le reste.
    /// Les cas nommés par la feuille de route (phase 2, point 2.2) côté
    /// analyseur : CRLF, chaînes vides, bords blancs, Unicode — chacun se
    /// normalise, et la normalisation tient le round-trip.
    #[test]
    fn la_normalisation_tient_sur_les_cas_nommes() {
        // CRLF et LF donnent le même document.
        assert_eq!(
            parse("Un paragraphe.\r\n\r\nUn autre.\r\n"),
            parse("Un paragraphe.\n\nUn autre.\n"),
            "CRLF et LF se lisent de même"
        );
        // Chaîne vide et blancs seuls.
        assert_eq!(parse("").blocs.len(), 0, "la chaîne vide est vide");
        // Une ligne de blancs n'est pas un paragraphe (mais trois lignes
        // vides consécutives font un saut de page — le design, EDITOR-04).
        let blancs = parse("   ");
        assert_eq!(
            blancs.blocs.len(),
            0,
            "les blancs seuls sont vides : {blancs:?}"
        );
        // Bords blancs : le texte se rogne.
        assert_eq!(
            parse("  Un paragraphe.  "),
            parse("Un paragraphe."),
            "les bords blancs se rognent"
        );
        // Unicode varié survit au round-trip.
        let unicode = "éàü 中文 🎉 a\u{0301} \u{202F}fin";
        let document = parse(unicode);
        let regenere = crate::langage::serialiser::serialiser(&document);
        assert_eq!(
            parse(&regenere),
            document,
            "l'Unicode tient le round-trip (sérialisé : {regenere:?})"
        );
    }

    #[test]
    fn une_cellule_en_plus_nest_jamais_perdue() {
        let source = "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 | 4 |\n";
        let document = super::parse(source);
        let super::super::schema::Bloc::Tableau {
            entete,
            lignes,
            alignements,
        } = document.blocs.first().expect("tableau")
        else {
            panic!("ce devrait être un tableau");
        };
        assert_eq!(lignes[0].len(), 4, "la cellule en plus devient une colonne");
        assert_eq!(entete.len(), 4, "l'en-tête suit");
        assert_eq!(alignements.len(), 4, "les alignements suivent");
        let relu = super::parse(&crate::langage::serialiser(&document));
        assert!(
            relu.texte_nu().contains('4'),
            "la cellule survit au round-trip"
        );
        assert_eq!(
            crate::langage::serialiser(&relu),
            crate::langage::serialiser(&document),
            "le round-trip est stable"
        );
    }

    /// Un tableau irrégulier ne perd aucune cellule, quelle que soit la forme
    /// des lignes — courtes, longues, seules.
    #[test]
    fn un_tableau_irregulier_ne_perd_aucune_cellule() {
        let source = "| A | B |\n| --- | --- |\n| 1 | 2 | 3 |\n| seul |\n| 5 | 6 |\n";
        let document = super::parse(source);
        let nu = document.texte_nu();
        for valeur in ["A", "B", "1", "2", "3", "seul", "5", "6"] {
            assert!(nu.contains(valeur), "« {valeur} » perdue : {nu}");
        }
    }

    /// Les cellules unicode se relisent telles quelles.
    #[test]
    fn les_cellules_unicode_se_relisent_telles_quelles() {
        let source = "| Été | été |\n| --- | --- |\n| ça | naïve — « guillemets » |\n";
        let document = super::parse(source);
        let relu = super::parse(&crate::langage::serialiser(&document));
        assert_eq!(relu.texte_nu(), document.texte_nu());
        assert!(relu.texte_nu().contains("naïve — « guillemets »"));
    }

    /// Un `_` intramot est du texte : `snake_case` ne devient pas
    /// `snake` + italique `case`, et ne se réécrit pas en le devenant.
    #[test]
    fn le_tiret_bas_intramot_est_du_texte() {
        let document = parse("snake_case et _italique_ ici.\n");

        let nu = document.texte_nu();
        assert!(nu.contains("snake_case"), "le mot technique survit : {nu}");
        assert!(
            document.blocs.iter().any(|bloc| matches!(
                bloc,
                super::super::schema::Bloc::Paragraphe(contenu)
                    if contenu.iter().any(|element| matches!(
                        element,
                        super::super::schema::Inline::Italique(_)
                    ))
            )),
            "l'italique aux frontières compose : {document:?}"
        );
    }

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
                Bloc::ListePuce(vec![ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("puce".to_string())],
                }]),
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
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Une ".to_string()),
                Inline::Litteral("*".to_string()),
                Inline::Texte("étoile".to_string()),
                Inline::Litteral("*".to_string()),
                Inline::Texte(" et du ".to_string()),
                Inline::Litteral("#".to_string()),
                Inline::Texte("texte.".to_string()),
            ])],
            "l'échappement fait un littéral : le caractère nu, sans interprétation"
        );
    }

    /// Une ligne échappée n'ouvre pas de bloc : elle reste du paragraphe.
    #[test]
    fn une_ligne_echappee_reste_du_paragraphe() {
        let doc = parse("Un paragraphe\n\\# pas un titre\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Un paragraphe ".to_string()),
                Inline::Litteral("#".to_string()),
                Inline::Texte(" pas un titre".to_string()),
            ])]
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
                Inline::Citation(vec![Reference::new("dupont2023")]),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Une mention de page se lit avec sa référence, et rien n'autre.
    #[test]
    fn une_page_se_lit_avec_sa_reference() {
        let doc = parse("Voir @dupont2023[p. 45].\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Voir ".to_string()),
                Inline::Citation(vec![Reference::new("dupont2023").avec_page("p. 45")]),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// Un point-virgule joint les références dans **un** appel — donc une
    /// note — et un mot entre deux rompt le groupe.
    #[test]
    fn un_point_virgule_reunit_les_references_d_un_meme_appel() {
        let doc = parse("Voir @dupont2023[p. 45]; @martin2020 et @bernard2021.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Voir ".to_string()),
                Inline::Citation(vec![
                    Reference::new("dupont2023").avec_page("p. 45"),
                    Reference::new("martin2020"),
                ]),
                Inline::Texte(" et ".to_string()),
                Inline::Citation(vec![Reference::new("bernard2021")]),
                Inline::Texte(".".to_string()),
            ])]
        );
    }

    /// L'appel autour du curseur se lit entier, groupe compris : c'est son
    /// texte tout entier que le composeur devra réécrire.
    #[test]
    fn l_appel_autour_se_lit_entier() {
        let texte = "Voir @dupont2023[p. 45]; @martin2020 ici.";
        let offset = texte.find("@martin").unwrap() + 3;

        let (references, debut, fin) = appel_autour(texte, offset).expect("un appel");

        assert_eq!(&texte[debut..fin], "@dupont2023[p. 45]; @martin2020");
        assert_eq!(references.len(), 2);

        // Le curseur en bout d'appel est encore dedans.
        assert!(appel_autour(texte, fin).is_some());
    }

    /// Hors d'un appel, il n'y en a pas — et un `@` échappé n'en ouvre pas.
    #[test]
    fn pas_d_appel_hors_citation() {
        let texte = "Un arobase nu @ n'ouvre rien, et \\@dupont2023 non plus.";
        assert_eq!(appel_autour(texte, 20), None);
        assert_eq!(appel_autour(texte, texte.find("dupont").unwrap()), None);
    }

    /// Un crochet jamais fermé n'ouvre pas de page : la référence reste nue,
    /// et les crochets redeviennent du texte.
    #[test]
    fn un_crochet_non_ferme_n_ouvre_pas_de_page() {
        let doc = parse("Voir @dupont2023[p. 45.\n");

        assert_eq!(
            doc.blocs,
            vec![Bloc::Paragraphe(vec![
                Inline::Texte("Voir ".to_string()),
                Inline::Citation(vec![Reference::new("dupont2023")]),
                Inline::Texte("[p. 45.".to_string()),
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
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("un".to_string())],
                },
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("deux".to_string())],
                },
            ])]
        );
    }

    /// Une liste multiniveau se lit — deux espaces de retrait par
    /// niveau (bug-report-10, 3.1).
    #[test]
    fn une_liste_multiniveau_se_lit() {
        let saut = &char::from(10).to_string();
        let source = ["- un", "  - sous", "- deux", ""].join(saut);
        let doc = parse(&source);

        assert_eq!(
            doc.blocs,
            vec![Bloc::ListePuce(vec![
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("un".to_string())],
                },
                ElementListe {
                    niveau: 1,
                    contenu: vec![Inline::Texte("sous".to_string())],
                },
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("deux".to_string())],
                },
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
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("un".to_string())],
                },
                ElementListe {
                    niveau: 0,
                    contenu: vec![Inline::Texte("deux".to_string())],
                },
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

#[cfg(test)]
mod tests_de_la_section_bibliographie {
    use super::super::schema::{Bloc, Inline};

    /// La ligne `{bibliographie}` est une section — et rien d'autre.
    #[test]
    fn la_ligne_bibliographie_est_un_bloc() {
        let doc = super::parse("Du texte.\n\n{bibliographie}\n");

        assert_eq!(
            doc.blocs,
            vec![
                Bloc::Paragraphe(vec![Inline::Texte("Du texte.".to_string())]),
                Bloc::Bibliographie,
            ]
        );

        // Une ligne habillée de plus n'en est pas une.
        let doc = super::parse("{bibliographie} de trop\n");
        assert!(
            !doc.blocs
                .iter()
                .any(|bloc| matches!(bloc, Bloc::Bibliographie)),
            "seule la ligne nue compte : {doc:?}"
        );
    }
}

#[cfg(test)]
mod tests_du_sommaire {
    use super::super::schema::{Bloc, Inline};

    /// La table des matières : `[[TOC]]` est une section (bug-report-10).
    #[test]
    fn la_ligne_toc_est_un_bloc() {
        let doc = super::parse("Du texte.\n\n[[TOC]]\n");

        assert_eq!(
            doc.blocs,
            vec![
                Bloc::Paragraphe(vec![Inline::Texte("Du texte.".to_string())]),
                Bloc::Sommaire,
            ]
        );
    }
}
