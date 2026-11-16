//! Making, renaming and deleting a folder.
//!
//! Each is checked first, in words, with nothing sent: the name, the special
//! folders, a folder that holds others. The server is then asked once, and a
//! refusal is shown as the server said it. When the server has done it, the
//! folder list is read again (the change's tally says so), so the sidebar
//! shows what the server holds. A rename keeps the folder's id for it and
//! every folder inside it, so the open folder, and the messages shown, stay
//! valid under the new name.

use model::{AccountId, Folder, FolderId, holds_folders, protected, saved_search_refusal};

use super::changes::Tally;
use super::worker::Worker;
use crate::folder_rules::{new_path, renamed_path};
use crate::imap_session::{ImapSession, SessionError};

/// Why a command names a folder this account does not have.
fn unknown(folder: FolderId) -> String {
    format!("unknown folder id: {}", folder.0)
}

impl Worker {
    /// The folder with this id when it is a mailbox of the account, or the
    /// refusal for a saved search or an id that names nothing.
    pub(super) fn mailbox_folder(&self, id: FolderId) -> Result<Folder, String> {
        match self.state.folder(id) {
            None => Err(unknown(id)),
            Some(folder) => saved_search_refusal(folder).map_or_else(|| Ok(folder.clone()), Err),
        }
    }

    /// Makes a folder inside `parent`, or at the top.
    pub(super) async fn create_folder(
        &mut self,
        session: &mut ImapSession,
        (account, parent, name): (&AccountId, Option<FolderId>, &str),
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        if *account != self.account {
            tally.notice(format!("unknown account: {}", account.0));
            return Ok(());
        }
        let inside = match parent.map(|id| self.mailbox_folder(id)).transpose() {
            Ok(inside) => inside,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let path = new_path(inside.as_ref().map(|folder| folder.path.as_str()), self.state.delimiter(), name, self.state.folders());
        let path = match path {
            Ok(path) => path,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let done = session.create_mailbox(&path).await;
        self.settle_folder_change(done, tally)
    }

    /// Gives a folder another name.
    pub(super) async fn rename_folder(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        name: &str,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let found = self.mailbox_folder(folder).and_then(|found| match protected(&found, "renamed") {
            Some(reason) => Err(reason),
            None => Ok(found),
        });
        let found = match found {
            Ok(found) => found,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let to = match renamed_path(&found.path, self.state.delimiter(), name, self.state.folders()) {
            Ok(to) => to,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let separator = self.state.delimiter().map(str::to_owned);
        session.leave_tree(&found.path, separator.as_deref()).await?;
        let done = session.rename_mailbox(&found.path, &to).await;
        if done.is_ok() {
            self.state.rename_folder_paths(&found.path, &to);
        }
        self.settle_folder_change(done, tally)
    }

    /// Removes a folder and its messages for good.
    pub(super) async fn delete_folder(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let found = self.mailbox_folder(folder).and_then(|found| {
            protected(&found, "deleted")
                .or_else(|| holds_folders(self.state.folders(), found.id, &found.name))
                .map_or(Ok(found), Err)
        });
        let found = match found {
            Ok(found) => found,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let separator = self.state.delimiter().map(str::to_owned);
        session.leave_tree(&found.path, separator.as_deref()).await?;
        let done = session.delete_mailbox(&found.path).await;
        if done.is_ok() {
            self.state.forget_folder(found.id);
            if self.wanted.folder == Some(found.id) {
                self.wanted.folder = None;
            }
        }
        self.settle_folder_change(done, tally)
    }

    /// What a folder command's answer comes to: done (the list is read
    /// again), refused (the server's words), or a lost connection.
    fn settle_folder_change(&mut self, done: Result<(), SessionError>, tally: &mut Tally) -> Result<(), SessionError> {
        match done {
            Ok(()) => {
                tally.relocated = true;
                tally.stale = true;
                Ok(())
            }
            Err(error) if error.is_lost() => Err(error),
            Err(error) => {
                tally.notice(error.text());
                Ok(())
            }
        }
    }
}

#[cfg(test)]
include!("folder_changes.tests.rs");
