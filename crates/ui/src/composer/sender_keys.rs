//! The keys and typed text of the From line: Left, Right and Space change the
//! account, Enter opens the list of accounts, and while the list is open Up
//! and Down move in it, Enter takes a row and Escape closes it.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::form::ComposeForm;
use super::input::Outcome;

/// A key while the From list is open. `None` when the list is closed. Every
/// key is the list's while it is open: Up and Down move, Enter takes the row,
/// and any other key closes the list.
pub(super) fn from_menu_key(form: &mut ComposeForm, key: &KeyEvent) -> Option<Outcome> {
    let cursor = form.from_menu?;
    let last = form.senders.len().saturating_sub(1);
    match key.key {
        LogicalKey::Named(NamedKey::Up) => form.from_menu = Some(cursor.saturating_sub(1)),
        LogicalKey::Named(NamedKey::Down) => form.from_menu = Some((cursor + 1).min(last)),
        LogicalKey::Named(NamedKey::Enter) => {
            form.from_menu = None;
            form.choose_sender(cursor);
        }
        _ => form.from_menu = None,
    }
    Some(Outcome::Changed)
}

/// Typed text while the From line has focus or its list is open: Space
/// changes the account and nothing else is typed into it.
pub(super) fn from_text(form: &mut ComposeForm, text: &str) -> Outcome {
    if form.from_menu.is_none() && text == " " && form.step_sender(true) {
        return Outcome::Changed;
    }
    Outcome::Ignored
}

#[cfg(test)]
include!("sender_keys.tests.rs");
