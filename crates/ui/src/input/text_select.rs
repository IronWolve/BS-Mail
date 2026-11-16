//! Selecting text in the reading pane: a press and a drag over the words, or
//! `Shift` with the arrow keys, `Home`, `End`, `PageUp` and `PageDown` while
//! the reader has focus. The selection is made of places in the wrapped text,
//! so it follows the words when the pane scrolls, and a drag past the top or
//! the bottom of the pane scrolls it. `y` copies the selection.

use super::action::Direction;
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::reader_tools::TextSelection;
use crate::widgets::{BodyPos, BodyText};

/// The place `direction` takes `from` to in `text`; `page` is how many rows a
/// page is.
fn moved(text: &BodyText, from: BodyPos, direction: Direction, page: usize) -> BodyPos {
    let last = text.len().saturating_sub(1);
    let on_row = |row: usize, col: u16| BodyPos { row, col: col.min(text.row_width(row).saturating_sub(1)) };
    match direction {
        Direction::Right if from.col.saturating_add(1) < text.row_width(from.row) => BodyPos { col: from.col + 1, ..from },
        Direction::Right if from.row < last => BodyPos { row: from.row + 1, col: 0 },
        Direction::Left if from.col > 0 => BodyPos { col: from.col - 1, ..from },
        Direction::Left if from.row > 0 => on_row(from.row - 1, u16::MAX),
        Direction::Down => on_row((from.row + 1).min(last), from.col),
        Direction::Up => on_row(from.row.saturating_sub(1), from.col),
        Direction::PageDown => on_row((from.row + page.max(1)).min(last), from.col),
        Direction::PageUp => on_row(from.row.saturating_sub(page.max(1)), from.col),
        Direction::Home => BodyPos { col: 0, ..from },
        Direction::End => on_row(from.row, u16::MAX),
        Direction::Right | Direction::Left => from,
    }
}

impl AppView {
    /// `Shift` and a movement key: in the reader the selection grows (from the
    /// top of what is shown when nothing is selected); anywhere else the key
    /// does what it does alone.
    pub(crate) fn extend_selection(&mut self, direction: Direction, grid: (u16, u16)) {
        let layout = self.layout(grid.0, grid.1);
        let Some((text, rect)) = self.reader_text(&layout) else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let key = text.key();
        let mut selection = match self.reading.selection.take().filter(|held| held.key == key && held.spread) {
            Some(held) => held,
            None => TextSelection::from_keys(BodyPos { row: text.first(), col: 0 }, key),
        };
        let to = moved(&text, selection.focus, direction, usize::from(rect.height).saturating_sub(1));
        selection.extend_to(to);
        selection.dragging = false;
        self.reading.selection = Some(selection);
        self.keep_row_on_screen(to.row, &layout);
    }

    /// Scrolls the least that brings `row` of the wrapped text on screen.
    fn keep_row_on_screen(&mut self, row: usize, layout: &Layout) {
        let Some((_, height)) = self.reader_extent(layout) else {
            return;
        };
        if row < self.scroll.reader {
            self.scroll.reader = row;
        } else if row >= self.scroll.reader + height {
            self.scroll.reader = row + 1 - height;
        }
    }

    /// A left press on the reader's text starts a selection; it selects
    /// nothing until the pointer moves to another cell.
    pub(crate) fn begin_selection(&mut self, layout: &Layout, col: u16, row: u16) {
        let Some(at) = self.reader_pos_at(layout, col, row) else {
            self.reading.selection = None;
            return;
        };
        let key = self.reader_text(layout).map(|(text, _)| text.key());
        self.reading.selection = key.map(|key| TextSelection::pressed(at, key));
    }

    /// The pointer moved with the button held: the selection follows it. Past
    /// the top or the bottom of the text the pane scrolls a row. Returns
    /// whether a selection is being dragged.
    pub(crate) fn drag_selection_to(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        if !self.reading.selection.as_ref().is_some_and(|held| held.dragging) {
            return false;
        }
        let layout = self.layout(grid.0, grid.1);
        let Some((text, rect)) = self.reader_text(&layout) else {
            return true;
        };
        let (lines, height) = (text.len(), usize::from(rect.height));
        if row < rect.y {
            self.scroll.reader = self.scroll.reader.saturating_sub(1);
        } else if row >= rect.bottom() {
            self.scroll.reader = (self.scroll.reader + 1).min(lines.saturating_sub(height));
        }
        let inside = row.clamp(rect.y, rect.bottom().saturating_sub(1));
        if let Some(at) = self.reader_pos_at(&layout, col, inside)
            && let Some(held) = self.reading.selection.as_mut()
        {
            held.extend_to(at);
        }
        true
    }

    /// The button came up at `col`, `row`. A drag leaves its selection. A click
    /// selects nothing, and opens the address it was on when it comes up on the
    /// same one. Returns whether a press on the text was being followed.
    pub(crate) fn end_selection(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let Some(held) = self.reading.selection.as_mut().filter(|held| held.dragging) else {
            return false;
        };
        held.dragging = false;
        let pending = self.reading.pending_link.take();
        if held.spread {
            return true;
        }
        self.reading.selection = None;
        if let Some(address) = pending {
            let layout = self.layout(grid.0, grid.1);
            if self.link_under(&layout, col, row).as_deref() == Some(address.as_str()) {
                self.open_link(address);
            }
        }
        true
    }
}

#[cfg(test)]
include!("text_select.tests.rs");
