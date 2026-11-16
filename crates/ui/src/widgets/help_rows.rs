//! The rows of the help screen. The list is data, in one place, so it is easy
//! to keep honest: a row here is a promise that the key does that, in the
//! host that is showing it.

use surface::Capabilities;

/// What a host must be able to do for a row to be true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Needs {
    Always,
    /// A host that changes the text size at run time (the window).
    FontZoom,
    /// A host with its own full-screen key (the window).
    FullScreen,
}

impl Needs {
    fn met_by(self, capabilities: &Capabilities) -> bool {
        match self {
            Self::Always => true,
            Self::FontZoom => capabilities.font_zoom,
            Self::FullScreen => capabilities.full_screen,
        }
    }
}

/// One row: the keys, what they do, and what the host must offer.
#[derive(Debug, Clone, Copy)]
pub struct HelpRow {
    pub keys: &'static str,
    pub meaning: &'static str,
    pub needs: Needs,
}

const fn row(keys: &'static str, meaning: &'static str) -> HelpRow {
    HelpRow { keys, meaning, needs: Needs::Always }
}

const fn row_needing(keys: &'static str, meaning: &'static str, needs: Needs) -> HelpRow {
    HelpRow { keys, meaning, needs }
}

/// Every row, in the order shown.
pub const HELP_ROWS: &[HelpRow] = &[
    row("Up Down  j k", "move in the pane that has focus"),
    row("PgUp PgDn  Space", "a page at a time (Space in the reader)"),
    row("Home End", "first and last"),
    row("Tab  Shift+Tab", "change pane"),
    row("Enter  Esc", "go in a pane, come back out"),
    row("Right  Left", "open and close a folder; in the other panes, in and out"),
    row("1  2  3", "focus the folders, the list, the message"),
    row("g then i s d a t j", "go to the Inbox, Sent, Drafts, Archive, Trash, Spam"),
    row("Ctrl+u  Ctrl+d", "half a page up and down in the pane that has focus"),
    row("click  double-click", "select a folder or message; a double-click opens the message"),
    row("click flag or star cell", "toggle read or star of that message"),
    row("Ctrl click  Shift click", "mark a row; mark every row from the last one clicked"),
    row("drag rows", "drop them on a folder to move them; Esc cancels"),
    row("right click  .  Menu", "the menu of the item under the pointer or selected"),
    row("wheel", "scroll the pane under the pointer"),
    row("Down (last folder)", "go on into the accounts; Enter switches account"),
    row("Space (list)", "mark a message; u * d e ! m then act on all marked"),
    row("a  A  i", "mark all shown, clear the marks (Esc too), invert the marks"),
    row("n  p  N", "next, previous and next unread message (list or reader)"),
    row("u  *  +", "mark read or unread; star or unstar; set the alert mark, or clear it when they all have it"),
    row("t", "tag: pick a tag or type a new one; Enter sets it, or clears it when they all have it"),
    row("d  Delete", "delete: asks first; to the Trash, in the Trash for good"),
    row("e  !", "move to the Archive, move to the Spam folder"),
    row("m", "move to another folder"),
    row("F then n N r d m e", "folders: new inside this one, new at the top, rename, delete, mark all read, empty Trash or Spam"),
    row("s", "sync now"),
    row("/", "search the account; Esc brings the folder back"),
    row("/ (reader)", "find in the message: Enter or n next match, N previous, Esc closes"),
    row("drag  Shift+arrows", "select text; y copies it, or the whole message (the subject in the list)"),
    row("Y", "copy the sender's address"),
    row("H  v", "show the message's raw headers, its whole source; Esc brings the message back"),
    row("b", "open a safe copy of the message's HTML in your browser"),
    row("c  F2", "write a new message"),
    row("r  R  f", "reply, reply to everyone, forward"),
    row("w", "copy a message as a new one to edit; Enter on a draft goes on writing it"),
    row("F10  Ctrl+Enter", "send the message being written; Esc cancels it; the mouse places, selects, scrolls"),
    row("Left Right Space (From)", "pick the account the message is sent from; Enter lists them"),
    row("Down Up Tab Enter (To)", "pick and take an address that matches; Esc closes the list"),
    row("drag a bar", "resize the sidebar and the list and reader split"),
    row_needing("Ctrl + - 0  Ctrl wheel", "text size, and back to normal", Needs::FontZoom),
    row_needing("F11", "full screen", Needs::FullScreen),
    row("F10", "options: accounts, appearance, data (sends, while writing)"),
    row("?", "this help"),
    row("q  Ctrl+Q", "quit"),
];

/// The rows the host showing the screen can keep, as (keys, meaning).
pub fn help_rows(capabilities: &Capabilities) -> Vec<(&'static str, &'static str)> {
    HELP_ROWS
        .iter()
        .filter(|row| row.needs.met_by(capabilities))
        .map(|row| (row.keys, row.meaning))
        .collect()
}
