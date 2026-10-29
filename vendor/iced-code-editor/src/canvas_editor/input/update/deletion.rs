//! Backspace, delete, and explicit-selection-delete message handlers.

use iced::Task;

use super::{EditType, adjust_other_cursors};
use crate::canvas_editor::editing::command::{
    Command, DeleteCharCommand, DeleteForwardCommand,
};
use crate::canvas_editor::{CodeEditor, Message};

impl CodeEditor {
    /// Sélectionne les notes dont le curseur occupe le bord, pour qu'une
    /// suppression les emporte entières.
    ///
    /// Une note est un seul caractère — **même sur plusieurs lignes** : ses
    /// marques portent le même groupe, et la suppression vise les deux bords
    /// du groupe, pas le rangon sous le curseur. Sur son bord, Backspace ou
    /// Suppr emporte toute la note — c'est même le seul geste qui en
    /// débarrasse, le clic étant réservé à son édition.
    fn selectionner_les_notes_au_bord(&mut self, avant: bool) {
        use crate::canvas_editor::marques::bords_du_groupe;

        // Les deux bords de chaque note du document.
        let mut groupes: Vec<((usize, usize), (usize, usize))> = Vec::new();
        let mut vus: Vec<usize> = Vec::new();

        for marque in &self.marques {
            if vus.contains(&marque.groupe) {
                continue;
            }
            vus.push(marque.groupe);

            if let Some(bords) = bords_du_groupe(&self.marques, marque.groupe) {
                groupes.push(bords);
            }
        }

        for curseur in self.cursors.as_mut_slice() {
            if curseur.anchor.is_some() {
                continue;
            }

            let pos = curseur.position;
            let cible = groupes.iter().find(|(debut, fin)| {
                if avant {
                    // Backspace : la note qui finit sous le curseur, ou dont
                    // il occupe l'intérieur. Sur son bord gauche, il efface le
                    // caractère d'avant — la note est devant lui.
                    *debut < pos && pos <= *fin
                } else {
                    // Suppr : la note qui commence sous le curseur, ou dont il
                    // occupe l'intérieur.
                    *debut <= pos && pos < *fin
                }
            });

            if let Some((debut, fin)) = cible {
                curseur.anchor = Some(*debut);
                curseur.position = *fin;
            }
        }
    }

    /// Étend les sélections qui coupent une note pour couvrir la note entière.
    ///
    /// Une note est un caractère — même quand elle court sur plusieurs lignes.
    /// Une sélection qui n'en prend qu'une partie laisserait le reste du texte
    /// fondu à nu : les caractères de la clé apparaîtraient, fantômes. Toute
    /// sélection qui touche un groupe s'étend aux deux bords de ce groupe.
    pub(crate) fn etendre_selections_aux_marques(&mut self) {
        use crate::canvas_editor::marques::bords_du_groupe;

        if self.marques.is_empty() {
            return;
        }

        // Les deux bords de chaque note du document.
        let mut groupes: Vec<((usize, usize), (usize, usize))> = Vec::new();
        let mut vus: Vec<usize> = Vec::new();
        for marque in &self.marques {
            if vus.contains(&marque.groupe) {
                continue;
            }
            vus.push(marque.groupe);
            if let Some(bords) = bords_du_groupe(&self.marques, marque.groupe) {
                groupes.push(bords);
            }
        }

        for curseur in self.cursors.as_mut_slice() {
            let Some(anchor) = curseur.anchor else { continue };
            let pos = curseur.position;
            let (sel_debut, sel_fin) = if anchor <= pos { (anchor, pos) } else { (pos, anchor) };

            let mut nouveau_debut = sel_debut;
            let mut nouveau_fin = sel_fin;

            // Une seule passe suffit : les marques ne se chevauchent pas, et
            // l'extension s'arrête aux bords de groupe, jamais à l'intérieur.
            for &(g_debut, g_fin) in &groupes {
                let touche = nouveau_debut < g_fin && g_debut < nouveau_fin;
                if !touche { continue; }
                nouveau_debut = nouveau_debut.min(g_debut);
                nouveau_fin = nouveau_fin.max(g_fin);
            }

            if (nouveau_debut, nouveau_fin) != (sel_debut, sel_fin) {
                if anchor <= pos {
                    curseur.anchor = Some(nouveau_debut);
                    curseur.position = nouveau_fin;
                } else {
                    curseur.anchor = Some(nouveau_fin);
                    curseur.position = nouveau_debut;
                }
            }
        }
    }

