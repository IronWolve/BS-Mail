//! The text selected in the reading pane: where it started, where the pointer
//! or the arrow keys have taken it to, and which body it is on. Both ends are
//! places in the wrapped body, so it stays on the same words while the body
//! scrolls; another body (another message, a resized pane) is not the same
//! selection and shows nothing.

use crate::widgets::BodyPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSelection {
    /// Where the selection started.
    pub anchor: BodyPos,
    /// The end that moves: where the pointer is, or the arrow keys took it.
    pub focus: BodyPos,
    /// The wrapped body it is on (`BodyText::key`).
    pub key: u64,
    /// The button is still held, so moves of the pointer go on selecting.
    pub dragging: bool,
    /// The focus has left the cell the press was on. A press with no move is
    /// a click, which selects nothing.
    pub spread: bool,
}

impl TextSelection {
    /// A press at `at` on the body `key`: nothing is selected until it moves.
    pub fn pressed(at: BodyPos, key: u64) -> Self {
        Self { anchor: at, focus: at, key, dragging: true, spread: false }
    }

    /// A selection made with the keys, which starts on one cell.
    pub fn from_keys(at: BodyPos, key: u64) -> Self {
        Self { anchor: at, focus: at, key, dragging: false, spread: true }
    }

    /// Takes the focus to `to`.
    pub fn extend_to(&mut self, to: BodyPos) {
        if to != self.anchor {
            self.spread = true;
        }
        self.focus = to;
    }

    /// The first and the last place selected, both included, when something
    /// is selected on the body `key`.
    pub fn range(&self, key: u64) -> Option<(BodyPos, BodyPos)> {
        if !self.spread || self.key != key {
            return None;
        }
        Some((self.anchor.min(self.focus), self.anchor.max(self.focus)))
    }
}

#[cfg(test)]
include!("selection.tests.rs");
