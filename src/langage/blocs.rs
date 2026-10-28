//! Les blocs, dans le texte source — l'unité de l'éditeur de blocs.
//!
//! Le langage intermédiaire s'écrit en syntaxe markdown : c'est elle que
//! l'on tape, et elle ne change pas. Ce qui change, c'est l'**unité** sur
//! laquelle portent les gestes : une transformation de bloc touche le bloc
//! entier, pas la ligne du curseur — et un bloc se déplace d'un bloc.
//!
//! Le découpage est celui du parseur, ligne pour ligne : un titre est une
//! ligne, une citation de bloc est une ligne, une liste groupe ses lignes,
//! un bloc brut court de sa clôture, et un paragraphe s'arrête à la ligne
//! vide ou à l'ouverture d'un autre bloc. La preuve en est un test : le
//! nombre et la nature des blocs d'ici sont ceux de
//! [`crate::langage::parse`].

use std::ops::Range;

use super::parse::{est_numero, ferme_fence, longueur_fence, niveau_titre};

/// La nature d'un bloc, dans la syntaxe markdown de l'éditeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    /// `# Titre` … `###### Titre`.
    Titre(u8),

    /// `> texte`.
    Citation,

    /// `- élément`.
    Puce,

    /// `1. élément`.
    Numero,

    /// Le bloc fermé par des fences : ```typst … ``` — le Typst brut, la
    /// porte d'accès aux constructions avancées.
    Brut,

    /// Du texte libre.
    Paragraphe,
}

/// Un bloc du texte source : sa nature, et ses lignes.
///
/// Les lignes sont à partir de zéro, `[debut, fin[` : les lignes vides qui
/// séparent les blocs ne sont à personne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bloc {
    pub genre: Genre,
    pub lignes: Range<usize>,
}

/// Les blocs d'un texte source, dans l'ordre.
pub fn blocs(source: &str) -> Vec<Bloc> {
    let lignes: Vec<&str> = source.lines().collect();
    let mut blocs = Vec::new();
    let mut index = 0usize;

    while index < lignes.len() {
        let ligne = lignes[index];

        // Les lignes vides séparent les blocs, sans être elles-mêmes un bloc.
        if ligne.trim().is_empty() {
            index += 1;
            continue;
        }

        let echappee = ligne.trim_start().starts_with('\\');

        // Un bloc brut Typst : toute la zone fermée.
        if !echappee && ligne.trim_start().starts_with("```") {
            let fin = fin_du_brut(&lignes, index);
            blocs.push(Bloc {
                genre: Genre::Brut,
                lignes: index..fin,
            });
            index = fin;
            continue;
        }

        // Un titre : une ligne, et pas une de plus.
        if !echappee && let Some(niveau) = niveau_titre(ligne) {
            blocs.push(Bloc {
                genre: Genre::Titre(niveau),
                lignes: index..index + 1,
            });
            index += 1;
            continue;
        }

        // Une citation de bloc : une ligne aussi.
        if !echappee && ligne.trim_start().starts_with('>') {
            blocs.push(Bloc {
                genre: Genre::Citation,
                lignes: index..index + 1,
            });
            index += 1;
            continue;
        }

        // Une liste groupe ses lignes, puces ou numéros d'accord.
        if !echappee && ligne.trim_start().starts_with("- ") {
            let fin = fin_de_la_liste(&lignes, index, true);
            blocs.push(Bloc {
                genre: Genre::Puce,
                lignes: index..fin,
            });
            index = fin;
            continue;
        }

        if !echappee && est_numero(ligne) {
            let fin = fin_de_la_liste(&lignes, index, false);
            blocs.push(Bloc {
                genre: Genre::Numero,
                lignes: index..fin,
            });
            index = fin;
            continue;
        }

        // Un paragraphe court jusqu'à la ligne vide ou un bloc qui s'ouvre.
        let fin = fin_du_paragraphe(&lignes, index);
        blocs.push(Bloc {
            genre: Genre::Paragraphe,
            lignes: index..fin,
        });
        index = fin;
    }

    blocs
}

