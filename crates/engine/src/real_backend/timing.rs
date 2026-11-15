//! The waits and limits of the worker, gathered so tests can shorten them
//! without any production setting changing.

use std::time::Duration;

use crate::imap_session::Timeouts;
use crate::send::SmtpTimeouts;

/// How long the worker waits between reconnect attempts, how often it checks
/// that the connection is alive, how long it waits on the server, and how
/// long a search may go on. How often it looks for new mail is the account's
/// `poll_seconds`, unless a test sets its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub(super) backoff_start: Duration,
    pub(super) backoff_max: Duration,
    /// A session that lasted this long counts as having worked: only then
    /// does the reconnect wait start over from its shortest.
    pub(super) stable_after: Duration,
    pub(super) keepalive: Duration,
    pub(super) timeouts: Timeouts,
    pub(super) smtp: SmtpTimeouts,
    /// A fixed wait between checks for new mail, for tests. `None`: the
    /// account's `poll_seconds`.
    pub(super) poll: Option<Duration>,
    /// After this long a search stops going on to further folders and
    /// answers with what it has.
    pub(super) search_deadline: Duration,
}

impl Timing {
    /// Reconnect after about 2 s, doubling to 5 min and starting over only
    /// after a session of a minute; a check every 60 s; a search goes on to
    /// another folder only within its first 20 s.
    pub const fn production() -> Self {
        Self {
            backoff_start: Duration::from_secs(2),
            backoff_max: Duration::from_secs(300),
            stable_after: Duration::from_secs(60),
            keepalive: Duration::from_secs(60),
            timeouts: Timeouts::production(),
            smtp: SmtpTimeouts::production(),
            poll: None,
            search_deadline: Duration::from_secs(20),
        }
    }

    /// The same limits with a different reconnect wait.
    #[cfg(feature = "test-support")]
    pub const fn with_backoff(mut self, start: Duration, max: Duration) -> Self {
        self.backoff_start = start;
        self.backoff_max = max;
        self
    }

    /// A session lasting this long counts as having worked, which is when the
    /// reconnect wait starts over from its shortest.
    #[cfg(feature = "test-support")]
    pub const fn with_stable_after(mut self, lasting: Duration) -> Self {
        self.stable_after = lasting;
        self
    }

    /// The limits of a send: reaching the server and each secure handshake,
    /// one answer, and the whole send. Tests use short ones to see a slow
    /// server given up on.
    #[cfg(feature = "test-support")]
    pub const fn with_send_limits(mut self, connect: Duration, reply: Duration, total: Duration) -> Self {
        self.smtp = SmtpTimeouts { connect, reply, finish: reply, total };
        self
    }

    /// Looks for new mail this often instead of every `poll_seconds`. An
    /// account that turned the check off (`poll_seconds = 0`) stays off.
    #[cfg(feature = "test-support")]
    pub const fn with_poll(mut self, every: Duration) -> Self {
        self.poll = Some(every);
        self
    }

    /// A search stops going on to further folders after this long.
    #[cfg(feature = "test-support")]
    pub const fn with_search_deadline(mut self, deadline: Duration) -> Self {
        self.search_deadline = deadline;
        self
    }

    /// Milliseconds instead of seconds, for tests that watch a reconnect.
    #[cfg(feature = "test-support")]
    pub const fn quick() -> Self {
        Self {
            backoff_start: Duration::from_millis(40),
            backoff_max: Duration::from_millis(160),
            stable_after: Duration::from_secs(1),
            keepalive: Duration::from_millis(60),
            timeouts: Timeouts {
                connect: Duration::from_secs(5),
                command: Duration::from_secs(10),
                body: Duration::from_secs(10),
            },
            smtp: SmtpTimeouts {
                connect: Duration::from_secs(5),
                reply: Duration::from_secs(10),
                finish: Duration::from_secs(10),
                total: Duration::from_secs(30),
            },
            poll: None,
            search_deadline: Duration::from_secs(20),
        }
    }
}
