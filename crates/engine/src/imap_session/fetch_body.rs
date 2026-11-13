//! Fetching a whole message. BODY.PEEK[] is used on purpose: plain BODY[]
//! would make the server mark the message as read.

use futures_util::TryStreamExt;

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

impl ImapSession {
    /// The raw message with this UID from the open mailbox, or `None` when
    /// the server no longer has it.
    pub async fn fetch_body(&mut self, uid: u32) -> Result<Option<Vec<u8>>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session
                .uid_fetch(uid.to_string(), "(UID BODY.PEEK[])")
                .await?
                .try_collect()
                .await
        })
        .await;
        let rows: Vec<_> = self.settle(result)?;
        Ok(rows
            .iter()
            .filter(|row| row.uid == Some(uid))
            .find_map(|row| row.body().map(<[u8]>::to_vec)))
    }
}
