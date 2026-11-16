//! One-key moves to a folder known by its role: `e` moves the marked
//! messages, or the selected one, to the Archive, and `!` to the Spam folder.
//!
//! Nothing is asked: both only move mail, and either can be undone by moving
//! it back. The footer says how many were moved once the backend has done it
//! (a failure says so instead), and says in plain words when the account has
//! no such folder or the mail is already in it. Like every change, it only
//! asks the backend; the list changes when the backend says the mail moved.

use model::{Command, Event, FolderId, FolderRole, MessageId};

use crate::app_view::AppView;

impl AppView {
    /// `e`: moves the targets to the account's Archive folder.
    pub(crate) fn archive_targets(&mut self) {
        self.move_to_role(FolderRole::Archive, "Archive");
    }

    /// `!`: moves the targets to the account's Spam folder.
    pub(crate) fn spam_targets(&mut self) {
        self.move_to_role(FolderRole::Spam, "Spam");
    }

    fn move_to_role(&mut self, role: FolderRole, name: &str) {
        let targets = self.targets();
        if targets.is_empty() || !self.changes_allowed_or_note() {
            return;
        }
        let Some(destination) = self.mail.folders.iter().find(|folder| folder.role == role).map(|folder| folder.id) else {
            self.mail.set_note(Some(format!(
                "This account has no {name} folder, so nothing was moved (m moves to a folder by name)"
            )));
            return;
        };
        let count_asked = targets.len();
        let movable = self.not_in(targets, destination);
        if movable.is_empty() {
            let these = if count_asked == 1 { "that message is" } else { "those messages are" };
            self.mail.set_note(Some(format!("Nothing moved: {these} already in {name}")));
            return;
        }
        let count = movable.len();
        self.select_neighbor_outside(&movable);
        self.clear_marks();
        self.mail.relocated.extend(&movable);
        self.queue.push(Command::Move { messages: movable, to: destination });
        self.moved_note = Some(format!("Moved {count} to {name}"));
    }

    /// The messages of `ids` that are not in `folder` already. A message whose
    /// folder cannot be told (it left the list) is kept: asking to move it is
    /// harmless.
    fn not_in(&self, ids: Vec<MessageId>, folder: FolderId) -> Vec<MessageId> {
        let shown = self.mail.page.as_ref().map(|page| page.loaded.as_slice()).unwrap_or_default();
        ids.into_iter()
            .filter(|id| shown.iter().find(|envelope| envelope.id == *id).is_none_or(|envelope| envelope.folder != folder))
            .collect()
    }

    /// What an event from the backend does to the "Moved N to ..." line that
    /// waits for the move to be done: a failure drops it (the footer then says
    /// what went wrong), and the newest page of a folder, which is what a
    /// finished move brings, returns it to be shown. Call before the event is
    /// applied, and show what it returns after.
    pub(crate) fn settle_moved_note(&mut self, event: &Event) -> Option<String> {
        match event {
            Event::OpFailed { .. } => {
                self.moved_note = None;
                None
            }
            Event::EnvelopePage { offset: 0, .. } => self.moved_note.take(),
            _ => None,
        }
    }
}

#[cfg(test)]
include!("role_moves.tests.rs");
