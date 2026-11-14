//! How long a send waits on the mail server, step by step and as a whole.

use std::time::Duration;

/// Every wait of one send. A step that takes longer than its limit ends the
/// send; so does a send that takes longer than `total`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmtpTimeouts {
    /// Reaching the server, and each secure handshake.
    pub connect: Duration,
    /// The answer to one command or to the greeting.
    pub reply: Duration,
    /// The answer after the message was sent, which the server may spend
    /// time on, and each write of the message.
    pub finish: Duration,
    /// The whole send, from the first connection attempt to the last answer.
    pub total: Duration,
}

impl SmtpTimeouts {
    pub const fn production() -> Self {
        Self {
            connect: Duration::from_secs(20),
            reply: Duration::from_secs(60),
            finish: Duration::from_secs(120),
            total: Duration::from_secs(300),
        }
    }
}
