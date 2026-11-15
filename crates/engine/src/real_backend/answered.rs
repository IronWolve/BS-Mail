//! Marking the message a reply answers as answered, once the server has
//! accepted the reply.
//!
//! The mark follows the send and never changes its outcome. It is made over
//! the mailbox connection like any flag change, so it shows in the list at
//! once and other programs see it too. While the connection is down the mark
//! waits and is made after the reconnect; it is kept in memory only, so a
//! program closed first does not make it. An account that may not change the
//! mailbox is never marked, and a message that is no longer in its folder
//! has nothing to mark.

use model::{Command, FlagChange, MessageId};

use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

impl Worker {
    /// Sets `\Answered` on the original of an accepted reply. Only a lost
    /// connection is an error, and the mark then waits for the reconnect.
    pub(super) async fn mark_answered(
        &mut self,
        session: &mut ImapSession,
        original: MessageId,
    ) -> Result<(), SessionError> {
        if self.config.read_only || self.state.message_key(original).is_none() {
            return Ok(());
        }
        let change = FlagChange { answered: Some(true), ..FlagChange::default() };
        let command = Command::SetFlags { messages: vec![original], change };
        let done = self.change(session, command).await;
        if let Err(error) = &done
            && error.is_lost()
        {
            self.answered_offline(original);
        }
        done
    }

    /// The mailbox connection is down: the mark waits for the reconnect.
    pub(super) fn answered_offline(&mut self, original: MessageId) {
        if !self.config.read_only && !self.answered_waiting.contains(&original) {
            self.answered_waiting.push(original);
        }
    }

    /// Makes the marks that waited for the mailbox.
    pub(super) async fn mark_waiting_answered(&mut self, session: &mut ImapSession) -> Result<(), SessionError> {
        let mut waiting = std::mem::take(&mut self.answered_waiting).into_iter();
        while let Some(original) = waiting.next() {
            if let Err(error) = self.mark_answered(session, original).await {
                // The one that was cut off is already waiting again.
                for rest in waiting {
                    self.answered_offline(rest);
                }
                return Err(error);
            }
        }
        Ok(())
    }
}
