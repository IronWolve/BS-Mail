//! Keys and typed text into the composer.
//!
//! A letter key arrives twice from a window: once as a key and once as the
//! text the layout made of it. Only the text is typed, so every layout and
//! compose sequence works; keys here are the ones with no text, and the
//! shortcuts held with Ctrl.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::form::{ComposeForm, Field, Progress};

/// What the composer wants done after an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Not something the composer uses.
    Ignored,
    /// The composer changed; draw it again.
    Changed,
    /// Send the message.
    Send,
    /// Close the composer, discarding the message.
    Close,
}

/// How big the body pane is, which vertical movement and scrolling need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pane {
    pub width: usize,
    pub rows: usize,
}

/// What Escape says while the message is with the server. It is not a way
/// out: a message that is on its way cannot be called back, and the text
/// stays here until the server answers.
const ESCAPE_WHILE_SENDING: &str = "this message is being sent and cannot be called back: it may still go out. \
     Wait for the server's answer; closing the window asks first";

pub fn handle_key(form: &mut ComposeForm, key: &KeyEvent, pane: Pane) -> Outcome {
    if form.progress == Progress::Sending {
        return sending_key(form, key);
    }
    let held = key.modifiers;
    let outcome = match key.key {
        _ if held.alt || held.meta => Outcome::Ignored,
        LogicalKey::Named(NamedKey::Escape) => return escape(form),
        LogicalKey::Named(NamedKey::Function(10)) => Outcome::Send,
        LogicalKey::Named(NamedKey::Enter) if held.ctrl => Outcome::Send,
        LogicalKey::Named(named) => named_key(form, named, held.shift, held.ctrl, pane),
        LogicalKey::Char(ch) if held.ctrl => shortcut(form, ch, held.shift),
        LogicalKey::Char(_) => Outcome::Ignored,
    };
    if outcome == Outcome::Changed {
        form.notice = None;
        form.discard_armed = false;
        form.close_armed = false;
        keep_cursor_in_view(form, pane);
    }
    outcome
}

/// Typed or pasted text.
pub fn handle_text(form: &mut ComposeForm, text: &str, pane: Pane) -> Outcome {
    if form.progress == Progress::Sending {
        return Outcome::Ignored;
    }
    if !form.focused().insert_text(&clean(text)) {
        return Outcome::Ignored;
    }
    form.notice = None;
    form.discard_armed = false;
    form.close_armed = false;
    keep_cursor_in_view(form, pane);
    Outcome::Changed
}

/// Every key is left alone while sending, except that Escape says what is
/// true instead of seeming to do nothing.
fn sending_key(form: &mut ComposeForm, key: &KeyEvent) -> Outcome {
    let plain = !key.modifiers.alt && !key.modifiers.meta;
    if plain && matches!(key.key, LogicalKey::Named(NamedKey::Escape)) {
        form.notice = Some(ESCAPE_WHILE_SENDING.to_string());
        return Outcome::Changed;
    }
    Outcome::Ignored
}

fn escape(form: &mut ComposeForm) -> Outcome {
    if !form.is_dirty() || form.discard_armed {
        return Outcome::Close;
    }
    form.discard_armed = true;
    form.notice = Some("this message has text in it: press Esc again to discard it".to_string());
    Outcome::Changed
}

fn named_key(form: &mut ComposeForm, key: NamedKey, shift: bool, ctrl: bool, pane: Pane) -> Outcome {
    let in_body = form.focus == Field::Body;
    match key {
        NamedKey::Tab => {
            form.focus = if shift { form.focus.previous() } else { form.focus.next() };
        }
        NamedKey::Enter if in_body => {
            form.focused().insert_newline();
        }
        NamedKey::Enter => form.focus = form.focus.next(),
        NamedKey::Up if !in_body => form.focus = form.focus.previous(),
        NamedKey::Down if !in_body => form.focus = form.focus.next(),
        NamedKey::Up => form.body.move_vertical(false, pane.width, shift),
        NamedKey::Down => form.body.move_vertical(true, pane.width, shift),
        NamedKey::PageUp | NamedKey::PageDown if in_body => {
            let down = key == NamedKey::PageDown;
            for _ in 0..pane.rows.saturating_sub(1).max(1) {
                form.body.move_vertical(down, pane.width, shift);
            }
        }
        NamedKey::Left if ctrl => form.focused().move_word_left(shift),
        NamedKey::Right if ctrl => form.focused().move_word_right(shift),
        NamedKey::Left => form.focused().move_left(shift),
        NamedKey::Right => form.focused().move_right(shift),
        NamedKey::Home if ctrl => form.focused().move_text_start(shift),
        NamedKey::End if ctrl => form.focused().move_text_end(shift),
        NamedKey::Home => form.focused().move_line_start(shift),
        NamedKey::End => form.focused().move_line_end(shift),
        NamedKey::Backspace if ctrl => {
            form.focused().delete_word_back();
        }
        NamedKey::Backspace => {
            form.focused().backspace();
        }
        NamedKey::Delete => {
            form.focused().delete_forward();
        }
        _ => return Outcome::Ignored,
    }
    Outcome::Changed
}

/// Ctrl with a letter: select all, undo and redo, cut, copy, paste.
fn shortcut(form: &mut ComposeForm, ch: char, shift: bool) -> Outcome {
    match ch.to_ascii_lowercase() {
        'a' => form.focused().select_all(),
        'z' if shift => {
            form.focused().redo();
        }
        'z' => {
            form.focused().undo();
        }
        'y' => {
            form.focused().redo();
        }
        'x' => {
            if let Some(text) = form.focused().cut() {
                form.clipboard = text.clone();
                form.copied = Some(text);
            }
        }
        'c' => {
            if let Some(text) = form.focused().selected_text() {
                form.clipboard = text.clone();
                form.copied = Some(text);
            }
        }
        'v' => {
            let text = form.clipboard.clone();
            form.focused().insert_text(&text);
        }
        _ => return Outcome::Ignored,
    }
    Outcome::Changed
}

/// Text with the control codes taken out, so a paste cannot carry an escape
/// sequence into a message. Newlines stay and tabs become four spaces.
fn clean(text: &str) -> String {
    text.chars()
        .filter_map(|ch| match ch {
            '\t' => Some("    ".to_string()),
            '\n' | '\r' => Some(ch.to_string()),
            ch if ch.is_control() => None,
            ch => Some(ch.to_string()),
        })
        .collect()
}

/// Scrolls the body so the line with the cursor shows.
fn keep_cursor_in_view(form: &mut ComposeForm, pane: Pane) {
    if form.focus != Field::Body || pane.rows == 0 {
        return;
    }
    let (row, _) = form.body.cursor_cell(pane.width);
    if row < form.body_scroll {
        form.body_scroll = row;
    } else if row >= form.body_scroll + pane.rows {
        form.body_scroll = row + 1 - pane.rows;
    }
}

#[cfg(test)]
include!("input.tests.rs");
