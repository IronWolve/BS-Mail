//! The exact UIDs a changing command acts on.
//!
//! A `UidSet` can only be built from a list of explicit, non-zero UIDs, and
//! it is never empty: an empty list gives `None`, so there is nothing to
//! send. It prints as a comma list, never with `*` or a range, so a changing
//! command cannot reach a message that was not named.

use std::fmt;

/// The most UIDs one command names; longer lists go in several commands.
pub const MAX_PER_COMMAND: usize = 100;

/// Explicit message UIDs: sorted, without repeats, never empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UidSet(Vec<u32>);

impl UidSet {
    /// The set of these UIDs. UID 0 does not exist and is dropped; no UIDs
    /// left gives `None`.
    pub fn new(uids: impl IntoIterator<Item = u32>) -> Option<Self> {
        let mut list: Vec<u32> = uids.into_iter().filter(|uid| *uid != 0).collect();
        list.sort_unstable();
        list.dedup();
        (!list.is_empty()).then_some(Self(list))
    }

    pub fn contains(&self, uid: u32) -> bool {
        self.0.binary_search(&uid).is_ok()
    }

    pub fn uids(&self) -> &[u32] {
        &self.0
    }
}

impl fmt::Display for UidSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (at, uid) in self.0.iter().enumerate() {
            if at > 0 {
                f.write_str(",")?;
            }
            write!(f, "{uid}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
include!("uid_set.tests.rs");
