//! The mouse in the search box: a click in the query puts the caret under the
//! pointer (with Shift it selects from where the caret was), the two buttons
//! on the bottom border search and cancel, and a click outside the box closes
//! it like Escape does.

use surface::{ButtonState, InputEvent, PointerButton, PointerShape, Rect};

use crate::app_view::AppView;
use crate::composer::pointer_moves::col_at_cell;
use crate::search::geometry::{holds, search_parts, text_skip};

impl AppView {
    /// A pointer event while the search box is open. Returns whether the
    /// screen changed.
    pub(super) fn search_pointer(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, modifiers } = event else {
            return false;
        };
        let Some(parts) = search_parts(Rect::new(0, 0, grid.0, grid.1)) else {
            return false;
        };
        let (col, row) = (pos.col, pos.row);
        if holds(parts.go, col, row) {
            self.run_search();
        } else if holds(parts.cancel, col, row) || !holds(parts.rect, col, row) {
            self.search_bar = None;
        } else if row == parts.line.y && col >= parts.text_x
            && let Some(bar) = self.search_bar.as_mut()
        {
            let line = bar.lines()[0].clone();
            let skip = text_skip(&line, bar.cursor().col, parts.room);
            let target = col_at_cell(&line, skip, usize::from(col - parts.text_x));
            bar.move_to_col(target, modifiers.shift);
        }
        true
    }

    /// The pointer shape over the open search box: a hand over its buttons,
    /// the text pointer over the query.
    pub(crate) fn search_pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        let Some(parts) = search_parts(Rect::new(0, 0, grid.0, grid.1)) else {
            return PointerShape::Arrow;
        };
        if holds(parts.go, col, row) || holds(parts.cancel, col, row) {
            PointerShape::Hand
        } else if row == parts.line.y && col >= parts.text_x && col < parts.line.right() {
            PointerShape::Text
        } else {
            PointerShape::Arrow
        }
    }
}

#[cfg(test)]
include!("search_mouse.tests.rs");
