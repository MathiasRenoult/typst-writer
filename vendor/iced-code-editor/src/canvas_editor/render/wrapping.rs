//! Line wrapping logic for the text editor.
//!
//! This module handles the calculation of visual lines from logical lines
//! when line wrapping is enabled. It supports both viewport-based wrapping
//! (dynamic) and fixed column wrapping.

use crate::buffer::TextBuffer;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Range;

use crate::canvas_editor::{compare_floats, measure_char_width};

/// Represents a visual line segment in the editor.
///
/// When line wrapping is enabled, a single logical line may be split into
/// multiple visual line segments that are displayed sequentially.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualLine {
    /// Index of the logical line in the text buffer
    pub logical_line: usize,
    /// Segment index (0 for first segment, 1+ for wrapped segments)
    pub segment_index: usize,
    /// Start column in the logical line (inclusive)
    pub start_col: usize,
    /// End column in the logical line (exclusive)
    pub end_col: usize,
}

impl VisualLine {
    /// Creates a new visual line segment.
    ///
    /// # Arguments
    ///
    /// * `logical_line` - Index of the logical line
    /// * `segment_index` - Index of the segment within the line
    /// * `start_col` - Start column (inclusive)
    /// * `end_col` - End column (exclusive)
    pub fn new(
        logical_line: usize,
        segment_index: usize,
        start_col: usize,
        end_col: usize,
    ) -> Self {
        Self { logical_line, segment_index, start_col, end_col }
    }

    /// Returns whether this is the first segment of the logical line.
    pub fn is_first_segment(&self) -> bool {
        self.segment_index == 0
    }

    /// Returns the length of this segment in characters.
    pub fn len(&self) -> usize {
        self.end_col - self.start_col
    }
}

/// Calculator for line wrapping operations.
///
/// Handles the conversion between logical lines (as stored in the text buffer)
/// and visual lines (as displayed on screen with wrapping applied).
pub struct WrappingCalculator {
    /// Whether wrapping is enabled
    wrap_enabled: bool,
    /// Fixed wrap column (None = wrap at viewport width)
    wrap_column: Option<usize>,
    /// Full character width for wide characters
    full_char_width: f32,
    /// Character width for narrow characters
    char_width: f32,
}

impl WrappingCalculator {
    /// Creates a new wrapping calculator.
    ///
    /// # Arguments
    ///
    /// * `wrap_enabled` - Whether line wrapping is enabled
    /// * `wrap_column` - Fixed wrap column, or None for viewport-based wrapping
    /// * `full_char_width` - Full character width in pixels
    /// * `char_width` - Character width in pixels
    ///
    /// # Example
    ///
    /// ```text
    /// // Wrap at viewport width
    /// let calc = WrappingCalculator::new(true, None, 14.0, 8.4);
    ///
    /// // Wrap at 80 characters
    /// let calc = WrappingCalculator::new(true, Some(80), 14.0, 8.4);
    /// ```
    pub fn new(
        wrap_enabled: bool,
        wrap_column: Option<usize>,
        full_char_width: f32,
        char_width: f32,
    ) -> Self {
        Self { wrap_enabled, wrap_column, full_char_width, char_width }
    }

    /// Calculates all visual lines from the text buffer.
    ///
    /// # Arguments
    ///
    /// * `text_buffer` - The text buffer to wrap
    /// * `viewport_width` - Width of the viewport in pixels (used if wrap_column is None)
    /// * `gutter_width` - Width of the line number gutter in pixels (subtracted from available width)
    /// * `hidden` - Logical lines hidden by collapsed code folds; these produce no visual lines
    ///
    /// # Returns
    ///
    /// A vector of visual line segments
    pub fn calculate_visual_lines(
        &self,
        text_buffer: &TextBuffer,
        viewport_width: f32,
        gutter_width: f32,
        hidden: &HashSet<usize>,
    ) -> Vec<VisualLine> {
        self.calculate_visual_lines_range(
            text_buffer,
            viewport_width,
            gutter_width,
            hidden,
            0..text_buffer.line_count(),
        )
    }

