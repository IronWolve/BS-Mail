//! What a poll returns: the account list when one is owed, then what the
//! screen lost, then what every member has produced.

use model::Event;

use super::composite::MultiBackend;
use super::intake::{Gate, admit_all};

impl MultiBackend {
    /// One poll, in this order: the account list when one is due (built after
    /// the members were polled, so it carries the connection states they just
    /// reported), the active member's remembered state when the screen lost
    /// the account it was showing, the composite's own failure notices, and
    /// then the members' events in member order.
    pub(super) fn poll(&mut self) -> Vec<Event> {
        // The remembered state is read before the members are polled, so it
        // is never sent twice for the same change.
        let replay = self.take_replay();
        let polled = self.poll_members();
        let mut events = Vec::new();
        if std::mem::take(&mut self.list_due) {
            events.push(self.account_list());
        }
        events.extend(replay);
        events.append(&mut self.notices);
        events.extend(polled);
        events
    }

    fn take_replay(&mut self) -> Vec<Event> {
        if !std::mem::take(&mut self.replay_due) {
            return Vec::new();
        }
        let active = self.active.as_ref();
        self.entries
            .iter()
            .find(|entry| Some(&entry.id) == active)
            .map(|entry| entry.cache.replay())
            .unwrap_or_default()
    }

    /// Polls every member, active or not, so every connection keeps working,
    /// and returns what is forwarded, in member order.
    fn poll_members(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        for entry in &mut self.entries {
            let mut raw = std::mem::take(&mut entry.backlog);
            raw.extend(entry.backend.poll_events());
            let mut gate = Gate {
                active: self.active.as_ref(),
                host_sent: &mut self.host_sent,
            };
            events.extend(admit_all(entry, raw, &mut gate));
        }
        events
    }
}
