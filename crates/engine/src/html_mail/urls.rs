//! What a link or a picture may point at. Only plain web, mail and page-local
//! links are kept, and only pictures that arrive inside the message itself.

use mail_parser::decoders::base64::base64_decode;

use super::limits::{MAX_LINK_BYTES, MAX_PICTURE_BYTES};
use crate::send::base64;

/// The address a browser would read from an attribute value: blanks and
/// control characters around it are dropped, and a tab or a line break
/// inside it is dropped too (a browser ignores them), so `java\nscript:` is
/// seen as what it is.
fn as_read(value: &str) -> String {
    value.trim_matches(|c: char| c <= ' ').chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect()
}

fn starts_with_ignoring_case(text: &str, start: &str) -> bool {
    text.get(..start.len()).is_some_and(|head| head.eq_ignore_ascii_case(start))
}

/// The `href` of a link, when it is one that is kept: `http`, `https`,
/// `mailto` or a place on the same page. Anything else (a `javascript:` or
/// `data:` address, a relative path, a file) is left out. The address is
/// returned as a browser reads it.
pub(super) fn safe_link(value: &str) -> Option<String> {
    let read = as_read(value);
    if read.is_empty() || read.len() > MAX_LINK_BYTES || read.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let kept = ["http://", "https://", "mailto:"].iter().any(|scheme| starts_with_ignoring_case(&read, scheme))
        || (read.starts_with('#') && read.len() > 1);
    kept.then_some(read)
}

/// Whether a value names something on the network: a web address, an address
/// with no scheme that a browser would fetch from the page's own host, or a
/// protocol-relative one.
pub(super) fn is_remote(value: &str) -> bool {
    let read = as_read(value);
    ["http:", "https:", "//", "ftp:"].iter().any(|start| starts_with_ignoring_case(&read, start))
}

/// A `cid:` reference's id, without the scheme and the angle brackets.
pub(super) fn content_id(value: &str) -> Option<String> {
    let read = as_read(value);
    let id = read.get(..4).filter(|start| start.eq_ignore_ascii_case("cid:")).map(|_| &read[4..])?;
    let id = id.trim_matches(|c| c == '<' || c == '>');
    (!id.is_empty()).then(|| id.to_owned())
}

/// The picture a `data:` address holds, as a clean `data:` address, when it is
/// a small png, jpeg, gif or webp whose bytes really are one. Nothing else
/// (an svg, which can hold script; a page; a mistake) is let through.
pub(super) fn inline_picture(value: &str) -> Option<String> {
    let read = as_read(value);
    if !starts_with_ignoring_case(&read, "data:image/") {
        return None;
    }
    let (head, data) = read.split_once(',')?;
    if !head.to_ascii_lowercase().ends_with(";base64") || data.len() > MAX_PICTURE_BYTES * 2 {
        return None;
    }
    picture_address(&base64_decode(data.as_bytes())?)
}

/// A clean `data:` address for the bytes of a picture, or `None` when they are
/// not a png, jpeg, gif or webp or are too many. The type is the one the
/// bytes show, never the one that was claimed.
pub(super) fn picture_address(bytes: &[u8]) -> Option<String> {
    if bytes.len() > MAX_PICTURE_BYTES {
        return None;
    }
    let kind = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if bytes.starts_with(b"\xFF\xD8\xFF") {
        "jpeg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "gif"
    } else if bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        return None;
    };
    Some(format!("data:image/{kind};base64,{}", base64::encode(bytes)))
}

#[cfg(test)]
include!("urls.tests.rs");
