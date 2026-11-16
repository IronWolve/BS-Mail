//! Carrying out a flag change, a move, a delete, a tag, a folder action or a
//! change to every message of a folder, and reporting it.
//!
//! Every change follows the same path: the message ids are resolved to exact
//! UIDs, each folder is opened for changes and its UIDVALIDITY checked against
//! the one the ids were issued under, the change is sent, and only then are
//! events produced, from what the server confirmed. A refusal or a lost
//! connection becomes a failure notice and nothing is changed locally.
//! Nothing is kept to try again later. A connection lost partway says which
//! steps the server had finished (see `change_lost`).

use model::{Command, Event, Flags, FolderId, MessageId};

use super::unsupported::{READ_ONLY_REASON, change_operation};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};
use crate::session_state::Validity;

/// Why a change did not go through because a folder is not what it was.
const FOLDER_CHANGED: &str = "the folder changed on the server, so nothing was done; refreshing";

/// What a change did so far, to be reported once it has finished.
#[derive(Debug, Default)]
pub(super) struct Tally {
    /// New flags of messages, as the server reported them.
    pub updates: Vec<(MessageId, Flags)>,
    /// New tags of messages, as the server reported them.
    pub tags: Vec<(MessageId, Vec<String>)>,
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
    /// True when the server is known to have taken part of the change: a
    /// flag it confirmed, or messages moved or removed.
    pub(super) fn landed(&self) -> bool {
        self.relocated || !self.updates.is_empty()
    }

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
        let mut tally = Tally::default();
        let done = match command {
            Command::SetFlags { messages, change } => {
                self.set_flags(session, &messages, change, &mut tally).await
            }
            Command::Move { messages, to } => self.move_messages(session, &messages, to, &mut tally).await,
            Command::Delete { messages } => self.delete_messages(session, &messages, &mut tally).await,
            Command::SetTag { messages, tag, on } => self.set_tag(session, &messages, (&tag, on), &mut tally).await,
            Command::CreateFolder { account, parent, name } => {
                self.create_folder(session, (&account, parent, &name), &mut tally).await
            }
            Command::RenameFolder { folder, name } => self.rename_folder(session, folder, &name, &mut tally).await,
            Command::DeleteFolder { folder } => self.delete_folder(session, folder, &mut tally).await,
            Command::MarkAllRead { folder } => self.mark_all_read(session, folder, &mut tally).await,
            Command::EmptyFolder { folder } => self.empty_folder(session, folder, &mut tally).await,
            _ => return Ok(()),
        };
        let finished = match done {
            Ok(()) => match self.finish_change(session, operation, &mut tally).await {
                Err(error) if error.is_lost() => Err(self.lost_after_change(operation, tally, error)),
                other => other,
            },
            Err(error) if error.is_lost() => Err(self.lost_during_change(operation, tally, error)),
            Err(error) => Err(error),
        };
        match finished {
            Err(error) if error.is_lost() => Err(error),
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
            Validity::Changed | Validity::Unknown => {
                self.poll.forget_flags(folder);
                Err(OpenError::Stale)
            }
        }
    }

    /// Sends the events of a finished change, then its failure notices.
    async fn finish_change(
        &mut self,
        session: &mut ImapSession,
        operation: &str,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        session.leave_write().await?;
        self.send_flag_updates(tally);
        if tally.stale {
            self.resync(session, operation, tally).await?;
        } else if !tally.touched.is_empty() {
            self.report_counts(session, operation, tally).await?;
        }
        for notice in std::mem::take(&mut tally.notices) {
            self.sink.send(self.sink.failure(operation, notice));
        }
        Ok(())
    }

    /// Tells the screen the tags the server confirmed.
    fn send_tag_updates(&mut self, tally: &mut Tally) {
        if !tally.tags.is_empty() {
            self.sink.send(Event::TagsChanged { updates: std::mem::take(&mut tally.tags) });
        }
    }

    /// Tells the screen the flags and tags the server confirmed.
    pub(super) fn send_flag_updates(&mut self, tally: &mut Tally) {
        self.send_tag_updates(tally);
        if tally.updates.is_empty() {
            return;
        }
        let updates = std::mem::take(&mut tally.updates);
        // Its own changes are not something that happened elsewhere.
        for (id, flags) in &updates {
            if let Some((folder, uid)) = self.state.message_key(*id) {
                self.poll.note_flags(folder, [(uid.0, *flags)]);
            }
        }
        self.sink.send(Event::FlagsChanged { updates });
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
                Ok((unread, total)) => {
                    moved |= self.state.set_counts(folder, unread, total);
                    self.poll.note_counts(&path, unread, total);
                }
                Err(error) if error.is_lost() => return Err(error),
                Err(_) => {}
            }
        }
        Ok(moved)
    }
}
