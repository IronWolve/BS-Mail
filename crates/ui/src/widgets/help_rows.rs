//! The rows of the help screen. The list is data, in one place, so it is easy
//! to keep honest: a row here is a promise that the key does that, in the
//! host that is showing it.

use surface::Capabilities;

use crate::keys::KeyMap;

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

/// Every row, in the order shown. A `{name}` stands for the key or keys the
/// action of that name has now (`keys::actions`), so a key the user rebound
/// shows as it is; the other keys are fixed and written as they are.
pub const HELP_ROWS: &[HelpRow] = &[
    row("Up Down  {down} {up}", "move in the pane that has focus"),
    row("PgUp PgDn  Space", "a page at a time; Space in the reader, at the end of a message, shows the next unread one"),
    row("Home End", "first and last"),
    row("Tab  Shift+Tab", "change pane; a pane that is switched off (the sidebar) is skipped"),
    row("Enter  Esc", "go in a pane, come back out"),
    row("Right  Left", "open and close a folder or a thread; in the other panes, in and out"),
    row("{focus_sidebar}  {focus_list}  {focus_reader}", "focus the folders, the list, the message"),
    row("{go_to} then i s d a t j", "go to the Inbox, Sent, Drafts, Archive, Trash, Spam; {go_to} again: pick any folder by name"),
    row("Ctrl+u  Ctrl+d", "half a page up and down in the pane that has focus"),
    row("click  double-click", "select a folder or message; a double-click opens the message"),
    row("click flag or star cell", "toggle read or star of that message"),
    row("Ctrl click  Shift click", "mark a row; mark every row from the last one clicked"),
    row("drag rows", "drop them on a folder to move them; Esc cancels"),
    row("right click  {context_menu}  Menu", "the menu of the item under the pointer or selected"),
    row("wheel", "scroll the pane under the pointer"),
    row("Down (last folder)", "go on into the accounts; Enter switches account"),
    row("Space (list)", "mark a message; {read} {star} {delete} {archive} {spam} {move} then act on all marked"),
    row("{mark_all}  {unmark_all}  {invert_marks}", "mark all shown, clear the marks (Esc too), invert the marks"),
    row("{sort}  {sort_reverse}  {threads}", "sort by the next column, reverse the order (or click a column header); threads on or off"),
    row("{next_message}  {previous_message}  {next_unread}", "show next, previous, next unread message in your pane; none left: {next_unread} names the next folder, {next_unread} again goes"),
    row("{read}  {star}  {alert}", "mark read or unread; star or unstar; set the alert mark, or clear it when they all have it"),
    row("{tag}", "tag: pick a tag or type a new one; Enter sets it, or clears it when they all have it"),
    row("{delete}  Delete", "delete: asks first; to the Trash, in the Trash for good"),
    row("{archive}  {spam}  {move}", "move to the Archive, move to the Spam folder, move to another folder you pick"),
    row("{folder_actions} then n N r d m e", "folders: new inside this one, new at the top, rename, delete, mark all read, empty Trash or Spam"),
    row("{sync}", "sync now"),
    row("{search}", "search the account; Esc brings the folder back"),
    row("{search} (reader)", "find in the message: Enter or n next match, N previous, Esc closes"),
    row("drag  Shift+arrows", "select text; {copy} copies it, or the whole message (the subject in the list); {copy_sender} copies the sender's address"),
    row("{headers}  {source}  {browser}", "show the raw headers, the whole source (Esc brings the message back); {browser} opens its HTML in your browser"),
    row("{next_attachment}  {previous_attachment}", "attachments: cursor to the next, previous one; then Enter saves it, o opens it, a saves it as"),
    row("{compose}  F2", "write a new message"),
    row("{reply}  {reply_all}  {forward}", "reply, reply to everyone, forward"),
    row("{edit_as_new}", "copy a message as a new one to edit; Enter on a draft goes on writing it"),
    row("F10  Ctrl+Enter", "send the message being written; Esc cancels it; the mouse places, selects, scrolls"),
    row("Ctrl+O", "attach a file: type its path (Tab completes, Down picks); Left Right Delete on the Attach row"),
    row("Left Right Space (From)", "pick the account the message is sent from; Enter lists them"),
    row("Down Up Tab Enter (To)", "pick and take an address that matches; Esc closes the list"),
    row("drag a bar", "resize the sidebar and the list and reader split"),
    row_needing("Ctrl + - 0  Ctrl wheel", "text size, and back to normal", Needs::FontZoom),
    row_needing("F11", "full screen", Needs::FullScreen),
    row("{options}", "options: accounts, appearance, data, about"),
    row("{help}", "this help"),
    row("q  Ctrl+Q", "quit"),
];

/// The rows the host showing the screen can keep, as (keys, meaning), with
/// the keys the user has filled in.
pub fn help_rows(capabilities: &Capabilities, keys: &KeyMap) -> Vec<(String, String)> {
    HELP_ROWS
        .iter()
        .filter(|row| row.needs.met_by(capabilities))
        .map(|row| (fill(row.keys, keys), fill(row.meaning, keys)))
        .collect()
}

/// `template` with each `{name}` replaced by the key or keys of that action.
fn fill(template: &str, keys: &KeyMap) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push_str(&rest[open..]);
            return out;
        };
        out.push_str(&keys.text_of(&after[..close]));
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
include!("help_rows.tests.rs");
