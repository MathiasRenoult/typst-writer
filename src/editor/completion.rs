//! La liste de complétion : ce qui la déclenche, ce qu'elle montre, ce qu'elle
//! écrit.
//!
//! Tout ce qui décide vit ici, et rien n'y touche à l'interface : ce sont des
//! fonctions pures, donc éprouvables, et c'est là que se cachent les erreurs qui
//! coûtent — le mot qu'on croyait remplacer, le gabarit inséré tel quel.

/// Un élément de complétion : ce que l'application propose d'elle-même —
/// une clé citable, un terme du glossaire, un gabarit. Le monde LSP
/// est retiré (feuille de route, phase 1) : la complétion locale garde
/// ses fonctions pures, et son type est à elle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Ce qui se lit dans la liste.
    pub label: String,

    /// Ce qui s'écrit au curseur — par défaut, le libellé.
    pub insert: Option<String>,

    /// Le texte sur lequel filtrer quand on continue de taper — par
    /// défaut, le libellé.
    pub filtre: Option<String>,
}

impl Element {
    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// Ce qui s'insère, gabarits compris.
    pub fn insertion(&self) -> Option<&str> {
        self.insert.as_deref()
    }

    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// Le texte de filtre : le libellé, sauf choix contraire.
    pub fn filter(&self) -> &str {
        self.filtre.as_deref().unwrap_or(&self.label)
    }
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Les caractères qui, tapés, demandent la liste.
///
/// Ce sont ceux que le serveur annonce comme déclencheurs : `#` ouvre ses
/// commandes, `@` ses citations. L'application ne devine pas — elle reprend ce
/// que le serveur a dit savoir faire.
pub const TRIGGERS: [char; 2] = ['#', '@'];

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Le nom d'une référence, où que soit le curseur dans le mot.
///
/// C'est la différence avec `context` : celui-ci ne regarde que ce qui a été
/// **tapé**, car c'est ce qu'il faut pour filtrer une liste. Mais on appelle un
/// aller à la définition depuis un mot déjà écrit — souvent cliqué au milieu —,
/// et il faut alors le nom entier.
pub fn reference(text: &str, offset: usize) -> Option<String> {
    let offset = offset.min(text.len());

    let start = text[..offset]
        .rfind(|character: char| !is_reference(character))
        .map_or(0, |index| index + 1);

    let end = text[offset..]
        .find(|character: char| !is_reference(character))
        .map_or(text.len(), |index| offset + index);

    let word = &text[start..end];

    // Le `@` n'est dans le mot que si le curseur est posé dessus ; partout
    // ailleurs il est **juste avant**, et c'est lui qui fait d'un mot une
    // référence.
    if let Some(name) = word.strip_prefix(CITATION) {
        return Some(name.to_string());
    }

    text[..start].ends_with(CITATION).then(|| word.to_string())
}

/// Ce qui peut composer un nom de référence.
///
/// Pas le point : une phrase se termine souvent juste après une référence, et
/// « @intro. » doit donner « intro ». Une étiquette qui contient vraiment un
/// point est rare, et le serveur reste là pour ces cas-là.
fn is_reference(character: char) -> bool {
    character.is_alphanumeric() || "_:-".contains(character)
}

/// Le caractère qui appelle une citation.
pub const CITATION: char = '@';

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
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

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
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

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Un caractère peut-il appartenir à un nom de commande ou à une clé ?
fn is_word(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
}

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Les éléments qui correspondent à ce qui a été tapé, dans l'ordre du serveur.
///
/// L'ordre est **celui du serveur** : il trie mieux que nous, et il le dit par
/// son `sortText`. On ne fait que retirer ce qui ne correspond pas.
///
/// Deux passes, parce que c'est ce que la main attend : d'abord ce qui *commence*
/// par ce qu'on a tapé, puis ce qui le *contient*. « figure » doit passer devant
/// « quad_figure ».
pub fn matching(items: &[Element], typed: &str) -> Vec<usize> {
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

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
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

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
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

#[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
/// Ce qu'il faut faire pour accepter un élément.
///
/// `typed` est ce qui a été tapé après le déclencheur, et `replacement` la
/// longueur que le serveur demande de remplacer — le plus souvent la longueur de
/// `typed`, mais pas toujours.
pub fn insertion(item: &Element, typed: &str, replacement: Option<usize>) -> Insertion {
    let (text, caret) = expand(item.insertion().unwrap_or(&item.label));

    Insertion {
        backspaces: replacement.unwrap_or_else(|| typed.chars().count()),
        text,
        caret,
    }
}

/// La liste de complétion, telle qu'elle s'affiche.
pub struct Completion {
    /// Ce que le serveur a proposé.
    pub items: Vec<crate::editor::completion::Element>,

    /// Les indices des éléments qui correspondent à ce qui est tapé, dans
    /// l'ordre voulu par le serveur.
    pub shown: Vec<usize>,

    /// La sélection, dans [`Self::shown`].
    pub selected: usize,

    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// Ce qui a été tapé après le déclencheur.
    pub typed: String,

    /// Le caractère qui a ouvert la liste.
    pub trigger: char,

    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// Le décalage, en octets, du début de ce qui serait remplacé.
    pub start: usize,

    /// Position source du curseur à la dernière mise à jour de la liste.
    pub cursor_offset: usize,
}

impl Completion {
    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// Ferme la liste si le curseur a été déplacé sans modifier le texte.
    pub fn should_close_after(&self, cursor_offset: usize, content_changed: bool) -> bool {
        !content_changed && cursor_offset != self.cursor_offset
    }

    #[allow(dead_code)] // Rebranché par la complétion locale (feuille de route).
    /// L'élément sélectionné, s'il y en a un.
    pub fn selected_item(&self) -> Option<&crate::editor::completion::Element> {
        self.shown
            .get(self.selected)
            .and_then(|index| self.items.get(*index))
    }

    /// Déplace la sélection, en boucle : on ne bute pas sur les extrémités.
    pub fn move_selection(&mut self, delta: isize) {
        if self.shown.is_empty() {
            return;
        }

        let count = self.shown.len() as isize;
        let wanted = (self.selected as isize + delta).rem_euclid(count);

        self.selected = wanted as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(label: &str, insert: Option<&str>) -> Element {
        Element {
            label: label.to_string(),
            insert: insert.map(str::to_string),
            filtre: None,
        }
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

    /// Le filtre suit le texte de filtre quand il en donne un.
    #[test]
    fn le_filtre_l_emporte_sur_le_libelle() {
        let item = Element {
            label: "Une étiquette".to_string(),
            insert: None,
            filtre: Some("figure".to_string()),
        };
        assert_eq!(matching(std::slice::from_ref(&item), "fig"), vec![0]);

        let autre = Element {
            filtre: Some("autre".to_string()),
            ..item.clone()
        };
        assert!(matching(&[autre], "fig").is_empty());
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

    /// Une référence se lit entière, même si le curseur est au milieu — et sans
    /// emporter la ponctuation qui la suit.
    #[test]
    fn a_reference_is_read_whole() {
        let text = "Voir @intro pour commencer, et @fig-reseau.";

        for offset in [6, 8, 11] {
            assert_eq!(reference(text, offset), Some("intro".to_string()));
        }

        assert_eq!(reference(text, 40), Some("fig-reseau".to_string()));
        assert_eq!(
            reference(text, text.len() - 1),
            Some("fig-reseau".to_string())
        );

        // Ce qui n'est pas une référence n'en donne pas.
        assert_eq!(reference("un mot simple", 5), None);
        assert_eq!(reference("", 0), None);
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