    /// Calculates visual lines for a contiguous logical-line range.
    ///
    /// This is used by the editor's incremental layout cache after localized
    /// edits, avoiding a full-file rewrap on every keystroke.
    pub(crate) fn calculate_visual_lines_range(
        &self,
        text_buffer: &TextBuffer,
        viewport_width: f32,
        gutter_width: f32,
        hidden: &HashSet<usize>,
        logical_range: Range<usize>,
    ) -> Vec<VisualLine> {
        let logical_range = logical_range.start.min(text_buffer.line_count())
            ..logical_range.end.min(text_buffer.line_count());

        if !self.wrap_enabled {
            // No wrapping: one visual line per (visible) logical line
            return logical_range
                .filter(|line| !hidden.contains(line))
                .map(|line| {
                    VisualLine::new(line, 0, 0, text_buffer.line_len(line))
                })
                .collect();
        }

        // Calculate wrap width in pixels
        // If wrap_column is set, width is columns * character width.
        // Otherwise, use viewport width minus gutter width.
        let wrap_width_pixels = if let Some(cols) = self.wrap_column {
            cols as f32 * self.char_width
        } else {
            (viewport_width - gutter_width).max(self.char_width)
        };

        let mut visual_lines = Vec::new();

        for logical_line in logical_range {
            if hidden.contains(&logical_line) {
                continue; // Hidden by a collapsed fold: emit no visual lines.
            }

            let line_content = text_buffer.line(logical_line);

            if line_content.is_empty() {
                visual_lines.push(VisualLine::new(logical_line, 0, 0, 0));
                continue;
            }

            let chars: Vec<char> = line_content.chars().collect();
            let mut segment_index = 0;
            let mut segment_start = 0;

            while segment_start < chars.len() {
                // Furthest character that still fits, measured from the start of
                // this segment. At least one character is always accepted, even
                // if it alone exceeds the wrap width.
                let mut segment_end = segment_start;
                let mut width = 0.0;

                while segment_end < chars.len() {
                    let char_width = measure_char_width(
                        chars[segment_end],
                        self.full_char_width,
                        self.char_width,
                    );

                    if compare_floats(width + char_width, wrap_width_pixels)
                        == Ordering::Greater
                        && segment_end > segment_start
                    {
                        break;
                    }

                    width += char_width;
                    segment_end += 1;
                }

                // The whole remainder fits: it is the last segment of this line.
                if segment_end == chars.len() {
                    visual_lines.push(VisualLine::new(
                        logical_line,
                        segment_index,
                        segment_start,
                        chars.len(),
                    ));
                    break;
                }

                // The line overflows at `segment_end`. Wrapping exactly there
                // would cut a word in two, which is unreadable in prose: back up
                // to the last space of the segment instead, and break after it.
                // The space stays at the end of the finished line, so no
                // character is dropped and columns in the logical line keep
                // their meaning.
                let break_at = chars[segment_start..segment_end]
                    .iter()
                    .rposition(|c| c.is_whitespace())
                    .map(|offset| segment_start + offset + 1)
                    // A single word wider than the line: cutting it is then the
                    // only option left. Same when the segment opens on a space,
                    // where falling back guarantees forward progress.
                    .filter(|&at| at > segment_start)
                    .unwrap_or(segment_end);

                visual_lines.push(VisualLine::new(
                    logical_line,
                    segment_index,
                    segment_start,
                    break_at,
                ));

                segment_index += 1;
                segment_start = break_at;
            }
        }

        visual_lines
    }

