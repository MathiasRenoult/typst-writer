//! La liste de complétion : ce qui la déclenche, ce qu'elle montre, ce qu'elle
//! écrit.
//!
//! Tout ce qui décide vit ici, et rien n'y touche à l'interface : ce sont des
//! fonctions pures, donc éprouvables, et c'est là que se cachent les erreurs qui
//! coûtent — le mot qu'on croyait remplacer, le gabarit inséré tel quel.

use super::messages::CompletionItem;

/// Les caractères qui, tapés, demandent la liste.
///
/// Ce sont ceux que le serveur annonce comme déclencheurs : `#` ouvre ses
/// commandes, `@` ses citations. L'application ne devine pas — elle reprend ce
/// que le serveur a dit savoir faire.
pub const TRIGGERS: [char; 2] = ['#', '@'];

/// Ce qu'un déclencheur a ouvert : le mot en cours, et où il commence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    /// Le caractère qui a ouvert la liste.
    pub trigger: char,

    /// Ce qui a été tapé après lui — ce sur quoi on filtre.
    pub typed: String,

    /// Le décalage, en octets, du **début du mot** (après le déclencheur) : ce
    /// qui serait remplacé si le serveur ne le disait pas lui-même.
    pub start: usize,
}

/// Ce que le texte, à cette position, demande-t-il ?
///
/// On remonte au début du mot en cours, puis on regarde le caractère qui le
/// précède : c'est la même règle pour déclencher la liste et pour savoir quoi
/// remplacer, ce qui évite que les deux divergent.
pub fn context(text: &str, offset: usize) -> Option<Context> {
    let offset = offset.min(text.len());
    let before = &text[..offset];

    // Le mot en cours : lettres, chiffres, et les caractères qui composent un
    // nom de commande ou une clé de citation.
    let word_start = before
        .char_indices()
        .rev()
        .take_while(|(_, character)| is_word(*character))
        .last()
        .map_or(offset, |(index, _)| index);

    let (_, trigger) = before[..word_start].char_indices().next_back()?;

    if !TRIGGERS.contains(&trigger) {
        return None;
    }

    Some(Context {
        trigger,
        typed: text[word_start..offset].to_string(),
        start: word_start,
    })
}

/// Un caractère peut-il appartenir à un nom de commande ou à une clé ?
fn is_word(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
}

/// Les éléments qui correspondent à ce qui a été tapé, dans l'ordre du serveur.
///
/// L'ordre est **celui du serveur** : il trie mieux que nous, et il le dit par
/// son `sortText`. On ne fait que retirer ce qui ne correspond pas.
///
/// Deux passes, parce que c'est ce que la main attend : d'abord ce qui *commence*
/// par ce qu'on a tapé, puis ce qui le *contient*. « figure » doit passer devant
/// « quad_figure ».
pub fn matching(items: &[CompletionItem], typed: &str) -> Vec<usize> {
    if typed.is_empty() {
        return (0..items.len()).collect();
    }

    let wanted = typed.to_lowercase();

    let mut starts: Vec<usize> = Vec::new();
    let mut contains: Vec<usize> = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let candidate = item.filter().to_lowercase();

        if candidate.starts_with(&wanted) {
            starts.push(index);
        } else if candidate.contains(&wanted) {
            contains.push(index);
        }
    }

    starts.extend(contains);

    starts
}