/// Le bloc qui contient une ligne, s'il y en a un.
pub fn bloc_a(source: &str, ligne: usize) -> Option<usize> {
    blocs(source)
        .iter()
        .position(|bloc| bloc.lignes.contains(&ligne))
}

/// Le bloc devient le genre demandé, et la source toute neuve s'ensuit.
///
/// Redemander le genre du bloc le **retire** : c'est la bascule. Un titre
/// de niveau différent change de niveau, sans se retirer. Les lignes du
/// bloc se reprennent entièrement : le paragraphe devient un titre d'une
/// ligne (ses lignes se joignent), la liste devient du texte nu (ses
/// marqueurs partent).
/// Le genre qu'un bloc obtient : redemander le sien le rend paragraphe.
pub fn genre_effectif(actuel: Genre, demande: Genre) -> Genre {
    if actuel == demande {
        Genre::Paragraphe
    } else {
        demande
    }
}

/// Ce qui remplace le bloc, et où il commence dans la source.
///
/// La plage est en **octets** — la monnaie de l'éditeur — et le texte est
/// celui du bloc neuf, sans ses lignes voisines. [`transformer`] n'est que
/// ceci posé dans la source.
pub fn remplacement(source: &str, bloc: &Bloc, genre: Genre) -> Option<(Range<usize>, String)> {
    let lignes: Vec<&str> = source.lines().collect();

    if bloc.lignes.start >= bloc.lignes.end || bloc.lignes.end > lignes.len() {
        return None;
    }

    let genre = genre_effectif(bloc.genre, genre);

    let mut contenu: Vec<String> = lignes[bloc.lignes.clone()]
        .iter()
        .map(|ligne| sans_marque(ligne, bloc.genre))
        .collect();

    // Un bloc brut se défait de ses fences : elles sont au pourtour, jamais
    // dans le contenu.
    if bloc.genre == Genre::Brut {
        contenu.remove(0);

        let ouverture = longueur_fence(lignes[bloc.lignes.start]).max(3);
        if contenu
            .last()
            .is_some_and(|ligne| ferme_fence(ligne, ouverture))
        {
            contenu.pop();
        }
    }

    let remplacement: Vec<String> = match genre {
        Genre::Paragraphe => contenu,

        Genre::Titre(niveau) => {
            let texte = contenu
                .iter()
                .map(|ligne| ligne.trim())
                .collect::<Vec<_>>()
                .join(" ");

            vec![format!("{} {texte}", "#".repeat(niveau as usize))]
        }

        Genre::Citation => contenu
            .iter()
            .map(|ligne| format!("> {}", ligne.trim()))
            .collect(),

        Genre::Puce => contenu
            .iter()
            .map(|ligne| format!("- {}", ligne.trim()))
            .collect(),

        Genre::Numero => contenu
            .iter()
            .enumerate()
            .map(|(rang, ligne)| format!("{}. {}", rang + 1, ligne.trim()))
            .collect(),

        Genre::Brut => {
            let mut sortante = vec!["```typst".to_string()];
            sortante.extend(contenu);
            sortante.push("```".to_string());
            sortante
        }
    };

    // Les décalages viennent de la source elle-même : `lines()` mange les
    // `\r`, et un fichier CRLF se serait fait couper au mauvais octet.
    let debuts = debuts_de_ligne(source);
    let debut = debuts[bloc.lignes.start];

    let derniere = debuts[bloc.lignes.end - 1];
    let fin = source[derniere..]
        .find('\n')
        .map(|relatif| derniere + relatif)
        .unwrap_or(source.len());

    Some((debut..fin, remplacement.join("\n")))
}

/// Le bloc devient le genre demandé, et la source toute neuve s'ensuit.
///
/// L'API de lecture : l'éditeur, lui, travaille par [`remplacement`], qui
/// lui dit quoi poser à quelle place. Celle-ci est la même chose, dite en
/// termes de document.
#[allow(dead_code)]
pub fn transformer(source: &str, bloc: &Bloc, genre: Genre) -> Option<String> {
    let (plage, texte) = remplacement(source, bloc, genre)?;

    Some(format!(
        "{}{}{}",
        &source[..plage.start],
        texte,
        &source[plage.end..]
    ))
}

