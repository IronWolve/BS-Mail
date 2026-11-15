//! A message's list-row data: the envelope with its flags, tags and thread.

use crate::address::Address;
use crate::flags::Flags;
use crate::ids::{FolderId, MessageId, ThreadId, Uid};

/// A point in time with the sender's UTC offset, so the reader can show the
/// original zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MailDate {
    /// UTC seconds since the Unix epoch.
    pub unix: i64,
    /// Offset east of UTC, in minutes.
    pub offset_minutes: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub id: MessageId,
    pub folder: FolderId,
    pub uid: Uid,
    pub flags: Flags,
    pub from: Vec<Address>,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub subject: String,
    pub date: MailDate,
    /// True when `date` is the time the server received the message because
    /// the message's own `Date` is more than a day later than that, which no
    /// message can truthfully say of itself.
    pub date_ahead: bool,
    /// Size in bytes.
    pub size: u64,
    pub has_attachments: bool,
    pub tags: Vec<String>,
    pub thread: ThreadId,
    /// The Message-ID header value, when present.
    pub message_id: Option<String>,
}
