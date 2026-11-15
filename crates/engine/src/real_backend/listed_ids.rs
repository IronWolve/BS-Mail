//! The thread ids of the messages the lists have shown, read from their
//! envelopes when a page arrives: the message's own `Message-ID`, and its
//! `In-Reply-To` as the one earlier id it names.
//!
//! This is the fallback for a reply whose original is not among the few
//! opened lately and cannot be looked up on the server (the mailbox is down,
//! the folder was rebuilt, the message was moved away). The ids are enough to
//! keep the reply in its conversation; the `References` of the original
//! itself, which only a look-up gives, is the part it lacks. A message id is
//! never given to a second message (see `session_state`), so what is kept
//! under an id cannot go stale.

use std::collections::{HashMap, VecDeque};

use model::MessageId;

use crate::send::ReplyIds;

/// How many messages are remembered: the ones listed most recently.
const KEPT: usize = 2000;

#[derive(Debug, Default)]
pub(super) struct ListedIds {
    ids: HashMap<MessageId, ReplyIds>,
    /// The messages in the order they were first kept, oldest first.
    order: VecDeque<MessageId>,
}

impl ListedIds {
    /// Remembers what a list row said, replacing an earlier entry. A row
    /// with no usable `Message-ID` is not kept: there is nothing to name.
    pub(super) fn keep(&mut self, id: MessageId, ids: Option<ReplyIds>) {
        let Some(ids) = ids else {
            return;
        };
        if self.ids.insert(id, ids).is_none() {
            self.order.push_back(id);
        }
        while self.order.len() > KEPT {
            if let Some(oldest) = self.order.pop_front() {
                self.ids.remove(&oldest);
            }
        }
    }

    /// What the list said about this message, if it is still remembered.
    pub(super) fn get(&self, id: MessageId) -> Option<ReplyIds> {
        self.ids.get(&id).cloned()
    }
}

#[cfg(test)]
include!("listed_ids.tests.rs");
