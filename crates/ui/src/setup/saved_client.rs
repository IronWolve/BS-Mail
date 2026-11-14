//! A client id the program already holds for a provider, so that a second
//! account with the same provider does not ask for it again.
//!
//! The program reads them from its own file and hands them over. The secret
//! is never part of any debug output.

use std::fmt;

/// What a provider's sign-in needs from the program itself.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct SavedClient {
    /// The provider's name as the account file spells it: `google`, `microsoft`.
    pub provider: String,
    pub client_id: String,
    /// Empty when the provider gave none.
    pub client_secret: String,
}

impl fmt::Debug for SavedClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SavedClient")
            .field("provider", &self.provider)
            .field("client_secret", &"<hidden>")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
include!("saved_client.tests.rs");
