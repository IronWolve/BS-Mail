//! The account list the screen gets: built here from the members.

use model::{Account, Event};

use super::composite::MultiBackend;

impl MultiBackend {
    /// One account per member, in member order, with the latest connection
    /// state seen from it and the active one marked selected.
    pub(super) fn account_list(&self) -> Event {
        let accounts = self
            .entries
            .iter()
            .map(|entry| Account {
                id: entry.id.clone(),
                name: entry.name.clone(),
                address: entry.address.clone(),
                state: entry.cache.connection_state(),
                selected: self.active.as_ref() == Some(&entry.id),
            })
            .collect();
        Event::AccountList(accounts)
    }
}
