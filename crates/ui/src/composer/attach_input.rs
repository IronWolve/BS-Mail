//! The keys and clicks of the Attach row and the path box over the composer.
//! `Ctrl+O` (or Enter on the row, or a click on Add or on the empty row) opens
//! the path box; while it is open it takes every key, text and click. A path
//! it accepts waits for the program, which reads the file (the form never
//! touches one). With the cursor on the row, Left and Right choose a file and
//! Delete or Backspace takes it off; a click on a chip chooses it, and on
//! Remove takes it off.

use surface::{InputEvent, KeyEvent, LogicalKey, NamedKey, Rect};

use super::attach_line::{attach_rects, chip_at, inside};
use super::attachments::AttachAsk;
use super::draw::ComposerRects;
use super::form::{ComposeForm, Field};
use super::input::Outcome;
use crate::path_entry::{PathOutcome, home_dir};

/// Input while the path box is open: every key, text and click is the box's.
/// `None` when it is not open.
pub fn path_box_input(form: &mut ComposeForm, event: &InputEvent, main: Rect) -> Option<Outcome> {
    let prompt = form.path.as_mut()?;
    let outcome = match event {
        InputEvent::Key(key) => prompt.key(key),
        InputEvent::Text(text) | InputEvent::Paste(text) => prompt.text(text),
        InputEvent::Button { .. } | InputEvent::Wheel { .. } => prompt.pointer(event, main),
        _ => PathOutcome::Ignored,
    };
    Some(match outcome {
        PathOutcome::Ignored => Outcome::Ignored,
        PathOutcome::Changed => Outcome::Changed,
        PathOutcome::Cancel => {
            form.path = None;
            Outcome::Changed
        }
        PathOutcome::Accept(path) => {
            form.path = None;
            let already = form.attached_wire();
            form.attach_asks.push(AttachAsk { path, already });
            Outcome::Changed
        }
    })
}

/// `Ctrl+O` anywhere in the form, and the keys of the Attach row while it has
/// focus. `None` for a key that is not one of them.
pub fn attach_key(form: &mut ComposeForm, key: &KeyEvent) -> Option<Outcome> {
    let held = key.modifiers;
    if held.alt || held.meta {
        return None;
    }
    if held.ctrl && !held.shift && key.key == LogicalKey::Char('o') {
        form.open_path_box(home_dir());
        return Some(Outcome::Changed);
    }
    if form.focus != Field::Attach || held.ctrl || held.shift {
        return None;
    }
    let changed = match key.key {
        LogicalKey::Named(NamedKey::Left) => form.step_attach(false),
        LogicalKey::Named(NamedKey::Right) => form.step_attach(true),
        LogicalKey::Named(NamedKey::Delete | NamedKey::Backspace) => form.remove_selected_file(),
        LogicalKey::Named(NamedKey::Enter) => {
            form.open_path_box(home_dir());
            true
        }
        _ => return None,
    };
    Some(if changed { Outcome::Changed } else { Outcome::Ignored })
}

/// A press at `at` on the Attach row: Add opens the path box, Remove takes the
/// chosen file off, a chip is chosen, and the empty row opens the box.
/// `None` for a press that is not on the row.
pub fn attach_press(form: &mut ComposeForm, rects: &ComposerRects, at: (u16, u16)) -> Option<Outcome> {
    let row = rects.attach;
    if at.1 != row.y || at.0 < row.x || at.0 >= row.right() {
        return None;
    }
    let parts = attach_rects(row, form);
    if inside(parts.add, at.0, at.1) {
        form.open_path_box(home_dir());
    } else if parts.remove.is_some_and(|remove| inside(remove, at.0, at.1)) {
        form.remove_selected_file();
    } else if let Some(file) = chip_at(&parts, at.0, at.1) {
        form.focus = Field::Attach;
        form.attach_cursor = Some(file);
    } else if form.can_focus_attach() {
        form.focus = Field::Attach;
    } else {
        form.open_path_box(home_dir());
    }
    Some(Outcome::Changed)
}

#[cfg(test)]
include!("attach_input.tests.rs");
