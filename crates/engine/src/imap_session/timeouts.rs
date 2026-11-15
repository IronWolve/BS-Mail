//! How long the session waits on the server before giving up.

use std::future::Future;
use std::time::{Duration, Instant};

use async_imap::error::Error as ImapError;

use super::error::SessionError;
use super::stream::ReadCounter;

/// Waits for the network, in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeouts {
    /// Opening the connection, the TLS handshake and the greeting.
    pub connect: Duration,
    /// One command, from sending it to its last response line.
    pub command: Duration,
    /// Fetching one message, which can be large: how long the server may
    /// send nothing at all while the message is coming. A download that keeps
    /// arriving is not cut by this, only by `BODY_CEILING`.
    pub body: Duration,
}

/// The longest one message download may take in all, however steadily it
/// arrives. The most that is asked for is 16 MiB, so this lets a link of about
/// 14 KiB a second finish; anything slower is given up on.
pub const BODY_CEILING: Duration = Duration::from_secs(20 * 60);

impl Timeouts {
    pub const fn production() -> Self {
        Self {
            connect: Duration::from_secs(20),
            command: Duration::from_secs(90),
            body: Duration::from_secs(180),
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

/// Runs one download. The library hands over a fetched message only when all
/// of it has arrived, so the clock alone cannot tell a slow download from a
/// dead one. The stream's byte count can: the wait ends as a lost connection
/// when a whole `silence` window passes with no new byte, which is after one
/// to two windows of nothing. A download that keeps sending is let be until
/// `ceiling`, which stops a connection that only trickles.
pub async fn bounded_by_progress<T>(
    silence: Duration,
    ceiling: Duration,
    arrived: &ReadCounter,
    work: impl Future<Output = Result<T, ImapError>>,
) -> Result<T, SessionError> {
    let started = Instant::now();
    let mut work = std::pin::pin!(work);
    let mut seen = arrived.total();
    loop {
        let left = ceiling.saturating_sub(started.elapsed());
        match tokio::time::timeout(silence.min(left), work.as_mut()).await {
            Ok(result) => return result.map_err(SessionError::from),
            Err(_) => {
                let now = arrived.total();
                if now == seen || started.elapsed() >= ceiling {
                    return Err(SessionError::timed_out());
                }
                seen = now;
            }
        }
    }
}

#[cfg(test)]
include!("timeouts.tests.rs");
