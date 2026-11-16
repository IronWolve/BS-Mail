//! Putting the cursor where the pointer is: a click, and a drag that selects.

use super::editor_buffer::{EditorBuffer, Pos};
use super::graphemes::{count, list, width};

impl EditorBuffer {
    /// The cursor to the position nearest screen row `row` (counted from the
    /// top of the text) and cell column `x` in a pane `room` cells wide. With
    /// `extend` the selection grows from where it began.
    pub fn move_to_cell(&mut self, row: usize, x: usize, room: usize, extend: bool) {
        self.start_move(extend);
        let visual = self.visual_lines(room);
        let at = row.min(visual.len().saturating_sub(1));
        if let Some(piece) = visual.get(at).copied() {
            self.cursor = self.position_at(&visual, at, piece, x);
        }
        self.goal = None;
    }

    /// The cursor to grapheme `col` of the one line of a field. With `extend`
    /// the selection grows from where it began.
    pub fn move_to_col(&mut self, col: usize, extend: bool) {
        self.start_move(extend);
        self.cursor = Pos { line: 0, col: col.min(count(&self.lines[0])) };
        self.goal = None;
    }

    /// The cursor as near `at` as the text allows, with nothing selected.
    pub fn place_near(&mut self, at: Pos) {
        let line = at.line.min(self.lines.len() - 1);
        self.cursor = Pos { line, col: at.col.min(count(&self.lines[line])) };
        self.anchor = None;
        self.goal = None;
    }

    /// Replaces graphemes `from` up to `to` of the first line with `text`,
    /// leaving the cursor after it. One step of undo.
    pub fn replace_cols(&mut self, from: usize, to: usize, text: &str) -> bool {
        let end = count(&self.lines[0]);
        self.anchor = Some(Pos { line: 0, col: from.min(end) });
        self.cursor = Pos { line: 0, col: to.min(end) };
        self.insert_text(text)
    }
}

/// The grapheme of `line` a click at cell `x` lands on, counting cells from
/// the grapheme `skip`: the cursor goes before the grapheme clicked, and at
/// the end of the line for a click past it.
pub fn col_at_cell(line: &str, skip: usize, x: usize) -> usize {
    let mut used = 0;
    for (index, grapheme) in list(line).iter().enumerate().skip(skip) {
        let cells = width(grapheme);
        if x < used + cells {
            return index;
        }
        used += cells;
    }
    count(line)
}

#[cfg(test)]
include!("pointer_moves.tests.rs");
