//! The handle that adds, removes and replaces members of a running
//! composite. It holds the composite's queue only weakly, so it never keeps
//! members alive after the composite is dropped; on a dropped composite every
//! call does nothing.

use std::sync::{Arc, Mutex, Weak};

use model::{AccountId, Event};

use super::member::Member;
use super::pending::{Pending, locked};

#[derive(Clone)]
pub struct MembersHandle {
    shared: Weak<Mutex<Pending>>,
}

impl MembersHandle {
    pub(super) fn new(shared: &Arc<Mutex<Pending>>) -> Self {
        Self {
            shared: Arc::downgrade(shared),
        }
    }

    /// Adds a member at the end. `early_events` are what it already produced;
    /// they are fed through as if it had just produced them. A member whose
    /// id is already there is not added and the screen gets a failure notice.
    /// Takes effect at the composite's next `handle` or `poll_events`.
    pub fn add(&self, member: Member, early_events: Vec<Event>) {
        self.with(|pending| pending.add(member, early_events));
    }

    /// Removes the member with this id, dropping its backend. An id that is
    /// not a member changes nothing.
    pub fn remove(&self, id: &AccountId) {
        self.with(|pending| pending.remove(id));
    }

    /// Swaps in a new backend for the member with the same id, which keeps
    /// its place in the list and gets a new slot; the old backend is dropped.
    /// An id that is not a member yet is added at the end.
    pub fn replace(&self, member: Member, early_events: Vec<Event>) {
        self.with(|pending| pending.replace(member, early_events));
    }

    /// The members' ids in order, as they will be once every change made so
    /// far has taken effect. Empty when the composite is gone.
    pub fn ids(&self) -> Vec<AccountId> {
        self.with(|pending| pending.ids()).unwrap_or_default()
    }

    fn with<R>(&self, work: impl FnOnce(&mut Pending) -> R) -> Option<R> {
        let shared = self.shared.upgrade()?;
        let mut pending = locked(&shared);
        Some(work(&mut pending))
    }
}

#[cfg(test)]
include!("handle.tests.rs");