/// L'expansion d'un gabarit, et la place du curseur après elle.
///
/// tinymist envoie des marqueurs (`figure(${1:})`) **même quand on lui déclare
/// ne pas savoir les expanser** : mesuré, et donc à traiter ici. Un marqueur est
/// remplacé par sa valeur par défaut, ou par rien, et le curseur se pose au
/// premier d'entre eux — à l'endroit où l'on veut écrire.
///
/// Ce n'est pas une implémentation complète des gabarits : il n'y a ni tabulation
/// entre les marqueurs, ni choix. C'est le strict nécessaire pour qu'aucun
/// marqueur ne se retrouve dans le document.
pub fn expand(snippet: &str) -> (String, Option<usize>) {
    let characters: Vec<char> = snippet.chars().collect();
    let mut expanded = String::with_capacity(snippet.len());
    let mut first_stop: Option<usize> = None;
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        // Un marqueur échappé est un caractère ordinaire.
        if character == '\\' && index + 1 < characters.len() {
            expanded.push(characters[index + 1]);
            index += 2;

            continue;
        }

        if character != '$' {
            expanded.push(character);
            index += 1;

            continue;
        }

        let stop = first_stop.is_none();

        // `$N` ou `${N:defaut}` : on saute le numéro, et on garde le défaut.
        index += 1;

        let braced = characters.get(index) == Some(&'{');

        if braced {
            index += 1;
        }

        let digits_start = index;

        while index < characters.len() && characters[index].is_ascii_digit() {
            index += 1;
        }

        if index == digits_start {
            // Un `$` qui n'ouvre pas de marqueur est un `$` ordinaire.
            expanded.push('$');

            continue;
        }

        if stop && first_stop.is_none() {
            first_stop = Some(expanded.chars().count());
        }

        if braced {
            // La valeur par défaut, jusqu'à l'accolade fermante correspondante.
            if characters.get(index) == Some(&':') {
                index += 1;

                let mut depth = 1;

                while index < characters.len() {
                    match characters[index] {
                        '\\' if index + 1 < characters.len() => {
                            expanded.push(characters[index + 1]);
                            index += 2;

                            continue;
                        }
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;

                            if depth == 0 {
                                index += 1;

                                break;
                            }
                        }
                        other => expanded.push(other),
                    }

                    index += 1;
                }
            } else if characters.get(index) == Some(&'}') {
                index += 1;
            }
        }
    }

    (expanded, first_stop)
}

/// Ce qu'il faut faire au document pour accepter un élément.
///
/// Le remplacement, et non l'ajout : `#fig` devient `#figure(…)`, il ne devient
/// pas `#figfigure(…)`. Le serveur donne la plage exacte quand il peut ; à
/// défaut, on remplace ce qu'on a soi-même compté.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insertion {
    /// Nombre de caractères à effacer avant d'écrire.
    pub backspaces: usize,

    /// Le texte à écrire.
    pub text: String,

    /// Où poser le curseur, en caractères depuis le début du texte écrit.
    pub caret: Option<usize>,
}

