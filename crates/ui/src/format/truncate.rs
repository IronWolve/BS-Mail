//! Cuts text to a width in terminal cells, ending in an ellipsis. Widths come
//! from the same tables the layout uses; wide characters count two cells and
//! combining marks stay with the character they belong to.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Three dots: the From column of the picture.
pub const ELLIPSIS_DOTS: &str = "...";
/// The single-character ellipsis, one cell wide.
pub const ELLIPSIS_CHAR: &str = "\u{2026}";

/// Width of `text` in cells.
pub fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// `text` cut to at most `max_width` cells. Text that already fits is
/// returned whole; otherwise it is cut at a character boundary and
/// `ellipsis` is appended inside the budget. A wide character that would
/// straddle the cut is dropped, so the result can be a cell short of
/// `max_width`. If even the ellipsis does not fit, the front of the ellipsis
/// is returned. Control characters (tab, newline) become spaces, so the
/// width is exact.
pub fn truncate(text: &str, max_width: usize, ellipsis: &str) -> String {
    let text = single_line(text);
    if display_width(&text) <= max_width {
        return text;
    }
    let ellipsis_width = display_width(ellipsis);
    if ellipsis_width > max_width {
        return take_width(ellipsis, max_width);
    }
    let mut cut = take_width(&text, max_width - ellipsis_width);
    cut.push_str(ellipsis);
    cut
}

fn single_line(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// The longest prefix of whole characters that fits in `limit` cells.
pub fn take_width(text: &str, limit: usize) -> String {
    let mut used = 0;
    let mut out = String::new();
    for grapheme in text.graphemes(true) {
        let width = display_width(grapheme);
        if used + width > limit {
            break;
        }
        used += width;
        out.push_str(grapheme);
    }
    out
}

#[cfg(test)]
include!("truncate.tests.rs");
