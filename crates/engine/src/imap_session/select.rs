//! Opening a mailbox for reading. Always EXAMINE: an examined mailbox is
//! read-only on the server, so reading can never change a flag. SELECT, which
//! opens a mailbox for changes, lives only in `write_select.rs`.

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// The mailbox the session has open, and whether it was opened for changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenMailbox {
    pub path: String,
    pub writable: bool,
}

/// What EXAMINE or SELECT reports about a mailbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailboxInfo {
    /// Messages in the mailbox right now.
    pub exists: u32,
    pub uid_validity: Option<u32>,
}

impl ImapSession {
    /// Opens `path` read-only and reports its size.
    pub async fn examine(&mut self, path: &str) -> Result<MailboxInfo, SessionError> {
        let limit = self.limits.command;
        self.open = None;
        let result = bounded(limit, self.inner.examine(path)).await;
        let found = self.settle(result)?;
        self.open = Some(OpenMailbox {
            path: path.to_owned(),
            writable: false,
        });
        Ok(MailboxInfo {
            exists: found.exists,
            uid_validity: found.uid_validity,
        })
    }

    /// Opens `path` for reading unless it is already open that way. A
    /// mailbox that is open for changes is opened again with EXAMINE, so
    /// reads never run in a changing session. Returns what EXAMINE reported
    /// when it had to open it.
    pub async fn ensure_examined(
        &mut self,
        path: &str,
    ) -> Result<Option<MailboxInfo>, SessionError> {
        let reading_it = self
            .open
            .as_ref()
            .is_some_and(|open| open.path == path && !open.writable);
        if reading_it {
            return Ok(None);
        }
        self.examine(path).await.map(Some)
    }
}
