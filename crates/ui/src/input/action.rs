//! What a key press means to the screen, independent of which pane has focus.

use surface::{KeyEvent, LogicalKey, NamedKey};

use crate::view_state::Focus;

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
    /// Space: a page down while reading, and in the list it marks the row and
    /// moves on; elsewhere it does nothing.
    SpaceDown,
    SpaceUp,
    ToggleRead,
    ToggleStar,
    /// `d` and the Delete key: delete, after asking.
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
    /// `w`: a copy of the message as a new one to edit.
    EditAsNew,
    /// `a`: mark every message shown.
    MarkAll,
    /// `A`: clear every mark.
    UnmarkAll,
    /// `i`: mark the unmarked messages and unmark the marked ones.
    InvertMarks,
    /// `e`: move the marked messages, or the selected one, to the Archive.
    Archive,
    /// `!`: move them to the Spam folder.
    Spam,
    /// `n`: show the next message of the list, from the list or the reader.
    NextMessage,
    /// `p`: show the previous message.
    PreviousMessage,
    /// `N`: show the next unread message.
    NextUnread,
    /// `1` `2` `3`: focus the sidebar, the list or the reader.
    FocusPane(Focus),
    /// `Ctrl+u` and `Ctrl+d`: half a page up or down in the pane that has focus.
    HalfPageUp,
    HalfPageDown,
    /// `.` and the Menu key: the context menu of the selected item.
    ContextMenu,
    /// `t`: the tag picker for the marked messages, or the selected one.
    Tag,
    /// `+`: set the alert mark on them, or clear it when they all have it.
    ToggleAlert,
    /// `F`: the next letter names a folder action (new, rename, delete, mark
    /// all read, empty).
    FolderKeys,
    /// `g`: the next letter names a well-known folder to go to.
    GoTo,
    /// `Shift` with an arrow, `Home`, `End`, `PageUp` or `PageDown`: select
    /// text in the reader; in the other panes it moves as the key does alone.
    Extend(Direction),
    /// `y`: copy the selected text of the reader, or the whole message when
    /// none is selected; in the list, the subject.
    Copy,
    /// `Y`: copy the sender's address.
    CopySender,
    /// `H`: show the message's raw headers in the reader.
    ViewHeaders,
    /// `v`: show the message's whole source in the reader.
    ViewSource,
    /// `b`: open a safe copy of the message's HTML in the browser.
    OpenInBrowser,
}

/// Where a selection key takes the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

impl Direction {
    /// The action the same key does without `Shift`.
    pub fn plain(self) -> Action {
        match self {
            Self::Up => Action::Up,
            Self::Down => Action::Down,
            Self::Left => Action::Left,
            Self::Right => Action::Right,
            Self::Home => Action::First,
            Self::End => Action::Last,
            Self::PageUp => Action::PageUp,
            Self::PageDown => Action::PageDown,
        }
    }

    fn of(named: NamedKey) -> Option<Self> {
        Some(match named {
            NamedKey::Up => Self::Up,
            NamedKey::Down => Self::Down,
            NamedKey::Left => Self::Left,
            NamedKey::Right => Self::Right,
            NamedKey::Home => Self::Home,
            NamedKey::End => Self::End,
            NamedKey::PageUp => Self::PageUp,
            NamedKey::PageDown => Self::PageDown,
            _ => return None,
        })
    }
}

/// The action for a key press, or `None` for a key the screen ignores.
/// Shortcuts with `Ctrl`, `Alt` or the logo key are never read here, except
/// `Ctrl+u` and `Ctrl+d`: the rest belong to the window and to the commands
/// that come later.
pub fn action_for(key: &KeyEvent) -> Option<Action> {
    let held = key.modifiers;
    if held.ctrl && !held.alt && !held.meta && !held.shift {
        return match key.key {
            LogicalKey::Char('u') => Some(Action::HalfPageUp),
            LogicalKey::Char('d') => Some(Action::HalfPageDown),
            _ => None,
        };
    }
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
        LogicalKey::Char('w') if !held.shift => Some(Action::EditAsNew),
        LogicalKey::Char('d') if !held.shift => Some(Action::Delete),
        LogicalKey::Char('a') if !held.shift => Some(Action::MarkAll),
        LogicalKey::Char('A') => Some(Action::UnmarkAll),
        LogicalKey::Char('i') if !held.shift => Some(Action::InvertMarks),
        LogicalKey::Char('e') if !held.shift => Some(Action::Archive),
        LogicalKey::Char('!') => Some(Action::Spam),
        LogicalKey::Char('n') if !held.shift => Some(Action::NextMessage),
        LogicalKey::Char('p') if !held.shift => Some(Action::PreviousMessage),
        LogicalKey::Char('N') => Some(Action::NextUnread),
        LogicalKey::Char('s') if !held.shift => Some(Action::Sync),
        LogicalKey::Char('g') if !held.shift => Some(Action::GoTo),
        LogicalKey::Char('.') => Some(Action::ContextMenu),
        LogicalKey::Char('t') if !held.shift => Some(Action::Tag),
        LogicalKey::Char('+') => Some(Action::ToggleAlert),
        LogicalKey::Char('F') => Some(Action::FolderKeys),
        LogicalKey::Char('y') if !held.shift => Some(Action::Copy),
        LogicalKey::Char('Y') => Some(Action::CopySender),
        LogicalKey::Char('H') => Some(Action::ViewHeaders),
        LogicalKey::Char('v') if !held.shift => Some(Action::ViewSource),
        LogicalKey::Char('b') if !held.shift => Some(Action::OpenInBrowser),
        LogicalKey::Char('1') => Some(Action::FocusPane(Focus::Sidebar)),
        LogicalKey::Char('2') => Some(Action::FocusPane(Focus::List)),
        LogicalKey::Char('3') => Some(Action::FocusPane(Focus::Reader)),
        LogicalKey::Char(_) => None,
    }
}

fn named_action(named: NamedKey, shift: bool) -> Option<Action> {
    if shift && let Some(direction) = Direction::of(named) {
        return Some(Action::Extend(direction));
    }
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
        NamedKey::Delete => Action::Delete,
        NamedKey::Menu => Action::ContextMenu,
        NamedKey::Function(2) => Action::Compose,
        NamedKey::Function(10) if shift => Action::ContextMenu,
        NamedKey::Function(10) => Action::Options,
        NamedKey::Escape | NamedKey::Backspace => Action::Back,
        _ => return None,
    })
}

#[cfg(test)]
include!("action.tests.rs");
