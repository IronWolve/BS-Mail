//! Looking for words in the wrapped body: every place the query comes up, in
//! the order of the body, ignoring case. A match is on one row; words that
//! the wrapping broke over two rows are not found as one.

use crate::widgets::cell_text::cells;
use crate::widgets::{BodyText, Match};

/// The most matches that are kept; a query that comes up more often than this
/// is as good as every word.
pub const MAX_MATCHES: usize = 10_000;

/// The characters a character is compared as.
fn folded(c: char) -> impl Iterator<Item = char> {
    c.to_lowercase()
}

/// Every match of `query` in the body, in order.
pub fn find_matches(body: &BodyText, query: &str) -> Vec<Match> {
    let wanted: Vec<char> = query.chars().flat_map(folded).collect();
    let mut found = Vec::new();
    if wanted.is_empty() {
        return found;
    }
    for row in 0..body.len() {
        let text = body.row_text(row);
        if text.is_empty() {
            continue;
        }
        // Each character as compared, with the cells it takes.
        let mut chars: Vec<(char, u16, u16)> = Vec::with_capacity(text.len());
        let mut at = 0u16;
        for c in text.chars() {
            let mut buffer = [0u8; 4];
            let end = at.saturating_add(cells(c.encode_utf8(&mut buffer)));
            chars.extend(folded(c).map(|f| (f, at, end)));
            at = end;
        }
        let mut start = 0;
        while start + wanted.len() <= chars.len() {
            let here = &chars[start..start + wanted.len()];
            if here.iter().map(|(c, ..)| *c).eq(wanted.iter().copied()) {
                found.push(Match { row, from: here[0].1, to: here[here.len() - 1].2 });
                if found.len() >= MAX_MATCHES {
                    return found;
                }
                start += wanted.len();
            } else {
                start += 1;
            }
        }
    }
    found
}

#[cfg(test)]
include!("find_text.tests.rs");
