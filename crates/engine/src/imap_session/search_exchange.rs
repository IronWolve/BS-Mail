//! Searching the open mailbox: `UID SEARCH`, and no other command.
//!
//! The command is `UID SEARCH` followed by the keys from `crate::search`.
//! The mailbox must be open for reading (EXAMINE). The answer is read to the
//! command's own tagged line and its status is checked: a refusal is an
//! error, never an empty result.
//!
//! Only searches whose strings are all plain quoted text are sent. A string
//! that needs a literal (accents and other non-ASCII text, line breaks,
//! braces) must not be sent on one line with the bytes behind it: a server
//! that refuses the literal would then read the bytes as a command of its
//! own. Sending a literal safely means waiting for the server's go-ahead
//! first, which needs the raw client the gate files own; until that is
//! allowed here, such a search is refused with a plain sentence before any
//! command is sent.

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;
use crate::search::{Keys, Part};

/// Whether this build sends searches that need a literal.
pub const SENDS_LITERALS: bool = false;

/// Why a search with a literal is refused.
pub const LITERAL_REFUSAL: &str = "searching for accents, braces or line breaks is not available yet; \
     use plain letters and digits";

/// The keys as one line of plain command text, or the reason they cannot be.
fn one_line(keys: &Keys) -> Result<String, SessionError> {
    if keys.needs_charset() {
        return Err(SessionError::Refused(LITERAL_REFUSAL.to_owned()));
    }
    let mut line = String::new();
    for part in keys.parts() {
        match part {
            Part::Text(text) if !text.contains(['\r', '\n']) => line.push_str(text),
            Part::Text(_) => {
                return Err(SessionError::Refused("the search holds text that cannot be sent safely".to_owned()));
            }
            Part::Literal(_) => return Err(SessionError::Refused(LITERAL_REFUSAL.to_owned())),
        }
    }
    Ok(line)
}

impl ImapSession {
    /// The UIDs of the messages in the open mailbox that match `keys`, lowest
    /// first. A read: it changes nothing.
    pub async fn uid_search(&mut self, keys: &Keys) -> Result<Vec<u32>, SessionError> {
        if !self.open.as_ref().is_some_and(|open| !open.writable) {
            return Err(SessionError::Refused("no mailbox is open for reading".to_owned()));
        }
        let line = one_line(keys)?;
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.uid_search(line)).await;
        let mut found: Vec<u32> = self.settle(result)?.into_iter().collect();
        found.sort_unstable();
        Ok(found)
    }
}

#[cfg(test)]
include!("search_exchange.tests.rs");
