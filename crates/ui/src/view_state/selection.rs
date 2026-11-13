//! What is selected: the account, the folder and the message shown.

use model::{Account, AccountId, Envelope, Folder, FolderId, FolderRole, MessageId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Selection {
    pub account: Option<AccountId>,
    pub folder: Option<FolderId>,
    pub message: Option<MessageId>,
    /// Row of the selected message in the loaded page.
    pub message_index: usize,
}

impl Selection {
    /// Picks the account the backend marked, else the first, unless one is
    /// already selected and still exists.
    pub fn choose_account(&mut self, accounts: &[Account]) {
        let still_there = self
            .account
            .as_ref()
            .is_some_and(|id| accounts.iter().any(|account| &account.id == id));
        if still_there {
            return;
        }
        self.account = accounts
            .iter()
            .find(|account| account.selected)
            .or_else(|| accounts.first())
            .map(|account| account.id.clone());
    }

    /// Picks the Inbox, else the first folder, unless one is already
    /// selected and still exists.
    pub fn choose_folder(&mut self, folders: &[Folder]) {
        let still_there = self
            .folder
            .is_some_and(|id| folders.iter().any(|folder| folder.id == id));
        if still_there {
            return;
        }
        self.folder = folders
            .iter()
            .find(|folder| folder.role == FolderRole::Inbox)
            .or_else(|| folders.first())
            .map(|folder| folder.id);
    }

    /// A page of `folder` was loaded: that folder is the one shown, and the
    /// selected message stays if it is on the page, else the first row.
    pub fn page_loaded(&mut self, folder: FolderId, envelopes: &[Envelope]) {
        self.folder = Some(folder);
        let kept = self
            .message
            .and_then(|id| envelopes.iter().position(|envelope| envelope.id == id));
        match kept {
            Some(index) => self.message_index = index,
            None => {
                self.message_index = 0;
                self.message = envelopes.first().map(|envelope| envelope.id);
            }
        }
    }
}

#[cfg(test)]
include!("selection.tests.rs");
