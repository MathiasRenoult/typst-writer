//! Source et historique uniques, projections graphiques indépendantes.
use super::*;

/// Modèle détenu par le document, et non par un onglet.
/// CommandHistory est déjà un Arc<Mutex<_>> : toutes les projections
/// partagent le même historique, jamais des piles indépendantes.
pub struct SourcePartagee {
    buffer: TextBuffer,
    history: CommandHistory,
    grouping: bool,
    revision: u64,
}

impl SourcePartagee {
    /// Texte canonique pour la compilation de toutes les sources ouvertes.
    pub fn contenu(&self) -> String { self.buffer.to_string() }

    /// Crée le modèle canonique depuis la première projection.
    pub fn nouvelle(editor: &CodeEditor) -> Self {
        Self { buffer: editor.buffer.clone(), history: editor.history.clone(), grouping: false, revision: editor.buffer_revision }
    }

    /// Prête l’historique unique à la vue qui reçoit un message.
    pub fn ouvrir_transaction(&mut self, editor: &mut CodeEditor) {
        self.projeter(editor);
        editor.history = self.history.clone();
        editor.is_grouping = self.grouping;
    }

    /// Rend l'historique unique et publie le texte atomiquement à l'hôte.
    pub fn fermer_transaction(&mut self, editor: &mut CodeEditor) -> bool {
        self.grouping = editor.is_grouping;
        let change = self.revision != editor.buffer_revision;
        if change {
            self.buffer = editor.buffer.clone();
            self.revision = editor.buffer_revision;
        }
        change
    }

    /// Ne modifie ni curseur, ni sélection, ni défilement, ni identité widget.
    pub fn projeter(&self, editor: &mut CodeEditor) {
        if editor.buffer_revision == self.revision { return; }
        self.projeter_force(editor);
    }

    /// Initialise une projection neuve, même si sa révision vaut aussi zéro.
    pub fn projeter_force(&self, editor: &mut CodeEditor) {
        editor.history = self.history.clone();
        editor.buffer = self.buffer.clone();
        editor.buffer_revision = self.revision;
        editor.content_cache.clear();
        editor.overlay_cache.clear();
        *editor.visual_lines_cache.borrow_mut() = None;
        *editor.foldable_regions_cache.borrow_mut() = None;
        *editor.max_content_width_cache.borrow_mut() = None;
        editor.pre_edit_line = 0;
        editor.pre_edit_last_line = usize::MAX;
        editor.invalidate_highlight_from(0);
        *editor.bracket_depth_cache.borrow_mut() = BracketDepthCache::new();
        editor.search_state.update_matches(&editor.buffer);
        editor.collapsed_folds.clear();
        editor.fold_revision = editor.fold_revision.wrapping_add(1);
        editor.enqueue_lsp_change();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deux_vues_partagent_le_texte_et_un_seul_historique() {
        let mut a = CodeEditor::new("bonjour", "txt");
        let mut b = CodeEditor::new("bonjour", "txt");
        let mut source = SourcePartagee::nouvelle(&a);
        source.ouvrir_transaction(&mut a);
        let _ = a.update(&Message::Paste("salut ".into()));
        assert!(source.fermer_transaction(&mut a));
        source.projeter(&mut b);
        assert_eq!(a.content(), b.content());
        assert!(a.can_undo());
        assert!(b.can_undo());
        source.ouvrir_transaction(&mut b);
        assert!(b.can_undo());
        let _ = b.update(&Message::Undo);
        source.fermer_transaction(&mut b);
        source.projeter(&mut a);
        assert_eq!(a.content(), "bonjour");
        assert_eq!(b.content(), "bonjour");
    }
}
