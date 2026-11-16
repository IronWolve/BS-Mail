//! Which folder rows the picker shows and where: the drawing and the pointer
//! both ask here, so a click lands on the row the user sees.

use surface::Rect;

use super::draw::picker_rect;

/// Where the folder rows of the box in `rect` start, how many fit, and the
/// index of the first folder showing when the highlight is at `cursor`.
pub fn rows_span(rect: Rect, cursor: usize) -> (u16, usize, usize) {
    // Border, prompt, rule: the rows start three cells below the top.
    let top = rect.y + 3;
    let room = usize::from(rect.bottom().saturating_sub(1).saturating_sub(top));
    (top, room, cursor.saturating_sub(room.saturating_sub(1)))
}

/// Whether cell (`col`, `row`) is in the box the picker draws on `grid` for
/// `shown` folders.
pub fn inside_box(grid: Rect, shown: usize, col: u16, row: u16) -> bool {
    let rect = picker_rect(grid, shown);
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

/// The folder (as an index into the folders shown) whose row is at cell
/// (`col`, `row`), when the highlight is at `cursor`.
pub fn row_at(grid: Rect, (shown, cursor): (usize, usize), col: u16, row: u16) -> Option<usize> {
    let rect = picker_rect(grid, shown);
    let (top, room, first) = rows_span(rect, cursor);
    let in_columns = col > rect.x && col + 1 < rect.right();
    if !in_columns || row < top || usize::from(row - top) >= room {
        return None;
    }
    let index = first + usize::from(row - top);
    (index < shown).then_some(index)
}

#[cfg(test)]
include!("rows.tests.rs");
