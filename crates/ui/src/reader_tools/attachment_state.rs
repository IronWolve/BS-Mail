//! What the reading pane keeps about the attachments of the open message: the
//! one the cursor is on, the downloads asked for and not yet answered, and the
//! files that arrived, which the program writes to disk or sends on.

use std::collections::VecDeque;

use model::{AttachmentInfo, FileBytes, MessageId};

/// What an attachment is wanted for once it has arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Want {
    /// Saved in the attachments folder of the data folder.
    Save,
    /// Saved there and handed to the system to open.
    Open,
    /// Saved where the user typed (a folder, or a new file name).
    SaveAs(String),
}

/// Who asked for a download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// The reader: save, open or save as.
    Reader(Want),
    /// A forward being written: the file goes on the message.
    Forward,
}

/// A download asked for and not answered yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetch {
    pub message: MessageId,
    pub attachment: AttachmentInfo,
    pub owner: Owner,
}

/// An attachment that arrived, for the program to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentFile {
    /// The name the sender gave it (not yet made safe: the program does that).
    pub name: String,
    pub content_type: String,
    pub data: FileBytes,
    pub want: Want,
}

#[derive(Debug, Default)]
pub struct AttachmentState {
    /// The attachment the cursor is on: the message it belongs to and its
    /// place in that message's list.
    pub cursor: Option<(MessageId, usize)>,
    /// The downloads asked for, oldest first, which the answers settle in order.
    pub fetches: VecDeque<Fetch>,
    /// The files that arrived and wait for the program.
    pub files: Vec<AttachmentFile>,
}

impl AttachmentState {
    /// Whether this download is already asked for for the same purpose.
    pub fn is_waiting(&self, message: MessageId, part: u32, owner: &Owner) -> bool {
        self.fetches.iter().any(|fetch| fetch.message == message && fetch.attachment.part == part && fetch.owner == *owner)
    }

    /// Takes the oldest download of this part, which an answer just settled.
    pub fn settle(&mut self, message: MessageId, part: u32) -> Option<Fetch> {
        let at = self.fetches.iter().position(|fetch| fetch.message == message && fetch.attachment.part == part)?;
        self.fetches.remove(at)
    }
}

#[cfg(test)]
include!("attachment_state.tests.rs");
