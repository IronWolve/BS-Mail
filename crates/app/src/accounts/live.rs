//! The accounts of the run in progress: where they are kept and the handle
//! that changes which ones are running.

use engine::MembersHandle;
use model::AccountId;
use ui::AccountSummary;

use crate::oauth_clients::ClientStore;

use super::place::Place;
use super::summary::summaries_in_order;

/// What the screen needs to add, change and remove accounts while the program
/// runs. Cheap to copy; copies share the one running list.
#[derive(Clone)]
pub struct Live {
    pub place: Place,
    pub members: MembersHandle,
}

impl Live {
    pub fn new(place: Place, members: MembersHandle) -> Self {
        Self { place, members }
    }

    /// The Options page's list: every account on disk, in the order the
    /// running accounts have.
    pub fn summaries(&self) -> Vec<AccountSummary> {
        let scan = self.place.scan();
        summaries_in_order(&scan.found, &self.members.ids())
    }

    /// The client ids this run's data folder holds for all accounts.
    pub fn clients(&self) -> ClientStore {
        ClientStore::in_folder(&self.place.data_folder())
    }

    /// The ids the running accounts hold, as text.
    pub fn ids_in_use(&self) -> Vec<String> {
        self.members.ids().into_iter().map(|AccountId(id)| id).collect()
    }
}

#[cfg(test)]
include!("live.tests.rs");
