//! The backend that serves several accounts as one.

use std::sync::{Arc, Mutex};

use model::{AccountId, Command, Event, MailBackend};

use super::entry::Entry;
use super::handle::MembersHandle;
use super::member::Member;
use super::pending::Pending;
use super::slots::Slot;

/// Serves several mail accounts as one backend: one member backend per
/// account, ids kept apart by slot (see `slots`), and one account on screen
/// at a time (the active one), which is the one whose mail is forwarded.
///
/// The first member is active at start. Which events go on and which are only
/// remembered is decided in `policy`; a switch is answered from what was
/// remembered. The account list is built here, never taken from a member.
pub struct MultiBackend {
    pub(super) entries: Vec<Entry>,
    /// The account the screen shows.
    pub(super) active: Option<AccountId>,
    /// The slot the next member gets; only ever grows.
    pub(super) next_slot: Slot,
    pub(super) host_sent: bool,
    /// An account list is owed to the screen.
    pub(super) list_due: bool,
    /// The active member must be described to the screen again, because the
    /// one it was showing is gone.
    pub(super) replay_due: bool,
    /// Failures of the composite's own, sent at the next poll.
    pub(super) notices: Vec<Event>,
    pub(super) shared: Arc<Mutex<Pending>>,
}

impl MultiBackend {
    /// Serves `members`, in this order. A later member with the id of an
    /// earlier one is not added, and the screen gets a failure notice.
    pub fn new(members: Vec<Member>) -> Self {
        let mut pending = Pending::default();
        for member in members {
            pending.add(member, Vec::new());
        }
        let mut multi = Self {
            entries: Vec::new(),
            active: None,
            next_slot: 0,
            host_sent: false,
            list_due: true,
            replay_due: false,
            notices: Vec::new(),
            shared: Arc::new(Mutex::new(pending)),
        };
        multi.apply_pending();
        multi
    }

    /// Starts on the member `id` instead of the first. An id that is not a
    /// member leaves the first one active.
    #[must_use]
    pub fn with_active(mut self, id: &AccountId) -> Self {
        if self.entries.iter().any(|entry| &entry.id == id) {
            self.active = Some(id.clone());
        }
        self
    }

    /// A handle for adding, removing and replacing members while the
    /// composite is in use.
    pub fn members(&self) -> MembersHandle {
        MembersHandle::new(&self.shared)
    }
}

impl MailBackend for MultiBackend {
    /// Routes the command to the member or members it is about and returns
    /// what they answer at once. Never blocks.
    fn handle(&mut self, command: Command) -> Vec<Event> {
        self.apply_pending();
        self.dispatch(command)
    }

    /// The account list when one is owed, then everything the members have
    /// produced, in member order.
    fn poll_events(&mut self) -> Vec<Event> {
        self.apply_pending();
        self.poll()
    }
}

#[cfg(test)]
include!("composite.tests.rs");
