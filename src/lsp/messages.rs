//! Les types du protocole LSP dont nous avons besoin, et rien de plus.
//!
//! Décrits à la main plutôt que tirés d'une bibliothèque : nous n'en consommons
//! qu'une poignée, et les écrire ici, c'est aussi écrire leurs tests.

use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize};

/// Une position dans un document, telle que le protocole la définit.
///
/// `line` et `character` sont à base zéro, et le caractère se compte en **unités
/// de code UTF-16** — la convention du protocole. Notre code, lui, compte en
/// octets UTF-8 : la conversion vit dans [`position_of`] et [`offset_of`], et
/// c'est là que se nichent les erreurs d'accents, de caractères chinois et
/// d'émojis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Un intervalle de document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Gravité d'un diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Severity {
    fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Error,
            2 => Self::Warning,
            3 => Self::Information,
            _ => Self::Hint,
        }
    }
}

impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_code(u32::deserialize(deserializer)?))
    }
}

impl Serialize for Severity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let code = match self {
            Self::Error => 1,
            Self::Warning => 2,
            Self::Information => 3,
            Self::Hint => 4,
        };

        serializer.serialize_u32(code)
    }
}

/// Un diagnostic du serveur, positionné dans un document.
#[derive(Debug, Clone, Deserialize)]
pub struct Diagnostic {
    pub range: Range,

    #[serde(default)]
    pub severity: Option<Severity>,

    #[serde(default)]
    pub message: String,

    /// Identifiant du diagnostic (`E001`, par exemple), parfois un nombre.
    #[serde(default, deserialize_with = "string_or_number")]
    pub code: Option<String>,

    /// Qui l'a produit — `tinymist`, en pratique.
    #[serde(default)]
    pub source: Option<String>,
}

/// Catégorie d'un élément de complétion, telle que le serveur l'annonce.
///
/// Le protocole la transmet par un nombre dont nous ne consommons que
/// l'affichage : une icône et un mot, dans la liste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
    Unknown,
}

impl CompletionKind {
    fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Text,
            2 => Self::Method,
            3 => Self::Function,
            4 => Self::Constructor,
            5 => Self::Field,
            6 => Self::Variable,
            7 => Self::Class,
            8 => Self::Interface,
            9 => Self::Module,
            10 => Self::Property,
            11 => Self::Unit,
            12 => Self::Value,
            13 => Self::Enum,
            14 => Self::Keyword,
            15 => Self::Snippet,
            16 => Self::Color,
            17 => Self::File,
            18 => Self::Reference,
            19 => Self::Folder,
            20 => Self::EnumMember,
            21 => Self::Constant,
            22 => Self::Struct,
            23 => Self::Event,
            24 => Self::Operator,
            25 => Self::TypeParameter,
            _ => Self::Unknown,
        }
    }

    /// Le mot affiché à droite de l'étiquette, dans la liste.
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "texte",
            Self::Method => "méthode",
            Self::Function => "fonction",
            Self::Constructor => "constructeur",
            Self::Field => "champ",
            Self::Variable => "variable",
            Self::Class => "classe",
            Self::Interface => "interface",
            Self::Module => "module",
            Self::Property => "propriété",
            Self::Unit => "unité",
            Self::Value => "valeur",
            Self::Enum => "énumération",
            Self::Keyword => "mot-clé",
            Self::Snippet => "gabarit",
            Self::Color => "couleur",
            Self::File => "fichier",
            Self::Reference => "référence",
            Self::Folder => "dossier",
            Self::EnumMember => "membre",
            Self::Constant => "constante",
            Self::Struct => "structure",
            Self::Event => "événement",
            Self::Operator => "opérateur",
            Self::TypeParameter => "paramètre",
            Self::Unknown => "",
        }
    }
}

impl<'de> Deserialize<'de> for CompletionKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_code(u32::deserialize(deserializer)?))
    }
}

/// La documentation d'un élément : le protocole autorise une chaîne ou un objet.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Documentation {
    Text(String),
    Markup { value: String },
}

impl Documentation {
    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) => text,
            Self::Markup { value } => value,
        }
    }
}

