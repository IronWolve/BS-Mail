//! Running the reading of one message where it can neither hold up the mail
//! worker nor take it down: on a thread of its own, with a roomy stack, a
//! time limit and a catch for a panic in the libraries that parse and convert.
//!
//! A thread cannot be stopped from outside, so one that runs past the limit is
//! given up on and left to finish by itself; while two such threads are still
//! running, no new one is started.

use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Once, mpsc};
use std::thread;
use std::time::Duration;

use super::limits::{MAX_RUNAWAY, RENDER_LIMIT, RENDER_STACK};

/// The name of the reading threads, which a panic hook can tell apart.
const THREAD_NAME: &str = "render";

/// Why a message was not read.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Failure {
    /// It took longer than the limit.
    TooSlow,
    /// A library panicked on it.
    Crashed,
    /// Earlier messages are still being worked through and no more are taken.
    Busy,
}

impl Failure {
    /// What the reading pane says in place of the message.
    pub(super) fn words(&self) -> &'static str {
        match self {
            Self::TooSlow => "This message could not be shown: reading it took too long.",
            Self::Crashed => "This message could not be shown: its contents stopped the reader.",
            Self::Busy => "This message could not be shown: the reader is still busy with an earlier one.",
        }
    }
}

/// How many given-up threads are still running.
static RUNAWAY: AtomicUsize = AtomicUsize::new(0);

/// The thread is still working.
const WORKING: u8 = 0;
/// The thread finished while the caller was still waiting.
const DELIVERED: u8 = 1;
/// The caller gave up on the thread.
const ABANDONED: u8 = 2;

/// Runs `work` under the usual limits.
pub(super) fn run<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T, Failure> {
    run_within(RENDER_LIMIT, &RUNAWAY, work)
}

/// Runs `work` on a thread of its own and waits at most `limit` for it.
/// `runaway` counts the threads given up on.
pub(super) fn run_within<T: Send + 'static>(
    limit: Duration,
    runaway: &'static AtomicUsize,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, Failure> {
    if runaway.load(Ordering::SeqCst) >= MAX_RUNAWAY {
        return Err(Failure::Busy);
    }
    quiet_panics();
    let (send, receive) = mpsc::channel();
    let state = Arc::new(AtomicU8::new(WORKING));
    let thread_state = Arc::clone(&state);
    let started = thread::Builder::new().name(THREAD_NAME.to_owned()).stack_size(RENDER_STACK).spawn(move || {
        let outcome = panic::catch_unwind(AssertUnwindSafe(work));
        if thread_state.compare_exchange(WORKING, DELIVERED, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            // Nobody is waiting any more: this thread is no longer running away.
            runaway.fetch_sub(1, Ordering::SeqCst);
            return;
        }
        let _ = send.send(outcome);
    });
    if started.is_err() {
        return Err(Failure::Crashed);
    }
    match receive.recv_timeout(limit) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err(Failure::Crashed),
        Err(_) => give_up(&state, &receive, runaway),
    }
}

/// The wait ran out. The thread may have finished in that very moment, in
/// which case its answer is taken after all.
fn give_up<T>(
    state: &AtomicU8,
    receive: &mpsc::Receiver<thread::Result<T>>,
    runaway: &AtomicUsize,
) -> Result<T, Failure> {
    if state.compare_exchange(WORKING, ABANDONED, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
        runaway.fetch_add(1, Ordering::SeqCst);
        return Err(Failure::TooSlow);
    }
    match receive.recv() {
        Ok(Ok(value)) => Ok(value),
        _ => Err(Failure::Crashed),
    }
}

/// A panic that is caught here is answered on screen, so it must not also be
/// printed over the screen or put the terminal back. Panics anywhere else go
/// to whatever hook was there before.
fn quiet_panics() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if thread::current().name() != Some(THREAD_NAME) {
                previous(info);
            }
        }));
    });
}

#[cfg(test)]
include!("guarded.tests.rs");
