//! The text being written: lines, a cursor, a selection and its edits.
//!
//! Positions are in graphemes, so the cursor never lands inside an accented
//! letter or a joined emoji. The buffer always holds at least one line.
//! Movement is in `cursor_moves.rs` and undo in `history.rs`; screen
//! wrapping is `wrap.rs`, because it depends on the width of the pane.

use super::graphemes::{byte_of, count, list};
use super::history::{History, Kind, Snapshot};
use super::wrap_cache::WrapCache;

/// A place in the text: the line, and the grapheme within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub(super) lines: Vec<String>,
    pub(super) cursor: Pos,
    /// The other end of the selection; `None` when nothing is selected.
    pub(super) anchor: Option<Pos>,
    /// The cell column vertical movement tries to keep, across short lines.
    pub(super) goal: Option<usize>,
    pub(super) history: History,
    single_line: bool,
    /// Counts the changes made to `lines`, so a kept wrapping is known stale.
    /// Every place that changes `lines` calls `text_changed`.
    revision: u64,
    pub(super) wrapped: WrapCache,
}

impl EditorBuffer {
    /// A buffer for several lines of text.
    pub fn new(text: &str) -> Self {
        Self::build(text, false)
    }

    /// A buffer for one line, such as a recipient field: a newline typed or
    /// pasted becomes a space.
    pub fn single_line(text: &str) -> Self {
        Self::build(text, true)
    }

    fn build(text: &str, single_line: bool) -> Self {
        let mut buffer = Self {
            lines: vec![String::new()],
            cursor: Pos::default(),
            anchor: None,
            goal: None,
            history: History::default(),
            single_line,
            revision: 0,
            wrapped: WrapCache::default(),
        };
        buffer.set_text(text);
        buffer
    }

    /// Replaces everything, with the cursor at the end and no undo history.
    pub fn set_text(&mut self, text: &str) {
        let text = normalize(text, self.single_line);
        self.lines = text.split('\n').map(str::to_string).collect();
        self.text_changed();
        let last = self.lines.len() - 1;
        self.cursor = Pos {
            line: last,
            col: count(&self.lines[last]),
        };
        self.anchor = None;
        self.goal = None;
        self.history = History::default();
    }

