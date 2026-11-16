//! The headers of one attached file: its content type and its name, in the
//! forms mail programs read. A name that is plain ASCII is a quoted string
//! (with quotes and backslashes escaped); any other name is written as RFC
//! 2231 parameters (`filename*=utf-8''%C3%A9...`, split into numbered
//! continuations when it is long), never as raw bytes. The callers have
//! already refused a name with a line break or another control character, so
//! nothing here can start a new header or end the part.

use model::OutgoingAttachment;

/// The type for a file whose kind is not known, or whose declared type is not
/// something a file can be sent as.
pub const UNKNOWN_TYPE: &str = "application/octet-stream";

/// The longest plain quoted name written on one line.
const PLAIN_LIMIT: usize = 60;

/// The most characters of an encoded name on one line of continuations.
const SEGMENT: usize = 44;

/// A declared content type made safe: `type/subtype` of token characters,
/// lower case, and not a multipart or a message (a file is sent as base64,
/// which those types forbid). Anything else is `application/octet-stream`.
pub fn content_type_of(declared: &str) -> String {
    let lower = declared.trim().to_ascii_lowercase();
    let token = |piece: &str| {
        !piece.is_empty()
            && piece.len() <= 60
            && piece.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$&^_.+-".contains(&byte))
    };
    match lower.split_once('/') {
        Some((kind, subtype)) if token(kind) && token(subtype) && kind != "multipart" && kind != "message" => lower,
        _ => UNKNOWN_TYPE.to_owned(),
    }
}

/// The three header lines of the part (type with the name, disposition with
/// the file name, and the transfer encoding) and the blank line that ends
/// them, each ended with CRLF.
pub fn part_headers(file: &OutgoingAttachment) -> String {
    let content_type = content_type_of(&file.content_type);
    format!(
        "Content-Type: {content_type}{}\r\nContent-Disposition: attachment{}\r\nContent-Transfer-Encoding: base64\r\n\r\n",
        parameter("name", &file.name),
        parameter("filename", &file.name),
    )
}

/// `; key="name"`, or the RFC 2231 form for a name that is not plain.
fn parameter(key: &str, name: &str) -> String {
    let plain = name.len() <= PLAIN_LIMIT && name.bytes().all(|byte| (0x20..0x7f).contains(&byte));
    if plain {
        let mut out = format!("; {key}=\"");
        for ch in name.chars() {
            if ch == '"' || ch == '\\' {
                out.push('\\');
            }
            out.push(ch);
        }
        out.push('"');
        return out;
    }
    let encoded = percent_encode(name);
    if encoded.len() <= SEGMENT {
        return format!(";\r\n {key}*=utf-8''{encoded}");
    }
    let mut out = String::new();
    for (number, piece) in segments(&encoded).iter().enumerate() {
        let head = if number == 0 { "utf-8''" } else { "" };
        out.push_str(&format!(";\r\n {key}*{number}*={head}{piece}"));
    }
    out
}

/// Every byte outside the characters RFC 2231 leaves as they are, as `%XX`.
fn percent_encode(name: &str) -> String {
    let mut out = String::with_capacity(name.len() * 2);
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `encoded` cut into pieces of at most `SEGMENT` characters, never inside a
/// `%XX`.
fn segments(encoded: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut start = 0;
    while start < encoded.len() {
        let mut end = (start + SEGMENT).min(encoded.len());
        if end < encoded.len() {
            // Back up out of a `%XX` the cut would split.
            if encoded.as_bytes()[end - 1] == b'%' {
                end -= 1;
            } else if end >= 2 && encoded.as_bytes()[end - 2] == b'%' {
                end -= 2;
            }
        }
        pieces.push(&encoded[start..end]);
        start = end;
    }
    pieces
}

#[cfg(test)]
include!("attachment_header.tests.rs");
