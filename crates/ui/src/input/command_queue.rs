//! The commands the screen sends to the mail backend.
//!
//! Selection is what the user changes; asking the backend for the folder or
//! the message that selection now points at is derived from it. So a click, a
//! key, a backend event that moves the selection, and a restart all end in
//! the same place, and nothing is asked for twice.

use model::{Command, FolderId, MessageId};

use crate::view_state::{MailSnapshot, Selection};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandQueue {
    explicit: Vec<Command>,
    asked_folder: Option<FolderId>,
    asked_body: Option<MessageId>,
}

impl CommandQueue {
    /// A command the user asked for outright, such as a sync.
    pub fn push(&mut self, command: Command) {
        self.explicit.push(command);
    }

    /// Forgets what was asked for, so the next call asks again. Used when the
    /// user asks for a refresh.
    pub fn forget(&mut self) {
        self.asked_folder = None;
        self.asked_body = None;
    }

    /// Every command due now: the explicit ones, then whatever the selection
    /// needs that was not asked for yet.
    pub fn take(&mut self, selection: &Selection, mail: &MailSnapshot) -> Vec<Command> {
        let mut commands = std::mem::take(&mut self.explicit);
        let Some(folder) = selection.folder else {
            return commands;
        };
        let page_is_shown = mail.page.as_ref().is_some_and(|page| page.folder == folder);
        if !page_is_shown {
            if self.asked_folder != Some(folder) {
                self.asked_folder = Some(folder);
                commands.push(Command::OpenFolder { folder });
            }
            return commands;
        }
        let Some(message) = selection.message else {
            return commands;
        };
        let have_body = mail.body.as_ref().is_some_and(|(id, _)| *id == message);
        if !have_body && self.asked_body != Some(message) {
            self.asked_body = Some(message);
            commands.push(Command::FetchBody { message });
        }
        commands
    }
}

#[cfg(test)]
include!("command_queue.tests.rs");
