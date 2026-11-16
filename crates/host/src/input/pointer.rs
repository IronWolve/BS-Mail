//! Pointer buttons and the wheel.
//!
//! A wheel that reports pixels is added up until it has turned a whole row,
//! so a touchpad scrolls smoothly instead of jumping a row per flick.

use surface::{ButtonState, InputEvent, PointerButton};
use winit::event::{MouseButton, MouseScrollDelta};

/// The button, or `None` for one the screen has no use for.
pub fn pointer_button(button: MouseButton) -> Option<PointerButton> {
    let button = match button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(_) => return None,
    };
    Some(button)
}

/// A press of the middle button pastes the primary selection into a text
/// field (the habit of X11 and Wayland): the paste event it is, when `primary`
/// has text. Every other press and release, and a middle press with nothing
/// selected, is not one. `primary` is only asked for a middle press.
pub fn middle_paste(
    button: PointerButton,
    state: ButtonState,
    primary: impl FnOnce() -> Option<String>,
) -> Option<InputEvent> {
    if button != PointerButton::Middle || state != ButtonState::Pressed {
        return None;
    }
    primary().filter(|text| !text.is_empty()).map(InputEvent::Paste)
}

/// Pixel-precise scrolling, kept until it adds up to whole rows and columns.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WheelCarry {
    rows: f64,
    cols: f64,
}

impl WheelCarry {
    /// Adds one wheel event and returns the whole rows and columns it
    /// completes, positive down and right.
    pub fn add(&mut self, delta: MouseScrollDelta, cell: (u32, u32)) -> (i16, i16) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                self.cols += f64::from(x);
                self.rows += f64::from(y);
            }
            MouseScrollDelta::PixelDelta(position) => {
                self.cols += position.x / f64::from(cell.0.max(1));
                self.rows += position.y / f64::from(cell.1.max(1));
            }
        }
        let rows = whole(&mut self.rows);
        let cols = whole(&mut self.cols);
        // The wheel reports up as positive; rows count down.
        (-rows, cols)
    }
}

/// The whole part, left behind in `value`.
fn whole(value: &mut f64) -> i16 {
    let whole = value.trunc();
    *value -= whole;
    whole.clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
}

#[cfg(test)]
include!("pointer.tests.rs");
