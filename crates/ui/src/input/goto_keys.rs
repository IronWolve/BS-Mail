//! `g` then a letter goes to a well-known folder of the account on screen:
//! `i` Inbox, `s` Sent, `d` Drafts, `a` Archive, `t` Trash, `j` Spam (junk).
//! `g` twice, or whatever key `go_to` was given twice, opens the picker for
//! going to any folder by its name (`folder_picker`). While the letter is
//! awaited the footer lists them; Escape, any other key or a click cancels.
//! An account that has no such folder says so.

use model::FolderRole;
use surface::{KeyEvent, LogicalKey};

use super::action::Action;
use crate::app_view::AppView;
use crate::keys::KeySpec;
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

/// How the footer's words while the letter is awaited begin; what ends the
/// wait takes down a note that begins so.
const WAITING_START: &str = "Go to folder: ";

impl AppView {
    /// `g` was pressed: the next letter says which folder.
    pub(crate) fn begin_go_to(&mut self) {
        self.memory.go_pending = true;
        let note = self.waiting_note();
        self.mail.set_note(Some(note));
    }

    /// What the footer says while the letter is awaited: the letters, and the
    /// key that opens the picker (the one `go_to` has now).
    fn waiting_note(&self) -> String {
        let again = self.keys.keys_of("go_to").first().map_or_else(|| "g".to_string(), KeySpec::text);
        format!("{WAITING_START}i Inbox, s Sent, d Drafts, a Archive, t Trash, j Spam, {again} any folder (Esc cancels)")
    }

    /// Stops waiting for the letter. Returns whether it was being awaited.
    pub(crate) fn cancel_go_to(&mut self) -> bool {
        if !std::mem::take(&mut self.memory.go_pending) {
            return false;
        }
        if self.mail.error.as_deref().is_some_and(|note| note.starts_with(WAITING_START)) {
            self.mail.set_note(None);
        }
        true
    }

    /// The key after `g`. Whatever it is, the wait is over: a letter of the
    /// list goes to its folder, the key of `go_to` again opens the picker for
    /// any folder, anything else changes nothing. A letter of the list wins
    /// over the key of `go_to` when the two are one (`go_to` rebound to `s`:
    /// `s` `s` goes to Sent, as it always did). The key held down repeating is
    /// not a second press: the wait goes on.
    pub(crate) fn go_key(&mut self, key: &KeyEvent) -> bool {
        let again = self.keys.action(key) == Some(Action::GoTo);
        if again && key.repeat {
            return false;
        }
        self.cancel_go_to();
        match jump_letter(key) {
            Some((role, name)) => self.go_to_role(role, name),
            None if again => self.start_folder_picker(),
            None => {}
        }
        true
    }

    /// Opens the first folder of the account that has `role`.
    fn go_to_role(&mut self, role: FolderRole, name: &str) {
        match self.mail.folders.iter().find(|folder| folder.role == role).map(|folder| folder.id) {
            Some(id) => {
                self.account_cursor = None;
                self.focus = Focus::List;
                self.change_folder(id);
            }
            None => self.mail.set_note(Some(format!("This account has no {name} folder"))),
        }
    }
}

/// The role and the footer's name of the folder a plain letter goes to, when
/// `key` is one of the list.
fn jump_letter(key: &KeyEvent) -> Option<(FolderRole, &'static str)> {
    let LogicalKey::Char(letter) = key.key else {
        return None;
    };
    let held = key.modifiers;
    if held.ctrl || held.alt || held.meta {
        return None;
    }
    GO_TO
        .iter()
        .find(|(wanted, ..)| *wanted == letter.to_ascii_lowercase())
        .map(|(_, role, name)| (*role, *name))
}

#[cfg(test)]
include!("goto_keys.tests.rs");
