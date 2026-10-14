use iced::Task;
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::path::PathBuf;

pub struct EditorState {
    pub editor: CodeEditor,
    pub path: Option<PathBuf>,
}

/// Taille du texte de rédaction.
///
/// Plus grande que les 13-14 px d'un éditeur de code : on lit des phrases, pas
/// des identifiants.
pub const BODY_SIZE: f32 = 17.0;

/// Nom de syntaxe passé à l'éditeur.
///
/// C'est **une extension de fichier**, pas un nom de langage : la résolution du
/// widget cherche dans les extensions déclarées par les définitions de syntaxe,
/// et Typst y répond à `typ`. Passer `"typst"` ne trouverait rien et
/// l'éditeur resterait en texte brut.
///
/// La définition vient du jeu embarqué par `two-face` (celui de `bat`), d'où la
/// feature du même nom sur la dépendance : le jeu par défaut de `syntect`, lui,
/// ne contient pas Typst.
pub const SYNTAX: &str = "typ";

/// Habille l'éditeur selon un thème résolu.
///
/// Trois choses dépendent du thème : les couleurs, les polices, et la palette de
/// coloration syntaxique. Elles sont posées ensemble, en un seul endroit, pour
/// qu'aucune ne puisse être oubliée quand on en ajoute une.
fn apply_look(editor: &mut iced_code_editor::CodeEditor, resolved: &crate::ui::theme::Resolved) {
    match &resolved.editor {
        // Un thème de fichier décrit précisément l'éditeur.
        Some(look) => {
            editor.set_theme(look.style.clone());
            editor.set_syntax_palette(Some(look.syntax.clone()));
        }
        // Un thème natif délègue au widget, comme avant les thèmes.
        //
        // Le fond est alors repris du panneau : c'est ce qui fait apparaître la
        // zone de rédaction comme une page, et donc ses marges. Sur un fond
        // uniforme, une marge existe en géométrie mais ne se voit pas.
        None => {
            let mut style = iced_code_editor::from_iced_theme(&resolved.iced);

            style.background = resolved.palette.surface;

            editor.set_theme(style);
            editor.set_syntax_palette(None);
        }
    }

    let typography = resolved.typography;

    // La police se pose **après** les couleurs : changer de police remesure la
    // largeur des caractères, et le faire à chaque changement de thème une
    // seule fois suffit.
    editor.set_font(typography.editor);
    editor.set_font_size(typography.editor_size, false);
    // `set_line_height` attend une hauteur **en pixels**, pas un multiple de la
    // taille du texte : la multiplier ici est ce qui rattache les deux unités.
    editor.set_line_height(typography.editor_size * typography.line_height);
}

/// Interligne, en multiple de la taille du texte.
///
/// 1,6 est l'intervalle où un paragraphe reste lisible sans que l'œil perde la
/// ligne en revenant à gauche.
const LINE_HEIGHT: f32 = 1.6;

impl Default for EditorState {
    fn default() -> Self {
        let mut state = Self {
            editor: CodeEditor::new("", SYNTAX),
            path: None,
        };

        state.apply_prose_profile();

        state
    }
}

impl EditorState {
    pub fn new_file(path: PathBuf, content: String) -> Self {
        let mut state = Self {
            editor: CodeEditor::new(&content, SYNTAX),
            path: Some(path),
        };

        state.apply_prose_profile();

        state
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        self.editor.update(&message)
    }

    pub fn filename(&self) -> &str {
        self.path
            .as_deref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Sans titre")
    }

    pub fn content(&self) -> String {
        self.editor.content()
    }

    pub fn is_modified(&self) -> bool {
        self.editor.is_modified()
    }

    /// Insère du texte à la position du curseur.
    ///
    /// Le mécanisme est le message `Paste` de l'éditeur — c'est son insertion
    /// documentée au curseur (utilisée par la palette de commandes), pas un
    /// accès au presse-papiers système.
    pub fn insert_at_cursor(&mut self, text: &str) -> Task<EditorMessage> {
        self.editor.update(&EditorMessage::Paste(text.to_string()))
    }

