//! The automatic check for new mail. Every `poll_seconds` the worker asks
//! the server for the counts of the open folder (and of the Inbox when that
//! is another folder, or when no folder is open at all: an account the screen
//! is not showing still has its Inbox looked at, so the account list can show
//! its unread count) with STATUS, which only reads. When something differs
//! from the last look it sends what a sync of the open folder sends: the
//! folder list with fresh counts, the newest page with the same ids for the
//! messages already known, and the sync status with the number of arrivals.
//! A look that finds nothing changed sends nothing at all. Counts do not say
//! when another program stars or answers a message, so a look that sees no
//! change in them also asks for the flags of the newest messages the screen
//! knows in the open folder, and of the next slice of the older mail loaded
//! below them (every look takes the next slice, so all the loaded rows come
//! round), and sends the ones that differ.
//!
//! Every fifth look also reads the counts of the whole folder list again,
//! quietly: it sends the list only if something in it differs.
//!
//! A lost connection is handed back to the supervisor like for any command,
//! so the reconnect rules apply and no failure notice is sent for it. A
//! refusal (a folder deleted meanwhile, say) is ignored; the next look tries
//! again.

use std::time::Duration;

use model::{Event, FolderId, FolderRole, SyncState, Uid};

use super::handlers::FOLDER_REBUILT;
use super::lifecycle::{now_unix, progress};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError, UidSet};

/// How many of the newest messages of the open folder have their flags looked
/// at in each check.
const FLAG_LOOK_ROWS: usize = 100;

/// How many more, taken from the older mail loaded below the newest ones, are
/// looked at in each check. Every look takes the next slice of them, so all
/// the loaded rows come round without any one look growing past
/// `FLAG_LOOK_ROWS + FLAG_SWEEP_ROWS` messages.
const FLAG_SWEEP_ROWS: usize = 100;

/// What one look found.
#[derive(Debug, Default)]
struct Found {
    /// The folders whose counts changed.
    changed: Vec<FolderId>,
    /// Messages that arrived in them.
    arrived: u32,
}

impl Worker {
    /// The wait between two looks, or `None` when they are switched off.
    pub(super) fn poll_period(&self) -> Option<Duration> {
        if self.config.poll_seconds == 0 {
            return None;
        }
        let every = self
            .timing
            .poll
            .unwrap_or_else(|| Duration::from_secs(u64::from(self.config.poll_seconds)));
        Some(every.max(Duration::from_millis(1)))
    }

    /// One look. Only a lost connection is an error.
    pub(super) async fn poll_check(&mut self, session: &mut ImapSession) -> Result<(), SessionError> {
        let open = self.wanted.folder;
        let full = self.poll.next_check();
        let found = self.look_at_counts(session, open).await?;
        let mut list_changed = !found.changed.is_empty();
        if full {
            list_changed |= self.reread_counts(session).await?;
        }
        let mut events = Vec::new();
        if list_changed {
            events.push(Event::FolderList {
                account: self.account.clone(),
                folders: self.state.folders().to_vec(),
            });
        }
        if let Some(open) = open.filter(|_| !self.search_shown) {
            if found.changed.contains(&open) {
                events.extend(self.refreshed_page(session, open).await?);
            } else {
                events.extend(self.look_at_flags(session, open).await?);
            }
        }
        if !events.is_empty() {
            self.poll.add_arrived(found.arrived);
            let now = now_unix();
            self.last_sync = Some(now);
            events.push(progress(&self.sink, SyncState::Idle, Some(now), None, self.poll.arrived()));
            self.sink.send_all(events);
        }
        Ok(())
    }

