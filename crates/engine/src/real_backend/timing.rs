//! The waits and limits of the worker, gathered so tests can shorten them
//! without any production setting changing.

use std::time::Duration;

use crate::imap_session::Timeouts;

/// How long the worker waits between reconnect attempts, how often it checks
/// that the connection is alive, and how long it waits on the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub(super) backoff_start: Duration,
    pub(super) backoff_max: Duration,
    pub(super) keepalive: Duration,
    pub(super) timeouts: Timeouts,
}

impl Timing {
    /// Reconnect after 2 s, doubling to 5 min; a check every 60 s.
    pub const fn production() -> Self {
        Self {
            backoff_start: Duration::from_secs(2),
            backoff_max: Duration::from_secs(300),
            keepalive: Duration::from_secs(60),
            timeouts: Timeouts::production(),
        }
    }

    /// The same limits with a different reconnect wait.
    #[cfg(feature = "test-support")]
    pub const fn with_backoff(mut self, start: Duration, max: Duration) -> Self {
        self.backoff_start = start;
        self.backoff_max = max;
        self
    }

    /// Milliseconds instead of seconds, for tests that watch a reconnect.
    #[cfg(feature = "test-support")]
    pub const fn quick() -> Self {
        Self {
            backoff_start: Duration::from_millis(40),
            backoff_max: Duration::from_millis(160),
            keepalive: Duration::from_millis(60),
            timeouts: Timeouts {
                connect: Duration::from_secs(5),
                command: Duration::from_secs(10),
            },
        }
    }
}
