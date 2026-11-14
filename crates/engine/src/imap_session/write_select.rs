//! Opening a mailbox for changes (SELECT) and going back to reading.
//!
//! This is the only file that sends SELECT. It refuses on its own when the
//! account is read-only, so a read-only session never opens a mailbox for
//! changes whatever the caller does.

use super::error::SessionError;
use super::select::{MailboxInfo, OpenMailbox};
use super::session::ImapSession;
use super::timeouts::bounded;

/// Why a changing operation was refused before anything was sent.
const READ_ONLY_SESSION: &str = "this account is read-only";

impl ImapSession {
    /// Refuses unless the account allows changes.
    pub(super) fn require_writes(&self) -> Result<(), SessionError> {
        if self.writes {
            Ok(())
        } else {
            Err(SessionError::Refused(READ_ONLY_SESSION.to_owned()))
        }
    }

    /// Refuses unless the open mailbox was opened for changes.
    pub(super) fn require_writable_mailbox(&self) -> Result<(), SessionError> {
        match &self.open {
            Some(open) if open.writable => Ok(()),
            _ => Err(SessionError::Refused(
                "no mailbox is open for changes".to_owned(),
            )),
        }
    }

    /// Opens `path` for changes and reports its size and UIDVALIDITY. The
    /// caller compares the validity with the one its message ids were issued
    /// under before changing anything.
    pub async fn select_for_write(&mut self, path: &str) -> Result<MailboxInfo, SessionError> {
        self.require_writes()?;
        let limit = self.limits.command;
        self.open = None;
        let result = bounded(limit, self.inner.select(path)).await;
        let found = self.settle(result)?;
        self.open = Some(OpenMailbox {
            path: path.to_owned(),
            writable: true,
        });
        Ok(MailboxInfo {
            exists: found.exists,
            uid_validity: found.uid_validity,
        })
    }

    /// Goes back to reading: a mailbox open for changes is opened again with
    /// EXAMINE. Does nothing when no mailbox is open for changes. A refusal
    /// leaves no mailbox open; only a lost connection is an error.
    pub async fn leave_write(&mut self) -> Result<(), SessionError> {
        let Some(path) = self
            .open
            .as_ref()
            .filter(|open| open.writable)
            .map(|open| open.path.clone())
        else {
            return Ok(());
        };
        match self.examine(&path).await {
            Err(error) if error.is_lost() => Err(error),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
include!("write_select.tests.rs");
