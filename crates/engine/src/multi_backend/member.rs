//! One account that the composite serves: who it is and the backend that
//! serves it.

use model::{AccountId, MailBackend};

/// A member of a [`MultiBackend`](super::MultiBackend). The backend must send
/// events that carry its own account id, which is `id`.
pub struct Member {
    pub id: AccountId,
    /// The account's display name, for the account list.
    pub name: String,
    /// The account's mail address, for the account list.
    pub address: String,
    pub backend: Box<dyn MailBackend>,
}

impl Member {
    pub fn new(
        id: AccountId,
        name: impl Into<String>,
        address: impl Into<String>,
        backend: Box<dyn MailBackend>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            address: address.into(),
            backend,
        }
    }
}

#[cfg(test)]
include!("member.tests.rs");
