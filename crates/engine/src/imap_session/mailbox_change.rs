//! Making, renaming and deleting a mailbox: the three commands about a
//! mailbox as a whole, for the folder actions.
//!
//! Each needs an account that allows changes and a name already in the
//! server's encoding. None needs a mailbox open, and none is run while the
//! mailbox it is about is the open one: `leave_tree` first opens the Inbox
//! (for reading) in its place. The answer is read to the command's own tagged
//! line, so a server's refusal is an error with its own words.

use super::error::SessionError;
use super::session::ImapSession;
use super::write_command::WriteCommand;

/// The one mailbox every server has, which is opened (for reading only) so
/// that no other is open while it is renamed or deleted.
const INBOX: &str = "INBOX";

impl ImapSession {
    /// Makes the mailbox `path`.
    pub async fn create_mailbox(&mut self, path: &str) -> Result<(), SessionError> {
        self.send_change(&WriteCommand::CreateMailbox { path }).await.map(|_| ())
    }

    /// Renames the mailbox `from` to `to`; a server renames what is inside it
    /// too.
    pub async fn rename_mailbox(&mut self, from: &str, to: &str) -> Result<(), SessionError> {
        self.send_change(&WriteCommand::RenameMailbox { from, to }).await.map(|_| ())
    }

    /// Removes the mailbox `path` and every message in it.
    pub async fn delete_mailbox(&mut self, path: &str) -> Result<(), SessionError> {
        self.send_change(&WriteCommand::DeleteMailbox { path }).await.map(|_| ())
    }

    /// Opens the Inbox in place of the open mailbox when that is `path` or
    /// inside it (`delimiter` is the server's separator), so a mailbox is
    /// never renamed or deleted while this connection has it open. Does
    /// nothing otherwise. A lost connection is the only error.
    pub async fn leave_tree(&mut self, path: &str, delimiter: Option<&str>) -> Result<(), SessionError> {
        let inside = self.open.as_ref().is_some_and(|open| {
            open.path == path
                || delimiter.is_some_and(|sep| !sep.is_empty() && open.path.strip_prefix(path).is_some_and(|rest| rest.starts_with(sep)))
        });
        if !inside {
            return Ok(());
        }
        match self.examine(INBOX).await {
            Err(error) if error.is_lost() => Err(error),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
include!("mailbox_change.tests.rs");
