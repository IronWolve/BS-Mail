//! The wait before the next reconnect attempt: doubling from a start value up
//! to a ceiling, spread by a random amount so many copies of the program that
//! lost the same server do not all come back at the same moment, and back to
//! the start only once a session has lasted.

use std::time::Duration;

use crate::send::random::random_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    start: Duration,
    max: Duration,
    next: Duration,
}

impl Backoff {
    pub fn new(start: Duration, max: Duration) -> Self {
        Self {
            start,
            max,
            next: start,
        }
    }

    /// The wait for this attempt; the one after it is twice as long, up to
    /// the ceiling.
    pub fn next_delay(&mut self) -> Duration {
        let now = self.next;
        self.next = self.next.saturating_mul(2).min(self.max);
        now
    }

    /// As `next_delay`, spread over a quarter either way of it.
    pub fn next_wait(&mut self) -> Duration {
        spread(self.next_delay())
    }

    /// Back to the first, shortest wait.
    pub fn reset(&mut self) {
        self.next = self.start;
    }

    /// Notes that a session that `lasted` this long has ended. One that
    /// lasted `stable` starts the waits over, unless it ended on a copy the
    /// mailbox keeps cutting off (`cut_off`): that session did not work, it
    /// only took its time, and the waits must keep growing.
    pub fn session_ended(&mut self, lasted: Duration, stable: Duration, cut_off: bool) {
        if lasted >= stable && !cut_off {
            self.reset();
        }
    }
}

/// `delay` made between three quarters and one and a quarter of itself, by a
/// random amount; unchanged if the system has no randomness to give.
fn spread(delay: Duration) -> Duration {
    let Ok(bytes) = random_bytes::<2>() else {
        return delay;
    };
    let roll = u128::from(u16::from_le_bytes(bytes));
    let top = u128::from(u16::MAX);
    let nanos = delay.as_nanos() * (75 * top + 50 * roll) / (100 * top);
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

#[cfg(test)]
include!("backoff.tests.rs");
