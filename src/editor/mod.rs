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
const BODY_SIZE: f32 = 17.0;

/// Interligne, en multiple de la taille du texte.
///
/// 1,6 est l'intervalle où un paragraphe reste lisible sans que l'œil perde la
/// ligne en revenant à gauche.
const LINE_HEIGHT: f32 = 1.6;

impl Default for EditorState {
    fn default() -> Self {
        let mut state = Self {
            editor: CodeEditor::new(crate::project::DOCUMENT_TEMPLATE, "typst"),
            path: None,
        };

        state.apply_prose_profile(&iced::Theme::Dark);

        state
    }
}

impl EditorState {
    pub fn new_file(path: PathBuf, content: String) -> Self {
        let mut state = Self {
            editor: CodeEditor::new(&content, "typst"),
            path: Some(path),
        };

        state.apply_prose_profile(&iced::Theme::Dark);

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
    /// À appeler à la création de l'éditeur, puis à chaque changement de thème
    /// puisque les couleurs en dépendent.
    pub fn apply_prose_profile(&mut self, theme: &iced::Theme) {
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

        // Confort de lecture.
        editor.set_font_size(BODY_SIZE, false);
        editor.set_line_height(BODY_SIZE * LINE_HEIGHT);

        // Couleurs déduites du thème de l'application : l'éditeur suit donc le
        // clair et le sombre sans réglage supplémentaire.
        editor.set_theme(iced_code_editor::from_iced_theme(theme));
    }

    /// Réapplique les couleurs après un changement de thème.
    pub fn retheme(&mut self, theme: &iced::Theme) {
        self.editor
            .set_theme(iced_code_editor::from_iced_theme(theme));
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