    /// Handles Backspace key press.
    ///
    /// If there's a selection, deletes the selection. Otherwise, deletes the
    /// character before the cursor.
    ///
    /// # Returns
    ///
    /// A `Task<Message>` that scrolls to keep the cursor visible if selection was deleted
    pub(crate) fn handle_backspace(&mut self) -> Task<Message> {
        // End grouping on backspace (separate from typing)
        if !self.keep_vim_insert_group() {
            self.end_grouping_if_active();
        }

        // Une note sur le bord gauche se sélectionne, pour partir entière.
        self.selectionner_les_notes_au_bord(true);

        // If any cursor has a selection, delete all selections first
        if self.delete_selection_if_present() {
            return self.scroll_to_cursor();
        }

        // A mouse click leaves a zero-length anchor in place so a following
        // drag can extend the selection (see `handle_enter`). Backspace must
        // clear it before moving the caret; otherwise the anchor is left
        // behind at the pre-edit position and a phantom one-character
        // selection appears next to the cursor, which the next Backspace or
        // Delete then eats instead of a single character.
        self.clear_selection();

        // Multi-cursor: process in descending document order
        let order = self.cursors.descending_order();

        for &idx in &order {
            let pos = self.cursors.as_slice()[idx].position;
            // Determine edit type for adjusting other cursors
            let edit_kind = if pos.1 > 0 {
                EditType::DeleteCharBack
            } else if pos.0 > 0 {
                let prev_line_len = self.buffer.line_len(pos.0 - 1);
                EditType::MergePrev { prev_line_len }
            } else {
                // At very start of document: nothing to delete
                continue;
            };
            let mut cmd =
                DeleteCharCommand::new(&self.buffer, pos.0, pos.1, pos);
            let mut cursor_pos = pos;
            cmd.execute(&mut self.buffer, &mut cursor_pos);
            self.cursors.as_mut_slice()[idx].position = cursor_pos;
            adjust_other_cursors(
                self.cursors.as_mut_slice(),
                idx,
                pos.0,
                pos.1,
                edit_kind,
            );
            self.history.push(Box::new(cmd));
        }

        self.finish_edit_operation();
        self.scroll_to_cursor()
    }

    /// Handles Delete key press.
    ///
    /// If there's a selection, deletes the selection. Otherwise, deletes the
    /// character after the cursor.
    ///
    /// # Returns
    ///
    /// A `Task<Message>` that scrolls to keep the cursor visible if selection was deleted
    pub(crate) fn handle_delete(&mut self) -> Task<Message> {
        // End grouping on delete
        if !self.keep_vim_insert_group() {
            self.end_grouping_if_active();
        }

        // Une note sur le bord droit se sélectionne, pour partir entière.
        self.selectionner_les_notes_au_bord(false);

        // If any cursor has a selection, delete all selections first
        if self.delete_selection_if_present() {
            return self.scroll_to_cursor();
        }

        // See the matching comment in `handle_backspace`: clear any
        // zero-length anchor left by a plain click before editing, so it
        // can't be mistaken for a real selection on a later edit.
        self.clear_selection();

        // Multi-cursor: process in descending document order
        let order = self.cursors.descending_order();

        for &idx in &order {
            let pos = self.cursors.as_slice()[idx].position;
            let line_len = self.buffer.line_len(pos.0);
            let edit_kind = if pos.1 < line_len {
                EditType::DeleteCharForward
            } else if pos.0 + 1 < self.buffer.line_count() {
                EditType::MergeNext { edit_line_len: line_len }
            } else {
                // At very end of document: nothing to delete
                continue;
            };
            let mut cmd =
                DeleteForwardCommand::new(&self.buffer, pos.0, pos.1, pos);
            let mut cursor_pos = pos;
            cmd.execute(&mut self.buffer, &mut cursor_pos);
            self.cursors.as_mut_slice()[idx].position = cursor_pos;
            adjust_other_cursors(
                self.cursors.as_mut_slice(),
                idx,
                pos.0,
                pos.1,
                edit_kind,
            );
            self.history.push(Box::new(cmd));
        }

        self.finish_edit_operation();
        Task::none()
    }

