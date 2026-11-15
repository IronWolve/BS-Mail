//! The message ids of sends that did not end well, so that sending the same
//! message again is one message to everyone who sees both attempts.
//!
//! An attempt is remembered from the moment it starts until the server accepts
//! the message. If it ends in a failure, or in "may or may not have been sent",
//! the same message sent again gets the same id. Once a message has been
//! accepted it is forgotten, so sending the same words to the same people on
//! purpose is a new message and is never taken for a copy of the first.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// How many unfinished sends are remembered.
const KEPT: usize = 8;

#[derive(Debug, Default)]
struct Entries(VecDeque<(u64, String)>);

/// Shared between the worker and the send tasks.
#[derive(Debug, Clone, Default)]
pub(super) struct RetryIds(Arc<Mutex<Entries>>);

impl RetryIds {
    /// The id of an earlier unfinished attempt at the message with this
    /// fingerprint, if there was one.
    pub(super) fn earlier(&self, key: u64) -> Option<String> {
        let entries = self.0.lock().ok()?;
        entries.0.iter().find(|(known, _)| *known == key).map(|(_, id)| id.clone())
    }

    /// An attempt at this message is under way with this id.
    pub(super) fn begin(&self, key: u64, id: &str) {
        let Ok(mut entries) = self.0.lock() else {
            return;
        };
        entries.0.retain(|(known, _)| *known != key);
        entries.0.push_back((key, id.to_owned()));
        while entries.0.len() > KEPT {
            entries.0.pop_front();
        }
    }

    /// The server accepted the message: it is not an unfinished send.
    pub(super) fn accepted(&self, key: u64) {
        if let Ok(mut entries) = self.0.lock() {
            entries.0.retain(|(known, _)| *known != key);
        }
    }
}

#[cfg(test)]
include!("retry_ids.tests.rs");
