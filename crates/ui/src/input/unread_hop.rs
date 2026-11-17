//! `N` across folders. With no other unread message in the open folder, `N`
//! says in the footer which folder it would go to next and how much unread
//! mail that one holds; the very next `N` opens it at its first unread message
//! and the pane that had the focus keeps it. The next folder is the one after
//! this one in the tree's order that has unread mail and is not the Trash or
//! the Spam folder, going on from the top; it is a folder of the account on
//! screen. With no folder of the account holding unread mail the footer says so.
//!
//! What was said holds only for the very next `N`: any other key, any click, a
//! key that is held down, another folder or another message ends it, so a stale
//! footer line can never take the user to a folder they did not mean to go to.

use model::{FolderId, FolderRole};
use surface::{ButtonState, InputEvent};

use super::memory::UnreadHop;
use crate::app_view::AppView;

/// What the footer says when the open folder is all there is to read.
const NONE_HERE: &str = "No more unread here";
const NONE_ANYWHERE: &str = "No unread mail";

impl AppView {
    /// Handles one input event, keeping the hop that `N` last announced only
    /// while this is the `N` that follows it. A key or a press ends it, and
    /// takes its words off the footer unless `N` says them again; the
    /// pointer moving, the wheel and typed text leave it be (a terminal sends
    /// the text of a key after the key).
    pub(crate) fn with_unread_hop(&mut self, event: &InputEvent, route: impl FnOnce(&mut Self) -> bool) -> bool {
        let ends = matches!(event, InputEvent::Key(_) | InputEvent::Button { state: ButtonState::Pressed, .. });
        let held = if ends { self.memory.unread_hop.take() } else { None };
        // The repeat of a key held down is not the press that was asked for.
        let repeated = matches!(event, InputEvent::Key(key) if key.repeat);
        self.memory.hop_held = held.clone().filter(|_| !repeated);
        let used = route(self);
        self.memory.hop_held = None;
        match held {
            Some(hop) if self.memory.unread_hop.is_none() => self.take_down_hop_note(&hop) || used,
            _ => used,
        }
    }

    /// `N` found no other unread message in the open list.
    pub(super) fn unread_elsewhere(&mut self) {
        let held = self.memory.hop_held.take();
        // A search's hits are not the folder: there is nowhere to go on to.
        let Some(open) = self.selection.folder.filter(|_| self.search.is_none()) else {
            self.mail.set_note(Some(NONE_HERE.to_string()));
            return;
        };
        let Some((to, name, unread)) = self.next_unread_folder(open) else {
            // The open folder's own count may still be unread mail that is not
            // loaded, or the message on screen: not "no unread mail" then.
            let any_here = self.mail.folder(open).is_some_and(|folder| folder.unread > 0);
            self.mail.set_note(Some((if any_here { NONE_HERE } else { NONE_ANYWHERE }).to_string()));
            return;
        };
        let from = (open, self.selection.message);
        if held.is_some_and(|hop| hop.from == from && hop.to == to) {
            self.open_first_unread_of(to);
            return;
        }
        let note = format!("{NONE_HERE}; N again goes to {name} ({unread} unread)");
        self.mail.set_note(Some(note.clone()));
        self.memory.unread_hop = Some(UnreadHop { from, to, note });
    }

    /// The folder after `open` in the tree's order that holds unread mail,
    /// with its name and its count: not the Trash or the Spam folder, and
    /// coming back round to the top when none follows. Never `open` itself.
    fn next_unread_folder(&self, open: FolderId) -> Option<(FolderId, String, u32)> {
        let folders = &self.mail.folders;
        let at = folders.iter().position(|folder| folder.id == open)?;
        let wanted = |folder: &&model::Folder| {
            folder.unread > 0 && !matches!(folder.role, FolderRole::Trash | FolderRole::Spam)
        };
        folders[at + 1..]
            .iter()
            .chain(&folders[..at])
            .find(wanted)
            .map(|folder| (folder.id, folder.name.clone(), folder.unread))
    }

    /// Opens `folder` as a click on it would, with its first unread message
    /// to be selected once its page is here, and opens the folders above it
    /// so it can be seen. The focus stays in the pane it was in.
    fn open_first_unread_of(&mut self, folder: FolderId) {
        self.mail.set_note(None);
        self.account_cursor = None;
        self.show_open_folder(folder);
        self.memory.open_at_unread = Some(folder);
    }

    /// What an event from the backend may have done to what `N` waits for.
    pub(crate) fn settle_unread_waits(&mut self) {
        self.settle_open_at_unread();
        self.end_stale_hop();
    }

    /// A hop is for the folder and message it was said about: when the
    /// backend's news moved the selection off them, it is over and its words
    /// come off the footer.
    fn end_stale_hop(&mut self) {
        let here = (self.selection.folder, self.selection.message);
        if self.memory.unread_hop.as_ref().is_some_and(|hop| (Some(hop.from.0), hop.from.1) != here)
            && let Some(hop) = self.memory.unread_hop.take()
        {
            self.take_down_hop_note(&hop);
        }
    }

    /// The page of the folder `N` opened has arrived: its first unread
    /// message is the one selected, read from its top. A page with no unread
    /// message among the ones loaded leaves the selection on the first row.
    pub(crate) fn settle_open_at_unread(&mut self) {
        let Some(folder) = self.memory.open_at_unread else {
            return;
        };
        if self.selection.folder != Some(folder) {
            self.memory.open_at_unread = None;
            return;
        }
        let Some(page) = self.shown_page() else {
            return;
        };
        let first = page.envelopes.iter().position(|envelope| !envelope.flags.seen).map(|row| (row, page.envelopes[row].id));
        self.memory.open_at_unread = None;
        if let Some((row, id)) = first {
            self.selection.message = Some(id);
            self.selection.message_index = row;
        }
    }

    /// Takes the words of an ended hop off the footer, when they are still
    /// the footer's. Returns whether it did.
    fn take_down_hop_note(&mut self, hop: &UnreadHop) -> bool {
        if self.mail.error.as_deref() != Some(hop.note.as_str()) {
            return false;
        }
        self.mail.set_note(None);
        true
    }
}

#[cfg(test)]
include!("unread_hop.tests.rs");
