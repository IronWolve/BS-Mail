//! `F` then a letter: the folder actions, from the keyboard. `n` makes a
//! folder inside the open one, `N` at the top level, `r` renames the open
//! folder, `d` deletes it (after asking), `m` marks it read and `e` empties it
//! when it is the Trash or Spam. While the letter is awaited the footer lists
//! them; Escape, any other key or a click cancels. They act on the folder that
//! is open (the highlighted one in the sidebar), and the folder's menu offers
//! the same actions to the mouse.

use surface::{KeyEvent, LogicalKey};

use crate::app_view::AppView;

/// What the footer says while the letter is awaited.
const WAITING_NOTE: &str =
    "Folder: n new inside this one, N new at the top, r rename, d delete, m mark all read, e empty Trash or Spam (Esc cancels)";

impl AppView {
    /// `F` was pressed: the next letter says what to do to the open folder.
    pub(crate) fn begin_folder_keys(&mut self) {
        self.memory.folder_pending = true;
        self.mail.set_note(Some(WAITING_NOTE.to_string()));
    }

    /// Stops waiting for the letter. Returns whether it was being awaited.
    pub(crate) fn cancel_folder_keys(&mut self) -> bool {
        if !std::mem::take(&mut self.memory.folder_pending) {
            return false;
        }
        if self.mail.error.as_deref() == Some(WAITING_NOTE) {
            self.mail.set_note(None);
        }
        true
    }

    /// The key after `F`. Whatever it is, the wait is over.
    pub(crate) fn folder_key(&mut self, key: &KeyEvent) -> bool {
        self.cancel_folder_keys();
        let held = key.modifiers;
        let LogicalKey::Char(letter) = key.key else {
            return true;
        };
        if held.ctrl || held.alt || held.meta {
            return true;
        }
        if letter == 'N' {
            self.start_new_folder(None);
            return true;
        }
        let Some(open) = self.selection.folder.filter(|id| self.mail.folder(*id).is_some()) else {
            self.mail.set_note(Some("Open a folder first: these keys act on the open folder".to_string()));
            return true;
        };
        match letter {
            'n' => self.start_new_folder(self.open_mailbox()),
            'r' => self.start_rename(open),
            'd' => self.start_delete_folder(open),
            'm' => self.mark_folder_read(open),
            'e' => self.start_empty_folder(open),
            _ => {}
        }
        true
    }
}

#[cfg(test)]
include!("folder_keys.tests.rs");
