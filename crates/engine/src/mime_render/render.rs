//! A raw message to the body the reading pane shows.

use mail_parser::MessageParser;
use model::Body;
use model::config::BodyPreference;

use super::attachments::list_attachments;
use super::body_select::pick_body;
use super::guarded;
use super::limits::{MAX_ATTACHMENTS, MAX_TEXT_BYTES, keep, megabytes};
use super::reply_to::reply_to_of;
use super::styled_lines::style_lines;

/// Parses `raw`, the start of a message of `total` bytes when that is known,
/// and builds the body: the text as styled lines and the list of attachments.
/// `prefer` says which part to show when a message has a plain and an HTML one.
/// Bytes that do not parse as a message are shown as text. The work is done
/// on a thread of its own with a time limit; a message that cannot be read in
/// time, or that stops the reader, is replaced by a line that says so.
pub fn render_message(raw: Vec<u8>, total: Option<u64>, prefer: BodyPreference) -> Body {
    match guarded::run(move || build(&raw, total, prefer)) {
        Ok(body) => body,
        Err(failure) => notice_only(failure.words()),
    }
}

/// A whole message, for the tests.
#[cfg(test)]
pub fn render_body(raw: &[u8]) -> Body {
    render_message(raw.to_vec(), None, BodyPreference::Plain)
}

fn build(raw: &[u8], total: Option<u64>, prefer: BodyPreference) -> Body {
    let mut notes = Vec::new();
    if let Some(total) = total.filter(|total| *total > raw.len() as u64) {
        notes.push(format!(
            "[Only the first {} MB of this {} MB message was read. The rest, attachments included, is not shown.]",
            megabytes(raw.len() as u64),
            megabytes(total)
        ));
    }
    let Some(message) = MessageParser::default().parse(raw) else {
        let text = String::from_utf8_lossy(raw);
        let shown = &text[..keep(&text, MAX_TEXT_BYTES)];
        return Body { lines: style_lines(&with_notes(shown, &notes)), attachments: Vec::new(), reply_to: Vec::new() };
    };
    let picked = pick_body(&message, prefer);
    let (text, more) = picked.map(|picked| (picked.text, picked.notes)).unwrap_or_default();
    notes.extend(more);
    let attachments = list_attachments(&message);
    if message.attachments.len() > MAX_ATTACHMENTS {
        notes.push(format!("[Only the first {MAX_ATTACHMENTS} attachments are listed.]"));
    }
    Body { lines: style_lines(&with_notes(&text, &notes)), attachments, reply_to: reply_to_of(&message) }
}

/// The text, then each note on a line of its own after a blank line.
fn with_notes(text: &str, notes: &[String]) -> String {
    if notes.is_empty() {
        return text.to_owned();
    }
    format!("{}\n\n{}", text.trim_end(), notes.join("\n"))
}

/// What the pane shows when the message could not be read at all.
fn notice_only(words: &str) -> Body {
    Body { lines: style_lines(&format!("[{words}]")), attachments: Vec::new(), reply_to: Vec::new() }
}

#[cfg(test)]
include!("render.tests.rs");