    /// Marks the text as changed, which drops any kept wrapping.
    fn text_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// The revision of the text, which differs after every change to it.
    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor(&self) -> Pos {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub fn is_single_line(&self) -> bool {
        self.single_line
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// The selection, start before end, or `None` when nothing is selected.
    pub fn selection(&self) -> Option<(Pos, Pos)> {
        let anchor = self.anchor.filter(|anchor| *anchor != self.cursor)?;
        Some(if anchor < self.cursor { (anchor, self.cursor) } else { (self.cursor, anchor) })
    }

    pub fn selected_text(&self) -> Option<String> {
        let (from, to) = self.selection()?;
        Some(self.text_between(from, to))
    }

    pub fn select_all(&mut self) {
        self.history.break_run();
        let last = self.lines.len() - 1;
        self.anchor = Some(Pos::default());
        self.cursor = Pos {
            line: last,
            col: count(&self.lines[last]),
        };
        self.goal = None;
    }

    /// Types `text` at the cursor, replacing the selection. Newlines make new
    /// lines, or spaces in a single-line buffer. Returns whether anything
    /// changed.
    pub fn insert_text(&mut self, text: &str) -> bool {
        let text = normalize(text, self.single_line);
        if text.is_empty() && self.selection().is_none() {
            return false;
        }
        let one = count(&text) == 1 && !text.contains('\n') && self.selection().is_none();
        self.begin_edit(if one { Kind::Typing } else { Kind::Other });
        self.remove_selection();
        self.insert_raw(&text);
        self.goal = None;
        true
    }

    pub fn insert_newline(&mut self) -> bool {
        !self.single_line && self.insert_text("\n")
    }

    /// Deletes the selection, else the grapheme before the cursor, else joins
    /// the line to the one above.
    pub fn backspace(&mut self) -> bool {
        if self.selection().is_some() {
            self.begin_edit(Kind::Other);
            self.remove_selection();
        } else if self.cursor.col > 0 {
            self.begin_edit(Kind::Deleting);
            self.remove_range(self.before(self.cursor), self.cursor);
        } else if self.cursor.line > 0 {
            self.begin_edit(Kind::Other);
            self.remove_range(self.before(self.cursor), self.cursor);
        } else {
            return false;
        }
        self.goal = None;
        true
    }

    /// Deletes the selection, else the grapheme after the cursor, else joins
    /// the next line up.
    pub fn delete_forward(&mut self) -> bool {
        if self.selection().is_some() {
            self.begin_edit(Kind::Other);
            self.remove_selection();
        } else if let Some(after) = self.after(self.cursor) {
            let kind = if after.line == self.cursor.line { Kind::Deleting } else { Kind::Other };
            self.begin_edit(kind);
            self.remove_range(self.cursor, after);
        } else {
            return false;
        }
        self.goal = None;
        true
    }

    /// Deletes back to the start of the word before the cursor.
    pub fn delete_word_back(&mut self) -> bool {
        if self.selection().is_some() {
            return self.backspace();
        }
        let start = self.word_start_before(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.begin_edit(Kind::Other);
        self.remove_range(start, self.cursor);
        self.goal = None;
        true
    }

    /// Takes the selection out of the text and returns it.
    pub fn cut(&mut self) -> Option<String> {
        let text = self.selected_text()?;
        self.begin_edit(Kind::Other);
        self.remove_selection();
        self.goal = None;
        Some(text)
    }

    pub fn undo(&mut self) -> bool {
        match self.history.undo(self.snapshot()) {
            Some(snapshot) => self.restore(snapshot),
            None => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        match self.history.redo(self.snapshot()) {
            Some(snapshot) => self.restore(snapshot),
            None => false,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        }
    }

    fn restore(&mut self, snapshot: Snapshot) -> bool {
        self.lines = snapshot.lines;
        self.text_changed();
        self.cursor = snapshot.cursor;
        self.anchor = None;
        self.goal = None;
        true
    }

    fn begin_edit(&mut self, kind: Kind) {
        let before = self.snapshot();
        self.history.record(before, kind);
    }

    fn remove_selection(&mut self) {
        if let Some((from, to)) = self.selection() {
            self.remove_range(from, to);
        }
        self.anchor = None;
    }

    /// The text from `from` up to but not including `to`.
    pub(super) fn text_between(&self, from: Pos, to: Pos) -> String {
        if from.line == to.line {
            let graphemes = list(&self.lines[from.line]);
            return graphemes[from.col.min(graphemes.len())..to.col.min(graphemes.len())].concat();
        }
        let mut parts = vec![tail_from(&self.lines[from.line], from.col)];
        parts.extend(self.lines[from.line + 1..to.line].iter().cloned());
        parts.push(head_to(&self.lines[to.line], to.col));
        parts.join("\n")
    }

    fn remove_range(&mut self, from: Pos, to: Pos) {
        let head = head_to(&self.lines[from.line], from.col);
        let tail = tail_from(&self.lines[to.line], to.col);
        self.lines.splice(from.line..=to.line, [head + &tail]);
        self.text_changed();
        self.cursor = from;
        self.anchor = None;
    }

    fn insert_raw(&mut self, text: &str) {
        let line = self.cursor.line;
        let head = head_to(&self.lines[line], self.cursor.col);
        let tail = tail_from(&self.lines[line], self.cursor.col);
        let pieces: Vec<&str> = text.split('\n').collect();
        let last = pieces.len() - 1;
        let end_col = if last == 0 {
            self.cursor.col + count(pieces[0])
        } else {
            count(pieces[last])
        };
        let mut new_lines: Vec<String> = pieces.iter().map(|piece| (*piece).to_string()).collect();
        new_lines[0].insert_str(0, &head);
        new_lines[last].push_str(&tail);
        self.lines.splice(line..=line, new_lines);
        self.text_changed();
        self.cursor = Pos {
            line: line + last,
            col: end_col,
        };
    }

    /// The position one grapheme before `pos`, or the end of the line above
    /// at the start of a line.
    pub(super) fn before(&self, pos: Pos) -> Pos {
        if pos.col > 0 {
            Pos { line: pos.line, col: pos.col - 1 }
        } else if pos.line > 0 {
            Pos { line: pos.line - 1, col: count(&self.lines[pos.line - 1]) }
        } else {
            pos
        }
    }

    /// The position one grapheme after `pos`, or the start of the next line
    /// at the end of a line; `None` at the very end.
    pub(super) fn after(&self, pos: Pos) -> Option<Pos> {
        if pos.col < count(&self.lines[pos.line]) {
            Some(Pos { line: pos.line, col: pos.col + 1 })
        } else if pos.line + 1 < self.lines.len() {
            Some(Pos { line: pos.line + 1, col: 0 })
        } else {
            None
        }
    }
}

/// Line endings made plain, and newlines made spaces for a one-line buffer.
fn normalize(text: &str, single_line: bool) -> String {
    let plain = text.replace("\r\n", "\n").replace('\r', "\n");
    if single_line { plain.replace('\n', " ") } else { plain }
}

fn head_to(line: &str, col: usize) -> String {
    line[..byte_of(line, col)].to_string()
}

fn tail_from(line: &str, col: usize) -> String {
    line[byte_of(line, col)..].to_string()
}

#[cfg(test)]
include!("editor_buffer.tests.rs");
