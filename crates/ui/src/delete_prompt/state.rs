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
    /// The messages that will be removed for good.
    pub targets: Vec<MessageId>,
    pub focus: Choice,
}

impl DeletePrompt {
    pub fn new(targets: Vec<MessageId>) -> Self {
        Self { targets, focus: Choice::Keep }
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