/// Le remplacement qu'un élément de complétion demande.
///
/// C'est **la** raison pour laquelle une étiquette ne suffit pas : elle ne dit
/// pas quoi remplacer.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Un élément de complétion, tel que tinymist le transmet.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,

    #[serde(default)]
    pub kind: Option<CompletionKind>,

    /// Ce qui sera inséré — [`Self::text_edit`] et `insert_text` ne viennent
    /// jamais tous les deux.
    #[serde(default)]
    pub insert_text: Option<String>,

    #[serde(default)]
    pub text_edit: Option<TextEdit>,

    #[serde(default)]
    pub detail: Option<String>,

    #[serde(default)]
    pub documentation: Option<Documentation>,

    /// Le texte sur lequel filtrer quand on continue de taper.
    #[serde(default)]
    pub filter_text: Option<String>,

    /// L'ordre voulu par le serveur : il trie mieux que nous, et il le dit.
    #[serde(default)]
    pub sort_text: Option<String>,
}

impl CompletionItem {
    /// Le texte à insérer, s'il y en a un.
    pub fn insertion(&self) -> Option<&str> {
        self.text_edit
            .as_ref()
            .map(|edit| edit.new_text.as_str())
            .or(self.insert_text.as_deref())
    }

    /// La plage à remplacer, si l'élément la précise.
    pub fn replacement(&self) -> Option<Range> {
        self.text_edit.as_ref().map(|edit| edit.range)
    }

    /// Le texte sur lequel filtrer.
    pub fn filter(&self) -> &str {
        self.filter_text.as_deref().unwrap_or(&self.label)
    }
}

/// Combien de diagnostics, et combien d'erreurs parmi eux.
///
/// C'est ce qu'une barre d'état résume. Le tri se fait ici, une fois, plutôt que
/// dans la vue : c'est une donnée que l'on affiche, donc une donnée que l'on
/// éprouve.
pub fn counts(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Some(Severity::Error))
        .count();

    (errors, diagnostics.len())
}

/// Une plage désigne une chaîne, un nombre ou rien : on n'en garde que le texte.
fn string_or_number<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    use serde_json::Value;

    let value = Option::<Value>::deserialize(deserializer)?;

    Ok(match value {
        Some(Value::String(text)) => Some(text),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    })
}

/// La position d'un décalage en octets dans un document.
///
/// Renvoie une position du protocole : ligne à base zéro, et caractère compté en
/// unités de code UTF-16. Le décalage est ramené à une frontière de caractère
/// s'il tombe au milieu d'un caractère — ce qu'un décalage d'octets peut
/// toujours faire.
pub fn position_of(text: &str, byte_offset: usize) -> Position {
    let mut end = byte_offset.min(text.len());

    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }

    let before = &text[..end];

    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    let character = before[line_start..].encode_utf16().count() as u32;

    Position { line, character }
}

/// Le décalage en octets d'une position du protocole.
///
/// Une position au-delà de la fin du document est ramenée à la fin : un serveur
/// qui parle d'une version du texte que nous n'avons pas encore ne doit pas
/// pouvoir nous faire sortir du document.
pub fn offset_of(text: &str, position: Position) -> usize {
    let mut line_start = 0usize;

    for _ in 0..position.line {
        match text[line_start..].find('\n') {
            Some(index) => line_start += index + 1,
            None => return text.len(),
        }
    }

    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |index| line_start + index);

    let mut units = 0u32;

    for (index, character) in text[line_start..line_end].char_indices() {
        if units >= position.character {
            return line_start + index;
        }

        units += character.len_utf16() as u32;
    }

    line_end
}

/// La colonne, en caractères, d'une position du protocole.
///
/// Le pas de plus que `offset_of` ne fait pas : l'éditeur compte ses colonnes en
/// **caractères**, le protocole en unités UTF-16, et une ligne accentuée les fait
/// diverger. C'est cette colonne-là qu'un saut doit donner à l'éditeur.
pub fn column_in_chars(text: &str, position: Position) -> usize {
    let offset = offset_of(text, position);
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);

    text[line_start..offset].chars().count()
}

/// Un diagnostic prêt à montrer : son libellé, et l'endroit où sauter.
///
/// Le libellé demande le **texte** du document : la colonne du protocole compte
/// en unités UTF-16, l'éditeur en caractères. Cette conversion se fait donc à
/// l'arrivée du diagnostic — une fois — et non à chaque image, où elle ferait
/// cloner le document entier soixante fois par seconde.
#[derive(Debug, Clone)]
pub struct Reported {
    pub label: String,
    pub line: usize,
    pub column: usize,
    pub severity: Option<Severity>,
}

