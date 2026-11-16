//! Keys, typed text and clicks into the second page of the account form. The
//! same keys work as on the first page: Tab and the arrows move, a letter is
//! typed where there is a box, Space and the arrows change a choice, F10 or
//! Ctrl+Enter connect and save. Escape goes back to the first page instead of
//! closing the form, so nothing typed is lost by a press that was meant to
//! leave the page. Nothing is ever copied out of the form.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::form::SetupForm;
use super::input::Outcome;
use super::settings_page::SettingsField;
use crate::composer::EditorBuffer;

/// A pane wide enough that a line of the signature never wraps, so a vertical
/// move is a move to the next line.
const NO_WRAP: usize = 4096;

pub fn handle_key(form: &mut SetupForm, key: &KeyEvent) -> Outcome {
    let held = key.modifiers;
    let focus = form.settings.focus;
    if key.key == LogicalKey::Named(NamedKey::Escape) {
        // While the program is connecting, Escape gives the try up, as it
        // does on the first page; otherwise it goes back a page.
        if form.waiting {
            return Outcome::Close;
        }
        form.show_details();
        return Outcome::Changed;
    }
    if form.waiting {
        return Outcome::Ignored;
    }
    match key.key {
        LogicalKey::Named(NamedKey::Function(10)) => Outcome::Submit,
        LogicalKey::Named(NamedKey::Enter) if held.ctrl => Outcome::Submit,
        LogicalKey::Named(NamedKey::Enter) => enter(form, focus),
        LogicalKey::Named(named) => named_key(form, named, held.shift, held.ctrl),
        LogicalKey::Char(ch) if held.ctrl => shortcut(form, ch, held.shift),
        LogicalKey::Char(_) => Outcome::Ignored,
    }
}

/// Typed or pasted text. On a choice a space changes it; on a button it
/// presses it.
pub fn handle_text(form: &mut SetupForm, text: &str) -> Outcome {
    if form.waiting {
        return Outcome::Ignored;
    }
    let focus = form.settings.focus;
    if focus.is_button() {
        return if text == " " { press(form, focus) } else { Outcome::Ignored };
    }
    if focus.is_choice() {
        if text != " " {
            return Outcome::Ignored;
        }
        form.change_settings_choice();
        return Outcome::Changed;
    }
    let typed = if focus == SettingsField::Signature { text.to_string() } else { one_line(text) };
    let Some(buffer) = form.settings_buffer_mut(focus) else {
        return Outcome::Ignored;
    };
    if !buffer.insert_text(&typed) {
        return Outcome::Ignored;
    }
    edited(form);
    Outcome::Changed
}

/// A click on a row: the keyboard moves there, a choice changes, a button
/// presses.
pub fn click(form: &mut SetupForm, field: SettingsField) -> Outcome {
    if form.waiting {
        return Outcome::Ignored;
    }
    form.settings.focus = field;
    form.quit_armed = false;
    if field.is_button() {
        return press(form, field);
    }
    if field.is_choice() {
        form.change_settings_choice();
    }
    Outcome::Changed
}

fn press(form: &mut SetupForm, field: SettingsField) -> Outcome {
    if field == SettingsField::Back {
        form.show_details();
        Outcome::Changed
    } else {
        Outcome::Submit
    }
}

fn enter(form: &mut SetupForm, focus: SettingsField) -> Outcome {
    match focus {
        SettingsField::Signature => {
            if let Some(buffer) = form.settings_buffer_mut(focus) {
                buffer.insert_newline();
            }
            edited(form);
        }
        SettingsField::Back | SettingsField::Save => return press(form, focus),
        field if field.is_choice() => form.change_settings_choice(),
        _ => step(form, true),
    }
    Outcome::Changed
}

fn named_key(form: &mut SetupForm, key: NamedKey, shift: bool, ctrl: bool) -> Outcome {
    let focus = form.settings.focus;
    match key {
        NamedKey::Tab if shift => step(form, false),
        NamedKey::Tab => step(form, true),
        NamedKey::Down | NamedKey::Up => vertical(form, key == NamedKey::Down),
        NamedKey::Left | NamedKey::Right if focus.is_choice() => {
            if !focus.is_button() {
                form.change_settings_choice();
            }
        }
        NamedKey::Left if ctrl => edit(form, |buffer| buffer.move_word_left(shift)),
        NamedKey::Right if ctrl => edit(form, |buffer| buffer.move_word_right(shift)),
        NamedKey::Left => edit(form, |buffer| buffer.move_left(shift)),
        NamedKey::Right => edit(form, |buffer| buffer.move_right(shift)),
        NamedKey::Home => edit(form, |buffer| buffer.move_line_start(shift)),
        NamedKey::End => edit(form, |buffer| buffer.move_line_end(shift)),
        NamedKey::Backspace | NamedKey::Delete => return delete(form, key, ctrl),
        _ => return Outcome::Ignored,
    }
    form.quit_armed = false;
    Outcome::Changed
}

/// Up and Down move a line in the signature while there is one to move to,
/// and move on to the next row otherwise.
fn vertical(form: &mut SetupForm, down: bool) {
    if form.settings.focus == SettingsField::Signature {
        let buffer = &mut form.settings.signature;
        let at = buffer.cursor().line;
        let can_move = if down { at + 1 < buffer.lines().len() } else { at > 0 };
        if can_move {
            buffer.move_vertical(down, NO_WRAP, false);
            return;
        }
    }
    step(form, down);
}

fn step(form: &mut SetupForm, forward: bool) {
    form.settings.focus = if forward { form.settings.focus.next() } else { form.settings.focus.previous() };
}

/// Cursor movement on the row that has the keyboard; nothing on a choice.
fn edit(form: &mut SetupForm, action: impl FnOnce(&mut EditorBuffer)) {
    let focus = form.settings.focus;
    if let Some(buffer) = form.settings_buffer_mut(focus) {
        action(buffer);
    }
}

fn delete(form: &mut SetupForm, key: NamedKey, ctrl: bool) -> Outcome {
    let focus = form.settings.focus;
    let Some(buffer) = form.settings_buffer_mut(focus) else {
        return Outcome::Ignored;
    };
    let changed = match key {
        NamedKey::Backspace if ctrl => buffer.delete_word_back(),
        NamedKey::Backspace => buffer.backspace(),
        _ => buffer.delete_forward(),
    };
    if !changed {
        return Outcome::Ignored;
    }
    edited(form);
    Outcome::Changed
}

/// Ctrl with a letter: select all, undo and redo. Nothing is copied or cut.
fn shortcut(form: &mut SetupForm, ch: char, shift: bool) -> Outcome {
    let focus = form.settings.focus;
    let Some(buffer) = form.settings_buffer_mut(focus) else {
        return Outcome::Ignored;
    };
    let edits_text = match ch.to_ascii_lowercase() {
        'a' => {
            buffer.select_all();
            false
        }
        'z' if shift => buffer.redo(),
        'z' => buffer.undo(),
        'y' => buffer.redo(),
        _ => return Outcome::Ignored,
    };
    if edits_text {
        edited(form);
    }
    Outcome::Changed
}

/// What an edit of a box does to the form: an old problem no longer applies.
/// (The first page's own bookkeeping, of which server the user wrote and
/// which name they chose, is not touched by this page.)
fn edited(form: &mut SetupForm) {
    form.notice = None;
    form.info = None;
    form.quit_armed = false;
}

/// Text on one line with the control codes taken out, so a paste cannot carry
/// a newline or an escape sequence into a box.
fn one_line(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

#[cfg(test)]
include!("settings_input.tests.rs");
