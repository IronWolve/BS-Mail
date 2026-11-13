//! One fetched message row to the model's envelope.

use async_imap::imap_proto::Envelope as WireEnvelope;
use async_imap::types::Flag;
use mail_parser::DateTime;
use model::{Envelope, FolderId, MailDate, MessageId, ThreadId, Uid};

use super::flag_map::map_flags;
use super::header_text::{decode_text, map_addresses};

/// What one FETCH response says about one message.
pub struct FetchedRow<'a> {
    pub uid: u32,
    pub flags: Vec<Flag<'a>>,
    pub envelope: Option<&'a WireEnvelope<'a>>,
    /// INTERNALDATE: when the server received the message.
    pub internal_date: Option<MailDate>,
    pub size: u32,
    pub has_attachments: bool,
}

/// The list row for a message. The date is the Date header when it parses
/// (it keeps the sender's zone), else the server's receive time.
pub fn build_envelope(id: MessageId, folder: FolderId, row: &FetchedRow<'_>) -> Envelope {
    let (flags, tags) = map_flags(&row.flags);
    let wire = row.envelope;
    let header_date = wire
        .and_then(|e| e.date.as_deref())
        .and_then(parse_date_header);
    Envelope {
        id,
        folder,
        uid: Uid(row.uid),
        flags,
        from: map_addresses(wire.and_then(|e| e.from.as_deref())),
        to: map_addresses(wire.and_then(|e| e.to.as_deref())),
        cc: map_addresses(wire.and_then(|e| e.cc.as_deref())),
        subject: wire
            .and_then(|e| e.subject.as_deref())
            .map(decode_text)
            .unwrap_or_default(),
        date: header_date.or(row.internal_date).unwrap_or_default(),
        size: u64::from(row.size),
        has_attachments: row.has_attachments,
        tags,
        thread: ThreadId(id.0),
        message_id: wire
            .and_then(|e| e.message_id.as_deref())
            .map(|raw| String::from_utf8_lossy(raw).trim().to_owned())
            .filter(|text| !text.is_empty()),
    }
}

/// A `Date:` header value as a point in time with the sender's offset.
pub fn parse_date_header(raw: &[u8]) -> Option<MailDate> {
    let text = String::from_utf8_lossy(raw);
    let parsed = DateTime::parse_rfc822(text.trim()).filter(DateTime::is_valid)?;
    let minutes = i16::try_from(u16::from(parsed.tz_hour) * 60 + u16::from(parsed.tz_minute)).ok()?;
    Some(MailDate {
        unix: parsed.to_timestamp(),
        offset_minutes: if parsed.tz_before_gmt { -minutes } else { minutes },
    })
}

#[cfg(test)]
include!("envelope_map.tests.rs");