/// Traduit les diagnostics du serveur pour l'affichage.
///
/// Le saut est toujours proposé : le client ne garde que les diagnostics du
/// document ouvert, le seul où l'éditeur puisse poser le curseur.
pub fn report(text: &str, items: &[Diagnostic]) -> Vec<Reported> {
    items
        .iter()
        .map(|diagnostic| {
            let position = diagnostic.range.start;
            let line = position.line as usize + 1;
            let column = column_in_chars(text, position) + 1;

            Reported {
                label: format!("ligne {line}, colonne {column} — {}", diagnostic.message),
                line,
                column,
                severity: diagnostic.severity,
            }
        })
        .collect()
}

/// Les modifications à apporter à un fichier.
#[derive(Debug, Clone)]
pub struct FileEdit {
    pub path: PathBuf,
    pub edits: Vec<TextEdit>,
}

/// Applique des remplacements à un texte.
///
/// Les remplacements se lisent **de la fin vers le début** : chaque application
/// déplace tout ce qui suit, et c'est à rebours que les positions du serveur
/// restent justes.
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut points: Vec<(usize, usize, String)> = Vec::new();

    for edit in edits {
        let debut = offset_of(text, edit.range.start);
        let fin = offset_of(text, edit.range.end);

        if debut <= fin {
            points.push((debut, fin, edit.new_text.clone()));
        }
    }

    points.sort_by_key(|(debut, fin, _)| (*debut, *fin));

    let mut out = text.to_string();

    for (debut, fin, nouveau) in points.iter().rev() {
        out.replace_range(*debut..*fin, nouveau);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Appliquer des remplacements ne déplace jamais ce qui les suit.
    #[test]
    fn edits_are_applied_from_the_end_so_the_others_stay_put() {
        let position = |line, character| Position { line, character };
        let edit = |start: (u32, u32), end: (u32, u32), new_text: &str| TextEdit {
            range: Range {
                start: position(start.0, start.1),
                end: position(end.0, end.1),
            },
            new_text: new_text.to_string(),
        };

        // « le chat dort » : deux remplacements, dont l'un déplacerait l'autre
        // s'ils se lisaient de la fin vers le début.
        let texte = "le chat dort\n";
        let edits = vec![edit((0, 3), (0, 7), "chien"), edit((0, 12), (0, 12), " !")];

        assert_eq!(apply_edits(texte, &edits), "le chien dort !\n");

        // Une insertion et une suppression, sur deux lignes.
        let texte = "un\ndeu\n";
        let edits = vec![edit((0, 2), (0, 2), "x"), edit((1, 0), (1, 3), "DEUX")];

        assert_eq!(apply_edits(texte, &edits), "unx\nDEUX\n");
    }

    /// Un émoji vaut deux unités UTF-16 et un seul caractère : la colonne du
    /// protocole ne peut pas être donnée telle quelle à l'éditeur.
    #[test]
    fn the_report_column_counts_characters_not_utf16_units() {
        let text = "a😀b";
        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 3,
                },
                end: Position {
                    line: 0,
                    character: 4,
                },
            },
            severity: Some(Severity::Error),
            message: "un problème".to_string(),
            code: None,
            source: None,
        };

        let rows = report(text, std::slice::from_ref(&diagnostic));

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].line, 1, "la ligne du protocole est 0-based");
        assert_eq!(
            rows[0].column, 3,
            "3 unités UTF-16 après « a😀 » = 2 caractères, donc la colonne 3"
        );
        assert!(rows[0].label.contains("ligne 1, colonne 3"));
        assert!(rows[0].label.contains("un problème"));
    }

    /// Une colonne accentuée : UTF-16 et caractères ne comptent pas pareil.
    #[test]
    fn a_column_is_counted_in_characters() {
        let text = "a\u{1D11E}b\néé";

        // Sur la première ligne : `a`, puis un caractère hors du plan de base, qui
        // vaut **deux** unités UTF-16 et un seul caractère.
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 0,
                    character: 0
                }
            ),
            0
        );
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 0,
                    character: 1
                }
            ),
            1
        );
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 0,
                    character: 3
                }
            ),
            2
        );
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 0,
                    character: 4
                }
            ),
            3
        );

        // Au-delà de la ligne, la colonne se ramène à sa longueur.
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 1,
                    character: 9
                }
            ),
            2
        );

        // Ligne inexistante : ramenée à la fin du document, comme `offset_of`,
        // et sans paniquer — un serveur peut parler d'un texte qu'on n'a pas.
        assert_eq!(
            column_in_chars(
                text,
                Position {
                    line: 9,
                    character: 0
                }
            ),
            2
        );
    }

    /// Un aller-retour ne doit rien déplacer, même avec des accents, des
    /// caractères chinois et des émojis — c'est exactement là que se trompe une
    /// conversion en UTF-16 écrite à la légère.
    #[test]
    fn positions_survive_every_alphabet() {
        let text = "Un été à Genève\n中文字符\nUn émoji : 🎉 et la suite\n";

        for (offset, _) in text.char_indices() {
            let position = position_of(text, offset);

            assert_eq!(
                offset_of(text, position),
                offset,
                "décalage {offset} perdu au passage par {position:?}"
            );
        }

        assert_eq!(offset_of(text, position_of(text, text.len())), text.len());
    }

    /// Le caractère se compte en unités UTF-16 : un émoji en vaut deux.
    #[test]
    fn a_character_is_a_utf16_unit() {
        let text = "🎉x";

        assert_eq!(
            position_of(text, 4),
            Position {
                line: 0,
                character: 2
            },
            "un émoji occupe deux unités de code"
        );
        assert_eq!(position_of(text, 0).character, 0);
        assert_eq!(
            offset_of(
                text,
                Position {
                    line: 0,
                    character: 2
                }
            ),
            4
        );
    }

    /// Les lignes se comptent à partir de zéro, le premier caractère aussi.
    #[test]
    fn lines_and_characters_start_at_zero() {
        let text = "première\ndeuxième\ntroisième";

        assert_eq!(
            position_of(text, 0),
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            position_of(text, text.find("deuxième").unwrap()),
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(
            position_of(text, text.len()),
            Position {
                line: 2,
                character: 9
            }
        );
    }

    /// Le décalage d'un retour à la ligne appartient à la ligne qu'il termine.
    #[test]
    fn a_line_break_belongs_to_the_line_it_ends() {
        let text = "a\nb";

        assert_eq!(
            offset_of(
                text,
                Position {
                    line: 0,
                    character: 1
                }
            ),
            1
        );
        assert_eq!(
            offset_of(
                text,
                Position {
                    line: 1,
                    character: 0
                }
            ),
            2
        );
    }

    /// Une position hors du document est ramenée dans le document.
    #[test]
    fn a_position_beyond_the_end_lands_on_the_end() {
        let text = "court";

        assert_eq!(
            offset_of(
                text,
                Position {
                    line: 0,
                    character: 99
                }
            ),
            text.len()
        );
        assert_eq!(
            offset_of(
                text,
                Position {
                    line: 9,
                    character: 0
                }
            ),
            text.len()
        );
        assert_eq!(
            offset_of(
                "",
                Position {
                    line: 0,
                    character: 3
                }
            ),
            0
        );
    }

    /// Le résumé compte les erreurs, et sépare les autres.
    #[test]
    fn the_summary_counts_errors_apart() {
        let diagnostics: Vec<Diagnostic> = serde_json::from_value(serde_json::json!([
            { "message": "une erreur", "severity": 1,
              "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } } },
            { "message": "un avertissement", "severity": 2,
              "range": { "start": { "line": 1, "character": 0 }, "end": { "line": 1, "character": 1 } } },
            { "message": "sans gravité déclarée",
              "range": { "start": { "line": 2, "character": 0 }, "end": { "line": 2, "character": 1 } } },
        ]))
        .expect("diagnostics");

        assert_eq!(counts(&diagnostics), (1, 3));
        assert_eq!(counts(&[]), (0, 0));
    }

    /// Un décalage au milieu d'un caractère est ramené à la frontière.
    #[test]
    fn an_offset_inside_a_character_moves_back() {
        let text = "é";

        assert_eq!(
            position_of(text, 1),
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            position_of(text, 2),
            Position {
                line: 0,
                character: 1
            }
        );
    }
}
