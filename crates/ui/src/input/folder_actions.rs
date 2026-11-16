//! The folder actions the user can start, from the keys (`F` then a letter,
//! see `folder_keys`) and from the folder's menu: a new folder, a new name for
//! a folder, deleting a folder, marking a folder read and emptying the Trash
//! or Spam. Each is refused first, in words, when the account is read-only or
//! the folder is one it cannot be done to; none is sent before its answer is
//! given (a name typed, a question said yes to), except marking read, which
//! only changes a flag. The folder list changes when the backend says so.

use model::{Command, FolderId, holds_folders, not_emptiable, protected, saved_search_refusal};

use super::done_note::Settles;
use crate::app_view::AppView;
use crate::ask::{Ask, ConfirmPrompt, NamePrompt};

impl AppView {
    /// The open folder, when it is a mailbox and not one of the saved searches.
    pub(crate) fn open_mailbox(&self) -> Option<FolderId> {
        let id = self.selection.folder?;
        self.mail.folder(id).filter(|folder| !folder.role.is_virtual()).map(|folder| folder.id)
    }

    /// Asks for the name of a new folder inside `inside`, or at the top level.
    pub(crate) fn start_new_folder(&mut self, inside: Option<FolderId>) {
        if !self.changes_allowed_or_note() {
            return;
        }
        let parent = match inside.map(|id| self.mail.folder(id).cloned()) {
            None => None,
            Some(None) => return,
            Some(Some(folder)) => {
                if let Some(reason) = saved_search_refusal(&folder) {
                    self.mail.set_note(Some(reason));
                    return;
                }
                Some(folder)
            }
        };
        let place = parent.as_ref().map(|folder| (folder.id, folder.name.as_str()));
        self.ask = Some(Ask::Name(NamePrompt::new_folder(place)));
    }

    /// Asks for the new name of `id`.
    pub(crate) fn start_rename(&mut self, id: FolderId) {
        let Some(folder) = self.mail.folder(id).cloned() else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        if let Some(reason) = protected(&folder, "renamed") {
            self.mail.set_note(Some(reason));
            return;
        }
        self.ask = Some(Ask::Name(NamePrompt::rename(id, &folder.name)));
    }

    /// Asks before `id` and the messages in it are deleted for good.
    pub(crate) fn start_delete_folder(&mut self, id: FolderId) {
        let Some(folder) = self.mail.folder(id).cloned() else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        if let Some(reason) = protected(&folder, "deleted").or_else(|| holds_folders(&self.mail.folders, id, &folder.name)) {
            self.mail.set_note(Some(reason));
            return;
        }
        self.ask = Some(Ask::Confirm(ConfirmPrompt::delete_folder(&folder)));
    }

    /// Asks before every message of the Trash or Spam folder `id` is deleted.
    pub(crate) fn start_empty_folder(&mut self, id: FolderId) {
        let Some(folder) = self.mail.folder(id).cloned() else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        if let Some(reason) = not_emptiable(&folder) {
            self.mail.set_note(Some(reason));
        } else if folder.total == 0 {
            self.mail.set_note(Some(format!("{} is already empty", folder.name)));
        } else {
            self.ask = Some(Ask::Confirm(ConfirmPrompt::empty_folder(&folder)));
        }
    }

    /// Marks every unread message of `id` read. Nothing is asked: it only
    /// changes a flag.
    pub(crate) fn mark_folder_read(&mut self, id: FolderId) {
        let Some(folder) = self.mail.folder(id).cloned() else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        if let Some(reason) = saved_search_refusal(&folder) {
            self.mail.set_note(Some(reason));
        } else if folder.unread == 0 {
            self.mail.set_note(Some(format!("No unread messages in {}", folder.name)));
        } else {
            self.queue.push(Command::MarkAllRead { folder: id });
            self.promise_note(format!("Marked all read in {}", folder.name), Settles::Folders);
        }
    }
}

#[cfg(test)]
include!("folder_actions.tests.rs");
