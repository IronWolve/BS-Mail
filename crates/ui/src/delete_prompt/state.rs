//! What the question holds: the messages it is about and which button has
//! focus. Focus starts on the safe one.

use model::MessageId;

/// A button of the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Delete,
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePrompt {
    /// The messages the delete is about.
    pub targets: Vec<MessageId>,
    /// How many of them are removed for good (the rest go to the Trash).
    pub for_good: usize,
    pub focus: Choice,
}

impl DeletePrompt {
    /// A question about messages that are all removed for good.
    pub fn new(targets: Vec<MessageId>) -> Self {
        let for_good = targets.len();
        Self::counting(targets, for_good)
    }

    /// A question about `targets`, `for_good` of which are removed for good.
    pub fn counting(targets: Vec<MessageId>, for_good: usize) -> Self {
        Self { targets, for_good, focus: Choice::Keep }
    }

    /// How many of the messages only move to the Trash.
    pub fn to_trash(&self) -> usize {
        self.targets.len().saturating_sub(self.for_good)
    }

    /// Moves focus to the other button.
    pub fn toggle(&mut self) {
        self.focus = match self.focus {
            Choice::Delete => Choice::Keep,
            Choice::Keep => Choice::Delete,
        };
    }
}

#[cfg(test)]
include!("state.tests.rs");
