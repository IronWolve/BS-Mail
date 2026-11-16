//! Writing text into the page: every character that could start markup, or
//! end a quoted value, is escaped, and control characters never get through.

use crate::text_clean::is_layout_control;

/// Whether a character is written at all: control codes (a line break and a
/// tab apart) and the layout controls of `text_clean` are not.
fn wanted(c: char) -> bool {
    c == '\n' || c == '\t' || !(c.is_control() || is_layout_control(c))
}

/// Writes `text` as the content of an element.
pub(super) fn text(out: &mut String, text: &str) {
    for c in text.chars().filter(|c| wanted(*c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

/// Writes `value` as the inside of a double-quoted attribute value.
pub(super) fn attribute(out: &mut String, value: &str) {
    for c in value.chars().filter(|c| wanted(*c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\n' | '\t' => out.push(' '),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
include!("write.tests.rs");
