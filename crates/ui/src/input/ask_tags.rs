//! The tag picker (see `ask::tags`). Typed letters narrow the tags, the arrows
//! move the highlight, Enter sets the highlighted tag on the messages (or
//! clears it when they all have it) and Escape cancels. A typed word that is no
//! known tag is offered as a new one when it could be a tag, and says why when
//! it could not. The mouse works: a click on a row does what Enter does, a click
//! outside the box cancels, the wheel moves the highlight and the pointer over
//! a row highlights it.

use model::Command;
use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, Rect};

use super::done_note::Settles;
use crate::app_view::AppView;
use crate::ask::tags::Held;
use crate::ask::{Ask, TagPicker};
use crate::move_picker::rows::{inside_box, row_at};

/// Rows the highlight moves for one notch of the wheel.
const WHEEL_ROWS: usize = 3;

impl AppView {
    fn tag_picker_mut(&mut self) -> Option<&mut TagPicker> {
        match self.ask.as_mut() {
            Some(Ask::Tag(picker)) => Some(picker),
            _ => None,
        }
    }

    /// Input while the picker is open. Returns whether the screen changed.
    pub(crate) fn tag_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.tag_key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                self.tag_picker_mut().map(|picker| picker.type_text(text)).is_some()
            }
            _ => self.tag_pointer(event, grid),
        }
    }

    fn tag_key(&mut self, key: &KeyEvent) -> bool {
        let LogicalKey::Named(named) = key.key else {
            return false;
        };
        let Some(picker) = self.tag_picker_mut() else {
            return false;
        };
        match named {
            NamedKey::Escape => self.ask = None,
            NamedKey::Up => picker.step(false),
            NamedKey::Down => picker.step(true),
            NamedKey::PageUp => (0..8).for_each(|_| picker.step(false)),
            NamedKey::PageDown => (0..8).for_each(|_| picker.step(true)),
            NamedKey::Backspace => picker.backspace(),
            NamedKey::Enter => self.finish_tag(),
            _ => return false,
        }
        true
    }

    /// Sets the highlighted tag on the messages, or clears it when they all
    /// have it. With nothing to choose the picker stays open to be corrected.
    pub(crate) fn finish_tag(&mut self) {
        let Some(Ask::Tag(picker)) = &self.ask else {
            return;
        };
        let Some(row) = picker.chosen() else {
            return;
        };
        let targets = picker.targets.clone();
        self.ask = None;
        let on = row.held != Held::All;
        let count = targets.len();
        let (messages, tag) = (if count == 1 { "1 message".to_owned() } else { format!("{count} messages") }, &row.tag);
        let note = if on { format!("Tagged {messages} {tag}") } else { format!("Took the tag {tag} off {messages}") };
        self.queue.push(Command::SetTag { messages: targets, tag: row.tag, on });
        self.promise_note(note, Settles::Tags);
    }

    /// A pointer event while the picker is open.
    fn tag_pointer(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let area = Rect::new(0, 0, grid.0, grid.1);
        let Some(Ask::Tag(picker)) = &self.ask else {
            return false;
        };
        let (shown, cursor) = (picker.rows().len(), picker.cursor);
        match event {
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } => {
                match row_at(area, (shown, cursor), pos.col, pos.row) {
                    Some(index) => {
                        if let Some(picker) = self.tag_picker_mut() {
                            picker.cursor = index;
                        }
                        self.finish_tag();
                    }
                    None if !inside_box(area, shown, pos.col, pos.row) => self.ask = None,
                    None => {}
                }
                true
            }
            InputEvent::PointerMoved { pos, .. } => match row_at(area, (shown, cursor), pos.col, pos.row) {
                Some(index) if index != cursor => {
                    if let Some(picker) = self.tag_picker_mut() {
                        picker.cursor = index;
                    }
                    true
                }
                _ => false,
            },
            InputEvent::Wheel { rows, .. } if *rows != 0 => {
                if let Some(picker) = self.tag_picker_mut() {
                    (0..usize::from(rows.unsigned_abs()) * WHEEL_ROWS).for_each(|_| picker.step(*rows > 0));
                }
                true
            }
            _ => false,
        }
    }

    /// Whether the pointer is over a row of the open picker.
    pub(crate) fn tag_row_under(&self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let Some(Ask::Tag(picker)) = &self.ask else {
            return false;
        };
        row_at(Rect::new(0, 0, grid.0, grid.1), (picker.rows().len(), picker.cursor), col, row).is_some()
    }
}

#[cfg(test)]
include!("ask_tags.tests.rs");
