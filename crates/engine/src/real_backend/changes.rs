//! Carrying out a flag change, a move or a delete, and reporting it.
//!
//! Every change follows the same path: the message ids are resolved to exact
//! UIDs, each folder is opened for changes and its UIDVALIDITY checked against
//! the one the ids were issued under, the change is sent, and only then are
//! events produced, from what the server confirmed. A refusal or a lost
//! connection becomes a failure notice and nothing is changed locally.
//! Nothing is kept to try again later.

use model::{Command, Event, Flags, FolderId, MessageId};

use super::unsupported::{READ_ONLY_REASON, change_operation};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};
use crate::session_state::Validity;

/// Why a change did not go through because a folder is not what it was.
const FOLDER_CHANGED: &str = "the folder changed on the server, so nothing was done; refreshing";

/// Why a change that was cut off by a dropped connection may or may not have
/// been applied.
const CONNECTION_LOST: &str = "the connection was lost, so the change may not have been \
     applied; the folder is read again once it is back";

/// What a change did so far, to be reported once it has finished.
#[derive(Debug, Default)]
pub(super) struct Tally {
    /// New flags of messages, as the server reported them.
    pub updates: Vec<(MessageId, Flags)>,
    /// Folders whose contents or flags changed, each once.
    pub touched: Vec<FolderId>,
    /// Messages left or arrived in a folder: pages are read again.
    pub relocated: bool,
    /// Refusals and problems, shown as failure notices after the events.
    pub notices: Vec<String>,
    /// A folder was not what the ids said: read everything again.
    pub stale: bool,
}

impl Tally {
    pub(super) fn touch(&mut self, folder: FolderId) {
        if !self.touched.contains(&folder) {
            self.touched.push(folder);
        }
    }

    pub(super) fn notice(&mut self, text: impl Into<String>) {
        let text = text.into();
        if !self.notices.contains(&text) {
            self.notices.push(text);
        }
    }

    /// Records why a folder could not be opened; a lost connection goes on.
    pub(super) fn open_failed(&mut self, error: OpenError) -> Result<(), SessionError> {
        match error {
            OpenError::Stale => {
                self.notice(FOLDER_CHANGED);
                self.stale = true;
                Ok(())
            }
            OpenError::Session(error) if error.is_lost() => Err(error),
            OpenError::Session(error) => {
                self.notice(error.text());
                Ok(())
            }
        }
    }

    /// Ids that name no known message: refused, and everything is read again
    /// rather than guessing which message was meant.
    pub(super) fn unknown(&mut self, ids: &[MessageId]) {
        if ids.is_empty() {
            return;
        }
        let list: Vec<String> = ids.iter().map(|id| id.0.to_string()).collect();
        self.notice(format!(
            "unknown message id: {}; the folder changed, refresh",
            list.join(", ")
        ));
        self.stale = true;
    }
}

/// Why a folder could not be opened for a change.
pub(super) enum OpenError {
    /// Its UIDVALIDITY is not the one the ids were issued under.
    Stale,
    Session(SessionError),
}

impl From<SessionError> for OpenError {
    fn from(error: SessionError) -> Self {
        Self::Session(error)
    }
}

impl Worker {
    /// Serves one changing command. Only a lost connection is an error here;
    /// everything else is reported as events and failure notices.
    pub(super) async fn change(
        &mut self,
        session: &mut ImapSession,
        command: Command,
    ) -> Result<(), SessionError> {
        let operation = change_operation(&command).unwrap_or("change");
        if self.config.read_only {
            self.sink.send(self.sink.failure(operation, READ_ONLY_REASON));
            return Ok(());
        }
        let done = match command {
            Command::SetFlags { messages, change } => {
                self.set_flags(session, &messages, change).await
            }
            Command::Move { messages, to } => self.move_messages(session, &messages, to).await,
            Command::Delete { messages } => self.delete_messages(session, &messages).await,
            _ => return Ok(()),
        };
        let finished = match done {
            Ok(tally) => self.finish_change(session, operation, tally).await,
            Err(error) => Err(error),
        };
        match finished {
            Err(error) if error.is_lost() => {
                self.sink.send(self.sink.failure(operation, CONNECTION_LOST));
                Err(error)
            }
            Err(error) => {
                self.sink.send(self.sink.failure(operation, error.text()));
                Ok(())
            }
            Ok(()) => Ok(()),
        }
    }

    /// Opens a folder for changes and checks that its ids still mean what
    /// they meant.
    pub(super) async fn open_for_change(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        path: &str,
    ) -> Result<(), OpenError> {
        let info = session.select_for_write(path).await?;
        match self.state.check_validity(folder, info.uid_validity) {
            Validity::Same => Ok(()),
            Validity::Changed | Validity::Unknown => Err(OpenError::Stale),
        }
    }

    /// Sends the events of a finished change, then its failure notices.
    async fn finish_change(
        &mut self,
        session: &mut ImapSession,
        operation: &str,
        mut tally: Tally,
    ) -> Result<(), SessionError> {
        session.leave_write().await?;
        if !tally.updates.is_empty() {
            let updates = std::mem::take(&mut tally.updates);
            self.sink.send(Event::FlagsChanged { updates });
        }
        if tally.stale {
            self.resync(session, operation, &mut tally).await?;
        } else if !tally.touched.is_empty() {
            self.report_counts(session, operation, &mut tally).await?;
        }
        for notice in std::mem::take(&mut tally.notices) {
            self.sink.send(self.sink.failure(operation, notice));
        }
        Ok(())
    }

    /// Reads the folder list and the open folder again, as a sync does.
    async fn resync(
        &mut self,
        session: &mut ImapSession,
        operation: &str,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let account = self.account.clone();
        match self.sync_now(session, &account).await {
            Ok(events) => self.sink.send_all(events),
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => tally.notice(format!("{operation}: {}", error.text())),
        }
        Ok(())
    }

    /// Counts of the touched folders, then (after a move or delete) the open
    /// folder's page.
    async fn report_counts(
        &mut self,
        session: &mut ImapSession,
        operation: &str,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let moved = self.recount(session, &tally.touched).await?;
        if moved || tally.relocated {
            self.sink.send(Event::FolderList {
                account: self.account.clone(),
                folders: self.state.folders().to_vec(),
            });
        }
        let open = self.wanted.folder.filter(|folder| tally.touched.contains(folder));
        if let (true, Some(folder)) = (tally.relocated, open) {
            match self.open_folder(session, folder).await {
                Ok(page) => self.sink.send_all(page),
                Err(error) if error.is_lost() => return Err(error),
                Err(error) => tally.notice(format!("{operation}: {}", error.text())),
            }
        }
        Ok(())
    }

    /// Reads the counts of these folders from the server. True when any
    /// differs from what was held.
    pub(super) async fn recount(
        &mut self,
        session: &mut ImapSession,
        folders: &[FolderId],
    ) -> Result<bool, SessionError> {
        let mut moved = false;
        for &folder in folders {
            let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
                continue;
            };
            match session.status_counts(&path).await {
                Ok((unread, total)) => moved |= self.state.set_counts(folder, unread, total),
                Err(error) if error.is_lost() => return Err(error),
                Err(_) => {}
            }
        }
        Ok(moved)
    }
}
