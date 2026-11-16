//! How the list is arranged: the sort of each kind of folder, whether threads
//! are on, and which threads are open. Plain data; the page is built from it
//! (see `arrange`) every time the rows or the arrangement change.

use std::collections::BTreeSet;

use model::MessageId;
use model::config::ListConfig;

use super::sort::Sort;
use super::sort_prefs::{RoleKey, SortPrefs};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Arrangement {
    /// What the user chose for each kind of folder.
    pub prefs: SortPrefs,
    /// The sort of a kind of folder nothing was chosen for: the setting
    /// `[list] default_sort`.
    pub default_sort: Option<Sort>,
    /// Whether the loaded rows are grouped into conversations.
    pub threaded: bool,
    /// The messages of the threads the user opened. A thread is open when any
    /// of its messages is here, so a message that joins an open thread shows
    /// in it.
    pub open: BTreeSet<MessageId>,
}

impl Arrangement {
    /// The arrangement the settings ask for.
    pub fn from_config(list: &ListConfig) -> Self {
        let default_sort = Sort::from_setting(&list.default_sort.column, list.default_sort.descending).flatten();
        Self { default_sort, threaded: list.threaded, ..Self::default() }
    }

    /// The sort in force for folders of `role`: the user's choice, else the
    /// setting.
    pub fn sort_for(&self, role: RoleKey) -> Option<Sort> {
        self.prefs.get(role).unwrap_or(self.default_sort)
    }

    /// Whether a thread with these messages is open.
    pub fn is_open(&self, members: &[MessageId]) -> bool {
        !self.open.is_empty() && members.iter().any(|id| self.open.contains(id))
    }
}

#[cfg(test)]
include!("arrangement.tests.rs");
