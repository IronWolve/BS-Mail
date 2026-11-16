//! A filter for the styles of a message: a style sheet or the `style`
//! attribute of one element is read, and a new, clean text is built from the
//! parts that are known to be harmless. Nothing is copied through: a rule is
//! kept only when its selector uses plain characters, and a declaration only
//! when its property is on a list and its value holds no address, no
//! function but arithmetic and colours, no escape and no markup.
//!
//! Left out whole: every at-rule (`@import`, `@font-face`, `@media`...),
//! every `url(...)`, `image-set(...)` or other function that can name a file,
//! and anything with a backslash, which could spell one in disguise.

use super::limits::{MAX_KEPT_CSS_BYTES, MAX_SHEET_BYTES};

/// What the filter kept, and how many references to the network it left out.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Filtered {
    pub css: String,
    pub remote: u32,
}

/// The properties that are kept: looks only, none that loads, moves a box over
/// the page or changes the pointer.
const PROPERTIES: &[&str] = &[
    "background", "background-color", "border", "border-bottom", "border-bottom-color", "border-bottom-style",
    "border-bottom-width", "border-collapse", "border-color", "border-left", "border-left-color", "border-left-style",
    "border-left-width", "border-radius", "border-right", "border-right-color", "border-right-style",
    "border-right-width", "border-spacing", "border-style", "border-top", "border-top-color", "border-top-style",
    "border-top-width", "border-width", "clear", "color", "direction", "display", "font", "font-family", "font-size",
    "font-style", "font-variant", "font-weight", "height", "letter-spacing", "line-height", "list-style",
    "list-style-type", "margin", "margin-bottom", "margin-left", "margin-right", "margin-top", "max-height",
    "max-width", "min-height", "min-width", "overflow-wrap", "padding", "padding-bottom", "padding-left",
    "padding-right", "padding-top", "table-layout", "text-align", "text-decoration", "text-indent", "text-transform",
    "vertical-align", "white-space", "width", "word-break", "word-spacing",
];

/// The functions a value may use.
const FUNCTIONS: &[&str] = &["rgb", "rgba", "hsl", "hsla", "calc", "min", "max", "clamp"];

/// Words that no value may hold, whatever else is true of it.
const BANNED: &[&str] = &["expression", "javascript", "behavior", "binding", "url", "src", "image"];

const MAX_VALUE_BYTES: usize = 300;
const MAX_SELECTOR_BYTES: usize = 300;

/// The kept part of a style sheet (the text of a `style` element).
pub(super) fn sheet(text: &str) -> Filtered {
    let mut out = Filtered::default();
    let end = (0..=MAX_SHEET_BYTES.min(text.len())).rev().find(|at| text.is_char_boundary(*at)).unwrap_or(0);
    let cut = &text[..end];
    let clean = strip_comments(cut).replace("<!--", " ").replace("-->", " ");
    let mut at = 0;
    while at < clean.len() {
        let rest = &clean[at..];
        let blank = rest.len() - rest.trim_start().len();
        at += blank;
        if at >= clean.len() {
            break;
        }
        if clean[at..].starts_with('@') {
            let end = skip_at_rule(&clean, at);
            if names_network(&clean[at..end]) {
                out.remote += 1;
            }
            at = end;
            continue;
        }
        let Some(open) = clean[at..].find('{') else {
            break;
        };
        let (inner, end) = block(&clean, at + open);
        let selector = &clean[at..at + open];
        at = end;
        let Some(selector) = clean_selector(selector) else {
            continue;
        };
        let kept = declarations(inner);
        out.remote += kept.remote;
        if !kept.css.is_empty() && out.css.len() + kept.css.len() + selector.len() < MAX_KEPT_CSS_BYTES {
            out.css.push_str(&format!("{selector}{{{}}}\n", kept.css));
        }
    }
    out
}

