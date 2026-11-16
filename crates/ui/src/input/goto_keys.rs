//! `g` then a letter goes to a well-known folder of the account on screen:
//! `i` Inbox, `s` Sent, `d` Drafts, `a` Archive, `t` Trash, `j` Spam (junk).
//! While the letter is awaited the footer lists them; Escape, any other key
//! or a click cancels. An account that has no such folder says so.

use model::FolderRole;
use surface::{KeyEvent, LogicalKey};

use crate::app_view::AppView;
use crate::view_state::Focus;

/// The letter, the role it goes to and the name the footer uses.
pub const GO_TO: &[(char, FolderRole, &str)] = &[
    ('i', FolderRole::Inbox, "Inbox"),
    ('s', FolderRole::Sent, "Sent"),
    ('d', FolderRole::Drafts, "Drafts"),
    ('a', FolderRole::Archive, "Archive"),
    ('t', FolderRole::Trash, "Trash"),
    ('j', FolderRole::Spam, "Spam"),
];

/// What the footer says while the letter is awaited.
const WAITING_NOTE: &str = "Go to folder: i Inbox, s Sent, d Drafts, a Archive, t Trash, j Spam (Esc cancels)";

impl AppView {
    /// `g` was pressed: the next letter says which folder.
    pub(crate) fn begin_go_to(&mut self) {
        self.memory.go_pending = true;
        self.mail.set_note(Some(WAITING_NOTE.to_string()));
    }

    /// Stops waiting for the letter. Returns whether it was being awaited.
    pub(crate) fn cancel_go_to(&mut self) -> bool {
        if !std::mem::take(&mut self.memory.go_pending) {
            return false;
        }
        if self.mail.error.as_deref() == Some(WAITING_NOTE) {
            self.mail.set_note(None);
        }
        true
    }

    /// The key after `g`. Whatever it is, the wait is over.
    pub(crate) fn go_key(&mut self, key: &KeyEvent) -> bool {
        self.cancel_go_to();
        let held = key.modifiers;
        let LogicalKey::Char(letter) = key.key else {
            return true;
        };
        if held.ctrl || held.alt || held.meta {
            return true;
        }
        let Some((_, role, name)) = GO_TO.iter().find(|(wanted, ..)| *wanted == letter.to_ascii_lowercase()) else {
            return true;
        };
        match self.mail.folders.iter().find(|folder| folder.role == *role).map(|folder| folder.id) {
            Some(id) => {
                self.account_cursor = None;
                self.focus = Focus::List;
                self.change_folder(id);
            }
            None => self.mail.set_note(Some(format!("This account has no {name} folder"))),
        }
        true
    }
}

#[cfg(test)]
include!("goto_keys.tests.rs");