/// Ce qu'il faut faire pour accepter un élément.
///
/// `typed` est ce qui a été tapé après le déclencheur, et `replacement` la
/// longueur que le serveur demande de remplacer — le plus souvent la longueur de
/// `typed`, mais pas toujours.
pub fn insertion(item: &CompletionItem, typed: &str, replacement: Option<usize>) -> Insertion {
    let (text, caret) = expand(item.insertion().unwrap_or(&item.label));

    Insertion {
        backspaces: replacement.unwrap_or_else(|| typed.chars().count()),
        text,
        caret,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::messages::CompletionItem;

    fn item(label: &str, insert: Option<&str>) -> CompletionItem {
        let mut value = serde_json::json!({ "label": label });

        if let Some(insert) = insert {
            value["insertText"] = serde_json::Value::String(insert.to_string());
        }

        serde_json::from_value(value).expect("élément")
    }

    /// Le déclencheur se reconnaît au mot en cours, et à rien d'autre.
    #[test]
    fn a_trigger_opens_the_list() {
        assert_eq!(
            context("#fig", 4),
            Some(Context {
                trigger: '#',
                typed: "fig".to_string(),
                start: 1
            })
        );
        assert_eq!(
            context("@cle", 4),
            Some(Context {
                trigger: '@',
                typed: "cle".to_string(),
                start: 1
            })
        );

        // Sans déclencheur, pas de liste : c'est de la prose, pas du code.
        assert_eq!(context("fig", 3), None);
        assert_eq!(context("", 0), None);
        assert_eq!(context("du texte ordinaire ", 20), None);

        // Le déclencheur seul, et rien derrière : la liste s'ouvre sur un mot
        // vide, avec tout à proposer.
        assert_eq!(
            context("un mot puis #", 13),
            Some(Context {
                trigger: '#',
                typed: String::new(),
                start: 13
            })
        );

        // Une position au-delà de la fin est ramenée à la fin du texte.
        assert_eq!(
            context("#fig", 99),
            Some(Context {
                trigger: '#',
                typed: "fig".to_string(),
                start: 1
            })
        );

        // Le mot du début d'une ligne n'a pas de déclencheur devant lui.
        assert_eq!(context("figure puis #note", 6), None);
    }

    /// Le filtrage garde l'ordre du serveur, et met devant ce qui commence pareil.
    #[test]
    fn matching_keeps_the_server_order() {
        let items = vec![
            item("quad_figure", None),
            item("figure", None),
            item("figure.paren", None),
            item("table", None),
        ];

        assert_eq!(matching(&items, ""), vec![0, 1, 2, 3]);
        assert_eq!(matching(&items, "fig"), vec![1, 2, 0]);
        assert_eq!(matching(&items, "FIGURE"), vec![1, 2, 0]);
        assert_eq!(matching(&items, "tab"), vec![3]);
        assert!(matching(&items, "rien").is_empty());
    }

    /// Le filtre suit `filterText` quand le serveur en donne un.
    #[test]
    fn the_filter_text_wins_over_the_label() {
        let mut value = serde_json::json!({ "label": "Une étiquette", "filterText": "figure" });
        let item: CompletionItem = serde_json::from_value(value.clone()).expect("élément");

        assert_eq!(matching(&[item.clone()], "fig"), vec![0]);

        value["filterText"] = serde_json::Value::String("autre".to_string());
        let other: CompletionItem = serde_json::from_value(value).expect("élément");

        assert!(matching(&[other], "fig").is_empty());
        assert_eq!(matching(&[item], "fig"), vec![0]);
    }

    /// Un gabarit ne laisse aucun marqueur dans le document.
    #[test]
    fn a_template_loses_its_markers() {
        assert_eq!(expand("figure"), ("figure".to_string(), None));
        assert_eq!(expand("figure(${1:})"), ("figure()".to_string(), Some(7)));
        assert_eq!(expand("figure[${1:}]"), ("figure[]".to_string(), Some(7)));
        assert_eq!(
            expand("figure(${1:titre})"),
            ("figure(titre)".to_string(), Some(7))
        );
        assert_eq!(expand("a$1b"), ("ab".to_string(), Some(1)));
        assert_eq!(
            expand("${2:x} puis ${1:y}"),
            ("x puis y".to_string(), Some(0))
        );

        // Un marqueur échappé reste un caractère, et un `$` seul aussi.
        assert_eq!(expand("un \\$ dollar"), ("un $ dollar".to_string(), None));
        assert_eq!(expand("prix : 5$"), ("prix : 5$".to_string(), None));
    }

    /// Accepter remplace ce qui a été tapé, et pose le curseur.
    #[test]
    fn accepting_replaces_what_was_typed() {
        let figure = item("figure", Some("figure"));

        assert_eq!(
            insertion(&figure, "fig", None),
            Insertion {
                backspaces: 3,
                text: "figure".to_string(),
                caret: None
            }
        );

        // Le serveur peut demander une plage plus large que ce qu'on a compté.
        assert_eq!(insertion(&figure, "fig", Some(4)).backspaces, 4);

        let paren = item("figure.paren", Some("figure(${1:})"));

        assert_eq!(
            insertion(&paren, "fig", None),
            Insertion {
                backspaces: 3,
                text: "figure()".to_string(),
                caret: Some(7)
            }
        );

        // Un élément sans texte d'insertion se rabat sur son étiquette.
        let bare = item("#figure", None);

        assert_eq!(insertion(&bare, "fig", None).text, "#figure");
    }
}
