//! The header lines of an outgoing message, in the order they are written.
//!
//! There is no `Bcc` in the block, ever: hidden recipients are in the
//! envelope only. The one place their names are written is the copy kept in
//! the Sent folder, which gets `bcc_line` and never goes to a server but the
//! user's own. And no header names the program that wrote the message. Every value
//! comes from text that was already checked for line breaks and controls,
//! and every line ends with CRLF.

use super::check::Mailbox;
use super::reply_ids::ReplyIds;
use super::word::{LINE_LIMIT, display_name, unstructured};

/// What goes into the headers.
pub struct HeaderParts<'a> {
    pub from: &'a Mailbox,
    pub to: &'a [Mailbox],
    pub cc: &'a [Mailbox],
    pub subject: &'a str,
    pub date: &'a str,
    pub message_id: &'a str,
    pub reply: Option<&'a ReplyIds>,
}

/// The whole block, ready to be followed by the MIME headers.
pub fn header_block(parts: &HeaderParts<'_>) -> String {
    let mut out = String::new();
    push(&mut out, "Date", parts.date);
    push(&mut out, "From", &address_list("From", std::slice::from_ref(parts.from)));
    if parts.to.is_empty() && parts.cc.is_empty() {
        push(&mut out, "To", "undisclosed-recipients:;");
    }
    if !parts.to.is_empty() {
        push(&mut out, "To", &address_list("To", parts.to));
    }
    if !parts.cc.is_empty() {
        push(&mut out, "Cc", &address_list("Cc", parts.cc));
    }
    if !parts.subject.is_empty() {
        push(&mut out, "Subject", &unstructured(parts.subject, "Subject: ".len()));
    }
    push(&mut out, "Message-ID", &format!("<{}>", parts.message_id));
    if let Some(reply) = parts.reply {
        push(&mut out, "In-Reply-To", &reply.in_reply_to());
        push(&mut out, "References", &id_list("References", &reply.thread()));
    }
    out
}

/// The `Bcc` header for the Sent copy, so the sender can see later who was
/// blind-copied; empty when nobody was.
pub fn bcc_line(bcc: &[Mailbox]) -> String {
    if bcc.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    push(&mut out, "Bcc", &address_list("Bcc", bcc));
    out
}

fn push(out: &mut String, name: &str, value: &str) {
    out.push_str(name);
    out.push_str(": ");
    out.push_str(value);
    out.push_str("\r\n");
}

/// One address as it appears in a header: `Name <address>`, or the bare
/// address when there is no name. `used` is how much of the first line is
/// taken already, so an encoded name is cut to fit.
pub fn mailbox_text(who: &Mailbox, used: usize) -> String {
    let Some(name) = &who.name else {
        return who.address.clone();
    };
    let shown = display_name(name, used);
    let last_line = match shown.rsplit_once("\r\n") {
        Some((_, last)) => last.len(),
        None => used + shown.len(),
    };
    let gap = if last_line + 1 + who.address.len() + 2 > LINE_LIMIT { "\r\n " } else { " " };
    format!("{shown}{gap}<{}>", who.address)
}

/// A comma-separated address list, folded before an address rather than
/// inside one.
fn address_list(header: &str, people: &[Mailbox]) -> String {
    join_folded(header, people, ",", mailbox_text)
}

/// Message ids, space-separated and folded.
fn id_list(header: &str, ids: &[String]) -> String {
    join_folded(header, ids, "", |id, _| format!("<{id}>"))
}

/// Joins items, each followed by `separator` except the last, starting a new
/// line before an item that would not fit on the current one. `render` makes
/// an item's text, given how much of its first line is already used.
fn join_folded<T>(
    header: &str,
    items: &[T],
    separator: &str,
    render: impl Fn(&T, usize) -> String,
) -> String {
    let mut out = String::new();
    let mut line_len = header.len() + 2;
    for (at, item) in items.iter().enumerate() {
        if at > 0 {
            out.push_str(separator);
            line_len += separator.len();
        }
        let mut text = render(item, line_len + usize::from(at > 0));
        if at > 0 {
            let first_line = text.split("\r\n").next().map_or(0, str::len);
            if line_len + 1 + first_line <= LINE_LIMIT {
                out.push(' ');
                line_len += 1;
            } else {
                out.push_str("\r\n ");
                line_len = 1;
                text = render(item, line_len);
            }
        }
        line_len = match text.rsplit_once("\r\n") {
            Some((_, last)) => last.len(),
            None => line_len + text.len(),
        };
        out.push_str(&text);
    }
    out
}

#[cfg(test)]
include!("header_block.tests.rs");
