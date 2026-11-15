//! Keys, typed text and clicks into the account form.
//!
//! A letter key arrives twice from a window: once as a key and once as the
//! text the layout made of it. Only the text is typed, so every layout works;
//! keys here are the ones with no text. Nothing is ever copied out of the
//! form, so the password cannot reach the clipboard from here.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::fields::SetupField;
use super::form::SetupForm;
use super::link_panel::PanelChoice;
use super::link_panel_input;

/// What the form wants done after an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Not something the form uses.
    Ignored,
    /// The form changed; draw it again.
    Changed,
    /// Check what was typed and connect.
    Submit,
    /// Close the form without saving.
    Close,
    /// A button on the sign-in link page, by key or click.
    Panel(PanelChoice),
}

const QUIT_NOTE: &str = "press Esc again to leave without saving";

pub fn handle_key(form: &mut SetupForm, key: &KeyEvent) -> Outcome {
    let held = key.modifiers;
    if held.alt || held.meta {
        return Outcome::Ignored;
    }
    if form.link.is_some() {
        return link_panel_input::key(key);
    }
    if form.provider_menu.is_some() {
        return match key.key {
            LogicalKey::Named(named) => form.menu_key(named),
            LogicalKey::Char(_) => Outcome::Ignored,
        };
    }
    if let LogicalKey::Named(NamedKey::Escape) = key.key {
        return escape(form);
    }
    if form.waiting {
        return Outcome::Ignored;
    }
    match key.key {
        LogicalKey::Named(NamedKey::Function(10)) => Outcome::Submit,
        LogicalKey::Named(NamedKey::Enter) if held.ctrl => Outcome::Submit,
        LogicalKey::Named(NamedKey::Enter) if form.focus == SetupField::Save => Outcome::Submit,
        LogicalKey::Named(NamedKey::Enter) if form.focus == SetupField::Provider => {
            form.open_provider_menu();
            Outcome::Changed
        }
        LogicalKey::Named(named) => named_key(form, named, held.shift, held.ctrl),
        LogicalKey::Char(ch) if held.ctrl => shortcut(form, ch, held.shift),
        LogicalKey::Char(_) => Outcome::Ignored,
    }
}

/// Typed or pasted text. On a choice a space changes it.
pub fn handle_text(form: &mut SetupForm, text: &str) -> Outcome {
    if form.link.is_some() {
        return link_panel_input::text(text);
    }
    if form.provider_menu.is_some() {
        return form.menu_text(text);
    }
    if form.waiting {
        return Outcome::Ignored;
    }
    if form.focus.is_choice() {
        if text == " " && form.focus != SetupField::Save {
            form.change_choice(true);
            return Outcome::Changed;
        }
        return if text == " " { Outcome::Submit } else { Outcome::Ignored };
    }
    let one_line = clean(text);
    let focus = form.focus;
    let Some(buffer) = form.focused_buffer() else {
        return Outcome::Ignored;
    };
    if !buffer.insert_text(&one_line) {
        return Outcome::Ignored;
    }
    form.note_edit();
    if focus == SetupField::Email {
        // The servers follow the address while it is being typed.
        form.apply_provider();
    }
    Outcome::Changed
}

/// A click on a field: focus moves there, and a choice changes.
pub fn click(form: &mut SetupForm, field: SetupField) -> Outcome {
    if form.waiting {
        return Outcome::Ignored;
    }
    if field == SetupField::Save {
        form.move_focus(field);
        return Outcome::Submit;
    }
    let was_focused = form.focus == field;
    if field == SetupField::Provider {
        // Opening the list changes nothing yet, so the first click opens it.
        form.move_focus(field);
        form.open_provider_menu();
        return Outcome::Changed;
    }
    form.move_focus(field);
    // A box ticks at the first click. A choice among several words changes
    // only when its row already has focus: the first click just goes there,
    // so clicking a row to look at it never changes the setting.
    if field == SetupField::AllowChanges || (field.is_choice() && was_focused) {
        form.change_choice(true);
    }
    form.quit_armed = false;
    Outcome::Changed
}

fn escape(form: &mut SetupForm) -> Outcome {
    if form.is_empty() || form.quit_armed || form.waiting {
        return Outcome::Close;
    }
    form.quit_armed = true;
    form.notice = Some(QUIT_NOTE.to_string());
    Outcome::Changed
}

fn named_key(form: &mut SetupForm, key: NamedKey, shift: bool, ctrl: bool) -> Outcome {
    match key {
        NamedKey::Tab if shift => step(form, false),
        NamedKey::Tab | NamedKey::Down | NamedKey::Enter => step(form, true),
        NamedKey::Up => step(form, false),
        NamedKey::Left | NamedKey::Right if form.focus.is_choice() => {
            form.change_choice(key == NamedKey::Right);
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

fn step(form: &mut SetupForm, forward: bool) {
    form.step_focus(forward);
}

/// Cursor movement on the focused text field; nothing on a choice.
fn edit(form: &mut SetupForm, action: impl FnOnce(&mut crate::composer::EditorBuffer)) {
    if let Some(buffer) = form.focused_buffer() {
        action(buffer);
    }
}

fn delete(form: &mut SetupForm, key: NamedKey, ctrl: bool) -> Outcome {
    let Some(buffer) = form.focused_buffer() else {
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
    form.note_edit();
    if form.focus == SetupField::Email {
        form.apply_provider();
    }
    Outcome::Changed
}

/// Ctrl with a letter: select all, undo and redo. Nothing is copied or cut.
fn shortcut(form: &mut SetupForm, ch: char, shift: bool) -> Outcome {
    let Some(buffer) = form.focused_buffer() else {
        return Outcome::Ignored;
    };
    // Whether the text really changed: an undo with nothing to undo leaves the
    // field as it was and must not count as the user's own edit, or a guessed
    // server would stop following the address.
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
        // Undo and redo change the address as typing does, so the servers
        // and the user name that follow from it are worked out again.
        form.note_edit();
        if form.focus == SetupField::Email {
            form.apply_provider();
        }
    }
    Outcome::Changed
}

/// Text on one line with the control codes taken out, so a paste cannot carry
/// a newline or an escape sequence into a field.
fn clean(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

#[cfg(test)]
include!("input.tests.rs");