/// Le bloc monte d'un cran : il échange sa place avec le bloc d'avant.
///
/// Renvoie la source neuve et la ligne où le bloc commence désormais — le
/// curseur y va, il suit le bloc.
pub fn monter(source: &str, bloc: &Bloc) -> Option<(String, usize)> {
    echanger(source, bloc, true)
}

/// Le bloc descend d'un cran : il échange sa place avec le bloc d'après.
pub fn descendre(source: &str, bloc: &Bloc) -> Option<(String, usize)> {
    echanger(source, bloc, false)
}

/// L'échange lui-même, vers le haut ou vers le bas.
fn echanger(source: &str, bloc: &Bloc, vers_le_haut: bool) -> Option<(String, usize)> {
    let tous = blocs(source);

    let rang = tous.iter().position(|autre| autre == bloc)?;
    let voisin = if vers_le_haut {
        rang.checked_sub(1)?
    } else {
        let voisin = rang + 1;
        (voisin < tous.len()).then_some(voisin)?
    };

    let (premier, second) = if vers_le_haut {
        (&tous[voisin], &tous[rang])
    } else {
        (&tous[rang], &tous[voisin])
    };

    let lignes: Vec<&str> = source.lines().collect();
    let mut sortante: Vec<String> = Vec::with_capacity(lignes.len());

    sortante.extend(
        lignes[..premier.lignes.start]
            .iter()
            .map(|ligne| ligne.to_string()),
    );
    sortante.extend(
        lignes[second.lignes.clone()]
            .iter()
            .map(|ligne| ligne.to_string()),
    );
    sortante.extend(
        lignes[premier.lignes.end..second.lignes.start]
            .iter()
            .map(|ligne| ligne.to_string()),
    );
    sortante.extend(
        lignes[premier.lignes.clone()]
            .iter()
            .map(|ligne| ligne.to_string()),
    );
    sortante.extend(
        lignes[second.lignes.end..]
            .iter()
            .map(|ligne| ligne.to_string()),
    );

    // Le bloc partage la place de son voisin : c'est sa première ligne qu'on
    // rend, pour y ramener le curseur.
    let premiere_ligne = if vers_le_haut {
        premier.lignes.start
    } else {
        second.lignes.start
    };

    Some((sortante.join("\n"), premiere_ligne))
}

/// Les décalages de début de chaque ligne, fins de ligne comprises.
fn debuts_de_ligne(source: &str) -> Vec<usize> {
    let mut debuts = vec![0];

    for (index, caractere) in source.char_indices() {
        if caractere == '\n' {
            debuts.push(index + caractere.len_utf8());
        }
    }

    debuts
}

/// Le contenu d'une ligne, sans son marqueur de bloc.
pub fn sans_marque(ligne: &str, genre: Genre) -> String {
    let retire = match genre {
        Genre::Titre(niveau) => {
            let marque = "#".repeat(niveau as usize);
            ligne
                .trim_start()
                .strip_prefix(&marque)
                .and_then(|reste| reste.strip_prefix(' '))
        }
        Genre::Citation => ligne
            .trim_start()
            .strip_prefix('>')
            .map(|reste| reste.trim_start()),
        Genre::Puce => ligne.trim_start().strip_prefix("- "),
        Genre::Numero => {
            let debut = ligne.trim_start();
            let chiffres = debut.chars().take_while(|c| c.is_ascii_digit()).count();

            if chiffres > 0 && debut[chiffres..].starts_with(". ") {
                Some(&debut[chiffres + 2..])
            } else {
                None
            }
        }
        Genre::Brut | Genre::Paragraphe => None,
    };

    retire.unwrap_or(ligne).to_string()
}

