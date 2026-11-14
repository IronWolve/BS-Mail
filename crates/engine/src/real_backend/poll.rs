//! The automatic check for new mail. Every `poll_seconds` the worker asks
//! the server for the counts of the open folder (and of the Inbox when that
//! is another folder) with STATUS, which only reads. When something differs
//! from the last look it sends what a sync of the open folder sends: the
//! folder list with fresh counts, the newest page with the same ids for the
//! messages already known, and the sync status with the number of arrivals.
//! A look that finds nothing changed sends nothing at all.
//!
//! Every fifth look also reads the counts of the whole folder list again,
//! quietly: it sends the list only if something in it differs.
//!
//! A lost connection is handed back to the supervisor like for any command,
//! so the reconnect rules apply and no failure notice is sent for it. A
//! refusal (a folder deleted meanwhile, say) is ignored; the next look tries
//! again.

use std::time::Duration;

use model::{Event, FolderId, FolderRole, SyncState};

use super::lifecycle::{now_unix, progress};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

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
        let Some(open) = self.wanted.folder else {
            return Ok(());
        };
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
        if found.changed.contains(&open) && !self.search_shown {
            events.extend(self.refreshed_page(session, open).await?);
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

    /// Asks STATUS about the open folder and the Inbox, keeps the counts in
    /// the folder list current, and says which folders changed.
    async fn look_at_counts(&mut self, session: &mut ImapSession, open: FolderId) -> Result<Found, SessionError> {
        let mut targets = vec![open];
        let inbox = self.state.folders().iter().find(|f| f.role == FolderRole::Inbox).map(|f| f.id);
        targets.extend(inbox.filter(|id| *id != open));
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
