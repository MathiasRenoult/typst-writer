use iced::Task;
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::path::PathBuf;

pub struct EditorState {
    pub editor: CodeEditor,
    pub path: Option<PathBuf>,

    /// AST courant du fichier `.tw`, recalculé à chaque modification du source.
    pub document: Option<crate::langage::Document>,
}

/// Taille du texte de rédaction.
///
/// Plus grande que les 13-14 px d'un éditeur de code : on lit des phrases, pas
/// des identifiants.
pub const BODY_SIZE: f32 = 17.0;

/// Extension de grammaire utilisée pour les fichiers Typst hérités.
///
/// Le résolveur de `iced-code-editor` attend une extension de fichier, pas un
/// nom de langage : `typ` est déclarée par le jeu embarqué via `two-face`.
pub const SYNTAX: &str = "typ";

/// Extension de coloration Markdown utilisée par `document.tw`.
pub const MARKDOWN_SYNTAX: &str = "md";

fn syntax_for_path(path: &std::path::Path) -> &'static str {
    if path.extension().is_some_and(|extension| extension == "tw") {
        MARKDOWN_SYNTAX
    } else {
        SYNTAX
    }
}

/// Habille l'éditeur selon un thème résolu.
///
/// Trois choses dépendent du thème : les couleurs, les polices, et la palette de
/// coloration syntaxique. Elles sont posées ensemble, en un seul endroit, pour
/// qu'aucune ne puisse être oubliée quand on en ajoute une.
fn apply_look(editor: &mut iced_code_editor::CodeEditor, resolved: &crate::ui::theme::Resolved) {
    match &resolved.editor {
        // Un thème de fichier décrit précisément l'éditeur.
        Some(look) => {
            editor.set_theme(look.style);
            editor.set_syntax_palette(Some(look.syntax.clone()));
        }
        // Un thème natif délègue au widget, comme avant les thèmes.
        //
        // Le fond est alors repris du panneau : c'est ce qui fait apparaître la
        // zone de rédaction comme une page, et donc ses marges. Sur un fond
        // uniforme, une marge existe en géométrie mais ne se voit pas.
        None => {
            let mut style = iced_code_editor::from_iced_theme(&resolved.iced);

            style.background = resolved.palette.background;

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
            document: None,
        };

        state.apply_prose_profile();

        state
    }
}

impl EditorState {
    pub fn new_file(path: PathBuf, content: String) -> Self {
        let document = path
            .extension()
            .is_some_and(|extension| extension == "tw")
            .then(|| crate::langage::parse(&content));
        let syntax = syntax_for_path(&path);
        let mut state = Self {
            editor: CodeEditor::new(&content, syntax),
            path: Some(path),
            document,
        };

        state.apply_prose_profile();
        if state.is_intermediate() {
            state.apply_markdown_profile();
        }

        state
    }

    /// Le fichier courant utilise-t-il le langage intermédiaire ?
    pub fn is_intermediate(&self) -> bool {
        self.path
            .as_deref()
            .and_then(|path| path.extension())
            .is_some_and(|extension| extension == "tw")
    }

    /// Change le chemin d'un document sauvegardé sous un autre nom.
    pub fn set_path(&mut self, path: PathBuf) {
        let syntax = syntax_for_path(&path);
        self.editor.set_syntax(syntax);
        self.path = Some(path);
        if self.is_intermediate() {
            self.apply_markdown_profile();
        } else {
            self.editor.set_auto_close_brackets(false);
        }
        self.refresh_document();
    }

    /// Met à jour l'AST après une modification du source sérialisé.
    fn refresh_document(&mut self) {
        self.document = self
            .is_intermediate()
            .then(|| crate::langage::parse(&self.editor.content()));
    }

