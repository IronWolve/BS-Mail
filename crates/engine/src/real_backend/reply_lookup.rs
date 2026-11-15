//! Finding the ids of the message a reply answers: the `Message-ID` and
//! `References` of the original. They are kept when a message is opened, so
//! the usual reply needs no look-up at all; otherwise they are read from the
//! server with PEEK, so the original is not marked as read.
//!
//! When the server cannot give them (the mailbox is down, the folder was
//! rebuilt, the message was moved away or deleted) the ids the list showed
//! for that message are used instead, which keep the reply in its
//! conversation. Only when even those are gone does the reply go without, and
//! the lookup says why, so the person is told.

use model::MessageId;

use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};
use crate::send::{ReplyIds, parse_threading_headers};

/// What a look for a reply's thread ids came to.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Lookup {
    /// The ids to put in the reply, when there are any.
    pub ids: Option<ReplyIds>,
    /// Why the reply goes without ids, when the original could not be read.
    /// `None` when they were found, and when the original is known to have
    /// no usable `Message-ID`, which leaves nothing to say.
    pub missing: Option<&'static str>,
}

/// The original is not one this run knows: its folder or its id is gone.
const UNKNOWN: &str = "the original message is not known any more";
/// The original's folder was rebuilt on the server, so its ids are void.
const REBUILT: &str = "the original's folder was rebuilt on the server";
/// The server no longer has the original.
const GONE: &str = "the original message is no longer on the server";
/// The server would not open the folder or give the headers.
const REFUSED: &str = "the server would not give the original's headers";
/// The connection to the mailbox failed while it was asked.
pub(super) const LOST: &str = "the connection to the mailbox was lost";
/// There is no connection to the mailbox.
pub(super) const OFFLINE: &str = "the mailbox is not connected";

/// What the server said, or why it could not say.
type Asked = Result<Option<ReplyIds>, &'static str>;

impl Worker {
    /// The thread ids of message `original`. Only a lost connection is an
    /// error, and the caller then still has `listed_lookup` to fall back on.
    pub(super) async fn reply_ids(
        &mut self,
        session: &mut ImapSession,
        original: MessageId,
    ) -> Result<Lookup, SessionError> {
        if let Some(known) = self.threads.get(original) {
            return Ok(Lookup { ids: known, missing: None });
        }
        match self.ask_server(session, original).await? {
            Ok(ids) => Ok(Lookup { ids, missing: None }),
            Err(why) => Ok(self.listed_lookup(original, why)),
        }
    }

    /// What is known without the server: the ids kept when the message was
    /// opened, else what the list showed. `why` is said when neither is
    /// there.
    pub(super) fn known_ids(&self, original: MessageId, why: &'static str) -> Lookup {
        match self.threads.get(original) {
            Some(known) => Lookup { ids: known, missing: None },
            None => self.listed_lookup(original, why),
        }
    }

    /// The ids the list showed for the message, or `why` they are missing.
    pub(super) fn listed_lookup(&self, original: MessageId, why: &'static str) -> Lookup {
        match self.listed.get(original) {
            Some(ids) => Lookup { ids: Some(ids), missing: None },
            None => Lookup { ids: None, missing: Some(why) },
        }
    }

    /// Reads the ids from the server. The answer is the ids (none when the
    /// message has no usable `Message-ID`), or the reason the server could
    /// not say. A folder found rebuilt has its page read again, as every id
    /// the screen holds for it is void.
    async fn ask_server(&mut self, session: &mut ImapSession, original: MessageId) -> Result<Asked, SessionError> {
        let Some((folder, uid)) = self.state.message_key(original) else {
            return Ok(Err(UNKNOWN));
        };
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(Err(UNKNOWN));
        };
        let reopened = match session.ensure_examined(&path).await {
            Ok(reopened) => reopened,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => return Ok(Err(REFUSED)),
        };
        if reopened.is_some_and(|info| self.note_validity(folder, info.uid_validity)) {
            if self.wanted.folder == Some(folder) {
                let page = self.replace_page(session, folder, &path, "reply").await?;
                self.sink.send_all(page);
            }
            return Ok(Err(REBUILT));
        }
        let raw = match session.fetch_threading_headers(uid.0).await {
            Ok(raw) => raw,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => return Ok(Err(REFUSED)),
        };
        let Some(raw) = raw else {
            return Ok(Err(GONE));
        };
        let found = parse_threading_headers(&raw);
        self.threads.keep(original, found.clone());
        Ok(Ok(found))
    }
}

#[cfg(test)]
include!("reply_lookup.tests.rs");
