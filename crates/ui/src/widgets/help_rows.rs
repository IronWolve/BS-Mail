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
    row("click", "select a folder or message"),
    row("wheel", "scroll the pane under the pointer"),
    row("Space (list)", "mark a message; u * d then act on all marked, Esc clears"),
    row("u", "mark read or unread"),
    row("*", "star or unstar"),
    row("d", "delete"),
    row("m", "move to another folder"),
    row("s", "sync now"),
    row("/", "search the account; Esc brings the folder back"),
    row("c  F2", "write a new message"),
    row("r  R  f", "reply, reply to everyone, forward"),
    row("Ctrl+Enter", "send the message being written"),
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
