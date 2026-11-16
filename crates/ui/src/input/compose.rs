//! Using and finishing a message from the screen.
//!
//! While the composer is open it takes every key, typed letter, paste and
//! click; nothing reaches the screen underneath. Sending only asks the
//! backend: the composer stays open, showing it is sending, until the backend
//! says the message was accepted, so a failure never costs the user their
//! text. Starting a message is in `compose_start`.

use model::{Command, Event, MessageId};
use surface::InputEvent;

use crate::app_view::AppView;
use crate::composer::attach_input::path_box_input;
use crate::composer::draw::pane;
use crate::composer::input::{Outcome, handle_key, handle_text};
use crate::composer::mouse::handle_pointer;
use crate::composer::Progress;

/// What kind of message to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeKind {
    New,
    Reply,
    ReplyAll,
    Forward,
    /// A draft in the Drafts folder, to go on writing; it is taken out of the
    /// folder once the message is sent.
    Draft,
    /// A copy of any message as a new one to edit.
    EditAsNew,
}

impl AppView {
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
        let outcome = match path_box_input(form, event, main) {
            Some(boxed) => boxed,
            None => match event {
                InputEvent::Key(key) => handle_key(form, key, pane),
                InputEvent::Text(text) | InputEvent::Paste(text) => handle_text(form, text, pane),
                InputEvent::Button { .. } | InputEvent::PointerMoved { .. } | InputEvent::Wheel { .. } => {
                    handle_pointer(form, event, main)
                }
                _ => Outcome::Ignored,
            },
        };
        match outcome {
            Outcome::Ignored => false,
            Outcome::Changed => {
                form.refresh_suggestions(&self.addresses);
                true
            }
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
        form.suggest = None;
        form.from_menu = None;
        if let Some(wait) = form.hold_send() {
            form.notice = Some(wait);
            return;
        }
        match form.to_outgoing() {
            Err(problem) => form.notice = Some(problem),
            Ok(message) => {
                form.progress = Progress::Sending;
                form.notice = None;
                self.queue.push(Command::Send { message });
            }
        }
    }

    /// The paths the user typed in the composer's path box, once, for the
    /// program to read the files they name (it answers with `file_attached`).
    pub fn take_attach_asks(&mut self) -> Vec<crate::composer::AttachAsk> {
        self.composer.as_mut().map(|form| form.take_attach_asks()).unwrap_or_default()
    }

    /// The program's answer about a file the user named: the file, to be
    /// attached, or why it was not read.
    pub fn file_attached(&mut self, result: Result<model::OutgoingAttachment, String>) {
        let Some(form) = self.composer.as_mut() else {
            return;
        };
        match result {
            Ok(file) => {
                form.add_file(file);
            }
            Err(reason) => form.notice = Some(reason),
        }
    }

    /// Text the user cut or copied in the composer, once, for the host to put
    /// on the system clipboard.
    pub fn take_copied(&mut self) -> Option<String> {
        if let Some(link) = self.setup.as_mut().and_then(|form| form.copied.take()) {
            return Some(link);
        }
        if let Some(link) = self.copied_link.take() {
            return Some(link);
        }
        self.composer.as_mut()?.copied.take()
    }

    /// The backend's answer to a send: accepted closes the composer, keeps the
    /// addresses the message went to for completing the next ones, and takes
    /// the draft it continued out of the Drafts folder; a failure gives the
    /// message back to the user with the reason.
    pub(crate) fn note_send_outcome(&mut self, event: &Event) {
        let Some(form) = self.composer.as_mut() else {
            return;
        };
        if form.progress != Progress::Sending {
            return;
        }
        match event {
            Event::MessageSent { .. } => {
                let sent = form.to_outgoing().ok();
                let draft = form.draft;
                self.composer = None;
                if let Some(message) = sent {
                    self.note_sent_to(message.to.iter().chain(&message.cc).chain(&message.bcc));
                }
                if let Some(id) = draft {
                    self.remove_sent_draft(id);
                }
            }
            Event::OpFailed { operation, reason, .. } if operation == "send" => {
                form.progress = Progress::Writing;
                form.close_armed = false;
                form.notice = Some(format!("not sent: {reason}"));
            }
            _ => {}
        }
    }

    /// Takes the draft the sent message continued out of the Drafts folder
    /// (it goes to the Trash, which keeps it for a while), and moves the
    /// selection off it when it is the one selected.
    fn remove_sent_draft(&mut self, id: MessageId) {
        self.select_neighbor_outside(&[id]);
        if self.shown_page().is_some_and(|page| page.envelopes.iter().any(|envelope| envelope.id == id)) {
            self.mail.relocated.push(id);
        }
        self.queue.push(Command::Delete { messages: vec![id] });
    }
}

#[cfg(test)]
include!("compose.tests.rs");