    /// Asks STATUS about the open folder, when there is one, and the Inbox,
    /// keeps the counts in the folder list current, and says which folders
    /// changed.
    async fn look_at_counts(&mut self, session: &mut ImapSession, open: Option<FolderId>) -> Result<Found, SessionError> {
        let mut targets: Vec<FolderId> = open.into_iter().collect();
        let inbox = self.state.folders().iter().find(|f| f.role == FolderRole::Inbox).map(|f| f.id);
        targets.extend(inbox.filter(|id| Some(*id) != open));
        let mut found = Found::default();
        for id in targets {
            let Some((path, listed)) = self.state.folder(id).map(|f| (f.path.clone(), (f.unread, f.total))) else {
                continue;
            };
            let counts = match session.status_for_check(&path).await {
                Ok(counts) => counts,
                Err(error) if error.is_lost() => return Err(error),
                Err(_) => continue,
            };
            if let Some(arrived) = self.poll.observe(&path, counts, listed) {
                self.state.set_counts(id, counts.unseen, counts.messages);
                found.changed.push(id);
                found.arrived = found.arrived.saturating_add(arrived);
            }
        }
        Ok(found)
    }

    /// Asks for the flags of the newest messages known in the open folder,
    /// and of a slice of the older ones loaded below them, and says which
    /// differ from what was known: another program may have starred or
    /// answered them. A refusal gives no events. A folder that was rebuilt in
    /// the meantime gives its newest page again, with fresh ids.
    async fn look_at_flags(&mut self, session: &mut ImapSession, folder: FolderId) -> Result<Vec<Event>, SessionError> {
        let asked = self.poll.flag_uids(folder, FLAG_LOOK_ROWS, FLAG_SWEEP_ROWS);
        let Some(set) = UidSet::new(asked.iter().copied()) else {
            return Ok(Vec::new());
        };
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(Vec::new());
        };
        let reopened = match session.ensure_examined(&path).await {
            Ok(reopened) => reopened,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => return Ok(Vec::new()),
        };
        if reopened.is_some_and(|info| self.note_validity(folder, info.uid_validity)) {
            return self.replace_page(session, folder, &path, "check for new mail").await;
        }
        let rows = match session.read_flags(&set).await {
            Ok(rows) => rows,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => return Ok(Vec::new()),
        };
        let changed = self.poll.changed_flags(folder, &asked, rows);
        if changed.is_empty() {
            return Ok(Vec::new());
        }
        let updates = changed.into_iter().map(|(uid, flags)| (self.state.message_id(folder, Uid(uid)), flags)).collect();
        Ok(vec![Event::FlagsChanged { updates }])
    }

    /// The open folder was rebuilt on the server, so every id the screen holds
    /// for it is void. Its newest page is read again, which hands out fresh
    /// ids and replaces the list. When that is refused the person is told to
    /// refresh, since nothing else would say so, under the name `operation`.
    pub(super) async fn replace_page(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        path: &str,
        operation: &str,
    ) -> Result<Vec<Event>, SessionError> {
        match self.read_first_page(session, folder, path).await {
            Ok(events) => Ok(events),
            Err(error) if error.is_lost() => Err(error),
            Err(_) => Ok(vec![self.sink.failure(operation, FOLDER_REBUILT)]),
        }
    }

    /// The newest page of the open folder, as a sync reads it. A refusal
    /// gives no events.
    async fn refreshed_page(&mut self, session: &mut ImapSession, folder: FolderId) -> Result<Vec<Event>, SessionError> {
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(Vec::new());
        };
        match self.read_first_page(session, folder, &path).await {
            Ok(events) => Ok(events),
            Err(error) if error.is_lost() => Err(error),
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Reads the counts of every folder again. True when the folder list
    /// differs from the one held, which is then replaced.
    async fn reread_counts(&mut self, session: &mut ImapSession) -> Result<bool, SessionError> {
        let folders = match self.read_tree(session).await {
            Ok(folders) => folders,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => return Ok(false),
        };
        if folders == self.state.folders() {
            return Ok(false);
        }
        self.state.set_folders(folders);
        Ok(true)
    }
}

#[cfg(test)]
include!("poll.tests.rs");
