//! How long the session waits on the server before giving up.

use std::future::Future;
use std::time::Duration;

use async_imap::error::Error as ImapError;

use super::error::SessionError;

/// Waits for the network, in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeouts {
    /// Opening the connection, the TLS handshake and the greeting.
    pub connect: Duration,
    /// One command, from sending it to its last response line.
    pub command: Duration,
}

impl Timeouts {
    pub const fn production() -> Self {
        Self {
            connect: Duration::from_secs(20),
            command: Duration::from_secs(90),
        }
    }
}

/// Runs one session operation under the command limit; a stuck server counts
/// as a lost connection.
pub async fn bounded<T>(
    limit: Duration,
    work: impl Future<Output = Result<T, ImapError>>,
) -> Result<T, SessionError> {
    match tokio::time::timeout(limit, work).await {
        Ok(result) => result.map_err(SessionError::from),
        Err(_) => Err(SessionError::timed_out()),
    }
}
