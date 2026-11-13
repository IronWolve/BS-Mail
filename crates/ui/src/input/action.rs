//! What a key press means to the screen, independent of which pane has focus.

use surface::{KeyEvent, LogicalKey, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FocusNext,
    FocusPrevious,
    Up,
    Down,
    PageUp,
    PageDown,
    First,
    Last,
    /// Go into the next pane: open the folder's list, read the message.
    Enter,
    /// Go back out to the previous pane.
    Back,
}

/// The action for a key press, or `None` for a key the screen ignores.
/// Shortcuts with `Ctrl`, `Alt` or the logo key are never read here: they
/// belong to the window and to the commands that come later.
pub fn action_for(key: &KeyEvent) -> Option<Action> {
    let held = key.modifiers;
    if held.ctrl || held.alt || held.meta {
        return None;
    }
    match key.key {
        LogicalKey::Named(named) => named_action(named, held.shift),
        LogicalKey::Char('j') if !held.shift => Some(Action::Down),
        LogicalKey::Char('k') if !held.shift => Some(Action::Up),
        LogicalKey::Char(' ') => Some(if held.shift { Action::PageUp } else { Action::PageDown }),
        LogicalKey::Char(_) => None,
    }
}

fn named_action(named: NamedKey, shift: bool) -> Option<Action> {
    Some(match named {
        NamedKey::Tab if shift => Action::FocusPrevious,
        NamedKey::Tab => Action::FocusNext,
        NamedKey::Up => Action::Up,
        NamedKey::Down => Action::Down,
        NamedKey::PageUp => Action::PageUp,
        NamedKey::PageDown => Action::PageDown,
        NamedKey::Home => Action::First,
        NamedKey::End => Action::Last,
        NamedKey::Enter | NamedKey::Right => Action::Enter,
        NamedKey::Escape | NamedKey::Left | NamedKey::Backspace => Action::Back,
        _ => return None,
    })
}

#[cfg(test)]
include!("action.tests.rs");
