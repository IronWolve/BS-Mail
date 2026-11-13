//! Addresses of web pages and `(path/like-tokens)` in parentheses.

use model::BodyRole;

use super::found::Found;

/// Link schemes the highlighter knows.
const SCHEMES: [&str; 2] = ["https://", "http://"];
/// Marks that end a sentence, not a link.
const TRAILING: [char; 9] = ['.', ',', ';', ':', '!', '?', ')', ']', '}'];

/// Marks every `http://` and `https://` address up to the next blank,
/// without the punctuation a sentence puts after it.
pub fn find_links(text: &str, found: &mut Found) {
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let Some(scheme) = SCHEMES.iter().filter_map(|s| rest.find(s).map(|i| (i, s.len()))).min()
        else {
            return;
        };
        let start = at + scheme.0;
        let tail = &text[start..];
        let stop = tail.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"')).unwrap_or(tail.len());
        let link = tail[..stop].trim_end_matches(TRAILING);
        if link.len() > scheme.1 {
            found.add(start..start + link.len(), BodyRole::Link);
        }
        at = start + stop.max(scheme.1);
    }
}

fn is_path_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '_' | '/' | '-' | '~')
}

/// Marks a path-like token (one that holds a `/`) that is the whole content
/// of a pair of parentheses; the parentheses stay plain.
pub fn find_paren_paths(text: &str, found: &mut Found) {
    for (open, _) in text.match_indices('(') {
        let inner = &text[open + 1..];
        let length = inner.find(|c: char| !is_path_char(c)).unwrap_or(inner.len());
        let closed = inner[length..].starts_with(')');
        let token = &inner[..length];
        if closed && token.contains('/') && !token.starts_with("//") {
            found.add(open + 1..open + 1 + length, BodyRole::Path);
        }
    }
}

#[cfg(test)]
include!("links.tests.rs");
