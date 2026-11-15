//! Waiting for a thread to end, but not forever.

use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How often a thread that has not ended yet is looked at again.
const STEP: Duration = Duration::from_millis(10);

/// Waits up to `limit` for `thread` to end and returns whether it did. A
/// thread that is still going after that is left to end by itself: closing an
/// account must never hang on one stuck worker.
pub(super) fn join_within(thread: JoinHandle<()>, limit: Duration) -> bool {
    let until = Instant::now() + limit;
    while !thread.is_finished() && Instant::now() < until {
        std::thread::sleep(STEP);
    }
    if !thread.is_finished() {
        return false;
    }
    let _ = thread.join();
    true
}

#[cfg(test)]
include!("bounded_join.tests.rs");
