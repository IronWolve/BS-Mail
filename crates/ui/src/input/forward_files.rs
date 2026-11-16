//! A forward carries the original's attachments: when the composer opens for
//! one, each attachment of the message is asked for from the engine, and the
//! composer adds each as it arrives and says so before the message is sent
//! ("2 attachments from the original will be sent"). One that would not fit in
//! a message with the others is not fetched and is named as left out. A reply
//! never carries them.

use model::attachments::megabytes;
use model::{AttachmentInfo, MAX_SEND_BYTES, MessageId, TEXT_RESERVE, wire_size};

use crate::app_view::AppView;
use crate::reader_tools::Owner;

impl AppView {
    /// Asks for the attachments of the message being forwarded, as many as fit
    /// in one message, and tells the composer which ones are coming.
    pub(crate) fn fetch_original_files(&mut self, message: MessageId, files: Vec<AttachmentInfo>) {
        let mut room = MAX_SEND_BYTES.saturating_sub(TEXT_RESERVE);
        let mut coming = Vec::new();
        let mut left_out = Vec::new();
        for info in files {
            let wire = wire_size(info.size);
            if wire > room {
                left_out.push(info.name.clone());
                continue;
            }
            room -= wire;
            coming.push(info.name.clone());
            self.fetch_attachment(message, info, Owner::Forward);
        }
        let Some(form) = self.composer.as_mut() else {
            return;
        };
        form.begin_forward(coming);
        for name in left_out {
            let reason = format!("too large to send with the others (a message may be {} MB with its files encoded)", megabytes(MAX_SEND_BYTES));
            form.forward_failed(&name, &reason);
        }
    }
}

#[cfg(test)]
include!("forward_files.tests.rs");
