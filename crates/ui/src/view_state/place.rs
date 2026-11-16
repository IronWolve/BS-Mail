//! Where the user was: the account and the folder open, named the way a file
//! can keep them (the account's id and the folder's path on the server), not
//! by the ids a run hands out.

/// An account and a folder, either of which may be unknown.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    /// The account's id.
    pub account: Option<String>,
    /// The folder's full path.
    pub folder: Option<String>,
}

impl Place {
    /// This place where it is known, else `saved`: a run that never found out
    /// where it was (no connection, no folder list yet) leaves what was
    /// remembered alone. A folder is only kept from `saved` for the same
    /// account, since a path means nothing in another account.
    #[must_use]
    pub fn over(self, saved: &Place) -> Place {
        match (self.account, self.folder) {
            (Some(account), Some(folder)) => Place { account: Some(account), folder: Some(folder) },
            (Some(account), None) => {
                let same = saved.account.as_ref() == Some(&account);
                Place { folder: saved.folder.clone().filter(|_| same), account: Some(account) }
            }
            (None, _) => saved.clone(),
        }
    }
}

#[cfg(test)]
include!("place.tests.rs");
