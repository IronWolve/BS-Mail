//! The commands the screen sends to the mail backend.
//!
//! Selection is what the user changes; asking the backend for the folder or
//! the message that selection now points at is derived from it. So a click, a
//! key, a backend event that moves the selection, and a restart all end in
//! the same place, and nothing is asked for twice.

use model::{Command, FolderId, MessageId};

use crate::view_state::{MailSnapshot, Selection};

/// How close to the end of the loaded messages the selection gets before the
/// next older page is asked for.
const NEAR_END: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandQueue {
    explicit: Vec<Command>,
    asked_folder: Option<FolderId>,
    asked_body: Option<MessageId>,
    /// The folder and the number of messages loaded when older mail was last
    /// asked for, so it is asked for once.
    asked_more: Option<(FolderId, usize)>,
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
        self.asked_more = None;
    }

    /// The newest page of a folder arrived again, from a refresh or after a
    /// reconnect. A request for older mail may have been lost on the way, so
    /// it can be made again.
    pub fn first_page_arrived(&mut self) {
        self.asked_more = None;
    }

    /// The request for older mail that is due: the selection, or the last row
    /// the wheel or a scrollbar has brought into view (`reach`), is near the
    /// end of what is loaded and the folder has more. Asked once for each
    /// number of messages loaded.
    pub fn take_more(&mut self, selection: &Selection, mail: &MailSnapshot, reach: usize) -> Option<Command> {
        let folder = selection.folder?;
        let page = mail.page.as_ref().filter(|page| page.folder == folder)?;
        let have = page.envelopes.len();
        let more = u64::from(page.total) > u64::from(page.offset) + u64::try_from(have).ok()?;
        let near = selection.message_index.max(reach) + NEAR_END >= have;
        if !(more && near) || self.asked_more == Some((folder, have)) {
            return None;
        }
        self.asked_more = Some((folder, have));
        Some(Command::LoadMore { folder, have: u32::try_from(have).ok()? })
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
