//! Two reads of a whole message that are not for the reading pane's text: its
//! raw header block or source, and the safe copy of its HTML part for the
//! browser. Both ask the server for the message the way a body is asked for
//! (`BODY.PEEK`, at most the same ceiling) and change nothing there.

use model::{Event, MessageId, RawPart};

use super::handlers::FOLDER_REBUILT;
use super::worker::Worker;
use crate::html_mail::Outcome;
use crate::imap_session::{ImapSession, SessionError};
use crate::mime_render::{raw_text, render_page};

/// What the answer to a message that is not there is called.
pub(super) const MISSING: &str = "the message is no longer on the server";

/// The message as the server holds it, or the failure to send instead.
type Read = Result<(Vec<u8>, Option<u64>), Event>;

pub(super) fn operation_of(part: RawPart) -> &'static str {
    match part {
        RawPart::Headers => "view headers",
        RawPart::Source => "view source",
    }
}

impl Worker {
    /// The raw header block or source of one message.
    pub(super) async fn fetch_raw(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
        part: RawPart,
    ) -> Result<Vec<Event>, SessionError> {
        Ok(match self.read_message(session, message, operation_of(part)).await? {
            Ok((bytes, total)) => vec![Event::RawText { message, part, text: raw_text(&bytes, total, part) }],
            Err(failure) => vec![failure],
        })
    }

    /// The safe copy of one message's HTML part.
    pub(super) async fn prepare_html(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
    ) -> Result<Vec<Event>, SessionError> {
        const OPERATION: &str = "open in browser";
        let (bytes, _) = match self.read_message(session, message, OPERATION).await? {
            Ok(read) => read,
            Err(failure) => return Ok(vec![failure]),
        };
        Ok(vec![match render_page(bytes).await {
            Ok(Outcome::Page(page)) => Event::HtmlPage { message, html: page.html, removed: page.removed },
            Ok(Outcome::NoHtmlPart) => self.sink.failure(OPERATION, "this message has no HTML part to open"),
            Ok(Outcome::Unreadable) => self.sink.failure(OPERATION, "this message could not be read"),
            Err(reason) => self.sink.failure(OPERATION, reason),
        }])
    }

    /// Reads one message from the server. A connection lost on the way is the
    /// session's error, to be dealt with by the caller; anything else that
    /// stops the read comes back as the failure notice to send.
    async fn read_message(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
        operation: &str,
    ) -> Result<Read, SessionError> {
        let uid = match self.open_message(session, message, operation).await? {
            Ok(uid) => uid,
            Err(failure) => return Ok(Err(failure)),
        };
        Ok(match session.fetch_body(uid).await? {
            Some(raw) => Ok((raw.bytes, raw.total)),
            None => Err(self.sink.failure(operation, MISSING)),
        })
    }

    /// Opens the folder a message is in, and gives its UID there; or the
    /// failure to send instead when the id is unknown, the folder is gone or
    /// was rebuilt since the message was listed.
    pub(super) async fn open_message(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
        operation: &str,
    ) -> Result<Result<u32, Event>, SessionError> {
        let Some((folder, uid)) = self.state.message_key(message) else {
            return Ok(Err(self.sink.failure(operation, format!("unknown message id: {}", message.0))));
        };
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(Err(self.sink.failure(operation, "the folder is gone")));
        };
        let reopened = session.ensure_examined(&path).await?;
        if reopened.is_some_and(|info| self.note_validity(folder, info.uid_validity)) {
            return Ok(Err(self.sink.failure(operation, FOLDER_REBUILT)));
        }
        Ok(Ok(uid.0))
    }
}
