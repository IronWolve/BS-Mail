//! The wrapped body as plain text, for what works on the words as drawn:
//! copying a selection, finding text and telling which cell the pointer is
//! over. Positions are a wrapped row (counted from the top of the whole body,
//! not of the screen) and a cell in it, so they stay on the same words while
//! the body scrolls.

use surface::Rect;
use unicode_segmentation::UnicodeSegmentation;

use super::cache::Wrapped;
use super::draw::{BodyView, TEXT_AT, wrap_width, wrapped};
use crate::widgets::DrawContext;
use crate::widgets::cell_text::cells;

/// A place in the wrapped body: a row of it and a cell of that row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BodyPos {
    pub row: usize,
    pub col: u16,
}

/// The wrapped rows of a body, as they are drawn at one width.
pub struct BodyText {
    body: Wrapped,
    first: usize,
    visible: usize,
}

/// The body drawn in `rect` with `view`, as text.
pub fn body_text(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> BodyText {
    let body = wrapped(ctx, rect, view);
    let visible = usize::from(rect.height);
    let first = view.first.min(body.len().saturating_sub(visible));
    BodyText { body, first, visible }
}

/// The place under the cell `col`, `row` of the screen, for a body drawn in
/// `rect`. A cell left or right of the text is the nearest place on its row.
/// `None` for a row outside the body or a body with no rows.
pub fn body_pos_at(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>, col: u16, row: u16) -> Option<BodyPos> {
    if rect.is_empty() || row < rect.y || row >= rect.bottom() {
        return None;
    }
    let text = body_text(ctx, rect, view);
    let at = text.first + usize::from(row - rect.y);
    if at >= text.len() {
        return None;
    }
    let room = wrap_width(ctx, rect).saturating_sub(1);
    let col = col.saturating_sub(rect.x.saturating_add(TEXT_AT)).min(room);
    Some(BodyPos { row: at, col })
}

impl BodyText {
    /// What the rows were made from; another number is another body.
    pub fn key(&self) -> u64 {
        self.body.key
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn is_empty(&self) -> bool {
        self.body.len() == 0
    }

    /// The first row on screen.
    pub fn first(&self) -> usize {
        self.first
    }

    /// How many rows the body has room for on screen.
    pub fn visible(&self) -> usize {
        self.visible
    }

    /// The text of one row.
    pub fn row_text(&self, row: usize) -> String {
        self.body.rows.get(row).map(|pieces| pieces.iter().map(|piece| piece.text.as_str()).collect()).unwrap_or_default()
    }

    /// How many cells one row takes.
    pub fn row_width(&self, row: usize) -> u16 {
        self.body.rows.get(row).map_or(0, |pieces| pieces.iter().fold(0u16, |sum, piece| sum.saturating_add(cells(&piece.text))))
    }

    /// Whether a row carries on the line before it, which was wrapped.
    fn continues(&self, row: usize) -> bool {
        self.body.continued.get(row).copied().unwrap_or(false)
    }

    /// Whether the end of the row before and the start of this one are the
    /// same web address, which was cut over the two.
    fn same_link_as_before(&self, row: usize) -> bool {
        let last = row.checked_sub(1).and_then(|before| self.body.rows.get(before)).and_then(|pieces| pieces.last());
        let first = self.body.rows.get(row).and_then(|pieces| pieces.first());
        matches!((last, first), (Some(a), Some(b)) if a.link.is_some() && a.link == b.link)
    }

    /// The text of the cells `from` to `to` (included; the end of the row for
    /// `None`) of a row. A character is in when its first cell is.
    pub fn cell_slice(&self, row: usize, from: u16, to: Option<u16>) -> String {
        let mut out = String::new();
        let mut at = 0u16;
        for grapheme in self.row_text(row).graphemes(true) {
            if to.is_some_and(|last| at > last) {
                break;
            }
            if at >= from {
                out.push_str(grapheme);
            }
            at = at.saturating_add(cells(grapheme));
        }
        out
    }

    /// The text from `start` to `end`, both included: rows end with a line
    /// break, except where a long line was wrapped, which is put together again.
    pub fn text_between(&self, start: BodyPos, end: BodyPos) -> String {
        let mut out = String::new();
        let last = end.row.min(self.len().saturating_sub(1));
        for row in start.row..=last {
            if row > start.row {
                if !self.continues(row) {
                    out.push('\n');
                } else if !self.same_link_as_before(row) {
                    out.push(' ');
                }
            }
            let from = if row == start.row { start.col } else { 0 };
            let to = (row == end.row).then_some(end.col);
            out.push_str(&self.cell_slice(row, from, to));
        }
        out
    }

    /// The whole body as text, lines put together again.
    pub fn all_text(&self) -> String {
        let Some(last) = self.len().checked_sub(1) else {
            return String::new();
        };
        self.text_between(BodyPos { row: 0, col: 0 }, BodyPos { row: last, col: u16::MAX })
    }
}

#[cfg(test)]
include!("text_map.tests.rs");
