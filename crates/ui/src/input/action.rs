//! What a key press means to the screen, independent of which pane has focus.

use surface::{KeyEvent, LogicalKey, NamedKey};

use crate::keys::KeyMap;
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
    /// Space: a page down while reading, and at the end of the message on to
    /// the next unread one; in the list it marks the row and moves on;
    /// elsewhere it does nothing.
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
    /// `x`: the cursor to the next attachment of the open message (the
    /// first when it is on none); `X` the one before. While it is on one,
    /// Enter saves it, `o` opens it and `a` asks where to save it.
    NextAttachment,
    PreviousAttachment,
    /// `o`: sort the list by the next column.
    SortNext,
    /// `O`: reverse the order of the sort.
    SortReverse,
    /// `T`: threads on or off for this session.
    ToggleThreads,
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

/// The action for a key press with no key rebound, or `None` for a key the
/// screen ignores. The screen itself asks its own `KeyMap`, which has the
/// user's `[keys]` laid over these.
pub fn action_for(key: &KeyEvent) -> Option<Action> {
    KeyMap::standard().action(key)
}

/// The action of a key that cannot be rebound: the keys that move, go in and
/// out, mark and select, the half pages, and the extra keys some actions have
/// (F2 composes, Delete deletes, Menu and Shift+F10 open the menu). The keys
/// that can be rebound are in `keys::actions`. Shortcuts with `Ctrl`, `Alt`
/// or the logo key are not read here, except `Ctrl+u` and `Ctrl+d`: the rest
/// belong to the window and to text boxes.
pub fn fixed_action(key: &KeyEvent) -> Option<Action> {
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
        LogicalKey::Char(' ') => Some(if held.shift { Action::SpaceUp } else { Action::SpaceDown }),
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
        NamedKey::Escape | NamedKey::Backspace => Action::Back,
        _ => return None,
    })
}

#[cfg(test)]
include!("action.tests.rs");