    /// Le message peut-il modifier le texte ou sa sérialisation ?
    pub fn message_may_change_content(message: &EditorMessage) -> bool {
        matches!(
            message,
            EditorMessage::CharacterInput(_)
                | EditorMessage::VimKey(_)
                | EditorMessage::Backspace
                | EditorMessage::Delete
                | EditorMessage::Enter
                | EditorMessage::Tab
                | EditorMessage::Cut
                | EditorMessage::Paste(_)
                | EditorMessage::DeleteSelection
                | EditorMessage::Undo
                | EditorMessage::Redo
                | EditorMessage::ImeCommit(_)
                | EditorMessage::ReplaceNext
                | EditorMessage::ReplaceAll
                | EditorMessage::ToggleComment
                | EditorMessage::MoveLineUp
                | EditorMessage::MoveLineDown
                | EditorMessage::DuplicateLineUp
                | EditorMessage::DuplicateLineDown
        )
    }

    pub fn update(&mut self, message: EditorMessage) -> Task<EditorMessage> {
        if self.is_intermediate() && matches!(&message, EditorMessage::Enter) {
            return self.markdown_enter();
        }

        let content_may_change = Self::message_may_change_content(&message);
        let task = self.editor.update(&message);

        if content_may_change {
            self.refresh_document();
        }

        task
    }

