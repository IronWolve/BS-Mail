//! Choosing which part of a message is its text.

use mail_parser::{Message, MessagePart, MimeHeaders, PartType};

use super::flowed::unflow;
use super::html_to_text::html_to_text;

/// The body text: the first non-empty `text/plain` part; failing that, the
/// first non-empty `text/html` part converted to text. `None` when the
/// message has neither.
pub fn pick_body(message: &Message<'_>) -> Option<String> {
    for &id in &message.text_body {
        let Some(part) = message.parts.get(id as usize) else {
            continue;
        };
        if let PartType::Text(text) = &part.body
            && is_plain(part)
            && !text.trim().is_empty()
        {
            return Some(plain_text(part, text));
        }
    }
    for &id in &message.html_body {
        let Some(part) = message.parts.get(id as usize) else {
            continue;
        };
        if let PartType::Html(html) = &part.body
            && !html.trim().is_empty()
        {
            return Some(html_to_text(html));
        }
    }
    None
}

/// A part with no type header is plain text by the standard's default.
fn is_plain(part: &MessagePart<'_>) -> bool {
    part.content_type().is_none_or(|found| {
        found.ctype().eq_ignore_ascii_case("text")
            && found
                .subtype()
                .is_none_or(|subtype| subtype.eq_ignore_ascii_case("plain"))
    })
}

fn plain_text(part: &MessagePart<'_>, text: &str) -> String {
    let attribute = |name: &str| {
        part.content_type()
            .and_then(|found| found.attribute(name))
    };
    let flowed = attribute("format").is_some_and(|v| v.eq_ignore_ascii_case("flowed"));
    if flowed {
        let delsp = attribute("delsp").is_some_and(|v| v.eq_ignore_ascii_case("yes"));
        unflow(text, delsp)
    } else {
        text.to_owned()
    }
}
