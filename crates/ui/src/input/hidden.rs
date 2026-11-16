//! Input while the screen is too small to draw anything but its warning.

use surface::{ButtonState, InputEvent};

use crate::app_view::AppView;

impl AppView {
    /// Input when only the line saying the window is too small is drawn: no
    /// key or click may act on what is not shown, so nothing is sent, deleted
    /// or moved. A question can still be dismissed, and a button let go of
    /// ends a drag that was begun.
    pub(crate) fn input_while_hidden(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            _ if self.delete_prompt.is_some() => self.delete_prompt_input(event, grid),
            _ if self.ask.is_some() => self.ask_input(event, grid),
            InputEvent::Button { state: ButtonState::Released, .. } => self.end_drag(),
            _ => false,
        }
    }
}