    /// Poursuit la structure Markdown du bloc courant au retour à la ligne.
    fn markdown_enter(&mut self) -> Task<EditorMessage> {
        let (line, column) = self.editor.cursor_position();
        let content = self.editor.content();
        let current = content.lines().nth(line).unwrap_or_default();
        let Some((marker, marker_bytes, marker_columns)) = markdown_continuation(current) else {
            let task = self.editor.update(&EditorMessage::Enter);
            self.refresh_document();
            return task;
        };

        if column < marker_columns {
            let task = self.editor.update(&EditorMessage::Enter);
            self.refresh_document();
            return task;
        }

        // Un marqueur vide suivi d'Entrée sort de la liste au lieu d'en créer
        // une nouvelle ligne vide, comme dans les éditeurs Markdown usuels.
        if column >= current.chars().count() && current[marker_bytes..].trim().is_empty() {
            let line_len = current.chars().count();
            let mut tasks = vec![self.editor.set_cursor(line, line_len)];

            for _ in 0..line_len {
                tasks.push(self.editor.update(&EditorMessage::Backspace));
            }

            tasks.push(self.editor.update(&EditorMessage::Enter));
            self.refresh_document();
            return Task::batch(tasks);
        }

        let enter = self.editor.update(&EditorMessage::Enter);
        let continuation = self.editor.update(&EditorMessage::Paste(marker));
        self.refresh_document();

        Task::batch([enter, continuation])
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
    /// documentée au curseur (utilisée par la palette de commandes). **Jamais
    /// avec un texte vide** : chez ce crate, `Paste("")` n'insère rien, il va
    /// *lire le presse-papiers* et le coller. Un collage fantôme s'est ainsi
    /// glissé dans le document à l'ouverture d'une fenêtre.
    pub fn insert_at_cursor(&mut self, text: &str) -> Task<EditorMessage> {
        if text.is_empty() {
            return Task::none();
        }

        self.update(EditorMessage::Paste(text.to_string()))
    }

    /// Remplace ce qui précède le curseur, puis écrit à sa place.
    ///
    /// Le crate n'expose pas de « remplace cette plage » : on le fait donc comme
    /// le ferait une main — autant de retours arrière que de caractères à
    /// effacer, puis l'insertion. C'est la seule voie qui n'ajoute pas de
    /// dépendance à l'intérieur du crate, et elle se comporte exactement comme
    /// la frappe (le curseur suit, l'historique enregistre une seule action).
    ///
    /// Un remplacement par le vide s'arrête aux retours arrière : le `Paste`
    /// qui les suit lirait le presse-papiers (voir [`Self::insert_at_cursor`]).
    pub fn replace_before(&mut self, characters: usize, text: &str) -> Task<EditorMessage> {
        let mut tasks = Vec::with_capacity(characters + 1);

        for _ in 0..characters {
            tasks.push(self.editor.update(&EditorMessage::Backspace));
        }

        if !text.is_empty() {
            tasks.push(self.editor.update(&EditorMessage::Paste(text.to_string())));
        }

        self.refresh_document();

        Task::batch(tasks)
    }

    /// Remplace une plage du document par du texte.
    ///
    /// Le curseur va d'abord en fin de plage : `replace_before` efface à
    /// rebours, et c'est la seule direction que le crate connaisse.
    pub fn replace_span(&mut self, start: usize, end: usize, text: &str) -> Task<EditorMessage> {
        let aller = self.set_cursor_offset(end);
        let remplacer = self.replace_before(end.saturating_sub(start), text);

        Task::batch([aller, remplacer])
    }

    /// Pose le curseur à un décalage, en caractères depuis le début du texte.
    ///
    /// Le crate ne se déplace qu'en (ligne, colonne) : la conversion se fait
    /// ici, une seule fois.
    pub fn set_cursor_offset(&mut self, offset: usize) -> Task<EditorMessage> {
        let content = self.content();
        let offset = offset.min(content.len());
        let avant = &content[..offset];
        let ligne = avant.matches('\n').count();
        let colonne = avant.chars().rev().take_while(|c| *c != '\n').count();

        self.set_cursor(ligne, colonne)
    }

    /// Applique une commande de mise en forme au document `.tw` courant.
    ///
    /// La sélection est lue depuis l'éditeur, mais la représentation structurée
    /// est reconstruite immédiatement après la modification et sert à produire
    /// le rendu Typst.
    pub fn apply_format(
        &mut self,
        action: crate::app::message::FormatAction,
    ) -> Task<EditorMessage> {
        let task = match action {
            crate::app::message::FormatAction::Gras => self.wrap_selection("**", "**"),
            crate::app::message::FormatAction::Italique => self.wrap_selection("*", "*"),
            crate::app::message::FormatAction::Couleur(couleur) => {
                self.wrap_selection(&format!("{{{couleur}:"), "}")
            }
            crate::app::message::FormatAction::Citation => self.insert_at_cursor("@"),
            crate::app::message::FormatAction::Note => self.wrap_selection("^[", "]"),
            crate::app::message::FormatAction::Titre(niveau) => {
                self.format_heading(niveau.clamp(1, 6))
            }
            crate::app::message::FormatAction::ListePuce => self.format_line_prefix("- "),
            crate::app::message::FormatAction::ListeNumero => self.format_numbered_line(),
            crate::app::message::FormatAction::CitationBloc => self.format_line_prefix("> "),
            crate::app::message::FormatAction::BlocTypst => self.format_raw_typst(),
        };

        self.refresh_document();
        task
    }

    /// Entoure la sélection ; sans sélection, insère deux marqueurs et place le
    /// curseur entre eux pour que l'écriture puisse continuer directement.
    fn wrap_selection(&mut self, prefix: &str, suffix: &str) -> Task<EditorMessage> {
        if let Some(selection) = self.editor.selected_text().filter(|text| !text.is_empty()) {
            return self.editor.update(&EditorMessage::Paste(format!(
                "{prefix}{selection}{suffix}"
            )));
        }

        let (line, column) = self.editor.cursor_position();
        let insert = format!("{prefix}{suffix}");
        let inserted_prefix = prefix.chars().count();
        let paste = self.editor.update(&EditorMessage::Paste(insert));
        let cursor = self.editor.set_cursor(line, column + inserted_prefix);

        Task::batch([paste, cursor])
    }

    /// Applique ou retire un préfixe simple de bloc sur la ligne du curseur.
    fn format_line_prefix(&mut self, prefix: &str) -> Task<EditorMessage> {
        let (line, column) = self.editor.cursor_position();
        let content = self.editor.content();
        let current = content.lines().nth(line).unwrap_or_default();
        let old = if current.starts_with(prefix) {
            prefix.chars().count()
        } else {
            0
        };
        let replacement = if old == 0 { prefix } else { "" };

        self.replace_line_prefix(line, column, old, replacement)
    }

    /// Applique ou retire un marqueur numéroté, quelle que soit sa valeur.
    fn format_numbered_line(&mut self) -> Task<EditorMessage> {
        let (line, column) = self.editor.cursor_position();
        let content = self.editor.content();
        let current = content.lines().nth(line).unwrap_or_default();
        let digits = current.chars().take_while(char::is_ascii_digit).count();
        let old = if digits > 0 && current[digits..].starts_with(". ") {
            digits + 2
        } else {
            0
        };
        let replacement = if old == 0 { "1. " } else { "" };

        self.replace_line_prefix(line, column, old, replacement)
    }

    /// Applique un niveau de titre sur la ligne courante ; répéter le même
    /// niveau le retire.
    fn format_heading(&mut self, niveau: u8) -> Task<EditorMessage> {
        let (line, column) = self.editor.cursor_position();
        let content = self.editor.content();
        let current = content.lines().nth(line).unwrap_or_default();
        let hashes = current
            .chars()
            .take_while(|character| *character == '#')
            .count();
        let old = if (1..=6).contains(&hashes) && current[hashes..].starts_with(' ') {
            hashes + 1
        } else {
            0
        };
        let prefix = format!("{} ", "#".repeat(niveau as usize));
        let replacement = if old == prefix.chars().count() {
            "".to_string()
        } else {
            prefix
        };

        self.replace_line_prefix(line, column, old, &replacement)
    }

    /// Insère un bloc Typst complet, en plaçant le caret dans son corps.
    fn format_raw_typst(&mut self) -> Task<EditorMessage> {
        if let Some(selection) = self.editor.selected_text().filter(|text| !text.is_empty()) {
            return self.editor.update(&EditorMessage::Paste(format!(
                "\n```typst\n{selection}\n```\n"
            )));
        }

        let (line, _) = self.editor.cursor_position();
        let paste = self
            .editor
            .update(&EditorMessage::Paste("\n```typst\n\n```\n".to_string()));
        let cursor = self.editor.set_cursor(line + 2, 0);

        Task::batch([paste, cursor])
    }

    fn replace_line_prefix(
        &mut self,
        line: usize,
        column: usize,
        old_characters: usize,
        replacement: &str,
    ) -> Task<EditorMessage> {
        let mut tasks = vec![self.editor.set_cursor(line, 0)];

        for _ in 0..old_characters {
            tasks.push(self.editor.update(&EditorMessage::Delete));
        }

        tasks.push(
            self.editor
                .update(&EditorMessage::Paste(replacement.to_string())),
        );
        tasks.push(self.editor.set_cursor(
            line,
            column.saturating_sub(old_characters) + replacement.chars().count(),
        ));

        Task::batch(tasks)
    }

    /// Où le curseur se trouve **dans la vue**, et non dans le document.
    ///
    /// `cursor_screen_position` porte mal son nom : elle rend l'ordonnée du
    /// curseur dans **tout le document** (le rang de la ligne visuelle multiplié
    /// par la hauteur de ligne, sans soustraction du défilement). Un menu posé à
    /// cette ordonnée se retrouve des milliers de pixels sous la fenêtre dès
    /// qu'on écrit au-delà du premier écran — c'est-à-dire toujours, dans un
    /// chapitre. Le défilement se soustrait donc ici, une fois.
    pub fn cursor_view_position(&self) -> iced::Point {
        let point = self
            .editor
            .cursor_screen_position()
            .unwrap_or(iced::Point::ORIGIN);

        iced::Point::new(point.x, point.y - self.editor.viewport_scroll())
    }

    /// Amène le curseur à une ligne et une colonne.
    ///
    /// Les colonnes se comptent en **caractères**, comme partout dans l'éditeur.
    /// La vue suit le curseur : c'est ce qui fait d'un saut une navigation, et
    /// non un simple changement de position.
    pub fn set_cursor(&mut self, line: usize, column: usize) -> Task<EditorMessage> {
        self.editor.set_cursor(line, column)
    }

    /// La position du curseur, et où il tombe dans le texte.
    ///
    /// La colonne du crate se compte en **caractères** ; tout le reste de
    /// l'application compte en **octets**. C'est ici que les deux se rejoignent.
    pub fn cursor_offset(&self) -> usize {
        let (line, column) = self.editor.cursor_position();
        let content = self.editor.content();

        let mut offset = 0usize;

        for (index, text_line) in content.split('\n').enumerate() {
            if index == line {
                return offset
                    + text_line
                        .char_indices()
                        .nth(column)
                        .map_or(text_line.len(), |(index, _)| index);
            }

            offset += text_line.len() + 1;
        }

        content.len()
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

    /// Active les paires Markdown utiles sans auto-fermer l'apostrophe française.
    fn apply_markdown_profile(&mut self) {
        self.editor.set_auto_close_brackets(true);
        self.editor.set_auto_close_apostrophes(false);
    }

    /// Réapplique les couleurs après un changement de thème.
    pub fn retheme(&mut self, resolved: &crate::ui::theme::Resolved) {
        apply_look(&mut self.editor, resolved);
    }
}

/// Détermine le marqueur Markdown à répéter après Entrée.
///
/// Le préfixe déjà présent fournit les colonnes d'arrêt ; une numérotation
/// continue au nombre suivant. Le texte et la position après ce marqueur restent
/// dans l'éditeur, tandis que le parseur/générateur Typst gère la sémantique.
fn markdown_continuation(line: &str) -> Option<(String, usize, usize)> {
    let body = line.trim_start();
    let indentation_bytes = line.len() - body.len();
    let indentation = &line[..indentation_bytes];
    let indentation_columns = indentation.chars().count();

    if body.starts_with("- ") {
        return Some((
            format!("{indentation}- "),
            indentation_bytes + 2,
            indentation_columns + 2,
        ));
    }

    if body.starts_with("> ") {
        return Some((
            format!("{indentation}> "),
            indentation_bytes + 2,
            indentation_columns + 2,
        ));
    }

    let digits = body.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && body[digits..].starts_with(". ") {
        let number = body[..digits].parse::<usize>().unwrap_or(usize::MAX);
        let next = number.saturating_add(1).max(1);
        let marker = format!("{indentation}{next}. ");

        return Some((
            marker,
            indentation_bytes + digits + 2,
            indentation_columns + digits + 2,
        ));
    }

    None
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

    /// Les fichiers `.tw` sont analysés dès l'ouverture : l'AST est disponible
    /// à l'éditeur et au générateur avant la première frappe.
    #[test]
    fn intermediate_documents_open_with_their_ast() {
        let editor = EditorState::new_file(
            PathBuf::from("document.tw"),
            "# Titre\n\nUn **mot**.\n".to_string(),
        );

        let document = editor.document.as_ref().expect("AST du document.tw");
        assert_eq!(editor.editor.syntax(), "md");
        assert_eq!(editor.editor.syntax_name(), "Markdown");
        assert_eq!(document.blocs.len(), 2);
        assert_eq!(document.texte_nu(), "Titre Un mot.");
    }

    /// Enregistrer sous `.tw` bascule aussi la grammaire, puis la quitte si
    /// l'utilisateur choisit de nouveau un fichier Typst.
    #[test]
    fn changing_the_path_changes_the_surface_syntax() {
        let mut editor = EditorState::new_file(PathBuf::from("source.typ"), "= Titre".to_string());
        assert_eq!(editor.editor.syntax(), "typ");
        assert!(!editor.editor.auto_close_brackets());

        editor.set_path(PathBuf::from("document.tw"));
        assert_eq!(editor.editor.syntax(), "md");
        assert!(editor.editor.auto_close_brackets());
        assert!(!editor.editor.auto_close_apostrophes());
        assert!(editor.document.is_some());

        editor.set_path(PathBuf::from("main.typ"));
        assert_eq!(editor.editor.syntax(), "typ");
        assert!(!editor.editor.auto_close_brackets());
        assert!(editor.document.is_none());
    }

    /// Le bouton Gras enveloppe la sélection et actualise le Document AST.
    #[test]
    fn formatting_selection_updates_intermediate_ast() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "mot".to_string());
        let _ = editor.update(EditorMessage::SelectAll);

        let _ = editor.apply_format(crate::app::message::FormatAction::Gras);

        assert_eq!(editor.content(), "**mot**");
        assert_eq!(
            editor.document.as_ref().expect("AST courant").texte_nu(),
            "mot"
        );
    }

