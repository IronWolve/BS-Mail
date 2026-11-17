//! The link the keys are on: `l` and `L` choose one of the open message's
//! links, and the reader then reads `Enter` as "open it" and `Esc` as "let it
//! go". It belongs to one message and is let go when another is opened.

use model::MessageId;

/// The chosen link of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCursor {
    /// The message whose body holds the link.
    pub message: MessageId,
    /// Its place among that body's links (see `LinkHit`), which is what is
    /// lit and what the pointer's light is told apart from.
    pub index: usize,
    /// Its whole address, kept so `Enter` and the footer need not look for it.
    pub address: String,
}

impl LinkCursor {
    /// Whether the cursor is on a link of `message`.
    pub fn is_on(&self, message: Option<MessageId>) -> bool {
        Some(self.message) == message
    }
}

#[cfg(test)]
include!("link_cursor.tests.rs");
