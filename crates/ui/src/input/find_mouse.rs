//! The mouse on the find box: a click in the query puts the caret there, the
//! three words at the right go to the previous match, the next one, and close
//! the box. A click anywhere else in the pane, or outside it, stops the box
//! taking letters and goes on to do what it does.

use surface::{ButtonState, InputEvent, PointerButton};

use crate::app_view::AppView;
use crate::composer::pointer_moves::col_at_cell;
use crate::layout::Layout;
use crate::search::geometry::holds;
use crate::widgets::find_bar::{FindParts, find_parts, text_skip};

impl AppView {
    /// Where the parts of the open find box are, when it is open and fits.
    fn find_parts_now(&self, layout: &Layout) -> Option<FindParts> {
        let find = self.reading.find.as_ref()?;
        let (text, rect) = self.reader_text(layout)?;
        let status = find.status(&find.matches(&text));
        find_parts(rect, u16::try_from(status.chars().count()).unwrap_or(u16::MAX))
    }

    /// Whether the cell is on one of the find box's three words, where a click acts.
    pub(crate) fn over_find_word(&self, layout: &Layout, col: u16, row: u16) -> bool {
        self.find_parts_now(layout).is_some_and(|parts| {
            [parts.previous, parts.next, parts.close].iter().any(|word| word.width > 0 && holds(*word, col, row))
        })
    }

    /// A press on the find box. Returns whether it was on the box.
    pub(crate) fn click_find_bar(&mut self, layout: &Layout, col: u16, row: u16, shift: bool, grid: (u16, u16)) -> bool {
        let Some(parts) = self.find_parts_now(layout) else {
            return false;
        };
        if !holds(parts.bar, col, row) {
            return false;
        }
        if holds(parts.close, col, row) {
            self.close_find();
        } else if holds(parts.next, col, row) {
            self.find_step(true, grid);
        } else if holds(parts.previous, col, row) {
            self.find_step(false, grid);
        } else if col >= parts.text_x
            && let Some(find) = self.reading.find.as_mut()
        {
            find.editing = true;
            let line = find.editor.lines()[0].clone();
            let skip = text_skip(&line, find.editor.cursor().col, parts.room);
            let at = col_at_cell(&line, skip, usize::from(col - parts.text_x));
            find.editor.move_to_col(at, shift);
        }
        true
    }

    /// A pointer event while the box takes the letters: a press on the box is
    /// its own; a press elsewhere ends the typing and acts as it would; moves
    /// and the wheel go on to the screen.
    pub(crate) fn find_pointer(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if let InputEvent::Button { button, state: ButtonState::Pressed, pos, modifiers } = event {
            let layout = self.layout(grid.0, grid.1);
            if *button == PointerButton::Left && self.click_find_bar(&layout, pos.col, pos.row, modifiers.shift, grid) {
                return true;
            }
            if let Some(find) = self.reading.find.as_mut() {
                find.editing = false;
            }
        }
        self.pointer_event(event, grid)
    }
}

#[cfg(test)]
include!("find_mouse.tests.rs");