    /// Sans sélection, la commande Typst laisse le curseur dans le corps brut.
    #[test]
    fn raw_typst_format_inserts_an_editable_block() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), String::new());

        let _ = editor.apply_format(crate::app::message::FormatAction::BlocTypst);

        assert_eq!(editor.editor.cursor_position(), (2, 0));
        assert_eq!(editor.content(), "\n```typst\n\n```\n");
        assert!(matches!(
            editor.document.as_ref().expect("AST courant").blocs.as_slice(),
            [crate::langage::schema::Bloc::Brut(contenu)] if contenu.is_empty()
        ));
    }

    /// Le bouton de titre transforme la ligne entière et préserve son texte AST.
    #[test]
    fn heading_format_updates_the_source_and_ast() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "Section".to_string());
        let _ = editor.editor.set_cursor(0, 0);

        let _ = editor.apply_format(crate::app::message::FormatAction::Titre(2));

        assert_eq!(editor.content(), "## Section");
        assert_eq!(
            editor.document.as_ref().expect("AST courant").texte_nu(),
            "Section"
        );
    }

    /// La puce bascule sur la ligne du curseur et est reconnue dans l'AST.
    #[test]
    fn bullet_format_toggles_the_current_line() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "Élément".to_string());
        let _ = editor.editor.set_cursor(0, 0);

        let _ = editor.apply_format(crate::app::message::FormatAction::ListePuce);

        assert_eq!(editor.content(), "- Élément");
        assert!(matches!(
            editor
                .document
                .as_ref()
                .expect("AST courant")
                .blocs
                .as_slice(),
            [crate::langage::schema::Bloc::ListePuce(_)]
        ));

        let _ = editor.apply_format(crate::app::message::FormatAction::ListePuce);
        assert_eq!(editor.content(), "Élément");
    }

    /// Une ligne numérotée devient une liste AST sans modifier son contenu.
    #[test]
    fn numbered_list_format_updates_the_current_line() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "Élément".to_string());
        let _ = editor.editor.set_cursor(0, 0);

        let _ = editor.apply_format(crate::app::message::FormatAction::ListeNumero);

        assert_eq!(editor.content(), "1. Élément");
        assert!(matches!(
            editor
                .document
                .as_ref()
                .expect("AST courant")
                .blocs
                .as_slice(),
            [crate::langage::schema::Bloc::ListeNumero(_)]
        ));
    }

    /// Une citation de bloc est formatée sur la ligne courante.
    #[test]
    fn block_quote_format_updates_the_ast() {
        let mut editor =
            EditorState::new_file(PathBuf::from("document.tw"), "Citation".to_string());
        let _ = editor.editor.set_cursor(0, 0);

        let _ = editor.apply_format(crate::app::message::FormatAction::CitationBloc);

        assert_eq!(editor.content(), "> Citation");
        assert!(matches!(
            editor
                .document
                .as_ref()
                .expect("AST courant")
                .blocs
                .as_slice(),
            [crate::langage::schema::Bloc::Citation(_)]
        ));
    }

    /// Entrée poursuit une liste à puces comme dans un éditeur Markdown.
    #[test]
    fn enter_continues_a_bullet_list() {
        let mut editor =
            EditorState::new_file(PathBuf::from("document.tw"), "- premier".to_string());
        let _ = editor.editor.set_cursor(0, "- premier".chars().count());

        let _ = editor.update(EditorMessage::Enter);

        assert_eq!(editor.content(), "- premier\n- ");
        assert!(matches!(
            editor.document.as_ref().expect("AST courant").blocs.as_slice(),
            [crate::langage::schema::Bloc::ListePuce(items)] if items.len() == 2
        ));
    }

    /// Entrée incrémente le numéro d'une liste ordonnée.
    #[test]
    fn enter_increments_a_numbered_list() {
        let mut editor =
            EditorState::new_file(PathBuf::from("document.tw"), "8. étape".to_string());
        let _ = editor.editor.set_cursor(0, "8. étape".chars().count());

        let _ = editor.update(EditorMessage::Enter);

        assert_eq!(editor.content(), "8. étape\n9. ");
        assert!(matches!(
            editor.document.as_ref().expect("AST courant").blocs.as_slice(),
            [crate::langage::schema::Bloc::ListeNumero(items)] if items.len() == 2
        ));
    }

    /// Entrée poursuit une citation de bloc avec son marqueur.
    #[test]
    fn enter_continues_a_block_quote() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "> texte".to_string());
        let _ = editor.editor.set_cursor(0, "> texte".chars().count());

        let _ = editor.update(EditorMessage::Enter);

        assert_eq!(editor.content(), "> texte\n> ");
        assert!(matches!(
            editor
                .document
                .as_ref()
                .expect("AST courant")
                .blocs
                .as_slice(),
            [
                crate::langage::schema::Bloc::Citation(_),
                crate::langage::schema::Bloc::Citation(_)
            ]
        ));
    }

    /// Entrée sur un marqueur vide quitte le mode liste.
    #[test]
    fn enter_on_an_empty_bullet_exits_the_list() {
        let mut editor = EditorState::new_file(PathBuf::from("document.tw"), "- ".to_string());
        let _ = editor.editor.set_cursor(0, 2);

        let _ = editor.update(EditorMessage::Enter);

        assert_eq!(editor.content(), "\n");
        assert!(
            editor
                .document
                .as_ref()
                .expect("AST courant")
                .blocs
                .is_empty()
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

    /// DIAGNOSTIC : la mémoire du chemin de frappe.
    ///
    /// Ignoré par défaut. Reproduit l'écriture sans interface, en passant par le
    /// message d'insertion documenté du widget, et relève la mémoire résidente
    /// au fil des insertions. C'est le seul moyen honnête de distinguer une
    /// croissance réelle d'une impression.
    ///
    ///     cargo test --release -- --ignored diagnose_editor_memory --nocapture
    #[test]
    #[ignore]
    fn diagnose_editor_memory() {
        fn resident_mb() -> usize {
            let statm = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();

            let pages: usize = statm
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);

            pages * 4096 / (1024 * 1024)
        }

        let path = std::env::temp_dir().join("typst-writer-memoire-edit.typ");

        let content = "Un paragraphe de départ, avec quelques mots pour que le rendu ait de quoi travailler.\n\n".repeat(200);

        std::fs::write(&path, &content).unwrap();

        let mut editor = EditorState::new_file(path.clone(), content);

        println!("départ : {} Mo", resident_mb());

        let mut total = 0usize;

        for _round in 1..=40 {
            for _ in 0..250 {
                let _ = editor.insert_at_cursor("une phrase de plus, écrite au clavier. ");
                total += 1;
            }

            println!("{total:>6} insertions : {:>5} Mo", resident_mb());
        }

        println!("contenu final : {} caractères", editor.content().len());

        let _ = std::fs::remove_file(&path);
    }
}
