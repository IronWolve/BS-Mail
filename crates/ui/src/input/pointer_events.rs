//! The pointer on the main screen: a press, a move with or without a button
//! held, a release and the wheel. Presses go to the bars and scrollbars first,
//! then to what is under them; the right button opens a context menu.

use surface::{ButtonState, InputEvent, PointerButton};

use crate::app_view::AppView;

impl AppView {
    /// One pointer event with nothing open over the mail. Returns whether the
    /// screen changed.
    pub(super) fn pointer_event(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, modifiers } => {
                // A drag whose release never came ends with the next press.
                self.cancel_row_drag();
                if self.begin_drag(pos.col, pos.row, grid) {
                    return true;
                }
                self.click(pos.col, pos.row, *modifiers, grid)
            }
            InputEvent::Button { button: PointerButton::Right, state: ButtonState::Pressed, pos, .. } => {
                self.cancel_row_drag();
                self.right_click(pos.col, pos.row, grid)
            }
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Released, pos, .. } => {
                let rows = self.drop_rows(*pos, grid);
                self.end_drag() || rows
            }
            InputEvent::PointerMoved { pos, .. } => self.drag_to(*pos, grid) || self.drag_rows_to(*pos, grid),
            InputEvent::Wheel { pos, rows, .. } => self.wheel(pos.col, pos.row, *rows, grid),
            _ => false,
        }
    }
}
