//! Mouse reports from the terminal as pointer events.
//!
//! A terminal gives the cell and nothing finer, so the position inside the
//! cell is the middle of it.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use surface::{ButtonState, CellPosition, InputEvent, Modifiers, PointerButton};

use super::keys_modifiers;

pub fn mouse_events(mouse: &MouseEvent) -> Vec<InputEvent> {
    let pos = CellPosition::at(mouse.column, mouse.row);
    let modifiers: Modifiers = keys_modifiers(mouse.modifiers);
    let button = |button: MouseButton, state: ButtonState| InputEvent::Button {
        button: match button {
            MouseButton::Left => PointerButton::Left,
            MouseButton::Right => PointerButton::Right,
            MouseButton::Middle => PointerButton::Middle,
        },
        state,
        pos,
        modifiers,
    };
    let wheel = |rows: i16, cols: i16| InputEvent::Wheel {
        pos,
        rows,
        cols,
        modifiers,
    };
    vec![match mouse.kind {
        MouseEventKind::Down(which) => button(which, ButtonState::Pressed),
        MouseEventKind::Up(which) => button(which, ButtonState::Released),
        MouseEventKind::Drag(_) | MouseEventKind::Moved => InputEvent::PointerMoved { pos, modifiers },
        MouseEventKind::ScrollUp => wheel(-1, 0),
        MouseEventKind::ScrollDown => wheel(1, 0),
        MouseEventKind::ScrollLeft => wheel(0, -1),
        MouseEventKind::ScrollRight => wheel(0, 1),
    }]
}

#[cfg(test)]
include!("mouse.tests.rs");
