//! The cheap question the automatic check asks: how many messages, how many
//! unread, and what UID the next arrival will get. STATUS is a read and does
//! not open the mailbox.

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// What STATUS says about one mailbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCounts {
    pub messages: u32,
    pub unseen: u32,
    /// The UID the next message will get; `None` when the server left it out.
    pub uid_next: Option<u32>,
}

impl ImapSession {
    /// The counts of the mailbox named `path`.
    pub async fn status_for_check(&mut self, path: &str) -> Result<StatusCounts, SessionError> {
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.status(path, "(MESSAGES UNSEEN UIDNEXT)")).await;
        let found = self.settle(result)?;
        Ok(StatusCounts {
            messages: found.exists,
            unseen: found.unseen.unwrap_or(0),
            uid_next: found.uid_next,
        })
    }
}
