//! The cue for which pane has focus: its title is underlined.
//!
//! Done after the pane is drawn, on the cells it already wrote, so no pane
//! has to know whether it is the focused one.

use surface::{Frame, Modifier, Rect};

/// Underlines every written cell of `title`, leaving blanks alone so the line
/// sits under the words and not across the gaps.
pub fn mark_focused(frame: &mut Frame, title: Rect) {
    for row in title.top()..title.bottom() {
        for col in title.left()..title.right() {
            let Some(cell) = frame.cells.cell_mut((col, row)) else {
                continue;
            };
            if !cell.symbol().trim().is_empty() {
                cell.modifier.insert(Modifier::UNDERLINED);
            }
        }
    }
}

#[cfg(test)]
include!("focus_mark.tests.rs");
