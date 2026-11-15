//! Starting, using and finishing a message from the screen.
//!
//! While the composer is open it takes every key, typed letter and paste;
//! nothing reaches the screen underneath. Sending only asks the backend: the
//! composer stays open, showing it is sending, until the backend says the
//! message was accepted, so a failure never costs the user their text.

use model::{Command, Event};
use surface::InputEvent;

use crate::app_view::AppView;
use crate::composer::draw::pane;
use crate::composer::input::{Outcome, handle_key, handle_text};
use crate::composer::reply::{Draft, Original, body_text, forward, reply};
use crate::composer::{ComposeForm, Progress};
use crate::format::{CivilTime, format_pattern};

/// What kind of message to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeKind {
    New,
    Reply,
    ReplyAll,
    Forward,
}

/// What the footer says when the message to answer has not arrived yet: its
/// Reply-To and its text come with its body, and a reply made without them
/// could go to the wrong address.
const BODY_NOT_HERE_NOTE: &str = "this message has not finished loading, so it cannot be answered or forwarded yet; try again in a moment";

/// Why no draft could be made from the selected message.
enum NoDraft {
    /// No message is selected.
    NothingSelected,
    /// Its body, which holds the Reply-To and the text, has not arrived.
    BodyNotHere,
}

/// What the footer says when the account does not allow sending.
const READ_ONLY_NOTE: &str = "this account is read-only: set read_only = false in its file to send mail";

impl AppView {
    /// Opens the composer for a new message, or for a reply or forward of the
    /// selected one.
    pub(crate) fn start_compose(&mut self, kind: ComposeKind) {
        if self.mail.changes_refused {
            self.mail.set_note(Some(READ_ONLY_NOTE.to_string()));
            return;
        }
        let Some((account, from, own)) = self.selected_account().map(|account| {
            let name = account.name.trim();
            let from = if name.is_empty() {
                account.address.clone()
            } else {
                format!("{name} <{}>", account.address)
            };
            (account.id.clone(), from, account.address.clone())
        }) else {
            return;
        };
        let (draft, reply_to) = match kind {
            ComposeKind::New => (Draft::default(), None),
            other => match self.draft_from_selected(other, &own) {
                Ok(found) => found,
                Err(NoDraft::NothingSelected) => return,
                Err(NoDraft::BodyNotHere) => {
                    self.mail.set_note(Some(BODY_NOT_HERE_NOTE.to_string()));
                    return;
                }
            },
        };
        self.composer = Some(ComposeForm::from_draft(account, from, &draft, reply_to));
    }

    /// The reply or forward of the selected message, with its id. Only once
    /// its body is here: without it the draft would have no Reply-To and no
    /// quoted text.
    fn draft_from_selected(&self, kind: ComposeKind, own: &str) -> Result<(Draft, Option<model::MessageId>), NoDraft> {
        let folder = self.selection.folder.ok_or(NoDraft::NothingSelected)?;
        let page = self.mail.page.as_ref().filter(|page| page.folder == folder).ok_or(NoDraft::NothingSelected)?;
        let envelope = page.envelopes.get(self.selection.message_index).ok_or(NoDraft::NothingSelected)?;
        let body = self
            .mail
            .body
            .as_ref()
            .filter(|(id, _)| *id == envelope.id)
            .map(|(_, body)| body)
            .ok_or(NoDraft::BodyNotHere)?;
        let text = body_text(body);
        let reply_to = body.reply_to.clone();
        let time = CivilTime::from_unix(envelope.date.unix, envelope.date.offset_minutes);
        let original = Original {
            envelope,
            text,
            date: format_pattern(&self.config.reader.date_format, &time),
            reply_to,
        };
        Ok(match kind {
            ComposeKind::Forward => (forward(&original), None),
            ComposeKind::ReplyAll => (reply(&original, own, true), Some(envelope.id)),
            _ => (reply(&original, own, false), Some(envelope.id)),
        })
    }

    /// Whether a message is with the server: the composer takes no text and
    /// no key then, so the keys that close the program are not for a text box.
    pub fn is_sending(&self) -> bool {
        self.composer.as_ref().is_some_and(|form| form.progress == Progress::Sending)
    }

    /// Whether the program may close now. A message with text in it, or one
    /// being sent, asks first: the first ask only says what closing costs,
    /// and the second goes ahead.
    pub fn may_close(&mut self) -> bool {
        self.composer.as_mut().is_none_or(|form| form.may_close_program())
    }

    /// Input while the composer is open. Returns whether the screen changed.
    pub(crate) fn composer_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let main = self.layout(grid.0, grid.1).main;
        let pane = pane(main);
        let Some(form) = self.composer.as_mut() else {
            return false;
        };
        let outcome = match event {
            InputEvent::Key(key) => handle_key(form, key, pane),
            InputEvent::Text(text) | InputEvent::Paste(text) => handle_text(form, text, pane),
            _ => Outcome::Ignored,
        };
        match outcome {
            Outcome::Ignored => false,
            Outcome::Changed => true,
            Outcome::Close => {
                self.composer = None;
                true
            }
            Outcome::Send => {
                self.send_composed();
                true
            }
        }
    }

    /// Hands the message to the backend, or says what is wrong with it.
    fn send_composed(&mut self) {
        let Some(form) = self.composer.as_mut() else {
            return;
        };
        // Pressing Send is something done since the warning about closing, so
        // the next close asks again.
        form.close_armed = false;
        match form.to_outgoing() {
            Err(problem) => form.notice = Some(problem),
            Ok(message) => {
                form.progress = Progress::Sending;
                form.notice = None;
                self.queue.push(Command::Send { message });
            }
        }
    }

    /// Text the user cut or copied in the composer, once, for the host to put
    /// on the system clipboard.
    pub fn take_copied(&mut self) -> Option<String> {
        if let Some(link) = self.setup.as_mut().and_then(|form| form.copied.take()) {
            return Some(link);
        }
        self.composer.as_mut()?.copied.take()
    }

    /// The backend's answer to a send: accepted closes the composer, a
    /// failure gives the message back to the user with the reason.
    pub(crate) fn note_send_outcome(&mut self, event: &Event) {
        let Some(form) = self.composer.as_mut() else {
            return;
        };
        if form.progress != Progress::Sending {
            return;
        }
        match event {
            Event::MessageSent { .. } => self.composer = None,
            Event::OpFailed { operation, reason, .. } if operation == "send" => {
                form.progress = Progress::Writing;
                form.close_armed = false;
                form.notice = Some(format!("not sent: {reason}"));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
include!("compose.tests.rs");
