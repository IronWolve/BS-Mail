//! Winding down a member that left the composite.
//!
//! Dropping a real backend can take seconds: it gives the sends that are on
//! their way time to finish before it stops its worker. That must not happen
//! on the thread that polls the other members and answers the screen, so a
//! member that left is dropped on a thread of its own. Nothing it says after
//! it left is ever polled, so no event of a removed account reaches the
//! screen; what it still has to finish (a send, its copy) gets the same time
//! as before.
//!
//! The composite waits for these threads when it is dropped itself, so closing
//! the program still waits for the sends under way. `winding_down` tells the
//! caller when a removed account has really stopped, which matters before its
//! files are deleted.

use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use model::{AccountId, MailBackend};

/// The ids of members that left and are not yet fully stopped. One id can
/// appear more than once: an account that was removed, added again and
/// removed again.
#[derive(Clone, Default)]
pub(super) struct Winding(Arc<Mutex<Vec<AccountId>>>);

impl Winding {
    fn list(&self) -> std::sync::MutexGuard<'_, Vec<AccountId>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn begin(&self, id: &AccountId) {
        self.list().push(id.clone());
    }

    fn end(&self, id: &AccountId) {
        let mut list = self.list();
        if let Some(at) = list.iter().position(|known| known == id) {
            list.remove(at);
        }
    }

    pub fn contains(&self, id: &AccountId) -> bool {
        self.list().contains(id)
    }
}

/// Marks the account as stopped when it is let go, however that happens.
struct Ending {
    winding: Winding,
    id: AccountId,
}

impl Drop for Ending {
    fn drop(&mut self) {
        self.winding.end(&self.id);
    }
}

/// The threads that are stopping members, and the list of what is stopping.
pub(super) struct Retired {
    threads: Vec<JoinHandle<()>>,
    winding: Winding,
}

impl Retired {
    pub fn new(winding: Winding) -> Self {
        Self { threads: Vec::new(), winding }
    }

    /// Stops `backend`, the one of account `id`, on a thread of its own and
    /// returns at once. If no thread can be made it is stopped here instead,
    /// which is slow but never loses a send.
    pub fn retire(&mut self, id: AccountId, backend: Box<dyn MailBackend>) {
        self.threads.retain(|thread| !thread.is_finished());
        self.winding.begin(&id);
        let ending = Ending { winding: self.winding.clone(), id };
        let spawned = std::thread::Builder::new().name("mail-retire".to_owned()).spawn(move || {
            drop(backend);
            drop(ending);
        });
        if let Ok(thread) = spawned {
            self.threads.push(thread);
        }
    }
}

impl Drop for Retired {
    /// Closing waits for the members that are still stopping.
    fn drop(&mut self) {
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
include!("retire.tests.rs");
