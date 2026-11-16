//! Showing the raw text of the selected message in the reading pane: `H` the
//! header block, `v` the whole source. The engine reads it from the server
//! (never changing it), the pane shows it as plain wrapped text that can be
//! selected, copied and searched, and Escape, or another message, brings the
//! message back. Pressing the same key again does too.

use model::{Command, ConnectionState, Event, MessageId, RawPart};

use crate::app_view::AppView;
use crate::view_state::Focus;

impl AppView {
    /// The message the keys act on, when one is selected.
    pub(crate) fn message_to_read(&self) -> Option<MessageId> {
        self.shown_page()?.envelopes.get(self.selection.message_index)?;
        self.selection.message
    }

    /// Shows `part` of the selected message in the reader, or brings the
    /// message back when that is what is showing.
    pub(crate) fn view_raw(&mut self, part: RawPart) {
        let Some(message) = self.message_to_read() else {
            return;
        };
        self.reading.find = None;
        if self.reading.raw_of(Some(message)).is_some_and(|raw| raw.part == part) {
            self.reading.raw = None;
            self.scroll.reader = 0;
            return;
        }
        self.reading.show_raw(message, part);
        self.focus = Focus::Reader;
        self.scroll.reader = 0;
        self.queue.push(Command::FetchRaw { message, part });
    }

    /// What the backend says about the raw text and the browser's page: the
    /// text arrives into the view that asked for it, and a failure ends the
    /// wait (the footer says why).
    pub(crate) fn note_reading_event(&mut self, event: &Event) {
        match event {
            Event::RawText { message, part, text } => {
                let waiting = self.reading.raw.as_mut().filter(|raw| raw.message == *message && raw.part == *part);
                if let Some(raw) = waiting {
                    raw.arrived(text);
                    self.scroll.reader = 0;
                }
            }
            Event::HtmlPage { message, html, removed } => self.page_arrived(*message, html, *removed),
            Event::AttachmentData { message, attachment, data } => self.attachment_arrived(*message, attachment, data),
            Event::ConnectionState { account, state } if *state != ConnectionState::Online && self.selection.account.as_ref() == Some(account) => {
                self.attachments_interrupted();
            }
            Event::OpFailed { operation, reason, .. } if operation == "save attachment" => self.attachment_failed(reason),
            Event::OpFailed { operation, .. } if operation == "view headers" || operation == "view source" => {
                self.reading.raw = None;
            }
            Event::OpFailed { operation, .. } if operation == "open in browser" => self.reading.page_asked = None,
            _ => {}
        }
    }
}

#[cfg(test)]
include!("raw_keys.tests.rs");
