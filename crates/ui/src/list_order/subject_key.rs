//! A subject as it is compared: without the `Re:` and `Fwd:` prefixes that
//! replies add, in lower case, with the spaces squeezed. Two messages of one
//! conversation have the same one, and a sort by subject puts a reply next to
//! what it answers.

/// The prefixes a reply or a forward puts in front of a subject.
const PREFIXES: [&str; 6] = ["re", "fw", "fwd", "aw", "sv", "antw"];

/// The most characters a subject is read for.
const MAX_CHARS: usize = 300;

/// `subject` without its reply prefixes, lower case and with single spaces.
pub fn base_subject(subject: &str) -> String {
    let mut rest = subject.trim();
    while let Some(after) = strip_prefix(rest) {
        rest = after;
    }
    let mut out = String::with_capacity(rest.len().min(MAX_CHARS));
    let mut gap = false;
    for ch in rest.chars().take(MAX_CHARS) {
        if ch.is_whitespace() || ch.is_control() {
            gap = true;
            continue;
        }
        if gap && !out.is_empty() {
            out.push(' ');
        }
        gap = false;
        out.extend(ch.to_lowercase());
    }
    out
}

/// Whether the subject starts with a reply or forward prefix.
pub fn has_reply_prefix(subject: &str) -> bool {
    strip_prefix(subject.trim()).is_some()
}

/// `text` without one leading `Re:` (or `Re[2]:`, `Fwd:`...), when it starts
/// with one.
fn strip_prefix(text: &str) -> Option<&str> {
    let colon = text.find(':')?;
    let head = &text[..colon];
    let word = head.split('[').next().unwrap_or(head).trim();
    let counted = head.len() == word.len() || head.trim_end().ends_with(']');
    let known = PREFIXES.iter().any(|prefix| word.eq_ignore_ascii_case(prefix));
    (known && counted).then(|| text[colon + 1..].trim_start())
}

#[cfg(test)]
include!("subject_key.tests.rs");
