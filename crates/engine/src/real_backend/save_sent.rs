//! Saving a copy of what was sent into the Sent folder.
//!
//! This only ever follows a send whose `MessageSent` has already gone out,
//! and it never turns that send into a failure: a copy that cannot be saved
//! is reported as its own notice, `save sent copy`. An account with no Sent
//! folder has nothing to save to and says nothing. The copy is the exact
//! bytes the server accepted, appended once the mailbox is reachable.
//!
//! The copy is the only record of what went out, so it is not given up
//! lightly. It is kept on disk (see `sent_queue`) from the moment the server
//! accepted the message until the Sent folder has it. While the mailbox
//! connection is down, or when it is lost during the save, the copy waits and
//! is saved after the reconnect; a copy left by a run that ended first is
//! found again at the next start. A server that refuses the copy keeps it on
//! disk for the next start rather than trying again and again.

use std::path::PathBuf;

use super::inbox::Inbox;
use super::sent_queue::forget;
use super::worker::Worker;
use crate::imap_session::{ImapSession, SentFolder, SessionError};
use model::{Event, FolderRole};

/// The name of this notice.
const OPERATION: &str = "save sent copy";

/// A message the server accepted, waiting for its copy to be saved.
pub(super) struct SentCopy {
    pub bytes: Vec<u8>,
    /// The file keeping the copy on disk; `None` when it is held in memory
    /// only.
    pub file: Option<PathBuf>,
}

/// The queue between the send tasks and the worker, which owns the session.
pub(super) type SentCopies = Inbox<SentCopy>;

impl Worker {
    /// Appends the copy to the Sent folder when there is one. A lost
    /// connection is the only error, and the copy then waits for the
    /// reconnect.
    pub(super) async fn save_sent(
        &mut self,
        session: &mut ImapSession,
        copy: SentCopy,
    ) -> Result<(), SessionError> {
        let folder = self.state.folders().iter().find(|f| f.role == FolderRole::Sent).cloned();
        let Some((folder, target)) = folder.and_then(|f| SentFolder::from_folder(&f).map(|t| (f, t))) else {
            drop_file(&copy);
            return Ok(());
        };
        match session.append_sent(&target, &copy.bytes).await {
            Ok(()) => {
                drop_file(&copy);
                self.report_new_copy(session, folder.id).await
            }
            Err(error) if error.is_lost() => {
                self.sink.send(self.sink.failure(
                    OPERATION,
                    "the connection was lost, so the copy in the Sent folder may not have been saved; \
                     it is kept and will be saved once the connection is back",
                ));
                self.unsaved.push(copy);
                Err(error)
            }
            Err(error) => {
                let kept = if copy.file.is_some() { "; it is kept, and the next start tries again" } else { "" };
                let reason = format!(
                    "the message was sent, but its copy could not be saved in {}: {}{kept}",
                    folder.name,
                    error.text()
                );
                self.sink.send(self.sink.failure(OPERATION, reason));
                Ok(())
            }
        }
    }

    /// Saves the copies that waited for the mailbox: those made while it was
    /// out of reach and those left by an earlier run. An account that may not
    /// change the mailbox saves nothing, and the copies stay where they are.
    pub(super) async fn save_unsaved(&mut self, session: &mut ImapSession) -> Result<(), SessionError> {
        if self.config.read_only {
            return Ok(());
        }
        let mut waiting = std::mem::take(&mut self.unsaved).into_iter();
        while let Some(copy) = waiting.next() {
            if let Err(error) = self.save_sent(session, copy).await {
                self.unsaved.extend(waiting);
                return Err(error);
            }
        }
        Ok(())
    }

    /// Tells the screen the Sent folder has one more message.
    async fn report_new_copy(
        &mut self,
        session: &mut ImapSession,
        folder: model::FolderId,
    ) -> Result<(), SessionError> {
        if self.recount(session, &[folder]).await? {
            self.sink.send(Event::FolderList {
                account: self.account.clone(),
                folders: self.state.folders().to_vec(),
            });
        }
        Ok(())
    }

    /// The copy cannot be saved yet: the mailbox connection is down. It waits
    /// for the reconnect. An account known to have no Sent folder has nothing
    /// to save to and hears nothing.
    pub(super) fn sent_copy_offline(&mut self, copy: SentCopy) {
        let folders = self.state.folders();
        if !folders.is_empty() && !folders.iter().any(|f| f.role == FolderRole::Sent) {
            drop_file(&copy);
            return;
        }
        let later = if copy.file.is_some() {
            "its copy is kept and will be saved in the Sent folder once the connection is back"
        } else {
            "its copy will be saved in the Sent folder once the connection is back, unless the program is closed first"
        };
        self.unsaved.push(copy);
        self.sink.send(self.sink.failure(
            OPERATION,
            format!("the message was sent, but this program is not connected to the mailbox, so {later}"),
        ));
    }
}

/// The copy is done with: take it off the disk.
fn drop_file(copy: &SentCopy) {
    if let Some(file) = &copy.file {
        forget(file);
    }
}
