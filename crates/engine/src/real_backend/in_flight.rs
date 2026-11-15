//! The sends that are on their way. A server can accept a message a moment
//! after the program was told to close, and the saved copy of what it accepted
//! is written then too, so closing waits a few seconds for the sends under way
//! instead of cutting them off.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many sends are on their way; shared between the worker and the owner
/// of the backend.
#[derive(Debug, Clone, Default)]
pub(super) struct SendsInFlight(Arc<AtomicUsize>);

/// One send on its way, for as long as it is held.
#[derive(Debug)]
pub(super) struct InFlight(Arc<AtomicUsize>);

impl SendsInFlight {
    /// A send starts: it counts until the returned value is dropped.
    pub(super) fn begin(&self) -> InFlight {
        self.0.fetch_add(1, Ordering::AcqRel);
        InFlight(Arc::clone(&self.0))
    }

    pub(super) fn count(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
include!("in_flight.tests.rs");
