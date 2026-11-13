//! Bullet markers at the start of a line and the lead word after them.

use model::BodyRole;

use super::found::Found;

/// Markers of a finished item, longest first so `- [x]` wins over `[x]`.
const DONE: [&str; 3] = ["- [x]", "[x]", "\u{2713}"];
/// Markers of an open item.
const OPEN: [&str; 3] = ["- [ ]", "[ ]", "\u{25cb}"];
/// Lead words that point at what comes next.
const NEXT_LEADS: [&str; 2] = ["Next", "Todo"];

/// The marker `rest` starts with and its role, when a blank or the end of
/// the text follows it.
fn marker_at(rest: &str) -> Option<(usize, BodyRole)> {
    let tables = [(DONE, BodyRole::BulletDone), (OPEN, BodyRole::BulletOpen)];
    for (markers, role) in tables {
        for marker in markers {
            let ends = rest
                .strip_prefix(marker)
                .is_some_and(|after| after.chars().next().is_none_or(char::is_whitespace));
            if ends {
                return Some((marker.len(), role));
            }
        }
    }
    None
}

/// Marks the bullet at the start of `text` (after any blanks) and, when
/// `lead` is on, the `Word:` that follows it. `text` must start a line.
pub fn find_bullet(text: &str, lead: bool, found: &mut Found) {
    let indent = text.len() - text.trim_start().len();
    let Some((length, role)) = marker_at(&text[indent..]) else {
        return;
    };
    let end = indent + length;
    found.add(indent..end, role);
    if lead {
        find_lead(text, end, found);
    }
}

/// A lead word: letters, digits and `/` ending in a colon, after the marker
/// and at least one blank. `Next:` and `Todo:` get their own role.
fn find_lead(text: &str, marker_end: usize, found: &mut Found) {
    let after = &text[marker_end..];
    let word_text = after.trim_start_matches(' ');
    if word_text.len() == after.len() {
        return;
    }
    let start = marker_end + (after.len() - word_text.len());
    let length = word_text
        .find(|c: char| !(c.is_alphanumeric() || c == '/'))
        .unwrap_or(word_text.len());
    let ends_ok = word_text[length..]
        .strip_prefix(':')
        .is_some_and(|rest| rest.chars().next().is_none_or(char::is_whitespace));
    if length == 0 || !ends_ok {
        return;
    }
    let role = if NEXT_LEADS.contains(&&word_text[..length]) {
        BodyRole::LeadNext
    } else {
        BodyRole::Lead
    };
    found.add(start..start + length + 1, role);
}

#[cfg(test)]
include!("bullets.tests.rs");