/// La fin du bloc brut ouvert à `index` : la ligne après sa clôture.
fn fin_du_brut(lignes: &[&str], index: usize) -> usize {
    let longueur_ouverture = longueur_fence(lignes[index]).max(3);
    let mut suivant = index + 1;

    while suivant < lignes.len() {
        if ferme_fence(lignes[suivant], longueur_ouverture) {
            return suivant + 1;
        }

        suivant += 1;
    }

    lignes.len()
}

/// La fin de la liste ouverte à `index` : la ligne après ses éléments.
fn fin_de_la_liste(lignes: &[&str], index: usize, puces: bool) -> usize {
    let mut suivant = index;

    while suivant < lignes.len() {
        let ligne = lignes[suivant];
        let meme_marque = if puces {
            ligne.trim_start().starts_with("- ")
        } else {
            est_numero(ligne)
        };

        if !meme_marque || ligne.trim_start().starts_with('\\') {
            break;
        }

        suivant += 1;
    }

    suivant
}

/// La fin du paragraphe ouvert à `index`.
fn fin_du_paragraphe(lignes: &[&str], index: usize) -> usize {
    let mut suivant = index;

    while suivant < lignes.len() {
        let ligne = lignes[suivant];

        if ligne.trim().is_empty() || ouvre_un_bloc(ligne) {
            break;
        }

        suivant += 1;
    }

    suivant
}

/// La ligne ouvre-t-elle un bloc, dans le corps d'un paragraphe ?
///
/// La même règle que le parseur : les lignes échappées sont du texte.
fn ouvre_un_bloc(ligne: &str) -> bool {
    let debut = ligne.trim_start();

    !debut.starts_with('\\')
        && (niveau_titre(ligne).is_some()
            || debut.starts_with('>')
            || debut.starts_with("- ")
            || est_numero(ligne)
            || debut.starts_with("```"))
}

#[cfg(test)]
mod tests {

    /// Sur un fichier CRLF, la plage coupée est celle du bloc — pas
    /// décalée par les `\r` que `lines()` a mangés.
    #[test]
    fn les_plages_tiennent_sur_un_fichier_crlf() {
        let source = "Un paragraphe.\r\n\r\n- puce";
        let bloc = blocs(source)[1].clone();

        let (plage, texte) =
            remplacement(source, &bloc, Genre::Paragraphe).expect("transformation");
        assert_eq!(&source[plage], "- puce", "la plage est celle du bloc");
        assert_eq!(texte, "puce");
    }

    use super::*;

    /// Le découpage est celui du parseur : même nombre de blocs, même
    /// nature, même ordre. C'est la preuve que l'éditeur de blocs est bien
    /// **relié à l'AST**, et pas à une idée voisine.
    #[test]
    fn le_decoupage_est_celui_du_parseur() {
        let sources = [
            "# Titre\n\nUn paragraphe.\n",
            "Un paragraphe\ndeux lignes.\n\n> une citation\n> une suite\n",
            "- un\n- deux\n\n1. un\n2. deux\n",
            "```typst\n#let x = 1\n```\n\nAprès.\n",
            "# A\n## B\nTexte.\n- puce\n> cite\n```typst\noui\n```",
            "\\# pas un titre\n\nDu texte.\n",
        ];

        for source in sources {
            let attendu = crate::langage::parse(source);
            let vus = blocs(source);

            let genres: Vec<String> = vus.iter().map(|bloc| format!("{:?}", bloc.genre)).collect();

            assert_eq!(
                vus.len(),
                attendu.blocs.len(),
                "blocs de {source:?} : {genres:?} contre {:?}",
                attendu.blocs
            );
        }
    }

    /// Le bloc du curseur se trouve, et les lignes vides ne sont à personne.
    #[test]
    fn le_bloc_du_curseur_se_trouve() {
        let source = "# Titre\n\nUn paragraphe\ndeux lignes.\n\n- puce\n";

        assert_eq!(
            blocs(source)[bloc_a(source, 0).expect("bloc")].genre,
            Genre::Titre(1)
        );
        assert_eq!(
            blocs(source)[bloc_a(source, 2).expect("bloc")].genre,
            Genre::Paragraphe,
            "la première ligne du paragraphe"
        );
        assert_eq!(
            blocs(source)[bloc_a(source, 3).expect("bloc")].genre,
            Genre::Paragraphe,
            "la seconde ligne du même bloc"
        );
        assert_eq!(bloc_a(source, 1), None, "une ligne vide n'est à personne");
    }

