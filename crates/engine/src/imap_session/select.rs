//! Opening a mailbox. Always EXAMINE, never SELECT: an examined mailbox is
//! read-only on the server, so this session physically cannot change a flag.

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// What EXAMINE reports about a mailbox.
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
        self.examined = None;
        let result = bounded(limit, self.inner.examine(path)).await;
        let found = self.settle(result)?;
        self.examined = Some(path.to_owned());
        Ok(MailboxInfo {
            exists: found.exists,
            uid_validity: found.uid_validity,
        })
    }

    /// Opens `path` unless it is the mailbox already open. Returns what
    /// EXAMINE reported when it had to open it.
    pub async fn ensure_examined(
        &mut self,
        path: &str,
    ) -> Result<Option<MailboxInfo>, SessionError> {
        if self.examined.as_deref() == Some(path) {
            return Ok(None);
        }
        self.examine(path).await.map(Some)
    }
}
