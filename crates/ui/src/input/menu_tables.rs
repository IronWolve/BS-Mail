//! The entries of every context menu, as data: one table per menu, so a
//! feature that adds an action adds one row here and one arm in `menu_run`.
//! Each row has the words, and the letter that chooses it, which is a letter
//! of the words (a test keeps the letters unique within a menu).

use crate::widgets::context_menu::{Menu, MenuEntry};

/// What choosing an entry does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Reply,
    ReplyAll,
    Forward,
    EditAsNew,
    Delete,
    Move,
    ToggleRead,
    ToggleStar,
    Archive,
    Spam,
    CopySender,
    CopySubject,
    OpenFolder,
    SyncNow,
    EditAccount,
    OpenLink,
    CopyLink,
}

/// Which menu: what was pointed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Message,
    Reader,
    Folder,
    Account,
    Link,
}

/// One row of a table: what it does, how it reads and its letter.
#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub action: MenuAction,
    pub label: &'static str,
    pub hotkey: char,
}

const fn row(action: MenuAction, label: &'static str, hotkey: char) -> Row {
    Row { action, label, hotkey }
}

use MenuAction::*;

/// The messages' own actions, shared by the list's menu and the reader's: the
/// reader shows the selected message.
const MESSAGE_ROWS: &[Row] = &[
    row(Reply, "Reply", 'r'),
    row(ReplyAll, "Reply all", 'a'),
    row(Forward, "Forward", 'f'),
    row(EditAsNew, "Edit as new", 'e'),
    row(Delete, "Delete", 'd'),
    row(Move, "Move...", 'm'),
    row(ToggleRead, "Mark read", 'k'),
    row(ToggleStar, "Star", 's'),
    row(Archive, "Archive", 'c'),
    row(Spam, "Spam", 'p'),
    row(CopySender, "Copy sender", 'o'),
    row(CopySubject, "Copy subject", 'b'),
];

const FOLDER_ROWS: &[Row] = &[row(OpenFolder, "Open", 'o'), row(SyncNow, "Sync now", 's')];

const ACCOUNT_ROWS: &[Row] = &[row(EditAccount, "Edit account", 'e'), row(SyncNow, "Sync now", 's')];

const LINK_ROWS: &[Row] = &[row(OpenLink, "Open", 'o'), row(CopyLink, "Copy link", 'c')];

/// The table of a menu.
pub fn rows(kind: MenuKind) -> &'static [Row] {
    match kind {
        MenuKind::Message | MenuKind::Reader => MESSAGE_ROWS,
        MenuKind::Folder => FOLDER_ROWS,
        MenuKind::Account => ACCOUNT_ROWS,
        MenuKind::Link => LINK_ROWS,
    }
}

/// What the words of some entries depend on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuFacts {
    /// Every message the entries act on is read: the entry then marks unread.
    pub all_seen: bool,
    /// Every one of them is starred: the entry then unstars.
    pub all_flagged: bool,
    /// How many messages are marked; the menu says so in its title.
    pub marked: usize,
}

/// The words of a row for `facts`.
fn label(row: &Row, facts: &MenuFacts) -> &'static str {
    match (row.action, facts.all_seen, facts.all_flagged) {
        (ToggleRead, true, _) => "Mark unread",
        (ToggleStar, _, true) => "Unstar",
        _ => row.label,
    }
}

/// The menu of `kind`, opened at `anchor`.
pub fn build_menu(kind: MenuKind, facts: &MenuFacts, anchor: (u16, u16)) -> Menu<MenuAction> {
    let entries = rows(kind).iter().map(|row| MenuEntry::new(row.action, label(row, facts), row.hotkey)).collect();
    let title = (facts.marked > 0 && matches!(kind, MenuKind::Message | MenuKind::Reader)).then(|| format!("{} marked", facts.marked));
    Menu::new(anchor, title, entries)
}

#[cfg(test)]
include!("menu_tables.tests.rs");
