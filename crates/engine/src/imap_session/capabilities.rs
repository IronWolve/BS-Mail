//! What the server says it can do, reduced to the few things the client
//! branches on.

use async_imap::types::{Capabilities, Capability};

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// The extensions this client cares about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerCaps {
    /// Folder roles come as LIST attributes (RFC 6154).
    pub special_use: bool,
    /// The LIST command accepts return options (RFC 5258).
    pub list_extended: bool,
    /// LIST can return each mailbox's counts with its name (RFC 5819).
    pub list_status: bool,
    /// Mailbox storage figures (RFC 2087).
    pub quota: bool,
    /// The server can push changes while idle (RFC 2177).
    pub idle: bool,
    /// Commands can name exact UIDs to expunge (RFC 4315). Without it the
    /// only expunge removes every deleted message in the mailbox, which this
    /// client never sends.
    pub uidplus: bool,
    /// Messages can be moved in one step (RFC 6851).
    pub move_ext: bool,
}

impl ServerCaps {
    /// Reads capability atoms, any letter case.
    pub fn from_atoms<'a>(atoms: impl IntoIterator<Item = &'a str>) -> Self {
        let mut caps = Self::default();
        for atom in atoms {
            match atom.to_ascii_uppercase().as_str() {
                "SPECIAL-USE" => caps.special_use = true,
                "LIST-EXTENDED" => caps.list_extended = true,
                "LIST-STATUS" => caps.list_status = true,
                "QUOTA" => caps.quota = true,
                "IDLE" => caps.idle = true,
                "UIDPLUS" => caps.uidplus = true,
                "MOVE" => caps.move_ext = true,
                _ => {}
            }
        }
        caps
    }

    fn from_capabilities(found: &Capabilities) -> Self {
        let atoms: Vec<&str> = found
            .iter()
            .filter_map(|capability| match capability {
                Capability::Atom(name) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        Self::from_atoms(atoms)
    }
}

impl ImapSession {
    /// Asks the server what it supports and remembers the answer.
    pub async fn load_capabilities(&mut self) -> Result<(), SessionError> {
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.capabilities()).await;
        let found = self.settle(result)?;
        self.caps = ServerCaps::from_capabilities(&found);
        Ok(())
    }
}

#[cfg(test)]
include!("capabilities.tests.rs");
