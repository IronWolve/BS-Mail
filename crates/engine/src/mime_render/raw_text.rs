//! A message as it was received, for the reading pane's "view headers" and
//! "view source": the header block or the whole text, with the codes that a
//! screen would obey taken out and the length held to the ceiling a body has.
//! Nothing here is interpreted; what the pane shows is what the sender sent.

use model::RawPart;

use super::limits::{MAX_TEXT_BYTES, keep, megabytes};
use crate::text_clean::strip_controls;

/// The header block of `raw`: everything before the first blank line, or all
/// of it when there is none.
fn header_block(raw: &[u8]) -> &[u8] {
    let crlf = raw.windows(4).position(|window| window == b"\r\n\r\n");
    let lf = raw.windows(2).position(|window| window == b"\n\n");
    match (crlf, lf) {
        (Some(a), Some(b)) => &raw[..a.min(b)],
        (Some(at), None) | (None, Some(at)) => &raw[..at],
        (None, None) => raw,
    }
}

/// The text to show for `part` of the message `raw`, the start of a message of
/// `total` bytes when that is known. Bytes that are not text are shown as the
/// replacement character; control codes are dropped (a line break and a tab
/// stay), so the text can be drawn but never acted on.
pub fn raw_text(raw: &[u8], total: Option<u64>, part: RawPart) -> String {
    let wanted = match part {
        RawPart::Headers => header_block(raw),
        RawPart::Source => raw,
    };
    let mut text = strip_controls(&String::from_utf8_lossy(wanted));
    if part == RawPart::Headers {
        text.truncate(text.trim_end().len());
    }
    let mut notes = Vec::new();
    let at = keep(&text, MAX_TEXT_BYTES);
    if at < text.len() {
        text.truncate(at);
        notes.push(format!("[Only the first {} MB is shown.]", megabytes(MAX_TEXT_BYTES as u64)));
    }
    if part == RawPart::Source
        && let Some(total) = total.filter(|total| *total > raw.len() as u64)
    {
        notes.push(format!(
            "[Only the first {} MB of this {} MB message was read. The rest is not shown.]",
            megabytes(raw.len() as u64),
            megabytes(total)
        ));
    }
    if !notes.is_empty() {
        text = format!("{}\n\n{}", text.trim_end(), notes.join("\n"));
    }
    text
}

#[cfg(test)]
include!("raw_text.tests.rs");
