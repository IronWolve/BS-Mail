//! Moving messages to another folder and deleting them.
//!
//! A delete outside the Trash folder is a move into it; inside the Trash
//! folder it removes the messages for good, and only on a server that can
//! expunge by UID. When the account has no Trash folder, or the server has no
//! safe way, the delete is refused; it never falls back to expunging a whole
//! mailbox.

use model::{FolderId, MessageId};

use super::change_rules::{DeletePlan, MoveHow, destination, move_how, plan_delete};
use super::changes::Tally;
use super::resolve::{Group, resolve};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

/// What to do with each batch of a group.
enum Action {
    /// Take the messages to another folder.
    MoveTo { folder: FolderId, path: String, how: MoveHow },
    /// Remove the messages from the folder they are in.
    Forever,
}

impl Worker {
    /// Moves the messages to a folder. Messages already there are left alone.
    pub(super) async fn move_messages(
        &mut self,
        session: &mut ImapSession,
        messages: &[MessageId],
        to: FolderId,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let (folder, path) = match destination(&self.state, to) {
            Ok(found) => found,
            Err(reason) => {
                tally.notice(reason);
                return Ok(());
            }
        };
        let how = move_how(session.caps());
        let resolved = resolve(&self.state, messages);
        tally.unknown(&resolved.unknown);
        for group in resolved.groups.iter().filter(|group| group.folder != folder) {
            if let MoveHow::Refuse(reason) = how {
                tally.notice(reason);
                continue;
            }
            let action = Action::MoveTo { folder, path: path.clone(), how };
            self.relocate_group(session, group, &action, tally).await?;
        }
        Ok(())
    }

    /// Deletes the messages: into the Trash folder, or for good from it.
    pub(super) async fn delete_messages(
        &mut self,
        session: &mut ImapSession,
        messages: &[MessageId],
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let resolved = resolve(&self.state, messages);
        tally.unknown(&resolved.unknown);
        for group in &resolved.groups {
            let action = match plan_delete(&self.state, group.folder, session.caps()) {
                DeletePlan::Refuse(reason) => {
                    tally.notice(reason);
                    continue;
                }
                DeletePlan::Forever => Action::Forever,
                DeletePlan::ToTrash { trash, path, how } => Action::MoveTo { folder: trash, path, how },
            };
            self.relocate_group(session, group, &action, tally).await?;
        }
        Ok(())
    }

    /// Carries the action out for one folder's messages, a command's worth at
    /// a time. A batch the server took stops resolving; one it refused is
    /// noted and the rest of the folder is left alone.
    async fn relocate_group(
        &mut self,
        session: &mut ImapSession,
        group: &Group,
        action: &Action,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        if let Err(error) = self.open_for_change(session, group.folder, &group.path).await {
            return tally.open_failed(error);
        }
        for (ids, set) in group.batches() {
            let outcome = match action {
                Action::MoveTo { path, how: MoveHow::Native, .. } => session.move_to(&set, path).await,
                Action::MoveTo { path, .. } => session.copy_then_remove(&set, path).await,
                Action::Forever => session.delete_forever(&set).await,
            };
            let failure = match outcome {
                Ok(()) => {
                    ids.iter().for_each(|id| self.state.forget_message(*id));
                    self.note_relocated(group.folder, action, tally);
                    continue;
                }
                Err(failure) => failure,
            };
            if failure.error.is_lost() {
                if failure.changed {
                    // The server already holds a copy or a deleted mark that
                    // the sequence did not finish with: say so, and void the
                    // ids so nothing is repeated from a stale list.
                    ids.iter().for_each(|id| self.state.forget_message(*id));
                    self.note_relocated(group.folder, action, tally);
                    tally.notice(failure.error.text());
                }
                return Err(failure.error);
            }
            tally.notice(failure.error.text());
            if failure.changed {
                self.note_relocated(group.folder, action, tally);
            }
            break;
        }
        Ok(())
    }

    fn note_relocated(&self, source: FolderId, action: &Action, tally: &mut Tally) {
        tally.relocated = true;
        tally.touch(source);
        if let Action::MoveTo { folder, .. } = action {
            tally.touch(*folder);
        }
    }
}
