//! Moving a position in a list of `len` rows.

use super::action::Action;

/// Where `action` takes `current` in a list of `len` rows when a page is
/// `page` rows. An action that is not a movement leaves it where it is.
pub fn step(action: Action, current: usize, len: usize, page: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    let page = page.max(1);
    let next = match action {
        Action::Up => current.saturating_sub(1),
        Action::Down => current.saturating_add(1),
        Action::PageUp => current.saturating_sub(page),
        Action::PageDown => current.saturating_add(page),
        Action::First => 0,
        Action::Last => last,
        _ => current,
    };
    next.min(last)
}

/// Rows a wheel notch moves by, whatever the pane.
pub const WHEEL_ROWS: usize = 3;

#[cfg(test)]
include!("movement.tests.rs");
