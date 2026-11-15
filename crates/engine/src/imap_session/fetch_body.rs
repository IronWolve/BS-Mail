//! Fetching one message. BODY.PEEK[] is used on purpose: plain BODY[] would
//! make the server mark the message as read.
//!
//! No more than `MAX_BODY_BYTES` of it is asked for, whatever its size: a
//! message can carry attachments of any weight, and the reading pane has no
//! use for most of them. The server says how large the whole message is, so
//! the caller can tell when it was cut.

use futures_util::TryStreamExt;

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// The most of one message that is downloaded to read it.
pub const MAX_BODY_BYTES: u32 = 16 * 1024 * 1024;

/// One message as the server gave it. Its debug output shows sizes, never
/// the message.
pub struct RawMessage {
    /// The message from its first byte, at most `MAX_BODY_BYTES` of it.
    pub bytes: Vec<u8>,
    /// The size of the whole message, when the server said.
    pub total: Option<u64>,
}

impl std::fmt::Debug for RawMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawMessage({} bytes of {:?})", self.bytes.len(), self.total)
    }
}

impl ImapSession {
    /// The start of the message with this UID from the open mailbox, or `None`
    /// when the server no longer has it.
    pub async fn fetch_body(&mut self, uid: u32) -> Result<Option<RawMessage>, SessionError> {
        let limit = self.limits.body;
        let session = &mut self.inner;
        let query = format!("(UID RFC822.SIZE BODY.PEEK[]<0.{MAX_BODY_BYTES}>)");
        let result = bounded(limit, async {
            session.uid_fetch(uid.to_string(), &query).await?.try_collect().await
        })
        .await;
        let rows: Vec<_> = self.settle(result)?;
        Ok(rows.iter().filter(|row| row.uid == Some(uid)).find_map(|row| {
            row.body().map(|bytes| RawMessage { bytes: bytes.to_vec(), total: row.size.map(u64::from) })
        }))
    }
}
