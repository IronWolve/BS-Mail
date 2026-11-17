//! Making one call to the keychain on a thread of its own, and waiting for it
//! for a limited time.
//!
//! Why a plain operating-system thread: the Linux client blocks its caller
//! (and deadlocks on a tokio worker), and a Windows credential call can wait
//! on the system too, so no call is ever made on the thread that draws the
//! screen or on one of the mail tasks. Why a time limit: a keychain that shows
//! a prompt nobody answers, or a service that hangs, must not hang the
//! program. When the time is up the answer is `Unavailable`; the thread is
//! left to finish by itself (it cannot be stopped), and what it gives later is
//! dropped, and wiped if it is a key. A call that was asked to store a key and
//! answers late may still store it: that is an entry that opens nothing,
//! because no vault names its key id until the switch has gone through.

use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use super::failure::KeychainFailure;

/// Runs `work` on a new thread and returns what it returns, or `Unavailable`
/// when it did not return within `limit`. A thread that panics, or cannot be
/// started, is a failure in fixed words, never a panic here.
pub(super) fn run_within<T: Send + 'static>(
    limit: Duration,
    work: impl FnOnce() -> Result<T, KeychainFailure> + Send + 'static,
) -> Result<T, KeychainFailure> {
    let (send, receive) = channel();
    let started = thread::Builder::new().name("keychain call".to_string()).spawn(move || {
        // The receiver may be gone by now (the time ran out): the answer is dropped.
        let _ = send.send(work());
    });
    if started.is_err() {
        return Err(KeychainFailure::Other("a thread for the call could not be started"));
    }
    match receive.recv_timeout(limit) {
        Ok(answer) => answer,
        Err(RecvTimeoutError::Timeout) => Err(KeychainFailure::Unavailable),
        Err(RecvTimeoutError::Disconnected) => Err(KeychainFailure::Other("the call ended without an answer")),
    }
}

#[cfg(test)]
include!("call.tests.rs");
