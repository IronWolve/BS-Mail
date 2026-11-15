//! What the automatic check for new mail remembers between two looks, and
//! how it tells that something changed.

use std::collections::HashMap;

use model::{Flags, FolderId};

use crate::imap_session::StatusCounts;

/// Every this many checks, the counts of the whole folder list are read
/// again too.
pub(super) const FULL_REFRESH_EVERY: u32 = 5;

/// The last STATUS answer of each folder looked at, the number of checks
/// made, and how many messages have arrived since the program started.
#[derive(Debug, Default)]
pub(super) struct PollState {
    known: HashMap<String, StatusCounts>,
    checks: u32,
    arrived: u32,
    /// The flags last known for the messages of each folder that was shown,
    /// by UID: what a page said, what the program itself changed, what the
    /// last look found. Another program can change a flag without changing
    /// any count STATUS reports, so the flags are asked for and compared.
    flags: HashMap<FolderId, HashMap<u32, Flags>>,
}

impl PollState {
    /// Counts one more check. True when it is one that also reads the whole
    /// folder list again.
    pub(super) fn next_check(&mut self) -> bool {
        self.checks += 1;
        self.checks % FULL_REFRESH_EVERY == 0
    }

    /// Takes in the flags of messages the screen was just given (a page) or
    /// that the program itself changed, so they are not mistaken later for a
    /// change made elsewhere.
    pub(super) fn note_flags(&mut self, folder: FolderId, rows: impl IntoIterator<Item = (u32, Flags)>) {
        self.flags.entry(folder).or_default().extend(rows);
    }

    /// The UIDs whose flags are looked at: the newest `most` known in the
    /// folder.
    pub(super) fn flag_uids(&self, folder: FolderId, most: usize) -> Vec<u32> {
        let mut uids: Vec<u32> = self.flags.get(&folder).map(|known| known.keys().copied().collect()).unwrap_or_default();
        uids.sort_unstable_by(|a, b| b.cmp(a));
        uids.truncate(most);
        uids
    }

    /// Compares what the server says now (`now`, for the UIDs asked about)
    /// with what was known, and keeps the new flags. Returns the messages
    /// whose flags differ. A message with nothing known before is only
    /// remembered, and one that is gone from the answer is forgotten.
    pub(super) fn changed_flags(&mut self, folder: FolderId, asked: &[u32], now: Vec<(u32, Flags)>) -> Vec<(u32, Flags)> {
        let known = self.flags.entry(folder).or_default();
        let present: Vec<u32> = now.iter().map(|(uid, _)| *uid).collect();
        known.retain(|uid, _| !asked.contains(uid) || present.contains(uid));
        let mut changed = Vec::new();
        for (uid, flags) in now {
            if let Some(before) = known.insert(uid, flags)
                && before != flags
            {
                changed.push((uid, flags));
            }
        }
        changed.sort_by_key(|(uid, _)| *uid);
        changed
    }

    /// Messages that arrived since the program started.
    pub(super) const fn arrived(&self) -> u32 {
        self.arrived
    }

    pub(super) fn add_arrived(&mut self, count: u32) {
        self.arrived = self.arrived.saturating_add(count);
    }

    /// Notes what STATUS says now about the folder at `path`, whose entry in
    /// the folder list says `listed` as `(unread, total)`. Returns `None`
    /// when nothing changed since the last look, else how many messages
    /// arrived.
    ///
    /// The first look at a folder has no earlier STATUS to compare with, so
    /// it compares with the folder list instead.
    pub(super) fn observe(&mut self, path: &str, now: StatusCounts, listed: (u32, u32)) -> Option<u32> {
        match self.known.insert(path.to_owned(), now) {
            Some(before) if before == now => None,
            Some(before) => Some(arrived_since(&before, &now)),
            None if (now.unseen, now.messages) == listed => None,
            None => Some(now.messages.saturating_sub(listed.1)),
        }
    }

    /// Takes in counts the program itself changed (marking mail, moving it),
    /// so its own work is not mistaken for something that happened on the
    /// server. A message that arrived in the folder used one UID.
    pub(super) fn note_counts(&mut self, path: &str, unseen: u32, messages: u32) {
        let Some(known) = self.known.get_mut(path) else {
            return;
        };
        let grown = messages.saturating_sub(known.messages);
        known.uid_next = known.uid_next.map(|next| next.saturating_add(grown));
        known.messages = messages;
        known.unseen = unseen;
    }
}

/// How many messages arrived between two looks: the UIDs handed out since,
/// never more than the messages there are now. Without UIDs, the growth.
fn arrived_since(before: &StatusCounts, now: &StatusCounts) -> u32 {
    match (before.uid_next, now.uid_next) {
        (Some(was), Some(is)) => is.saturating_sub(was).min(now.messages),
        _ => now.messages.saturating_sub(before.messages),
    }
}

#[cfg(test)]
include!("poll_state.tests.rs");
