//! The wait before the next reconnect attempt: doubling from a start value up
//! to a ceiling, back to the start after a success.

use std::time::Duration;

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

    /// Back to the first, shortest wait.
    pub fn reset(&mut self) {
        self.next = self.start;
    }
}

#[cfg(test)]
include!("backoff.tests.rs");