    /// Écrit le contenu courant sur le disque.
    ///
    /// Renvoie `Ok(())` si l'écriture a réussi, l'erreur sinon.
    pub fn save(&mut self) -> std::io::Result<()> {
        let Some(path) = self.path.clone() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "aucun chemin de fichier",
            ));
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::write(&path, self.content())?;

        self.editor.mark_saved();

        Ok(())
    }

    /// Configure l'éditeur pour de la rédaction, pas du code.
    ///
    /// C'est le réglage qui compte le plus pour le confort d'écriture : le
    /// widget est un éditeur de code, et tout ce qui le signale comme tel est
    /// du bruit dans un texte.
    ///
    /// À appeler à la création de l'éditeur. Les **couleurs et les polices**
    /// n'en font pas partie : elles viennent de [`Self::retheme`], et les mêler
    /// ici obligerait à rappeler tout le profil à chaque changement de thème.
    pub fn apply_prose_profile(&mut self) {
        let editor = &mut self.editor;

        // Le point litigieux : l'éditeur traite l'apostrophe comme une paire
        // auto-fermante, si bien que taper « l' » produisait « l'' » et qu'il
        // fallait ensuite franchir le second apostrophe. En français, c'est
        // plusieurs fois par phrase. Le réglage du widget est tout-ou-rien —
        // il couvre aussi les parenthèses et les crochets — mais en prose le
        // jeu n'en vaut pas la chandelle : on désactive l'ensemble.
        editor.set_auto_close_brackets(false);

        // Repères de code : inutiles dans un texte, et visuellement bruyants.
        editor.set_line_numbers_enabled(false);
        editor.set_bracket_match_highlight_enabled(false);
        editor.set_bracket_pair_colorization_enabled(false);
        editor.set_show_indent_guides(false);
        editor.set_show_whitespace(false);
        editor.set_show_color_previews(false);

        // Structures de blocs : une prose n'a pas de fonction à replier, et
        // l'indentation automatique surprend quand on revient à la ligne au
        // milieu d'une phrase.
        editor.set_folding_enabled(false);
        editor.set_sticky_scroll_enabled(false);
        editor.set_auto_indent_enabled(false);

        // Le retour à la ligne, en revanche, est indispensable : un paragraphe
        // ne doit pas défiler horizontalement.
        editor.set_wrap_enabled(true);

        // Confort de lecture, en attendant que `retheme` pose la typographie du
        // thème : ces valeurs sont celles du thème natif.
        editor.set_font_size(BODY_SIZE, false);
        editor.set_line_height(BODY_SIZE * LINE_HEIGHT);
    }

    /// Réapplique les couleurs après un changement de thème.
    pub fn retheme(&mut self, resolved: &crate::ui::theme::Resolved) {
        apply_look(&mut self.editor, resolved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'apostrophe ne doit plus être appairée.
    ///
    /// C'est le réglage qui gênait le plus à l'usage : taper « l' » produisait
    /// « l'' », plusieurs fois par phrase en français. Le widget ne permet pas
    /// de ne désactiver que l'apostrophe — le réglage couvre aussi les
    /// crochets — mais en prose le compromis est le bon.
    #[test]
    fn apostrophes_are_not_auto_paired() {
        let editor = EditorState::default();

        assert!(
            !editor.editor.auto_close_brackets(),
            "l'auto-pairage doit être désactivé"
        );
    }

    /// Les repères de code sont retirés : dans un texte, ils sont du bruit.
    #[test]
    fn code_chrome_is_removed() {
        let editor = EditorState::default();
        let inner = &editor.editor;

        assert!(!inner.line_numbers_enabled(), "pas de numéros de ligne");
        assert!(
            !inner.bracket_match_highlight_enabled(),
            "pas de surbrillance de paire"
        );
        assert!(
            !inner.bracket_pair_colorization_enabled(),
            "pas de crochets arc-en-ciel"
        );
        assert!(!inner.show_indent_guides(), "pas de guides d'indentation");
        assert!(!inner.show_whitespace(), "pas de marqueurs d'espaces");
        assert!(!inner.folding_enabled(), "pas de repliage de blocs");
        assert!(!inner.sticky_scroll_enabled(), "pas d'en-têtes épinglés");
        assert!(
            !inner.auto_indent_enabled(),
            "pas d'indentation automatique"
        );
    }

    /// Le retour à la ligne, en revanche, reste : un paragraphe ne doit jamais
    /// défiler horizontalement.
    #[test]
    fn word_wrapping_stays_on() {
        let editor = EditorState::default();

        assert!(
            editor.editor.wrap_enabled(),
            "le retour à la ligne est requis"
        );
    }

    /// La coloration Typst est réellement active.
    ///
    /// `syntax_name` renvoie le nom de la définition effectivement résolue :
    /// « Typst » si la syntaxe a été trouvée, « Plain Text » sinon. C'est ce
    /// qui distingue une coloration qui marche d'un repli silencieux sur du
    /// texte brut — les deux ayant exactement la même allure de code.
    #[test]
    fn typst_highlighting_is_actually_resolved() {
        let editor = EditorState::default();

        let resolved = editor.editor.syntax_name();

        assert_eq!(
            resolved, "Typst",
            "la syntaxe Typst doit être résolue, pas remplacée par du texte brut"
        );
    }

    /// Le nom de syntaxe doit être une extension connue des définitions.
    ///
    /// L'éditeur résout par extension de fichier : « typst » ne trouverait
    /// rien et retomberait sur du texte brut sans le dire.
    #[test]
    fn the_syntax_name_matches_a_declared_extension() {
        let editor = EditorState::default();

        assert_eq!(editor.editor.syntax(), SYNTAX);
        assert_ne!(
            editor.editor.syntax_name(),
            "Plain Text",
            "« {SYNTAX} » doit correspondre à une définition, pas au repli brut"
        );
    }

    /// Le confort de lecture : texte plus grand et interligne plus ouvert
    /// qu'un éditeur de code.
    #[test]
    fn reading_comfort_is_tuned() {
        let editor = EditorState::default();

        assert_eq!(editor.editor.font_size(), BODY_SIZE);
        assert_eq!(editor.editor.line_height(), BODY_SIZE * LINE_HEIGHT);
        assert!(
            editor.editor.line_height() > editor.editor.font_size(),
            "l'interligne doit dépasser la taille du texte"
        );
    }

    /// Le profil est appliqué à chaque ouverture de document, pas seulement à
    /// l'état par défaut : sinon rouvrir un fichier ferait revenir le bruit.
    #[test]
    fn profile_applies_to_opened_documents() {
        let path = std::env::temp_dir().join("typst-writer-prose-test.typ");
        let editor = EditorState::new_file(path, "= Titre\n\nDu texte.\n".to_string());

        assert!(!editor.editor.auto_close_brackets());
        assert!(!editor.editor.line_numbers_enabled());
        assert!(editor.editor.wrap_enabled());
        assert_eq!(editor.editor.font_size(), BODY_SIZE);
    }
}