    /// Converts a logical position to a visual line index.
    ///
    /// # Arguments
    ///
    /// * `visual_lines` - Pre-calculated visual lines
    /// * `line` - Logical line index
    /// * `col` - Column in the logical line
    ///
    /// # Returns
    ///
    /// The visual line index containing this position
    pub fn logical_to_visual(
        visual_lines: &[VisualLine],
        line: usize,
        col: usize,
    ) -> Option<usize> {
        // Visual lines are ordered by logical line, then by segment. Locate the
        // small slice for the requested logical line with binary partitioning
        // instead of scanning from the start of a potentially huge file.
        let start =
            visual_lines.partition_point(|visual| visual.logical_line < line);
        let end =
            visual_lines.partition_point(|visual| visual.logical_line <= line);
        let line_segments = visual_lines.get(start..end)?;

        // At a wrap boundary the cursor belongs to the following segment. At
        // the logical end of line it belongs to the final segment.
        let segment =
            line_segments.partition_point(|visual| visual.end_col <= col);
        if let Some(visual) = line_segments.get(segment)
            && col >= visual.start_col
        {
            return Some(start + segment);
        }

        line_segments
            .last()
            .filter(|visual| col == visual.end_col)
            .map(|_| end.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas_editor::{CHAR_WIDTH, FONT_SIZE};

    #[test]
    fn test_no_wrap_when_disabled() {
        let buffer = TextBuffer::new("line 1\nline 2\nline 3");
        let calc = WrappingCalculator::new(false, None, FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert_eq!(visual_lines.len(), 3);
        assert_eq!(visual_lines[0].logical_line, 0);
        assert_eq!(visual_lines[1].logical_line, 1);
        assert_eq!(visual_lines[2].logical_line, 2);
    }

    #[test]
    fn test_hidden_lines_are_skipped() {
        let buffer = TextBuffer::new("line 1\nline 2\nline 3\nline 4");
        let calc = WrappingCalculator::new(false, None, FONT_SIZE, CHAR_WIDTH);
        // Hide logical lines 1 and 2 (e.g. a collapsed fold).
        let hidden: HashSet<usize> = [1, 2].into_iter().collect();
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &hidden);

        // Only lines 0 and 3 remain visible.
        assert_eq!(visual_lines.len(), 2);
        assert_eq!(visual_lines[0].logical_line, 0);
        assert_eq!(visual_lines[1].logical_line, 3);
    }

    #[test]
    fn test_hidden_lines_are_skipped_when_wrapping() {
        let buffer = TextBuffer::new("aaaa\nbbbb\ncccc");
        let calc =
            WrappingCalculator::new(true, Some(80), FONT_SIZE, CHAR_WIDTH);
        let hidden: HashSet<usize> = [1].into_iter().collect();
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &hidden);

        assert_eq!(visual_lines.len(), 2);
        assert_eq!(visual_lines[0].logical_line, 0);
        assert_eq!(visual_lines[1].logical_line, 2);
    }

    /// Un texte en prose se coupe aux frontières de mots, jamais au milieu.
    ///
    /// Le découpage d'origine rompait au dernier caractère qui tenait : « aa
    /// bbbbbbbb cc » sur une ligne de dix devenait « aa bbbbbbb » puis
    /// « b cc », coupant le mot en deux alors que l'espace précédent était un
    /// point de rupture tout trouvé.
    #[test]
    fn test_wrap_at_word_boundary() {
        let text = "aa bbbbbbbb cc";
        let buffer = TextBuffer::new(text);
        let calc =
            WrappingCalculator::new(true, Some(10), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        // La rupture recule jusqu'à l'espace, au lieu de trancher le mot.
        assert_eq!(visual_lines.len(), 3);
        assert_eq!((visual_lines[0].start_col, visual_lines[0].end_col), (0, 3));
        assert_eq!((visual_lines[1].start_col, visual_lines[1].end_col), (3, 12));
        assert_eq!(
            (visual_lines[2].start_col, visual_lines[2].end_col),
            (12, 14)
        );

        // Aucun caractère n'est perdu : les segments pavent la ligne entière.
        let rebuilt: String = visual_lines
            .iter()
            .map(|segment| {
                buffer.line(0).chars().collect::<Vec<_>>()
                    [segment.start_col..segment.end_col]
                    .iter()
                    .collect::<String>()
            })
            .collect();

        assert_eq!(rebuilt, text);
    }

    /// Une ligne de prose ne doit jamais être coupée au milieu d'un mot.
    #[test]
    fn test_no_word_is_cut_across_segments() {
        let text =
            "this is a very long line that should be wrapped at word boundaries";
        let buffer = TextBuffer::new(text);
        let calc =
            WrappingCalculator::new(true, Some(9), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert!(visual_lines.len() > 1, "la ligne doit être découpée");

        let chars: Vec<char> = text.chars().collect();
        let last = visual_lines.len() - 1;

        // Chaque segmentation se poursuit là où la précédente s'arrête.
        for (index, pair) in visual_lines.windows(2).enumerate() {
            assert_eq!(
                pair[0].end_col,
                pair[1].start_col,
                "trou ou recouvrement après le segment {index}"
            );
        }

        assert_eq!(visual_lines[0].start_col, 0);
        assert_eq!(visual_lines[last].end_col, chars.len());

        // Toute rupture, sauf la dernière, tombe juste après une espace : c'est
        // exactement ce qui garantit qu'aucun mot n'est tranché.
        //
        // L'exception est le mot plus long que la ligne — « boundaries » sur
        // une ligne de neuf caractères — qui n'a aucune espace où se rompre.
        // Un segment qui contient une espace **doit** y avoir rompu.
        for segment in &visual_lines[..last] {
            let text: String = chars[segment.start_col..segment.end_col]
                .iter()
                .collect();

            if !text.chars().any(char::is_whitespace) {
                continue;
            }

            assert!(
                chars[segment.end_col - 1].is_whitespace(),
                "mot coupé alors qu'une espace était disponible : {text:?}"
            );
        }
    }

    /// Un mot plus long que la ligne reste coupé : il n'y a pas d'alternative.
    #[test]
    fn test_word_longer_than_the_line_is_still_cut() {
        let buffer = TextBuffer::new("supercalifragilistic");
        let calc =
            WrappingCalculator::new(true, Some(8), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert_eq!(visual_lines.len(), 3);
        assert_eq!(visual_lines[0].end_col, 8);
        assert_eq!(visual_lines[2].end_col, 20);
    }

    #[test]
    fn test_wrap_at_fixed_column() {
        let buffer =
            TextBuffer::new("this is a very long line that should be wrapped");
        let calc =
            WrappingCalculator::new(true, Some(10), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        // Line is 47 chars, should wrap into 5 segments (10+10+10+10+7)
        assert_eq!(visual_lines.len(), 5);
        assert_eq!(visual_lines[0].start_col, 0);
        assert_eq!(visual_lines[0].end_col, 10);
        assert_eq!(visual_lines[1].start_col, 10);
        assert_eq!(visual_lines[1].end_col, 20);
        assert_eq!(visual_lines[4].start_col, 40);
        assert_eq!(visual_lines[4].end_col, 47);
    }

    #[test]
    fn test_logical_to_visual_mapping() {
        let buffer =
            TextBuffer::new("short\nthis is a very long line that wraps\nend");
        let calc =
            WrappingCalculator::new(true, Some(15), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        // First line (short) - no wrap
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 0, 0),
            Some(0)
        );

        // Second line (long) - wraps
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 1, 0),
            Some(1)
        );
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 1, 14),
            Some(1)
        );
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 1, 15),
            Some(2)
        );
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 1, 30),
            Some(3)
        );
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 1, 35),
            Some(3)
        );
        assert_eq!(
            WrappingCalculator::logical_to_visual(&visual_lines, 99, 0),
            None
        );
    }

    #[test]
    fn test_calculate_visual_lines_range_keeps_logical_indices() {
        let buffer = TextBuffer::new("zero\none\ntwo\nthree");
        let calc = WrappingCalculator::new(false, None, FONT_SIZE, CHAR_WIDTH);
        let visual_lines = calc.calculate_visual_lines_range(
            &buffer,
            800.0,
            60.0,
            &HashSet::new(),
            1..3,
        );

        assert_eq!(
            visual_lines
                .iter()
                .map(|visual| visual.logical_line)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn test_wrap_empty_lines() {
        let buffer = TextBuffer::new("line1\n\nline3");
        let calc =
            WrappingCalculator::new(true, Some(10), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert_eq!(visual_lines.len(), 3);
        assert_eq!(visual_lines[1].logical_line, 1);
        assert_eq!(visual_lines[1].len(), 0);
    }

    #[test]
    fn test_wrap_very_long_line() {
        let long_text = "a".repeat(100);
        let buffer = TextBuffer::new(&long_text);
        let calc =
            WrappingCalculator::new(true, Some(20), FONT_SIZE, CHAR_WIDTH);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        // 100 chars / 20 per line = 5 lines
        assert_eq!(visual_lines.len(), 5);
        assert!(visual_lines.iter().all(|vl| vl.logical_line == 0));
    }

    #[test]
    fn test_visual_line_is_first_segment() {
        let vl1 = VisualLine::new(0, 0, 0, 10);
        let vl2 = VisualLine::new(0, 1, 10, 20);

        assert!(vl1.is_first_segment());
        assert!(!vl2.is_first_segment());
    }

    #[test]
    fn test_wrap_cjk() {
        // CJK characters are wide (FONT_SIZE = 14.0)
        // Latin characters are narrow (CHAR_WIDTH = 8.4)
        // Wrap width = 10 columns * 8.4 = 84.0 pixels

        // 6 CJK characters = 6 * 14.0 = 84.0 pixels. Matches exactly.
        let text = "你好世界你好"; // 6 chars
        let buffer = TextBuffer::new(text);
        let calc =
            WrappingCalculator::new(true, Some(10), FONT_SIZE, CHAR_WIDTH); // 84.0 px
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert_eq!(visual_lines.len(), 1);
        assert_eq!(visual_lines[0].len(), 6);

        // 7 CJK characters = 7 * 14.0 = 98.0 pixels.
        // Wrap width is 84.0 pixels.
        // First 6 chars = 6 * 14.0 = 84.0 pixels. They fit exactly (84.0 <= 84.0).
        // 7th char adds 14.0, total 98.0 > 84.0. Triggers wrap before 7th char.
        let text = "你好世界你好世"; // 7 chars
        let buffer = TextBuffer::new(text);
        let visual_lines =
            calc.calculate_visual_lines(&buffer, 800.0, 60.0, &HashSet::new());

        assert_eq!(visual_lines.len(), 2);
        assert_eq!(visual_lines[0].len(), 6); // First 6 fit
        assert_eq!(visual_lines[1].len(), 1); // 7th wraps
        assert_eq!(visual_lines[1].start_col, 6); // Starts at 7th char (index 6)
    }
}
