//! The thread ids of messages that were opened, kept so that a reply to one
//! of them needs no look-up on the server. That matters most while the
//! mailbox connection is down: a reply written then still carries
//! `In-Reply-To` and `References`, and so stays in its conversation for the
//! people who get it.
//!
//! A message id is never given to a second message (see `session_state`), so
//! what is kept under an id cannot go stale.

use std::collections::VecDeque;

use model::MessageId;

use crate::send::ReplyIds;

/// How many messages are remembered: the ones opened most recently.
const KEPT: usize = 64;

/// What was found for one message: its ids, or the fact that it has no
/// usable `Message-ID`, which is as good as an answer.
type Found = Option<ReplyIds>;

#[derive(Debug, Default)]
pub(super) struct ThreadCache(VecDeque<(MessageId, Found)>);

impl ThreadCache {
    /// What is known about this message: `None` when nothing is, so it has
    /// to be looked up; `Some(None)` when it is known to have no ids.
    pub(super) fn get(&self, id: MessageId) -> Option<Found> {
        self.0.iter().find(|(known, _)| *known == id).map(|(_, found)| found.clone())
    }

    /// Remembers what was found for a message, replacing an earlier entry.
    pub(super) fn keep(&mut self, id: MessageId, found: Found) {
        self.0.retain(|(known, _)| *known != id);
        self.0.push_back((id, found));
        while self.0.len() > KEPT {
            self.0.pop_front();
        }
    }
}

#[cfg(test)]
include!("thread_cache.tests.rs");
