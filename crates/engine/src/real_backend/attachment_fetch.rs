//! Fetching one attachment of a message for the program to save, open or
//! send on (`Command::FetchAttachment`). The part is found by the number the
//! reader gave it, checked against the server's own account of the message
//! (it must be the same kind of file, so a message that was laid out
//! differently on the server is never saved under the wrong name), refused in
//! plain words when it is over the ceiling, and otherwise downloaded in slices
//! and decoded (`imap_session::fetch_part`). Nothing is written to the
//! mailbox and the message is not marked read.

use model::attachments::megabytes;
use model::{AttachmentInfo, Event, FileBytes, MAX_FETCH_BYTES, MessageId};

use super::worker::Worker;
use crate::imap_session::{Fetched, ImapSession, SessionError};

/// The name of the notice, which the screen listens for.
pub const OPERATION: &str = "save attachment";

/// The number of a part as the list of numbers it is made of ("1.3" is
/// `[1, 3]`), or `None` for anything that is not digits and dots.
pub fn parse_section(text: &str) -> Option<Vec<u32>> {
    if text.is_empty() || text.len() > 64 {
        return None;
    }
    let numbers: Option<Vec<u32>> = text
        .split('.')
        .map(|piece| {
            let plain = !piece.is_empty() && piece.len() <= 9 && piece.bytes().all(|byte| byte.is_ascii_digit());
            plain.then(|| piece.parse().ok()).flatten().filter(|number| *number > 0)
        })
        .collect();
    numbers.filter(|numbers| numbers.len() <= 16)
}

/// The words for a part that is over the ceiling; `wire` is what the server
/// said it weighs as sent.
pub fn too_big(wire: u64) -> String {
    format!(
        "this attachment is {} MB as the server holds it (about {} MB as a file), and this program downloads \
         attachments up to {} MB as sent (about {} MB as a file); nothing was saved",
        megabytes(wire),
        megabytes(wire / 4 * 3),
        megabytes(MAX_FETCH_BYTES),
        megabytes(MAX_FETCH_BYTES / 4 * 3),
    )
}

/// The words for a part that came to more than the ceiling although the server
/// said it was smaller.
pub fn grew_too_big() -> String {
    format!(
        "this attachment turned out to be larger than the {} MB (as sent) this program downloads; nothing was saved",
        megabytes(MAX_FETCH_BYTES)
    )
}

/// The words for an attachment whose number does not lead to a file.
const NOT_FETCHABLE: &str = "this attachment cannot be fetched: its place in the message is not known";

/// The words for a part the server lays out differently from the reader.
const LAID_OUT_DIFFERENTLY: &str = "the server holds this message differently from what was read, so the attachment \
     was not saved; open the message again and try once more";

impl Worker {
    pub(super) async fn fetch_attachment(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
        attachment: AttachmentInfo,
    ) -> Result<Vec<Event>, SessionError> {
        let Some(path) = parse_section(&attachment.section) else {
            return Ok(vec![self.sink.failure(OPERATION, NOT_FETCHABLE)]);
        };
        let uid = match self.open_message(session, message, OPERATION).await? {
            Ok(uid) => uid,
            Err(failure) => return Ok(vec![failure]),
        };
        let Some(facts) = session.part_facts(uid, &path).await? else {
            return Ok(vec![self.sink.failure(OPERATION, MISSING_PART)]);
        };
        if facts.content_type != attachment.content_type {
            return Ok(vec![self.sink.failure(OPERATION, LAID_OUT_DIFFERENTLY)]);
        }
        if facts.wire_size > MAX_FETCH_BYTES {
            return Ok(vec![self.sink.failure(OPERATION, too_big(facts.wire_size))]);
        }
        Ok(vec![match session.fetch_part(uid, &path, &facts, MAX_FETCH_BYTES).await? {
            Fetched::File(bytes) => Event::AttachmentData { message, attachment, data: FileBytes::new(bytes) },
            Fetched::TooBig => self.sink.failure(OPERATION, grew_too_big()),
            Fetched::TooSlow => self.sink.failure(
                OPERATION,
                "the download took longer than this program waits for one attachment, so it was given up; nothing was saved",
            ),
        }])
    }
}

/// The words for a message or part the server no longer has.
const MISSING_PART: &str = "the message or the attachment is no longer on the server";

#[cfg(test)]
include!("attachment_fetch.tests.rs");
