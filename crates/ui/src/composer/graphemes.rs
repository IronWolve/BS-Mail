//! Small helpers for working in grapheme clusters, the unit a user thinks of
//! as one character: a letter with its accent, an emoji with its modifier.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// How many graphemes `text` holds.
pub fn count(text: &str) -> usize {
    text.graphemes(true).count()
}

/// The byte offset where grapheme number `index` starts; the length of the
/// text when `index` is past the end.
pub fn byte_of(text: &str, index: usize) -> usize {
    text.grapheme_indices(true)
        .nth(index)
        .map_or(text.len(), |(offset, _)| offset)
}

/// The graphemes of `text`, in order.
pub fn list(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

/// How many cells a grapheme takes: two for a wide one, one otherwise, and
/// at least one so nothing has no width at all.
pub fn width(grapheme: &str) -> usize {
    UnicodeWidthStr::width(grapheme).max(1)
}

/// The cells taken by graphemes `from` up to but not including `to`.
pub fn cells_between(text: &str, from: usize, to: usize) -> usize {
    text.graphemes(true).skip(from).take(to.saturating_sub(from)).map(width).sum()
}

/// A character that separates words.
pub fn is_blank(grapheme: &str) -> bool {
    grapheme.chars().all(char::is_whitespace)
}

#[cfg(test)]
include!("graphemes.tests.rs");
