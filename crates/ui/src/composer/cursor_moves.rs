//! Moving the cursor, and extending the selection while moving.
//!
//! Every move takes `extend`: with it the selection grows from where the move
//! began, without it the selection is dropped. Vertical moves follow the
//! screen's wrapped lines, so they need the width of the pane.

use std::sync::Arc;

use super::editor_buffer::{EditorBuffer, Pos};
use super::graphemes::{cells_between, count, is_blank, list, width};
use super::wrap::{VisualLine, visual_index};

impl EditorBuffer {
    /// Where a move starts from: ends the typing run, and sets or drops the
    /// selection anchor.
    pub(super) fn start_move(&mut self, extend: bool) {
        self.history.break_run();
        if extend {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
    }

    pub fn move_left(&mut self, extend: bool) {
        if !extend && let Some((from, _)) = self.selection() {
            self.collapse_to(from);
            return;
        }
        self.start_move(extend);
        self.cursor = self.before(self.cursor);
        self.goal = None;
    }

    pub fn move_right(&mut self, extend: bool) {
        if !extend && let Some((_, to)) = self.selection() {
            self.collapse_to(to);
            return;
        }
        self.start_move(extend);
        if let Some(next) = self.after(self.cursor) {
            self.cursor = next;
        }
        self.goal = None;
    }

    fn collapse_to(&mut self, at: Pos) {
        self.history.break_run();
        self.anchor = None;
        self.cursor = at;
        self.goal = None;
    }

    pub fn move_line_start(&mut self, extend: bool) {
        self.start_move(extend);
        self.cursor.col = 0;
        self.goal = None;
    }

    pub fn move_line_end(&mut self, extend: bool) {
        self.start_move(extend);
        self.cursor.col = count(&self.lines[self.cursor.line]);
        self.goal = None;
    }

    pub fn move_text_start(&mut self, extend: bool) {
        self.start_move(extend);
        self.cursor = Pos::default();
        self.goal = None;
    }

    pub fn move_text_end(&mut self, extend: bool) {
        self.start_move(extend);
        let last = self.lines.len() - 1;
        self.cursor = Pos {
            line: last,
            col: count(&self.lines[last]),
        };
        self.goal = None;
    }

    pub fn move_word_left(&mut self, extend: bool) {
        self.start_move(extend);
        self.cursor = self.word_start_before(self.cursor);
        self.goal = None;
    }

    pub fn move_word_right(&mut self, extend: bool) {
        self.start_move(extend);
        self.cursor = self.word_end_after(self.cursor);
        self.goal = None;
    }

    /// Up or down one screen line in a pane `room` cells wide. At the first
    /// line, up goes to the start of the text; at the last, down goes to its
    /// end, as most editors do.
    pub fn move_vertical(&mut self, down: bool, room: usize, extend: bool) {
        self.start_move(extend);
        let visual = self.visual_lines(room);
        let here = visual_index(&visual, self.cursor.line, self.cursor.col);
        let target = if down { here + 1 } else { here.wrapping_sub(1) };
        let Some(piece) = visual.get(target).copied() else {
            if down {
                self.cursor = self.end_position();
            } else {
                self.cursor = Pos::default();
            }
            self.goal = None;
            return;
        };
        let x = self.goal.unwrap_or_else(|| self.cell_in_piece(visual[here]));
        self.cursor = self.position_at(&visual, target, piece, x);
        self.goal = Some(x);
    }

    /// The screen lines of the text in a pane `room` cells wide. They are
    /// kept until the text changes or the width does, so a paint and a run of
    /// vertical steps wrap the text once.
    pub(super) fn visual_lines(&self, room: usize) -> Arc<[VisualLine]> {
        self.wrapped.visual(self.revision(), &self.lines, room)
    }

    /// The cell column of the cursor within its screen line.
    fn cell_in_piece(&self, piece: VisualLine) -> usize {
        cells_between(&self.lines[piece.line], piece.start, self.cursor.col)
    }

    /// The position in screen line `piece` nearest cell column `x`.
    pub(super) fn position_at(&self, visual: &[VisualLine], at: usize, piece: VisualLine, x: usize) -> Pos {
        let line = &self.lines[piece.line];
        let mut used = 0;
        let mut col = piece.start;
        for grapheme in list(line)[piece.start..piece.end].iter() {
            if used + width(grapheme) > x {
                break;
            }
            used += width(grapheme);
            col += 1;
        }
        // The end of a wrapped piece is the start of the next one, so stop
        // one short of it.
        let continues = visual.get(at + 1).is_some_and(|next| next.line == piece.line);
        if continues && col >= piece.end {
            col = piece.end.saturating_sub(1).max(piece.start);
        }
        Pos { line: piece.line, col }
    }

    /// Where the cursor is on screen in a pane `room` cells wide: the screen
    /// line from the top of the text, and the cell within it.
    pub fn cursor_cell(&self, room: usize) -> (usize, usize) {
        let visual = self.visual_lines(room);
        let at = visual_index(&visual, self.cursor.line, self.cursor.col);
        (at, self.cell_in_piece(visual[at]))
    }

    fn end_position(&self) -> Pos {
        let last = self.lines.len() - 1;
        Pos {
            line: last,
            col: count(&self.lines[last]),
        }
    }

    /// The start of the word at or before `pos`, crossing line ends.
    pub(super) fn word_start_before(&self, pos: Pos) -> Pos {
        let mut at = pos;
        // Skip blanks (and line breaks) going back, then the word itself.
        while at != Pos::default() && self.grapheme_before(at).is_none_or(is_blank) {
            at = self.before(at);
        }
        while at.col > 0 && self.grapheme_before(at).is_some_and(|g| !is_blank(g)) {
            at = self.before(at);
        }
        at
    }

    /// The end of the word at or after `pos`, crossing line ends.
    pub(super) fn word_end_after(&self, pos: Pos) -> Pos {
        let mut at = pos;
        while self.grapheme_at(at).is_none_or(is_blank) && self.after(at).is_some() {
            at = self.after(at).unwrap_or(at);
        }
        while self.grapheme_at(at).is_some_and(|g| !is_blank(g)) {
            at.col += 1;
        }
        at
    }

    fn grapheme_at(&self, pos: Pos) -> Option<&str> {
        list(&self.lines[pos.line]).get(pos.col).copied()
    }

    fn grapheme_before(&self, pos: Pos) -> Option<&str> {
        if pos.col == 0 {
            return None;
        }
        list(&self.lines[pos.line]).get(pos.col - 1).copied()
    }
}

#[cfg(test)]
include!("cursor_moves.tests.rs");
