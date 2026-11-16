//! The attachments of the open message. `x` puts the cursor on the first
//! attachment (the next one, and `X` the one before), and the reader then
//! reads a few keys as its own: `Enter` saves the attachment in the
//! attachments folder of the data folder, `o` saves it and opens it with the
//! system, `a` asks where to save it (a path box, with the folders and files
//! that match listed), `x` and `X` step on and `Esc` leaves. The mouse does the
//! same through the menu a click opens (see `attachment_mouse`). The screen
//! only asks the engine for the attachment and hands the file that arrives to
//! the program, which writes it (see `attachment_events`).

use model::{AttachmentInfo, Command, MessageId};
use surface::{KeyEvent, LogicalKey, NamedKey};

use crate::app_view::AppView;
use crate::format::format_size;
use crate::reader_tools::{Fetch, Owner, Want};
use crate::view_state::Focus;

/// What the footer says when the open message has no attachment.
const NO_ATTACHMENTS: &str = "This message has no attachments";

impl AppView {
    /// The attachments of the message in the reader: none while its raw text
    /// shows, or before its body has come.
    pub(crate) fn reader_attachments(&self) -> &[AttachmentInfo] {
        if self.reading.raw_of(self.selection.message).is_some() {
            return &[];
        }
        match self.mail.body.as_ref() {
            Some((id, body)) if Some(*id) == self.selection.message => &body.attachments,
            _ => &[],
        }
    }

    /// The attachment the cursor is on, when it is on one of the open message's.
    pub(crate) fn attachment_cursor(&self) -> Option<usize> {
        let (message, at) = self.reading.attachments.cursor?;
        (Some(message) == self.selection.message && at < self.reader_attachments().len()).then_some(at)
    }

    /// Whether the attachment keys are read: the reader has focus and the
    /// cursor is on an attachment.
    pub(crate) fn attachment_mode(&self) -> bool {
        self.focus == Focus::Reader && self.attachment_cursor().is_some()
    }

    /// `x` and `X`: the cursor to the next attachment, or the one before; the
    /// first (or the last) when it was on none.
    pub(crate) fn step_attachment(&mut self, forward: bool) {
        let count = self.reader_attachments().len();
        let Some(message) = self.selection.message.filter(|_| count > 0) else {
            self.mail.set_note(Some(NO_ATTACHMENTS.to_owned()));
            return;
        };
        let next = match (self.attachment_cursor(), forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
        };
        self.reading.attachments.cursor = Some((message, next));
        self.focus = Focus::Reader;
        self.scroll.reader = self.scroll.reader.min(next);
        let name = self.reader_attachments().get(next).map(|each| each.name.clone()).unwrap_or_default();
        self.mail.set_note(None);
        self.mail.done_text =
            Some(format!("Attachment {} of {count}: {name}. Enter saves it, o opens it, a saves it as, x next, Esc done", next + 1));
    }

    /// The keys of the attachment mode: Enter saves, `o` opens, `a` saves as,
    /// `x` and `X` step, Esc leaves. False for any other key.
    pub(crate) fn attachment_mode_key(&mut self, key: &KeyEvent) -> bool {
        let held = key.modifiers;
        if held.ctrl || held.alt || held.meta {
            return false;
        }
        match key.key {
            LogicalKey::Named(NamedKey::Enter) => self.save_attachment(None, Want::Save),
            LogicalKey::Named(NamedKey::Escape) => self.reading.attachments.cursor = None,
            LogicalKey::Char('o') if !held.shift => self.save_attachment(None, Want::Open),
            LogicalKey::Char('a') if !held.shift => self.ask_save_as(None),
            LogicalKey::Char('x') if !held.shift => self.step_attachment(true),
            LogicalKey::Char('X') => self.step_attachment(false),
            _ => return false,
        }
        true
    }

    /// The attachment an action is about: the one asked for, else the one the
    /// cursor is on, else the first, with its place in the list.
    pub(crate) fn attachment_choice(&self, index: Option<usize>) -> Option<(MessageId, AttachmentInfo, usize)> {
        let message = self.selection.message?;
        let at = index.or_else(|| self.attachment_cursor()).unwrap_or(0);
        Some((message, self.reader_attachments().get(at)?.clone(), at))
    }

    /// Saves (or opens) an attachment of the open message: the one given, the
    /// cursor's, or the first. The engine is asked for it; what arrives is
    /// written by the program.
    pub(crate) fn save_attachment(&mut self, index: Option<usize>, want: Want) {
        let Some((message, attachment, at)) = self.attachment_choice(index) else {
            self.mail.set_note(Some(NO_ATTACHMENTS.to_owned()));
            return;
        };
        self.reading.attachments.cursor = Some((message, at));
        self.fetch_attachment(message, attachment, Owner::Reader(want));
    }

    /// Asks the engine for an attachment, once for each purpose at a time.
    pub(crate) fn fetch_attachment(&mut self, message: MessageId, attachment: AttachmentInfo, owner: Owner) {
        if self.reading.attachments.is_waiting(message, attachment.part, &owner) {
            return;
        }
        if matches!(owner, Owner::Reader(_)) {
            let size = format_size(attachment.size, self.config.list.size_format);
            self.mail.set_note(Some(format!("Fetching {} ({size})...", attachment.name)));
        }
        self.queue.push(Command::FetchAttachment { message, attachment: attachment.clone() });
        self.reading.attachments.fetches.push_back(Fetch { message, attachment, owner });
    }
}

#[cfg(test)]
include!("attachment_keys.tests.rs");
