//! Clicks and the wheel on the Options screen.

use super::control::{Control, Part};
use super::input::Outcome;
use super::layout::{OptionsRects, max_scroll, page_at};
use super::place::hit;
use super::rows::Row;
use super::state::{OptionsState, Stop};
use crate::input::movement::WHEEL_ROWS;

/// A left click at a cell. A page name shows that page; a control is given
/// the keyboard and pressed; a row of the account list is highlighted.
pub fn click(state: &mut OptionsState, rects: &OptionsRects, rows: &[Row], col: u16, row: u16) -> Outcome {
    if !state.confirming
        && let Some(page) = page_at(rects, col, row)
    {
        state.select_page(page);
        return Outcome::Changed;
    }
    let scroll = state.scroll.min(max_scroll(rows.len(), rects.page.height));
    match hit(rows, rects.page, scroll, col, row) {
        Some((Control::AccountList, Part::Row(index))) => {
            state.cursor = index;
            state.stop = Stop::Control(Control::AccountList);
            Outcome::Changed
        }
        Some((control, part)) => {
            state.stop = Stop::Control(control);
            Outcome::Press(control, part)
        }
        None => Outcome::Ignored,
    }
}

/// The wheel, `notches` positive for down, over a page of `total` rows.
pub fn wheel(state: &mut OptionsState, rects: &OptionsRects, total: usize, notches: i16) -> Outcome {
    let last = max_scroll(total, rects.page.height);
    let from = state.scroll.min(last);
    let by = usize::from(notches.unsigned_abs()) * WHEEL_ROWS;
    let to = if notches < 0 { from.saturating_sub(by) } else { from.saturating_add(by).min(last) };
    state.scroll = to;
    if to == from { Outcome::Ignored } else { Outcome::Changed }
}

/// Whether a cell is on something a click does.
pub fn is_clickable(state: &OptionsState, rects: &OptionsRects, rows: &[Row], col: u16, row: u16) -> bool {
    if !state.confirming && page_at(rects, col, row).is_some() {
        return true;
    }
    let scroll = state.scroll.min(max_scroll(rows.len(), rects.page.height));
    hit(rows, rects.page, scroll, col, row).is_some()
}

#[cfg(test)]
include!("mouse.tests.rs");
