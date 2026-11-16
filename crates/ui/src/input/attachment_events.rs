//! What the engine's answer about an attachment does, and what the screen
//! hands the program: a file that arrived for the reader (to be written to
//! disk, and opened for `o`) or for a forward being written (it joins the
//! message), and the words the program says back in the footer. A failure is
//! shown in the footer by the usual notice; here it also ends the wait of the
//! forward it belonged to, so the composer can say which file is missing.

use model::{AttachmentInfo, FileBytes, MessageId};

use crate::app_view::AppView;
use crate::reader_tools::{AttachmentFile, Owner};

impl AppView {
    /// An attachment arrived: it goes to whoever asked for it.
    pub(crate) fn attachment_arrived(&mut self, message: MessageId, attachment: &AttachmentInfo, data: &FileBytes) {
        let Some(fetch) = self.reading.attachments.settle(message, attachment.part) else {
            return;
        };
        match fetch.owner {
            Owner::Reader(want) => self.reading.attachments.files.push(AttachmentFile {
                name: attachment.name.clone(),
                content_type: attachment.content_type.clone(),
                data: data.clone(),
                want,
            }),
            Owner::Forward => {
                if let Some(form) = self.composer.as_mut() {
                    form.forwarded_arrived(attachment, data.clone());
                }
            }
        }
    }

    /// A download failed: the oldest one waiting is the one that did, as the
    /// engine answers in the order it was asked.
    pub(crate) fn attachment_failed(&mut self, reason: &str) {
        let Some(fetch) = self.reading.attachments.fetches.pop_front() else {
            return;
        };
        if fetch.owner == Owner::Forward
            && let Some(form) = self.composer.as_mut()
        {
            form.forward_failed(&fetch.attachment.name, reason);
        }
    }

    /// The connection to the account on screen was lost: a download in flight
    /// is never answered (it is not asked again by itself), so every wait ends
    /// here. A forward is told which of its files did not come; for the reader
    /// the footer says to try again.
    pub(crate) fn attachments_interrupted(&mut self) {
        let lost = std::mem::take(&mut self.reading.attachments.fetches);
        let mut reader = None;
        for fetch in lost {
            match fetch.owner {
                Owner::Forward => {
                    if let Some(form) = self.composer.as_mut() {
                        form.forward_failed(&fetch.attachment.name, "the connection to the server was lost");
                    }
                }
                Owner::Reader(_) => reader = Some(fetch.attachment.name),
            }
        }
        if let Some(name) = reader {
            self.mail.set_note(None);
            self.mail.done_text = Some(format!("Fetching {name} was interrupted: the connection to the server was lost. Try again once it is back"));
        }
    }

    /// The attachments that arrived and are not yet taken, once, for the
    /// program to write.
    pub fn take_attachment_files(&mut self) -> Vec<AttachmentFile> {
        std::mem::take(&mut self.reading.attachments.files)
    }

    /// The footer's words about an attachment, from the program: where it was
    /// saved, or why it was not. They stay through the pages that follow, until
    /// the next key.
    pub fn attachment_done(&mut self, text: String) {
        self.mail.set_note(None);
        self.mail.done_text = Some(text);
    }
}

#[cfg(test)]
include!("attachment_events.tests.rs");
