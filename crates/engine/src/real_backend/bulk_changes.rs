//! Changes to every message of a folder at once: marking all read, and
//! emptying a Trash or Spam folder.
//!
//! Neither names a message: the folder is opened for reading, the messages
//! that match are found with a search (`UNSEEN`, or `ALL`), and the change is
//! then sent on exactly those UIDs, a command's worth at a time, like any
//! other change. A message that arrives in between is not touched.

use model::{FolderId, not_emptiable};

use super::changes::Tally;
use super::worker::Worker;
use crate::imap_session::{ImapSession, MAX_PER_COMMAND, SessionError, StoreFlag, StoreMode, UidSet};
use crate::search::Keys;

impl Worker {
    /// Marks every unread message of the folder read.
    pub(super) async fn mark_all_read(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let found = match self.mailbox_folder(folder) {
            Ok(found) => found,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let Some(unread) = self.find_uids(session, folder, &found.path, &Keys::unseen(), tally).await? else {
            return Ok(());
        };
        if let Err(error) = self.open_for_change(session, folder, &found.path).await {
            return tally.open_failed(error);
        }
        for chunk in unread.chunks(MAX_PER_COMMAND) {
            let Some(set) = UidSet::new(chunk.iter().copied()) else {
                continue;
            };
            match session.store_flags(&set, StoreMode::Add, &[StoreFlag::Seen]).await {
                Ok(_) => {
                    tally.relocated = true;
                    tally.touch(folder);
                }
                Err(error) if error.is_lost() => return Err(error),
                Err(error) => {
                    tally.notice(error.text());
                    break;
                }
            }
        }
        Ok(())
    }

    /// Removes every message of a Trash or Spam folder for good, and no other
    /// folder's.
    pub(super) async fn empty_folder(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let found = match self.mailbox_folder(folder) {
            Ok(found) => found,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        if let Some(reason) = not_emptiable(&found) {
            tally.notice(reason);
            return Ok(());
        }
        if !session.caps().uidplus {
            tally.notice("this server cannot delete safely (it does not offer UIDPLUS); nothing was deleted");
            return Ok(());
        }
        let Some(all) = self.find_uids(session, folder, &found.path, &Keys::all(), tally).await? else {
            return Ok(());
        };
        if let Err(error) = self.open_for_change(session, folder, &found.path).await {
            return tally.open_failed(error);
        }
        for chunk in all.chunks(MAX_PER_COMMAND) {
            let Some(set) = UidSet::new(chunk.iter().copied()) else {
                continue;
            };
            match session.delete_forever(&set).await {
                Ok(()) => self.note_emptied(folder, tally),
                Err(failure) if failure.error.is_lost() => {
                    if failure.changed {
                        self.note_emptied(folder, tally);
                        tally.notice(failure.error.text());
                    }
                    return Err(failure.error);
                }
                Err(failure) => {
                    tally.notice(failure.error.text());
                    if failure.changed {
                        self.note_emptied(folder, tally);
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// Part of the folder is gone: the ids it had are void, and its page and
    /// counts are read again.
    fn note_emptied(&mut self, folder: FolderId, tally: &mut Tally) {
        self.state.forget_folder(folder);
        tally.relocated = true;
        tally.touch(folder);
    }

    /// The UIDs of the folder's messages that match `keys`, found with the
    /// folder open for reading; `None` when there are none or the folder
    /// could not be searched (the reason is noted).
    async fn find_uids(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        path: &str,
        keys: &Keys,
        tally: &mut Tally,
    ) -> Result<Option<Vec<u32>>, SessionError> {
        let info = match session.examine(path).await {
            Ok(info) => info,
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => {
                tally.notice(error.text());
                return Ok(None);
            }
        };
        self.note_validity(folder, info.uid_validity);
        match session.uid_search(keys).await {
            Ok(found) => Ok((!found.is_empty()).then_some(found)),
            Err(error) if error.is_lost() => Err(error),
            Err(error) => {
                tally.notice(error.text());
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
include!("bulk_changes.tests.rs");
