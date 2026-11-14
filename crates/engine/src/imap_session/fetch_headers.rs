//! Fetching two header fields of one message, for the thread headers of a
//! reply. PEEK on purpose, like every read here: the message is not marked
//! as read.

use futures_util::TryStreamExt;

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

impl ImapSession {
    /// The `Message-ID` and `References` fields of the message with this UID
    /// in the open mailbox, as header text, or `None` when the server no
    /// longer has it.
    pub async fn fetch_threading_headers(&mut self, uid: u32) -> Result<Option<Vec<u8>>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session
                .uid_fetch(
                    uid.to_string(),
                    "(UID BODY.PEEK[HEADER.FIELDS (MESSAGE-ID REFERENCES)])",
                )
                .await?
                .try_collect()
                .await
        })
        .await;
        let rows: Vec<_> = self.settle(result)?;
        Ok(rows
            .iter()
            .filter(|row| row.uid == Some(uid))
            .find_map(|row| row.header().map(<[u8]>::to_vec)))
    }
}
