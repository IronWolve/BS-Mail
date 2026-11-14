//! Finding the ids of the message a reply answers: the `Message-ID` and
//! `References` of the original, read from the server with PEEK so the
//! original is not marked as read.

use model::MessageId;

use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};
use crate::send::{ReplyIds, parse_threading_headers};

impl Worker {
    /// The thread ids of message `original`, or `None` when they cannot be
    /// found: the id is unknown, the folder was rebuilt, the message is gone
    /// or has no usable `Message-ID`. Only a lost connection is an error.
    pub(super) async fn reply_ids(
        &mut self,
        session: &mut ImapSession,
        original: MessageId,
    ) -> Result<Option<ReplyIds>, SessionError> {
        let Some((folder, uid)) = self.state.message_key(original) else {
            return Ok(None);
        };
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(None);
        };
        let reopened = session.ensure_examined(&path).await?;
        if reopened.is_some_and(|info| self.state.note_validity(folder, info.uid_validity)) {
            return Ok(None);
        }
        let raw = session.fetch_threading_headers(uid.0).await?;
        Ok(raw.as_deref().and_then(parse_threading_headers))
    }
}
