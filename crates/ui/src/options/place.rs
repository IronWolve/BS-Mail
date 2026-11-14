//! Where the items of the visible rows sit on screen, and which part of which
//! control a cell belongs to. Drawing and clicking both use this, so they
//! cannot disagree.

use surface::Rect;

use super::control::{Control, Part};
use super::layout::LABEL_CELLS;
use super::rows::{ITEM_GAP, Item, Row, STEP_CELLS};

/// One item of a visible row, with its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub rect: Rect,
    /// Index into the page's rows, and into that row's items.
    pub row: usize,
    pub item: usize,
}

/// The column where a row's items start.
pub fn items_x(page: Rect, row: &Row) -> u16 {
    if row.label.is_some() { page.x + LABEL_CELLS } else { page.x }
}

/// The screen row of page row `index`, when it is in view.
pub fn screen_row(page: Rect, scroll: usize, index: usize) -> Option<u16> {
    let offset = index.checked_sub(scroll)?;
    (offset < usize::from(page.height)).then(|| page.y + u16::try_from(offset).unwrap_or(u16::MAX))
}

/// The items of every row in view, left to right on each. An item that does
/// not fit is cut at the right edge; items past it are left out.
pub fn slots(rows: &[Row], page: Rect, scroll: usize) -> Vec<Slot> {
    let mut found = Vec::new();
    for (index, row) in rows.iter().enumerate().skip(scroll).take(usize::from(page.height)) {
        let Some(y) = screen_row(page, scroll, index) else {
            break;
        };
        let mut x = items_x(page, row);
        for (at, item) in row.items.iter().enumerate() {
            let room = page.right().saturating_sub(x);
            let width = item.width(room).min(room);
            if width == 0 {
                break;
            }
            found.push(Slot { rect: Rect::new(x, y, width, 1), row: index, item: at });
            x = x.saturating_add(width + ITEM_GAP);
        }
    }
    found
}

/// The control and part under a cell, if it is on one.
pub fn hit(rows: &[Row], page: Rect, scroll: usize, col: u16, row: u16) -> Option<(Control, Part)> {
    let slot = slots(rows, page, scroll)
        .into_iter()
        .find(|slot| row == slot.rect.y && col >= slot.rect.x && col < slot.rect.right())?;
    let rect = slot.rect;
    match &rows[slot.row].items[slot.item] {
        Item::Text { .. } => None,
        Item::Choice { control, .. } => Some((*control, if col < rect.x + 2 { Part::Prev } else { Part::Next })),
        Item::Stepper { control, .. } if col < rect.x + STEP_CELLS => Some((*control, Part::Minus)),
        Item::Stepper { control, .. } if col + STEP_CELLS >= rect.right() => Some((*control, Part::Plus)),
        Item::Stepper { .. } => None,
        Item::Button { control, .. } => Some((*control, Part::Whole)),
        Item::Account { index, .. } => Some((Control::AccountList, Part::Row(*index))),
    }
}

#[cfg(test)]
include!("place.tests.rs");
