//! The marks laid over the reader's body once it is drawn: the selection and
//! the matches of a find. They are restyled in place on the cells that hold
//! the words, so the text under them is not drawn twice.

use surface::{Frame, Rect, Style};

use super::draw::{BodyView, TEXT_AT};
use super::text_map::{BodyPos, BodyText, body_text};
use crate::theme::roles;
use crate::widgets::DrawContext;

/// One match of a find: cells `from` up to (not including) `to` of a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    pub row: usize,
    pub from: u16,
    pub to: u16,
}

/// What to mark.
#[derive(Debug, Clone, Copy, Default)]
pub struct Highlights<'a> {
    /// The selected text, first place to last, both included.
    pub selection: Option<(BodyPos, BodyPos)>,
    /// The matches of the find, in the order they come in the body.
    pub found: &'a [Match],
    /// Which of them is the current one.
    pub current: Option<usize>,
}

impl Highlights<'_> {
    pub fn is_empty(&self) -> bool {
        self.selection.is_none() && self.found.is_empty()
    }

    /// The cells of `row` the selection covers, `from` up to `to` (not included).
    fn selected_cells(&self, text: &BodyText, row: usize) -> Option<(u16, u16)> {
        let (start, end) = self.selection?;
        if row < start.row || row > end.row {
            return None;
        }
        let width = text.row_width(row);
        let from = if row == start.row { start.col } else { 0 };
        let to = if row == end.row { end.col.saturating_add(1).min(width) } else { width };
        (from < to).then_some((from, to))
    }
}

/// Restyles the marked cells of the body drawn in `rect` with `view`.
pub fn draw_highlights(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>, marks: &Highlights<'_>) {
    if rect.is_empty() || marks.is_empty() {
        return;
    }
    let text = body_text(ctx, rect, view);
    let text_x = rect.x.saturating_add(TEXT_AT);
    let last = (text.first() + text.visible()).min(text.len());
    let first_match = marks.found.partition_point(|found| found.row < text.first());
    for (index, found) in marks.found.iter().enumerate().skip(first_match) {
        if found.row >= last {
            break;
        }
        let role = if marks.current == Some(index) { roles::READER_FOUND_CURRENT } else { roles::READER_FOUND };
        paint(frame, rect, text_x, found.row - text.first(), (found.from, found.to), ctx.full(role));
    }
    for row in text.first()..last {
        if let Some(cells) = marks.selected_cells(&text, row) {
            paint(frame, rect, text_x, row - text.first(), cells, ctx.full(roles::READER_SELECTION));
        }
    }
}

fn paint(frame: &mut Frame, rect: Rect, text_x: u16, line: usize, (from, to): (u16, u16), style: Style) {
    let y = rect.y.saturating_add(u16::try_from(line).unwrap_or(u16::MAX));
    for col in from..to {
        let x = text_x.saturating_add(col);
        if x >= rect.right() {
            break;
        }
        if let Some(cell) = frame.cells.cell_mut((x, y)) {
            cell.set_style(style);
        }
    }
}

#[cfg(test)]
include!("marks.tests.rs");
