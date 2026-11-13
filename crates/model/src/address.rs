//! One mail address with an optional display name.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Address {
    /// Display name, when the header carried one.
    pub name: Option<String>,
    /// The bare address, such as `user@example.org`.
    pub email: String,
}

impl Address {
    /// An address with no display name.
    pub fn bare(email: impl Into<String>) -> Self {
        Self {
            name: None,
            email: email.into(),
        }
    }

    /// An address with a display name.
    pub fn named(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: email.into(),
        }
    }
}
