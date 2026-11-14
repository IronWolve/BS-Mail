//! Moving and deleting messages: the three sequences that remove mail from a
//! mailbox, each on an explicit list of UIDs in the mailbox open for changes.
//!
//! - A native move: `UID MOVE`.
//! - A move by copy: `UID COPY`, and only once that succeeded `\Deleted` on
//!   the same messages and `UID EXPUNGE` of exactly them.
//! - A permanent delete: `\Deleted` and `UID EXPUNGE` of exactly the same
//!   messages.
//!
//! The extension a sequence needs (MOVE, or UIDPLUS for expunging by UID) is
//! checked before its first command, so a server that cannot finish the
//! sequence safely is refused before anything is marked or copied.

use super::error::SessionError;
use super::session::ImapSession;
use super::uid_set::UidSet;
use super::write_command::{Needs, WriteCommand};

/// A sequence that stopped, and whether the server may already have changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFailure {
    pub error: SessionError,
    /// True when an earlier step of the sequence went through, so the server
    /// holds a copy or a deleted mark that was not cleaned up.
    pub changed: bool,
}

impl StepFailure {
    fn clean(error: SessionError) -> Self {
        Self { error, changed: false }
    }

    fn dirty(error: SessionError, context: &str) -> Self {
        let text = format!("{context}: {}", error.text());
        let error = match error {
            SessionError::Lost(_) => SessionError::Lost(text),
            SessionError::Refused(_) => SessionError::Refused(text),
        };
        Self { error, changed: true }
    }
}

impl ImapSession {
    /// Moves the messages to `to` with one `UID MOVE`.
    pub async fn move_to(&mut self, set: &UidSet, to: &str) -> Result<(), StepFailure> {
        self.send_change(&WriteCommand::Move { set, to })
            .await
            .map(|_| ())
            .map_err(StepFailure::clean)
    }

    /// Moves the messages to `to` for a server without MOVE: copy, then mark
    /// the originals deleted and expunge exactly them.
    pub async fn copy_then_remove(&mut self, set: &UidSet, to: &str) -> Result<(), StepFailure> {
        self.require_writes().map_err(StepFailure::clean)?;
        self.require_extension(Needs::UidPlus).map_err(StepFailure::clean)?;
        self.send_change(&WriteCommand::Copy { set, to })
            .await
            .map_err(StepFailure::clean)?;
        self.send_change(&WriteCommand::MarkDeleted { set })
            .await
            .map_err(|e| StepFailure::dirty(e, "copied, but the originals could not be marked"))?;
        self.send_change(&WriteCommand::Expunge { set })
            .await
            .map_err(|e| StepFailure::dirty(e, "copied and marked, but the originals were not removed"))?;
        Ok(())
    }

    /// Removes the messages for good: mark them deleted, then expunge exactly
    /// them.
    pub async fn delete_forever(&mut self, set: &UidSet) -> Result<(), StepFailure> {
        self.require_writes().map_err(StepFailure::clean)?;
        self.require_extension(Needs::UidPlus).map_err(StepFailure::clean)?;
        self.send_change(&WriteCommand::MarkDeleted { set })
            .await
            .map_err(StepFailure::clean)?;
        self.send_change(&WriteCommand::Expunge { set })
            .await
            .map_err(|e| StepFailure::dirty(e, "marked as deleted, but the server did not remove them"))?;
        Ok(())
    }
}

#[cfg(test)]
include!("move_copy.tests.rs");
