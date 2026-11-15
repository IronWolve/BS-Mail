//! Choosing which part of a message is its text.

use mail_parser::{Message, MessagePart, MimeHeaders, PartType};
use model::config::BodyPreference;

use super::flowed::unflow;
use super::html_to_text::{html_to_text, too_nested};
use super::limits::{MAX_HTML_BYTES, MAX_TEXT_BYTES, keep, megabytes};
use crate::text_clean::has_visible_text;

/// The text of a message and what was left out of it, for the notices.
pub struct Picked {
    pub text: String,
    pub notes: Vec<String>,
}

/// The body text. By default (`Plain`) it is the first `text/plain` part that
/// shows something, and failing that the first non-empty `text/html` part
/// converted to text. With `Html` the order is the other way round: the
/// converted HTML part when it shows something, else the plain part. A part
/// made only of blanks, controls and zero-width characters shows nothing and
/// is passed over. `None` when the message has neither. A very long text, or
/// HTML too large or too deeply nested for the converter, is shortened or
/// simplified, and says so.
pub fn pick_body(message: &Message<'_>, prefer: BodyPreference) -> Option<Picked> {
    match prefer {
        BodyPreference::Plain => plain_body(message).or_else(|| html_body(message)),
        BodyPreference::Html => match html_body(message) {
            Some(html) if has_visible_text(&html.text) => Some(html),
            other => plain_body(message).or(other),
        },
    }
}

/// The first plain text part that shows something.
fn plain_body(message: &Message<'_>) -> Option<Picked> {
    for &id in &message.text_body {
        let Some(part) = message.parts.get(id as usize) else {
            continue;
        };
        if let PartType::Text(text) = &part.body
            && is_plain(part)
            && has_visible_text(text)
        {
            let mut notes = Vec::new();
            return Some(Picked { text: cut_text(plain_text(part, text), &mut notes), notes });
        }
    }
    None
}

/// The first non-empty HTML part, converted to text.
fn html_body(message: &Message<'_>) -> Option<Picked> {
    for &id in &message.html_body {
        let Some(part) = message.parts.get(id as usize) else {
            continue;
        };
        if let PartType::Html(html) = &part.body
            && !html.trim().is_empty()
        {
            return Some(from_html(html));
        }
    }
    None
}

fn from_html(html: &str) -> Picked {
    let mut notes = Vec::new();
    let shown = &html[..keep(html, MAX_HTML_BYTES)];
    if shown.len() < html.len() {
        notes.push(format!(
            "[This message is very large: only the first {} MB of its formatted text is shown.]",
            megabytes(MAX_HTML_BYTES as u64)
        ));
    }
    if too_nested(shown) {
        notes.push("[This message is formatted in a way that is too complicated to lay out, so it is shown in a simpler form.]".to_owned());
    }
    Picked { text: cut_text(html_to_text(shown), &mut notes), notes }
}

/// Keeps at most `MAX_TEXT_BYTES` of `text`, noting when it cut.
fn cut_text(mut text: String, notes: &mut Vec<String>) -> String {
    let at = keep(&text, MAX_TEXT_BYTES);
    if at < text.len() {
        text.truncate(at);
        notes.push(format!("[This text is very long: only the first {} MB is shown.]", megabytes(MAX_TEXT_BYTES as u64)));
    }
    text
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