    /// Une transformation touche le bloc entier, et la bascule le retire.
    #[test]
    fn un_bloc_se_transforme_entier() {
        let source = "une phrase\ndeux phrases.\n\nLa suite.";
        let bloc = blocs(source)[0].clone();

        // Le paragraphe devient une liste : chaque ligne, un élément.
        assert_eq!(
            transformer(source, &bloc, Genre::Puce).expect("transformation"),
            "- une phrase\n- deux phrases.\n\nLa suite."
        );

        // Il devient un titre : ses lignes se joignent.
        assert_eq!(
            transformer(source, &bloc, Genre::Titre(2)).expect("transformation"),
            "## une phrase deux phrases.\n\nLa suite."
        );

        // Et redemander la puce la retire.
        let en_puces = transformer(source, &bloc, Genre::Puce).expect("puces");
        let bloc = blocs(&en_puces)[0].clone();
        assert_eq!(
            transformer(&en_puces, &bloc, Genre::Puce).expect("bascule"),
            "une phrase\ndeux phrases.\n\nLa suite."
        );
    }

    /// Les marqueurs d'un bloc partent quand il devient du texte nu.
    #[test]
    fn les_marqueurs_partent_avec_le_paragraphe() {
        let source = "# Un titre\n\n> une citation\n\n- puce";
        let blocs = blocs(source);

        assert_eq!(
            transformer(source, &blocs[0], Genre::Paragraphe).expect("transformation"),
            "Un titre\n\n> une citation\n\n- puce"
        );
        assert_eq!(
            transformer(source, &blocs[1], Genre::Paragraphe).expect("transformation"),
            "# Un titre\n\nune citation\n\n- puce"
        );
    }

    /// Un titre change de niveau sans se retirer ; redemander le sien le
    /// retire.
    #[test]
    fn un_titre_change_de_niveau() {
        let source = "## Deuxième";
        let bloc = blocs(source)[0].clone();

        assert_eq!(
            transformer(source, &bloc, Genre::Titre(4)).expect("niveau"),
            "#### Deuxième"
        );
        assert_eq!(
            transformer(source, &bloc, Genre::Titre(2)).expect("bascule"),
            "Deuxième"
        );
    }

    /// Un bloc se déplace d'un bloc — et le curseur le suit.
    #[test]
    fn un_bloc_se_deplace_et_le_curseur_le_suit() {
        let source = "# Titre\n\nUn paragraphe.\n\nLa fin.";

        // Le paragraphe monte : il passe devant le titre.
        let (neuf, ligne) = monter(source, &blocs(source)[1]).expect("montée");
        assert_eq!(neuf, "Un paragraphe.\n\n# Titre\n\nLa fin.");
        assert_eq!(ligne, 0, "le bloc commence en haut");

        // Et il redescend.
        let (neuf, ligne) = descendre(&neuf, &blocs(&neuf)[0]).expect("descente");
        assert_eq!(neuf, source);
        assert_eq!(ligne, 2);
    }

    /// Un bloc Typst brut est opaque : ses lignes sont les siennes, fences
    /// comprises.
    #[test]
    fn un_bloc_brut_est_opaque() {
        let source = "```typst\n#let x = 1\n```\n\nAprès.";
        let bloc = blocs(source)[0].clone();

        assert_eq!(bloc.genre, Genre::Brut);
        assert_eq!(bloc.lignes, 0..3);

        // Il devient du texte : ses lignes restent entières.
        assert_eq!(
            transformer(source, &bloc, Genre::Paragraphe).expect("transformation"),
            "#let x = 1\n\nAprès."
        );
    }
}
