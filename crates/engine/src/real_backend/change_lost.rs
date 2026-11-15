//! What is said when the connection is lost while a change is under way, or
//! just after it went through.
//!
//! A flag change, a move or a delete is a few commands in a row. A loss
//! before the first one leaves nothing but doubt, and says so. A loss after a
//! step the server answered leaves something behind: a copy in the
//! destination while the originals are still in the source, or a deleted mark
//! that was not followed by the removal. That is reported with the step that
//! got through (the text of the failed sequence names it), and the folders
//! are read again after the reconnect, which shows what the server holds
//! before anything is tried a second time.

use super::changes::Tally;
use super::worker::Worker;
use crate::imap_session::SessionError;

/// A loss before any step of the change is known to have gone through.
const CONNECTION_LOST: &str = "the connection was lost, so the change may not have been \
     applied; the folder is read again once it is back";

/// A loss after part of the change went through.
const PART_DONE: &str = "the connection was lost partway through, so part of the change is on the server; \
     the folders are read again once it is back, so look at them before trying again";

/// A loss after the whole change went through, before what followed it.
const DONE_THEN_LOST: &str = "the change reached the server, but the connection was lost before \
     it could be confirmed; the folders are read again once it is back";

impl Worker {
    /// The connection was lost while the commands of the change were going
    /// out. What the server confirmed before that is still told, and so is
    /// the step a sequence got to. The error is handed back for the caller
    /// to act on.
    pub(super) fn lost_during_change(&mut self, operation: &str, tally: Tally, error: SessionError) -> SessionError {
        let sentence = if tally.landed() { PART_DONE } else { CONNECTION_LOST };
        self.tell_lost(operation, tally, sentence);
        error
    }

    /// The change went through, and the connection was lost afterwards
    /// (leaving the mailbox for reading, or reading the counts again).
    pub(super) fn lost_after_change(&mut self, operation: &str, tally: Tally, error: SessionError) -> SessionError {
        self.tell_lost(operation, tally, DONE_THEN_LOST);
        error
    }

    /// Sends what the tally holds that was not sent yet, then the sentence.
    fn tell_lost(&mut self, operation: &str, mut tally: Tally, sentence: &str) {
        self.send_flag_updates(&mut tally);
        for notice in std::mem::take(&mut tally.notices) {
            self.sink.send(self.sink.failure(operation, notice));
        }
        self.sink.send(self.sink.failure(operation, sentence));
    }
}

#[cfg(test)]
include!("change_lost.tests.rs");
