//! The files of the message being written: attached by path (the path box,
//! `Ctrl+O`), removed again, and for a forward the original's attachments,
//! fetched from the server and added as they arrive. The form only keeps the
//! list and its totals; reading a file is the program's, and fetching an
//! attachment is the engine's (see `input::attachment_keys`).
//!
//! A message may weigh 25 MB with its files encoded for sending, so every file
//! is counted at its encoded size, and one that does not fit is refused in
//! words. A forward waits for the original's attachments before it is sent,
//! and one that could not be fetched is named and left out only when the user
//! sends again knowing so.

use std::path::PathBuf;

use model::attachments::megabytes;
use model::{AttachmentInfo, FileBytes, MAX_SEND_BYTES, OutgoingAttachment, TEXT_RESERVE, sendable_name, wire_size};

use super::form::{ComposeForm, Field};
use crate::path_entry::PathPrompt;

/// A path the user typed in the path box, waiting for the program to read the
/// file (see `AppView::take_attach_asks`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachAsk {
    /// The path as typed.
    pub path: String,
    /// How much of the message the files already attached take once encoded.
    pub already: u64,
}

impl ComposeForm {
    /// The files attached, in the order they were added.
    pub fn files(&self) -> &[OutgoingAttachment] {
        &self.files
    }

    /// How many of the files came from the message being forwarded.
    pub fn from_original(&self) -> usize {
        self.forwarded.iter().filter(|from_original| **from_original).count()
    }

    /// What the files take of a message once encoded for sending.
    pub fn attached_wire(&self) -> u64 {
        self.files.iter().fold(0u64, |sum, file| sum.saturating_add(wire_size(file.data.len() as u64)))
    }

    /// Whether the Attach row is a stop for Tab and the arrow keys: it is once
    /// there is something on it to choose.
    pub fn can_focus_attach(&self) -> bool {
        !self.files.is_empty() || !self.fetching.is_empty() || !self.missing.is_empty()
    }

    /// Opens the path box to ask for a file, starting from `home`.
    pub fn open_path_box(&mut self, home: Option<PathBuf>) {
        self.path = Some(PathPrompt::attach(home));
        self.notice = None;
    }

    /// Takes the paths typed and accepted, once, for the program to read.
    pub fn take_attach_asks(&mut self) -> Vec<AttachAsk> {
        std::mem::take(&mut self.attach_asks)
    }

    /// Adds a file the user attached. False, with the reason in the notice,
    /// when it does not fit.
    pub fn add_file(&mut self, file: OutgoingAttachment) -> bool {
        match self.room_for(&file) {
            Ok(()) => {
                self.files.push(file);
                self.forwarded.push(false);
                self.attach_cursor = Some(self.files.len() - 1);
                true
            }
            Err(reason) => {
                self.notice = Some(reason);
                false
            }
        }
    }

    /// Whether the message has room for `file` on top of what is attached.
    fn room_for(&self, file: &OutgoingAttachment) -> Result<(), String> {
        let after = self.attached_wire().saturating_add(wire_size(file.data.len() as u64));
        if after.saturating_add(TEXT_RESERVE) > MAX_SEND_BYTES {
            return Err(format!(
                "{} does not fit: the message may be {} MB in all, with its files encoded for sending",
                file.name,
                megabytes(MAX_SEND_BYTES)
            ));
        }
        Ok(())
    }

    /// Takes the file under the Attach row's cursor off the message.
    pub fn remove_selected_file(&mut self) -> bool {
        let Some(at) = self.attach_cursor.filter(|at| *at < self.files.len()) else {
            return false;
        };
        self.files.remove(at);
        self.forwarded.remove(at);
        self.attach_cursor = if self.files.is_empty() { None } else { Some(at.min(self.files.len() - 1)) };
        if !self.can_focus_attach() && self.focus == Field::Attach {
            self.focus = Field::Subject;
        }
        true
    }

    /// Moves the Attach row's cursor to the next file or the one before,
    /// round the ends.
    pub fn step_attach(&mut self, forward: bool) -> bool {
        let count = self.files.len();
        if count == 0 {
            return false;
        }
        let at = match (self.attach_cursor, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
        };
        self.attach_cursor = Some(at);
        true
    }

    /// The original's attachments this forward is about to fetch, by name.
    pub fn begin_forward(&mut self, names: Vec<String>) {
        self.fetching = names;
    }

    /// An attachment of the original arrived: it is added like a file the
    /// user attached, under a name fit to send.
    pub fn forwarded_arrived(&mut self, attachment: &AttachmentInfo, data: FileBytes) {
        if let Some(at) = self.fetching.iter().position(|name| *name == attachment.name) {
            self.fetching.remove(at);
        }
        let file = OutgoingAttachment { name: sendable_name(&attachment.name), content_type: attachment.content_type.clone(), data };
        match self.room_for(&file) {
            Ok(()) => {
                self.files.push(file);
                self.forwarded.push(true);
            }
            Err(reason) => self.missing.push(reason),
        }
    }

    /// An attachment of the original could not be fetched: it is named, and the
    /// message is not sent until the user knows.
    pub fn forward_failed(&mut self, name: &str, reason: &str) {
        if let Some(at) = self.fetching.iter().position(|each| each == name) {
            self.fetching.remove(at);
        }
        self.missing.push(format!("{name}: {reason}"));
    }

    /// What stops Send for now, said as the notice, or `None` to go ahead: the
    /// original's attachments are still coming, or some could not be fetched
    /// (the first Send says which and the second sends without them).
    pub fn hold_send(&mut self) -> Option<String> {
        if !self.fetching.is_empty() {
            let left = self.fetching.len();
            return Some(format!(
                "{left} attachment{} of the original {} still being fetched: wait a moment, or take {} off the Attach line",
                if left == 1 { "" } else { "s" },
                if left == 1 { "is" } else { "are" },
                if left == 1 { "it" } else { "them" },
            ));
        }
        if self.missing.is_empty() || self.missing_armed {
            return None;
        }
        self.missing_armed = true;
        Some(format!("not attached: {}. Press Send again to send without {}", self.missing.join("; "), if self.missing.len() == 1 { "it" } else { "them" }))
    }

    /// What the notice row says while nothing else is to be said: that a forward
    /// carries the original's attachments, so the user knows before sending.
    pub fn original_note(&self) -> Option<String> {
        let sent = self.from_original();
        if !self.fetching.is_empty() {
            return Some(format!("Fetching the original's attachments ({} still to come)...", self.fetching.len()));
        }
        (sent > 0).then(|| format!("{sent} attachment{} from the original will be sent", if sent == 1 { "" } else { "s" }))
    }

    /// Everything that was waiting for a second press is forgotten: the
    /// message changed since.
    pub(super) fn disarm(&mut self) {
        self.discard_armed = false;
        self.close_armed = false;
        self.missing_armed = false;
    }
}

#[cfg(test)]
include!("attachments.tests.rs");
