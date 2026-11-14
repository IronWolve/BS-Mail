//! Saving a copy of what was sent into the Sent folder.
//!
//! This only ever follows a send whose `MessageSent` has already gone out,
//! and it never turns that send into a failure: a copy that cannot be saved
//! is reported as its own notice, `save sent copy`. An account with no Sent
//! folder has nothing to save to and says nothing. The copy is the exact
//! bytes the server accepted, appended once; it is not kept for later, so a
//! copy that cannot be saved now (the mailbox connection is down) is not
//! saved after a reconnect.

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::worker::Worker;
use crate::imap_session::{ImapSession, SentFolder, SessionError};
use model::{Event, FolderRole};

/// The name of this notice.
const OPERATION: &str = "save sent copy";

/// A message the server accepted, waiting for its copy to be saved.
pub(super) struct SentCopy {
    pub bytes: Vec<u8>,
}

/// The queue between the send tasks and the worker, which owns the session.
pub(super) struct SentCopies {
    sender: UnboundedSender<SentCopy>,
    receiver: UnboundedReceiver<SentCopy>,
}

impl SentCopies {
    pub(super) fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self { sender, receiver }
    }

    pub(super) fn sender(&self) -> UnboundedSender<SentCopy> {
        self.sender.clone()
    }

    /// The next copy to save. The queue never ends while the worker lives,
    /// because the worker holds a sender itself.
    pub(super) async fn recv(&mut self) -> Option<SentCopy> {
        self.receiver.recv().await
    }
}

impl Worker {
    /// Appends the copy to the Sent folder when there is one. Only a lost
    /// connection is an error.
    pub(super) async fn save_sent(
        &mut self,
        session: &mut ImapSession,
        copy: SentCopy,
    ) -> Result<(), SessionError> {
        let folder = self.state.folders().iter().find(|f| f.role == FolderRole::Sent).cloned();
        let Some((folder, target)) = folder.and_then(|f| SentFolder::from_folder(&f).map(|t| (f, t))) else {
            return Ok(());
        };
        match session.append_sent(&target, &copy.bytes).await {
            Ok(()) => self.report_new_copy(session, folder.id).await,
            Err(error) if error.is_lost() => {
                self.sink.send(self.sink.failure(
                    OPERATION,
                    "the connection was lost, so the copy in the Sent folder may not have been saved",
                ));
                Err(error)
            }
            Err(error) => {
                let reason = format!(
                    "the message was sent, but its copy could not be saved in {}: {}",
                    folder.name,
                    error.text()
                );
                self.sink.send(self.sink.failure(OPERATION, reason));
                Ok(())
            }
        }
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

    /// The copy cannot be saved: the mailbox connection is down. An account
    /// known to have no Sent folder has nothing to save to and hears nothing.
    pub(super) fn sent_copy_offline(&self) {
        let folders = self.state.folders();
        if !folders.is_empty() && !folders.iter().any(|f| f.role == FolderRole::Sent) {
            return;
        }
        self.sink.send(self.sink.failure(
            OPERATION,
            "the message was sent, but this program is not connected to the mailbox, so its copy \
             was not saved in the Sent folder",
        ));
    }
}
