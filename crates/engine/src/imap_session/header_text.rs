//! Header text as people read it: encoded words decoded, addresses joined.

use std::borrow::Cow;

use async_imap::imap_proto::Address as WireAddress;
use mail_parser::MessageParser;
use model::Address;

use crate::text_clean::single_line;

/// Text from an envelope field's bytes. Plain text passes straight through;
/// anything with an encoded word (`=?charset?B?...?=`) is decoded.
pub fn decode_text(raw: &[u8]) -> String {
    single_line(&decode_words(&unescaped(raw)))
}

/// The text of an envelope field as the server meant it. The parser hands a
/// quoted string over with its escapes still in it: a `"` or a `\` in the text
/// arrives as `\"` or `\\`, so a subject of `Quarterly "figures"` would show
/// its backslashes. In the payload of a quoted string every `"` and `\` is half
/// of such a pair, so that is what is undone. Text that cannot be a quoted
/// string's payload (a bare `"`, or a `\` before anything else) came as a
/// literal, which has no escapes, and is left as it is.
pub fn unescaped(raw: &[u8]) -> Cow<'_, [u8]> {
    if !raw.contains(&b'\\') {
        return Cow::Borrowed(raw);
    }
    let mut out = Vec::with_capacity(raw.len());
    let mut bytes = raw.iter().copied();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\\' => match bytes.next() {
                Some(next @ (b'\\' | b'"')) => out.push(next),
                _ => return Cow::Borrowed(raw),
            },
            b'"' => return Cow::Borrowed(raw),
            other => out.push(other),
        }
    }
    Cow::Owned(out)
}

fn decode_words(raw: &[u8]) -> String {
    if !raw.windows(2).any(|pair| pair == b"=?") {
        return String::from_utf8_lossy(raw).into_owned();
    }
    let mut field = b"Subject: ".to_vec();
    field.extend(raw.iter().filter(|b| **b != b'\r' && **b != b'\n'));
    field.extend_from_slice(b"\r\n\r\n");
    MessageParser::default()
        .parse_headers(&field)
        .and_then(|message| message.subject().map(str::to_owned))
        .unwrap_or_else(|| String::from_utf8_lossy(raw).into_owned())
}

/// The addresses of one envelope field. Group markers (no host) are skipped
/// and entries without a mailbox part are dropped.
pub fn map_addresses(list: Option<&[WireAddress<'_>]>) -> Vec<Address> {
    let Some(list) = list else {
        return Vec::new();
    };
    list.iter().filter_map(map_address).collect()
}

fn map_address(wire: &WireAddress<'_>) -> Option<Address> {
    let mailbox = wire.mailbox.as_deref()?;
    let host = wire.host.as_deref()?;
    let email = format!(
        "{}@{}",
        String::from_utf8_lossy(mailbox),
        String::from_utf8_lossy(host)
    );
    let name = wire
        .name
        .as_deref()
        .map(decode_text)
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty());
    Some(Address { name, email })
}

#[cfg(test)]
include!("header_text.tests.rs");
