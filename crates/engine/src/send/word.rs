//! Header text that is not plain ASCII, or is not safe as it stands: RFC 2047
//! encoded words for subjects and names, quoting for names, and folding of
//! long subjects. The callers have already refused control characters.
//!
//! A folded header holds `\r\n ` inside its value. An encoded word is never
//! longer than 75 characters, and a line never holds more than 76 where a
//! fold is possible.

use super::base64;

/// The longest line a fold aims for, counting everything but the line break.
pub const LINE_LIMIT: usize = 76;

/// The longest encoded word RFC 2047 allows.
const WORD_LIMIT: usize = 75;
const PREFIX: &str = "=?utf-8?B?";
const SUFFIX: &str = "?=";

/// True when `text` cannot go into a header as it is: it has non-ASCII
/// characters, or something that would be read as an encoded word.
pub fn needs_encoding(text: &str) -> bool {
    !text.is_ascii() || text.contains("=?")
}

/// `text` as encoded words, each at most 75 characters. The first word is
/// kept short enough to share a line with `first_line_used` characters.
pub fn encode_words(text: &str, first_line_used: usize) -> Vec<String> {
    let overhead = PREFIX.len() + SUFFIX.len();
    let capacity = |budget: usize| (budget.saturating_sub(overhead) / 4).max(1) * 3;
    let mut words = Vec::new();
    let mut chunk = String::new();
    let mut limit = capacity(LINE_LIMIT.saturating_sub(first_line_used).clamp(24, WORD_LIMIT));
    for c in text.chars() {
        if chunk.len() + c.len_utf8() > limit {
            words.push(word_of(&chunk));
            chunk.clear();
            limit = capacity(WORD_LIMIT);
        }
        chunk.push(c);
    }
    if !chunk.is_empty() || words.is_empty() {
        words.push(word_of(&chunk));
    }
    words
}

fn word_of(chunk: &str) -> String {
    format!("{PREFIX}{}{SUFFIX}", base64::encode(chunk.as_bytes()))
}

/// A subject (or any unstructured text) ready to follow `Name: `, folded.
/// `name_len` is the length of the header name and its colon and blank.
pub fn unstructured(text: &str, name_len: usize) -> String {
    if needs_encoding(text) {
        return encode_words(text, name_len).join("\r\n ");
    }
    fold_ascii(text, name_len)
}

/// Folds plain text at its blanks so lines stay within the limit where they
/// can. A word longer than a line stays whole.
fn fold_ascii(text: &str, first_used: usize) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut line_len = first_used;
    for (at, word) in text.split(' ').enumerate() {
        if at > 0 {
            if line_len + 1 + word.len() > LINE_LIMIT && line_len > 0 && !word.is_empty() {
                out.push_str("\r\n ");
                line_len = 1;
            } else {
                out.push(' ');
                line_len += 1;
            }
        }
        out.push_str(word);
        line_len += word.len();
    }
    out
}

/// A display name as it goes before `<address>`: left alone when it is made
/// of plain words, quoted when it has punctuation, encoded when it is not
/// ASCII. `used` is how much of the line the name starts on is taken.
pub fn display_name(name: &str, used: usize) -> String {
    if needs_encoding(name) {
        return encode_words(name, used).join("\r\n ");
    }
    let plain = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || "!#$%&'*+-/=?^_`{|}~".contains(c));
    if plain {
        return name.to_owned();
    }
    let mut out = String::with_capacity(name.len() + 2);
    out.push('"');
    for c in name.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

#[cfg(test)]
include!("word.tests.rs");
