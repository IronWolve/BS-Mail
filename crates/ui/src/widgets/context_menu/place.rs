//! Where a menu sits: at the cell it was opened at, moved so that the whole
//! box stays on the grid, and which entry is at a cell.

use surface::Rect;

use super::state::Menu;
use crate::format::display_width;

/// Cells of blank before and after each label.
const PAD: u16 = 1;
/// Fewest cells a menu is wide, so a short title still fits.
const MIN_WIDTH: u16 = 12;

fn cells(text: &str) -> u16 {
    u16::try_from(display_width(text)).unwrap_or(u16::MAX)
}

/// The box for `menu` on `grid`: its top-left corner at the anchor, pushed
/// left and up as far as it takes to fit, and cut short only when the grid
/// is smaller than the menu.
pub fn menu_rect<A: Copy>(grid: Rect, menu: &Menu<A>) -> Rect {
    let labels = menu.entries.iter().map(|entry| cells(&entry.label)).max().unwrap_or(0);
    // The title sits two cells in from the corner, between blanks.
    let title = menu.title.as_deref().map_or(0, |title| cells(title) + 2);
    let width = (labels + PAD * 2 + 2).max(title + 3).max(MIN_WIDTH).min(grid.width);
    let wanted = u16::try_from(menu.entries.len()).unwrap_or(u16::MAX).saturating_add(2);
    let height = wanted.min(grid.height);
    let x = menu.anchor.0.min(grid.right().saturating_sub(width)).max(grid.x);
    let y = menu.anchor.1.min(grid.bottom().saturating_sub(height)).max(grid.y);
    Rect::new(x, y, width, height)
}

/// How many entries the box has rows for.
pub fn rows_showing(rect: Rect) -> usize {
    usize::from(rect.height.saturating_sub(2))
}

/// The index of the first entry showing: the highlight is always in view.
pub fn first_showing(cursor: usize, room: usize) -> usize {
    cursor.saturating_sub(room.saturating_sub(1))
}

/// The entry at cell (`col`, `row`) of a menu drawn in `rect`, if one is.
pub fn entry_at<A: Copy>(rect: Rect, menu: &Menu<A>, col: u16, row: u16) -> Option<usize> {
    let room = rows_showing(rect);
    let inside = col > rect.x && col + 1 < rect.right() && row > rect.y && row + 1 < rect.bottom();
    if !inside {
        return None;
    }
    let index = first_showing(menu.cursor, room) + usize::from(row - rect.y - 1);
    (index < menu.entries.len()).then_some(index)
}

#[cfg(test)]
include!("place.tests.rs");