/// The kept part of the text of a `style` attribute.
pub(super) fn declarations(text: &str) -> Filtered {
    let mut out = Filtered::default();
    for part in split_top(text, ';') {
        let Some((property, value)) = part.split_once(':') else {
            continue;
        };
        let property = property.trim().to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if PROPERTIES.contains(&property.as_str()) && value_is_clean(value) {
            if !out.css.is_empty() {
                out.css.push(';');
            }
            out.css.push_str(&format!("{property}:{value}"));
        } else if names_network(value) {
            out.remote += 1;
        }
    }
    out
}

/// Whether a text refers to something on the network.
fn names_network(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("http:") || lower.contains("https:") || lower.contains("//") || lower.contains("ftp:")
}

fn value_is_clean(value: &str) -> bool {
    if value.len() > MAX_VALUE_BYTES || value.chars().any(is_forbidden_char) {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    if BANNED.iter().any(|word| lower.contains(word)) {
        return false;
    }
    // Every `(` must belong to a function on the list, or to none at all.
    let bytes = lower.as_bytes();
    bytes.iter().enumerate().filter(|(_, byte)| **byte == b'(').all(|(at, _)| {
        let start = lower[..at].rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')).map_or(0, |i| i + 1);
        let name = &lower[start..at];
        name.is_empty() || FUNCTIONS.contains(&name)
    })
}

/// Characters that never appear in a value that is kept: anything that could
/// end a rule or a text, start a tag or an escape, or is a control code.
fn is_forbidden_char(c: char) -> bool {
    c.is_control() || matches!(c, '\\' | '<' | '>' | '{' | '}' | '@' | ';' | '`')
}

/// A selector as plain characters only, or `None`.
fn clean_selector(text: &str) -> Option<String> {
    let text = text.trim();
    let plain = |c: char| {
        c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.' | '#' | '*' | '>' | '+' | '~' | ',' | ':' | '[' | ']' | '=' | '"' | '\'' | '^' | '$' | '|' | '(' | ')')
    };
    let ok = !text.is_empty() && text.len() <= MAX_SELECTOR_BYTES && text.chars().all(plain);
    ok.then(|| text.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// The text without `/* ... */` comments (an unfinished one runs to the end).
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        out.push(' ');
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Where a quoted text that starts at `at` (on the quote) ends, after the
/// closing quote; the end of the text when it is not closed.
fn skip_string(text: &str, at: usize) -> usize {
    let quote = text.as_bytes()[at];
    let mut index = at + 1;
    let bytes = text.as_bytes();
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    text.len()
}

/// The inside of the block whose `{` is at `open`, and where it ends, after
/// its `}`. A block inside it is part of it, so a rule that holds another is
/// skipped whole by its caller, which finds nothing it likes inside.
fn block(text: &str, open: usize) -> (&str, usize) {
    let bytes = text.as_bytes();
    let (mut depth, mut index) = (0usize, open);
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return (&text[open + 1..index], index + 1);
                }
            }
            b'"' | b'\'' => {
                index = skip_string(text, index);
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    (&text[open + 1..], text.len())
}

/// Where the at-rule that starts at `at` ends: after its `;`, or after its
/// block.
fn skip_at_rule(text: &str, at: usize) -> usize {
    let bytes = text.as_bytes();
    let mut index = at;
    while index < bytes.len() {
        match bytes[index] {
            b';' => return index + 1,
            b'{' => return block(text, index).1,
            b'"' | b'\'' => {
                index = skip_string(text, index);
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    text.len()
}

/// `text` split at `separator`, except inside quotes and parentheses.
fn split_top(text: &str, separator: char) -> Vec<&str> {
    let bytes = text.as_bytes();
    let (mut parts, mut start, mut depth, mut index) = (Vec::new(), 0usize, 0usize, 0usize);
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'"' | b'\'' => {
                index = skip_string(text, index);
                continue;
            }
            byte if depth == 0 && char::from(byte) == separator => {
                parts.push(&text[start..index]);
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    parts.push(&text[start..]);
    parts
}

#[cfg(test)]
include!("css.tests.rs");
