//! Every kind of terminal event as the events the screen reads.

use crossterm::event::Event;
use surface::InputEvent;

use super::keys::key_events;
use super::mouse::mouse_events;

/// The screen's events for one terminal event; none for one it ignores.
pub fn translate(event: &Event) -> Vec<InputEvent> {
    match event {
        Event::Key(key) => key_events(key),
        Event::Mouse(mouse) => mouse_events(mouse),
        Event::Paste(text) => vec![InputEvent::Paste(text.clone())],
        Event::FocusGained => vec![InputEvent::Focus { gained: true }],
        Event::FocusLost => vec![InputEvent::Focus { gained: false }],
        Event::Resize(cols, rows) => vec![InputEvent::Resized {
            cols: *cols,
            rows: *rows,
        }],
    }
}

#[cfg(test)]
include!("events.tests.rs");
