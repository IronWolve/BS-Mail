//! The queue that `MembersHandle` fills and the composite empties. Changes
//! wait here until the composite's next `handle` or `poll_events`, so the
//! handle never touches a running member.

use std::sync::{Mutex, MutexGuard, PoisonError};

use model::{AccountId, Event};

use super::member::Member;
use super::retire::Winding;

/// One membership change, in the order it was asked for.
pub(super) enum Change {
    Add { member: Member, early: Vec<Event> },
    Remove(AccountId),
    Replace { member: Member, early: Vec<Event> },
    /// A request that was refused; the screen is told at the next poll.
    Notice(Event),
}

#[derive(Default)]
pub(super) struct Pending {
    changes: Vec<Change>,
    /// The ids the members will have once every queued change is applied.
    ids: Vec<AccountId>,
    /// The members that left and are still stopping.
    winding: Winding,
}

impl Pending {
    /// Queues a new member. An id that is already a member is refused: the
    /// new backend is dropped and the screen gets a notice.
    pub fn add(&mut self, member: Member, early: Vec<Event>) {
        if self.ids.contains(&member.id) {
            self.changes.push(Change::Notice(already_there(&member.id)));
            return;
        }
        self.ids.push(member.id.clone());
        self.changes.push(Change::Add { member, early });
    }

    /// Queues the removal of a member; an id that is not one changes nothing.
    pub fn remove(&mut self, id: &AccountId) {
        let Some(at) = self.ids.iter().position(|known| known == id) else {
            return;
        };
        self.ids.remove(at);
        self.changes.push(Change::Remove(id.clone()));
    }

    /// Queues a new backend for an existing member; an id that is not a
    /// member yet is added at the end.
    pub fn replace(&mut self, member: Member, early: Vec<Event>) {
        if self.ids.contains(&member.id) {
            self.changes.push(Change::Replace { member, early });
        } else {
            self.add(member, early);
        }
    }

    pub fn ids(&self) -> Vec<AccountId> {
        self.ids.clone()
    }

    /// The list of members that left and are still stopping, which the
    /// composite adds to and the handle reads.
    pub fn winding(&self) -> Winding {
        self.winding.clone()
    }

    pub fn take(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }
}

/// Locks the queue. A panic elsewhere cannot leave it half written, so a
/// poisoned lock is still used.
pub(super) fn locked(shared: &Mutex<Pending>) -> MutexGuard<'_, Pending> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

fn already_there(id: &AccountId) -> Event {
    Event::OpFailed {
        account: Some(id.clone()),
        operation: "account".to_owned(),
        reason: "this account is already there; the new one was not added".to_owned(),
    }
}

#[cfg(test)]
include!("pending.tests.rs");