    /// Handles explicit selection deletion (Shift+Delete).
    ///
    /// Deletes the selected text if a selection exists.
    ///
    /// # Returns
    ///
    /// A `Task<Message>` that scrolls to keep the cursor visible
    pub(crate) fn handle_delete_selection(&mut self) -> Task<Message> {
        // End grouping on delete selection
        self.end_grouping_if_active();

        if self.delete_selection_if_present() {
            self.scroll_to_cursor()
        } else {
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_selection_message() {
        let mut editor = CodeEditor::new("hello world", "py");
        editor.cursors.primary_mut().position = (0, 0);
        editor.cursors.primary_mut().anchor = Some((0, 0));
        editor.cursors.primary_mut().position = (0, 5);

        let _ = editor.update(&Message::DeleteSelection);
        assert_eq!(editor.buffer.line(0), " world");
        assert_eq!(editor.cursors.primary_position(), (0, 0));
        assert!(editor.cursors.primary().anchor.is_none());
        assert!(!editor.cursors.primary().has_selection());
    }

    #[test]
    fn test_delete_selection_multiline() {
        let mut editor = CodeEditor::new("line1\nline2\nline3", "py");
        editor.cursors.primary_mut().position = (0, 2);
        editor.cursors.primary_mut().anchor = Some((0, 2));
        editor.cursors.primary_mut().position = (2, 2);

        let _ = editor.update(&Message::DeleteSelection);
        assert_eq!(editor.buffer.line(0), "line3");
        assert_eq!(editor.cursors.primary_position(), (0, 2));
        assert!(editor.cursors.primary().anchor.is_none());
    }

    #[test]
    fn test_delete_selection_no_selection() {
        let mut editor = CodeEditor::new("hello world", "py");
        editor.cursors.primary_mut().position = (0, 5);

        let _ = editor.update(&Message::DeleteSelection);
        // Should do nothing if there's no selection
        assert_eq!(editor.buffer.line(0), "hello world");
        assert_eq!(editor.cursors.primary_position(), (0, 5));
    }

    #[test]
    fn test_delete_key_with_selection() {
        let mut editor = CodeEditor::new("hello world", "py");
        editor.cursors.primary_mut().anchor = Some((0, 0));
        editor.cursors.primary_mut().position = (0, 5);
        editor.cursors.primary_mut().position = (0, 5);

        let _ = editor.update(&Message::Delete);

        assert_eq!(editor.buffer.line(0), " world");
        assert_eq!(editor.cursors.primary_position(), (0, 0));
        assert!(editor.cursors.primary().anchor.is_none());
        assert!(!editor.cursors.primary().has_selection());
    }

    #[test]
    fn test_delete_key_without_selection() {
        let mut editor = CodeEditor::new("hello", "py");
        editor.cursors.primary_mut().position = (0, 0);

        let _ = editor.update(&Message::Delete);

        // Should delete the 'h'
        assert_eq!(editor.buffer.line(0), "ello");
        assert_eq!(editor.cursors.primary_position(), (0, 0));
    }

    #[test]
    fn test_backspace_with_selection() {
        let mut editor = CodeEditor::new("hello world", "py");
        editor.cursors.primary_mut().anchor = Some((0, 6));
        editor.cursors.primary_mut().position = (0, 11);
        editor.cursors.primary_mut().position = (0, 11);

        let _ = editor.update(&Message::Backspace);

        assert_eq!(editor.buffer.line(0), "hello ");
        assert_eq!(editor.cursors.primary_position(), (0, 6));
        assert!(editor.cursors.primary().anchor.is_none());
        assert!(!editor.cursors.primary().has_selection());
    }

    #[test]
    fn test_backspace_without_selection() {
        let mut editor = CodeEditor::new("hello", "py");
        editor.cursors.primary_mut().position = (0, 5);

        let _ = editor.update(&Message::Backspace);

        // Should delete the 'o'
        assert_eq!(editor.buffer.line(0), "hell");
        assert_eq!(editor.cursors.primary_position(), (0, 4));
    }

    #[test]
    fn test_delete_multiline_selection() {
        let mut editor = CodeEditor::new("line1\nline2\nline3", "py");
        editor.cursors.primary_mut().anchor = Some((0, 2));
        editor.cursors.primary_mut().position = (2, 2);
        editor.cursors.primary_mut().position = (2, 2);

        let _ = editor.update(&Message::Delete);

        assert_eq!(editor.buffer.line(0), "line3");
        assert_eq!(editor.cursors.primary_position(), (0, 2));
        assert!(editor.cursors.primary().anchor.is_none());
    }

    /// Une sélection qui coupe une note à moitié l'emporte entière — jamais
    /// de texte fondu laissé à nu.
    #[test]
    fn une_selection_partielle_emporte_la_note_entiere() {
        let mut editor = CodeEditor::new("avant @dupont[p.45] apres", "py");
        // `@dupont[p.45]` occupe les colonnes 6 à 19.
        editor.set_marques(vec![crate::canvas_editor::marques::Marque::new(0, 0, 6, 19, "1")]);

        // Sélection qui coupe la note : de la colonne 4 à la colonne 10.
        editor.cursors.primary_mut().anchor = Some((0, 4));
        editor.cursors.primary_mut().position = (0, 10);

        editor.delete_selection();

        // La note entière est partie : plus de clé de citation à nu.
        assert_eq!(editor.buffer.line(0), "avan apres");
        assert!(!editor.buffer.line(0).contains("dupont"));
    }
}
