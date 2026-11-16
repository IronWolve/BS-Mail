//! The tags the screen has seen on each account's mail, kept so the tag picker
//! can offer them: only the messages loaded so far tell what tags exist, and a
//! tag seen once is still offered after the list moves on.

use std::collections::BTreeSet;

use model::{AccountId, Envelope, MessageId};

use super::mail_snapshot::MailSnapshot;
use super::selection::Selection;

impl MailSnapshot {
    /// Notes the tags of `envelopes`, which belong to `account`.
    pub(crate) fn remember_tags(&mut self, account: Option<&AccountId>, envelopes: &[Envelope]) {
        let Some(account) = account else {
            return;
        };
        let seen: Vec<&String> = envelopes.iter().flat_map(|envelope| envelope.tags.iter()).collect();
        if seen.is_empty() {
            return;
        }
        let known = self.known_tags.entry(account.clone()).or_default();
        for tag in seen {
            // One spelling per tag, whatever the letter case: the first seen.
            if !known.iter().any(|have| have.eq_ignore_ascii_case(tag)) {
                known.insert(tag.clone());
            }
        }
    }

    /// The server's new tags for messages: the rows shown take them, and the
    /// tags are remembered. A change that went through ends a failure shown,
    /// as a page or a body arriving does.
    pub(super) fn apply_tags(&mut self, updates: Vec<(MessageId, Vec<String>)>, selection: &Selection) {
        for (id, tags) in updates {
            let Some(envelope) = self.page.as_mut().and_then(|page| page.envelopes.iter_mut().find(|e| e.id == id)) else {
                continue;
            };
            envelope.tags = tags;
            let kept = [envelope.clone()];
            self.remember_tags(selection.account.as_ref(), &kept);
        }
        self.clear_failure(selection.account.as_ref());
    }

    /// The tags seen on the account, by name.
    pub fn known_tags_of(&self, account: &AccountId) -> BTreeSet<String> {
        self.known_tags.get(account).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
include!("known_tags.tests.rs");
