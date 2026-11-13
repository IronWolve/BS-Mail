//! Search-syntax tokens and version numbers.

use model::BodyRole;

use super::found::Found;

/// Words that are search keywords when followed by a colon. Any other
/// `word:` stays plain text.
pub const SEARCH_KEYWORDS: [&str; 12] = [
    "from",
    "to",
    "cc",
    "bcc",
    "subject",
    "tag",
    "date",
    "is",
    "has",
    "folder",
    "body",
    "attachment",
];

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// True when a `:word` token may start after `before`: the start of the text,
/// a blank or an opening mark.
fn starts_token(before: Option<char>) -> bool {
    before.is_none_or(|c| c.is_whitespace() || matches!(c, '(' | ',' | '[' | '"'))
}

/// Marks `keyword:` (the keyword and its colon) and `:word` (the colon and
/// the word).
pub fn find_search_tokens(text: &str, found: &mut Found) {
    for (colon, _) in text.match_indices(':') {
        let before = &text[..colon];
        let word_start = before
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_word_char(*c))
            .last()
            .map_or(colon, |(index, _)| index);
        if SEARCH_KEYWORDS.contains(&&text[word_start..colon]) {
            found.add(word_start..colon + 1, BodyRole::Token);
            continue;
        }
        let after = &text[colon + 1..];
        let length = after.find(|c: char| !is_word_char(c)).unwrap_or(after.len());
        if length > 0 && starts_token(before.chars().next_back()) {
            found.add(colon..colon + 1 + length, BodyRole::Token);
        }
    }
}

/// The length of a version number at the start of `text`: three or more
/// dot-separated numbers, optionally behind a `v`, not followed by a letter.
fn version_length(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut at = usize::from(bytes.first() == Some(&b'v'));
    let mut parts = 0;
    loop {
        let digits = bytes[at..].iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 {
            break;
        }
        at += digits;
        parts += 1;
        let dot_then_digit =
            bytes.get(at) == Some(&b'.') && bytes.get(at + 1).is_some_and(u8::is_ascii_digit);
        if !dot_then_digit {
            break;
        }
        at += 1;
    }
    let next = text[at..].chars().next();
    (parts >= 3 && !next.is_some_and(is_word_char)).then_some(at)
}

/// Marks version numbers such as `v1.3.0` and `1.3.0`.
pub fn find_versions(text: &str, found: &mut Found) {
    let mut before: Option<char> = None;
    for (index, c) in text.char_indices() {
        let starts = (c == 'v' || c.is_ascii_digit())
            && !before.is_some_and(|b| is_word_char(b) || b == '.');
        if starts {
            if let Some(length) = version_length(&text[index..]) {
                found.add(index..index + length, BodyRole::Token);
            }
        }
        before = Some(c);
    }
}

#[cfg(test)]
include!("tokens.tests.rs");
