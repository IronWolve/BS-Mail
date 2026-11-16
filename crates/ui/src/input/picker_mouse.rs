//! The mouse in the move picker: a click on a folder row moves the messages
//! there, a click outside the box cancels, the wheel moves the highlight
//! (and with it what shows), and the pointer over a row highlights it.

use surface::{ButtonState, InputEvent, PointerButton, Rect};

use crate::app_view::AppView;
use crate::move_picker::rows::{inside_box, row_at};

/// Folders the highlight moves for one notch of the wheel.
const WHEEL_ROWS: usize = 3;

impl AppView {
    /// A pointer event while the picker is open. Returns whether the screen
    /// changed.
    pub(super) fn picker_pointer(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let area = Rect::new(0, 0, grid.0, grid.1);
        let Some(picker) = self.picker.as_ref() else {
            return false;
        };
        let shown = picker.candidates(&self.mail.folders).len();
        let cursor = picker.cursor;
        match event {
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } => {
                match row_at(area, (shown, cursor), pos.col, pos.row) {
                    Some(index) => {
                        self.set_picker_cursor(index);
                        self.finish_move();
                    }
                    None if !inside_box(area, shown, pos.col, pos.row) => self.picker = None,
                    None => {}
                }
                true
            }
            InputEvent::PointerMoved { pos, .. } => match row_at(area, (shown, cursor), pos.col, pos.row) {
                Some(index) if index != cursor => {
                    self.set_picker_cursor(index);
                    true
                }
                _ => false,
            },
            InputEvent::Wheel { rows, .. } if *rows != 0 => {
                if let Some(picker) = self.picker.as_mut() {
                    (0..usize::from(rows.unsigned_abs()) * WHEEL_ROWS).for_each(|_| picker.step(*rows > 0, shown));
                }
                true
            }
            _ => false,
        }
    }

    fn set_picker_cursor(&mut self, index: usize) {
        if let Some(picker) = self.picker.as_mut() {
            picker.cursor = index;
        }
    }

    /// Whether the pointer is over a folder row of the open picker.
    pub(crate) fn picker_row_under(&self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let Some(picker) = self.picker.as_ref() else {
            return false;
        };
        let shown = picker.candidates(&self.mail.folders).len();
        row_at(Rect::new(0, 0, grid.0, grid.1), (shown, picker.cursor), col, row).is_some()
    }
}

#[cfg(test)]
include!("picker_mouse.tests.rs");
