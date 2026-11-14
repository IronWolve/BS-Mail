//! Applying the changes that were queued through the handle.

use model::{AccountId, Event};

use super::composite::MultiBackend;
use super::entry::Entry;
use super::member::Member;
use super::pending::{Change, locked};
use super::slots::Slot;

impl MultiBackend {
    /// Takes the queued changes and applies them in order. Any change to who
    /// is a member makes an account list due.
    pub(super) fn apply_pending(&mut self) {
        // Taken and released before any member is dropped, so a backend that
        // takes a moment to stop never holds the queue.
        let changes = locked(&self.shared).take();
        if changes.is_empty() {
            return;
        }
        for change in changes {
            match change {
                Change::Add { member, early } => self.insert(member, early),
                Change::Remove(id) => self.remove(&id),
                Change::Replace { member, early } => self.replace(member, early),
                Change::Notice(event) => self.notices.push(event),
            }
        }
        self.settle_active();
    }

    /// A slot no member has had; a removed one is never handed out again.
    fn take_slot(&mut self) -> Slot {
        let slot = self.next_slot;
        self.next_slot = slot.saturating_add(1);
        slot
    }

    fn insert(&mut self, member: Member, early: Vec<Event>) {
        let slot = self.take_slot();
        self.entries.push(Entry::new(slot, member, early));
        self.list_due = true;
    }

    fn remove(&mut self, id: &AccountId) {
        if let Some(at) = self.entries.iter().position(|entry| &entry.id == id) {
            self.entries.remove(at);
            self.list_due = true;
        }
    }

    /// The member keeps its place and gets a slot of its own, so ids the old
    /// backend handed out can never reach the new one.
    fn replace(&mut self, member: Member, early: Vec<Event>) {
        let Some(at) = self.entries.iter().position(|entry| entry.id == member.id) else {
            self.insert(member, early);
            return;
        };
        let slot = self.take_slot();
        self.entries[at] = Entry::new(slot, member, early);
        self.list_due = true;
    }

    /// Keeps the active account a member: when it is gone, or none was chosen
    /// yet, the first member takes over. If the screen was showing the one
    /// that went, it is told what it now needs.
    fn settle_active(&mut self) {
        let present = self
            .active
            .as_ref()
            .is_some_and(|id| self.entries.iter().any(|entry| &entry.id == id));
        if present {
            return;
        }
        let was_showing = self.active.is_some();
        self.active = self.entries.first().map(|entry| entry.id.clone());
        self.replay_due = was_showing && self.active.is_some();
    }
}
