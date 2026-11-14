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
    /// Right and left arrows: in the folder tree they open and close a
    /// folder, elsewhere they go in and out like Enter and Escape.
    Right,
    Left,
    /// Space: a page down while reading, and nothing yet in the list, where
    /// the list's own hint gives it to marking rows.
    SpaceDown,
    SpaceUp,
    ToggleRead,
    ToggleStar,
    Delete,
    Sync,
    Help,
    /// F10: open the Options screen.
    Options,
    Compose,
    Move,
    Search,
    Reply,
    ReplyAll,
    Forward,
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
        LogicalKey::Char(' ') => Some(if held.shift { Action::SpaceUp } else { Action::SpaceDown }),
        LogicalKey::Char('u') if !held.shift => Some(Action::ToggleRead),
        LogicalKey::Char('*') => Some(Action::ToggleStar),
        LogicalKey::Char('?') => Some(Action::Help),
        LogicalKey::Char('c') if !held.shift => Some(Action::Compose),
        LogicalKey::Char('m') if !held.shift => Some(Action::Move),
        LogicalKey::Char('/') => Some(Action::Search),
        LogicalKey::Char('r') if !held.shift => Some(Action::Reply),
        LogicalKey::Char('R') => Some(Action::ReplyAll),
        LogicalKey::Char('f') if !held.shift => Some(Action::Forward),
        LogicalKey::Char('d') if !held.shift => Some(Action::Delete),
        LogicalKey::Char('s') if !held.shift => Some(Action::Sync),
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
        NamedKey::Enter => Action::Enter,
        NamedKey::Right => Action::Right,
        NamedKey::Left => Action::Left,
        NamedKey::Function(2) => Action::Compose,
        NamedKey::Function(10) => Action::Options,
        NamedKey::Escape | NamedKey::Backspace => Action::Back,
        _ => return None,
    })
}

#[cfg(test)]
include!("action.tests.rs");
