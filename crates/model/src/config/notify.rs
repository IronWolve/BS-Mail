//! The `[notify]` table: what the program tells the user on its own.

use serde::{Deserialize, Serialize};

/// A notice inside the program only: no bell, no desktop notification and no
/// sound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotifyConfig {
    /// The footer says "New mail: N in <address>" for a few seconds when an
    /// account's Inbox unread count goes up.
    pub new_mail: bool,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self { new_mail: true }
    }
}
