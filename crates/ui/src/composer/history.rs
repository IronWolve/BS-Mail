//! Undo and redo for the editor: whole-buffer snapshots, which is simple and
//! cheap for the size of a message.
//!
//! A run of typing, or a run of deleting, is one step, so one undo takes back
//! a word typed and not a letter. Moving the cursor ends the run.

use super::editor_buffer::Pos;

/// The most steps kept; the oldest are dropped past this.
const LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub lines: Vec<String>,
    pub cursor: Pos,
}

/// What kind of edit this is, which decides whether it joins the run before.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Other,
    Typing,
    Deleting,
}

#[derive(Debug, Clone, Default)]
pub struct History {
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    run: Option<Kind>,
}

impl History {
    /// Records the state before an edit. An edit of the same run kind as the
    /// last one adds nothing: the snapshot at the start of the run covers it.
    pub fn record(&mut self, before: Snapshot, kind: Kind) {
        let joins = kind != Kind::Other && self.run == Some(kind);
        if !joins {
            self.undo.push(before);
            if self.undo.len() > LIMIT {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.run = Some(kind);
    }

    /// Ends the current run, so the next edit is a step of its own.
    pub fn break_run(&mut self) {
        self.run = None;
    }

    /// The state to go back to, given the current one for redo.
    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        self.run = None;
        Some(previous)
    }

    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        self.run = None;
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
}

#[cfg(test)]
include!("history.tests.rs");
