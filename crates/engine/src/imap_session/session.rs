//! A signed-in IMAP session and what it remembers between commands.

use std::time::Duration;

use async_imap::Session;

use super::capabilities::ServerCaps;
use super::connect;
use super::error::{ConnectError, SessionError};
use super::login;
use super::select::OpenMailbox;
use super::stream::MailStream;
use super::timeouts::{Timeouts, bounded};
use crate::account_config::AccountConfig;

/// How long a goodbye may take before the connection is dropped anyway.
const GOODBYE_WAIT: Duration = Duration::from_millis(500);

/// One signed-in connection. Reads are always safe: mailboxes are opened with
/// EXAMINE and bodies fetched with PEEK, so the server cannot see a change.
/// Changes are possible only when the account allows them, and then only
/// through the few operations in `write_exchange.rs`, `store_flags.rs` and
/// `move_copy.rs`, each on an explicit list of UIDs in a mailbox opened for
/// the purpose.
pub struct ImapSession {
    pub(super) inner: Session<MailStream>,
    pub(super) caps: ServerCaps,
    pub(super) limits: Timeouts,
    /// Whether the account may change the server. Fixed when connecting; a
    /// read-only session refuses every changing operation before it sends
    /// anything.
    pub(super) writes: bool,
    /// The mailbox open now, as the server names it, and how it was opened.
    pub(super) open: Option<OpenMailbox>,
}

impl ImapSession {
    /// Opens the connection, signs in and learns the server's capabilities.
    pub async fn connect(
        config: &AccountConfig,
        limits: Timeouts,
    ) -> Result<Self, ConnectError> {
        let client = connect::open(config, &limits).await?;
        let inner = login::sign_in(client, config, limits.connect).await?;
        let mut session = Self {
            inner,
            caps: ServerCaps::default(),
            limits,
            writes: !config.read_only,
            open: None,
        };
        session
            .load_capabilities()
            .await
            .map_err(|e| ConnectError::Unreachable(e.text().to_owned()))?;
        Ok(session)
    }

    /// What the server said it can do.
    pub fn caps(&self) -> &ServerCaps {
        &self.caps
    }

    /// Keeps the connection alive and notices a dead one.
    pub async fn ping(&mut self) -> Result<(), SessionError> {
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.noop()).await;
        self.settle(result)
    }

    /// A response that ended because the server hung up is not a complete
    /// answer: the client library reads a closed connection as "no more
    /// lines", so a half-finished fetch would look like an empty one.
    pub(super) fn settle<T>(&self, result: Result<T, SessionError>) -> Result<T, SessionError> {
        if self.inner.get_ref().peer_closed() {
            Err(SessionError::Lost("the server closed the connection".to_owned()))
        } else {
            result
        }
    }

    /// Says goodbye, waiting only a moment for the answer; the connection is
    /// being dropped either way.
    pub async fn close(&mut self) {
        let _ = tokio::time::timeout(GOODBYE_WAIT, self.inner.logout()).await;
    }
}
