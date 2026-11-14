//! The message being written: its fields, which one has focus, and the
//! message they make.

use model::{AccountId, Address, MessageId, OutgoingMessage};

use super::address_list::{format, parse};
use super::editor_buffer::EditorBuffer;
use super::reply::Draft;

/// The fields, in the order Tab visits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    To,
    Cc,
    Bcc,
    Subject,
    Body,
}

impl Field {
    pub const ALL: [Field; 5] = [Field::To, Field::Cc, Field::Bcc, Field::Subject, Field::Body];

    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|field| *field == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::To => "To:",
            Self::Cc => "Cc:",
            Self::Bcc => "Bcc:",
            Self::Subject => "Subject:",
            Self::Body => "",
        }
    }
}

/// Whether the message has been handed to the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    Writing,
    Sending,
}

#[derive(Debug, Clone)]
pub struct ComposeForm {
    pub account: AccountId,
    /// Who the message is from, as the From line shows it.
    pub from: String,
    pub to: EditorBuffer,
    pub cc: EditorBuffer,
    pub bcc: EditorBuffer,
    pub subject: EditorBuffer,
    pub body: EditorBuffer,
    pub focus: Field,
    pub in_reply_to: Option<MessageId>,
    pub progress: Progress,
    /// What to tell the user: a missing recipient, a failed send.
    pub notice: Option<String>,
    /// Text cut or copied inside the composer, for paste.
    pub clipboard: String,
    /// Escape was pressed on a message with text in it; once more discards.
    pub discard_armed: bool,
    /// The first screen line of the body shown.
    pub body_scroll: usize,
    /// The text the form started with, to tell whether anything was written.
    started_with: String,
}

impl ComposeForm {
    /// An empty message from an account.
    pub fn new(account: AccountId, from: String) -> Self {
        Self::from_draft(account, from, &Draft::default(), None)
    }

    /// A message started from a reply or a forward.
    pub fn from_draft(account: AccountId, from: String, draft: &Draft, reply_to: Option<MessageId>) -> Self {
        let mut body = EditorBuffer::new(&draft.body);
        body.move_text_start(false);
        let focus = if draft.to.is_empty() && !draft.is_reply() { Field::To } else { Field::Body };
        let mut form = Self {
            account,
            from,
            to: EditorBuffer::single_line(&format(&draft.to)),
            cc: EditorBuffer::single_line(&format(&draft.cc)),
            bcc: EditorBuffer::single_line(""),
            subject: EditorBuffer::single_line(&draft.subject),
            body,
            focus,
            in_reply_to: reply_to,
            progress: Progress::Writing,
            notice: None,
            clipboard: String::new(),
            discard_armed: false,
            body_scroll: 0,
            started_with: String::new(),
        };
        form.started_with = form.fingerprint();
        form
    }

    pub fn focused(&mut self) -> &mut EditorBuffer {
        match self.focus {
            Field::To => &mut self.to,
            Field::Cc => &mut self.cc,
            Field::Bcc => &mut self.bcc,
            Field::Subject => &mut self.subject,
            Field::Body => &mut self.body,
        }
    }

    pub fn buffer(&self, field: Field) -> &EditorBuffer {
        match field {
            Field::To => &self.to,
            Field::Cc => &self.cc,
            Field::Bcc => &self.bcc,
            Field::Subject => &self.subject,
            Field::Body => &self.body,
        }
    }

    fn fingerprint(&self) -> String {
        [&self.to, &self.cc, &self.bcc, &self.subject, &self.body]
            .iter()
            .map(|buffer| buffer.text())
            .collect::<Vec<_>>()
            .join("\u{1f}")
    }

    /// Whether anything was written beyond what the form started with.
    pub fn is_dirty(&self) -> bool {
        self.fingerprint() != self.started_with
    }

    /// The message these fields make, or what is wrong with them.
    pub fn to_outgoing(&self) -> Result<OutgoingMessage, String> {
        let list = |buffer: &EditorBuffer, name: &str| -> Result<Vec<Address>, String> {
            parse(&buffer.text()).map_err(|problem| format!("{name}: {problem}"))
        };
        let to = list(&self.to, "To")?;
        let cc = list(&self.cc, "Cc")?;
        let bcc = list(&self.bcc, "Bcc")?;
        if to.is_empty() && cc.is_empty() && bcc.is_empty() {
            return Err("add at least one recipient".to_string());
        }
        Ok(OutgoingMessage {
            account: self.account.clone(),
            to,
            cc,
            bcc,
            subject: self.subject.text(),
            body: self.body.text(),
            in_reply_to: self.in_reply_to,
        })
    }
}

#[cfg(test)]
include!("form.tests.rs");
