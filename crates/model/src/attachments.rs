//! Files that go out with a message, and the ceilings that apply to files in
//! either direction, worded once for the screens and the engine.

use crate::file_bytes::FileBytes;

/// One file attached to a message being sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingAttachment {
    /// The file name as the recipient will see it; no folder part.
    pub name: String,
    /// The content type, `type/subtype`; `application/octet-stream` when the
    /// file's kind is not known.
    pub content_type: String,
    pub data: FileBytes,
}

/// The most a message may weigh once built, in bytes (25 MB). The text, the
/// headers and every attached file as it travels (base64) all count.
pub const MAX_SEND_BYTES: u64 = 25 * 1024 * 1024;

/// The most one received attachment may weigh on the wire, in bytes (50 MB;
/// about 37 MB once decoded to the file). A larger one is not downloaded.
pub const MAX_FETCH_BYTES: u64 = 50 * 1024 * 1024;

/// What the headers of one attached part take beyond its encoded body.
const PART_OVERHEAD: u64 = 512;

/// The room the text of the message and its headers are given when the files
/// are counted against `MAX_SEND_BYTES` before the text is final.
pub const TEXT_RESERVE: u64 = 256 * 1024;

/// What a file of `len` bytes takes inside a message: base64 in lines of 76
/// characters with a CRLF each, and the part's own headers.
pub fn wire_size(len: u64) -> u64 {
    let characters = len.div_ceil(3).saturating_mul(4);
    characters.saturating_add(characters.div_ceil(76).saturating_mul(2)).saturating_add(PART_OVERHEAD)
}

/// The most bytes a file may have for it to take no more than `room` bytes
/// of a message (the largest `len` with `wire_size(len) <= room`); 0 when
/// there is no room for a file at all.
pub fn max_raw_len(room: u64) -> u64 {
    // One line of 76 characters and its CRLF carries 57 bytes.
    let mut len = room.saturating_sub(PART_OVERHEAD) / 78 * 57;
    if wire_size(len) > room {
        return 0;
    }
    while wire_size(len + 1) <= room {
        len += 1;
    }
    len
}

/// The longest file name that is sent, in characters.
const MAX_NAME_CHARS: usize = 200;

/// A received attachment's name made fit to be sent on: only its last part
/// (a `/` or `\` ends a folder), without control characters or the characters
/// that hide in a name (direction overrides, zero-width spaces), cut to a
/// length the sending side accepts with its extension kept; `attachment` when
/// nothing is left.
pub fn sendable_name(raw: &str) -> String {
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let kept: String = last.chars().filter(|ch| !ch.is_control() && !is_hidden(*ch)).collect();
    let kept = kept.trim().to_owned();
    if kept.is_empty() {
        return "attachment".to_owned();
    }
    if kept.chars().count() <= MAX_NAME_CHARS {
        return kept;
    }
    let extension = kept.rfind('.').map(|at| &kept[at..]).filter(|ext| ext.chars().count() <= 16).unwrap_or("");
    let stem: String = kept.chars().take(MAX_NAME_CHARS - extension.chars().count()).collect();
    format!("{stem}{extension}")
}

/// The characters that change how text is drawn or hide in it (the sending
/// side refuses them in a name).
fn is_hidden(ch: char) -> bool {
    matches!(
        ch,
        '\u{00AD}' | '\u{180E}' | '\u{200B}' | '\u{2028}' | '\u{2029}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}' | '\u{FEFF}' | '\u{FFF9}'..='\u{FFFB}' | '\u{E0001}' | '\u{E0020}'..='\u{E007F}'
    )
}

/// A size in whole megabytes, rounded up, for the sentences about a limit.
pub fn megabytes(bytes: u64) -> u64 {
    bytes.div_ceil(1024 * 1024)
}

#[cfg(test)]
include!("attachments.tests.rs");
