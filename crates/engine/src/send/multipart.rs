//! A message with files: `multipart/mixed` with the text first, then one
//! part for each file, each file as base64 in lines of 76 characters with its
//! own content type and file name (see `attachment_header`). Only built when
//! there are files; a message with none is the plain text message it always
//! was.

use model::OutgoingAttachment;

use super::attachment_header::part_headers;
use super::base64::encode_wrapped;

/// The `Content-Type` header line (ended with CRLF) of the whole message.
pub fn top_header(boundary: &str) -> String {
    format!("Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n")
}

/// The bytes that follow the message's own headers: the MIME version line, the
/// multipart type, a blank line, the text part (`text_headers` ends with the
/// blank line, `text` is its encoded body) and every file, closed by the last
/// boundary.
pub fn assemble(boundary: &str, text_headers: &str, text: &[u8], files: &[OutgoingAttachment]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() + 512);
    out.extend_from_slice(b"MIME-Version: 1.0\r\n");
    out.extend_from_slice(top_header(boundary).as_bytes());
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(format!("--{boundary}\r\n{text_headers}").as_bytes());
    out.extend_from_slice(text);
    for file in files {
        out.extend_from_slice(format!("\r\n--{boundary}\r\n{}", part_headers(file)).as_bytes());
        out.extend_from_slice(&encode_wrapped(file.data.as_slice()));
    }
    out.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    out
}

#[cfg(test)]
include!("multipart.tests.rs");
