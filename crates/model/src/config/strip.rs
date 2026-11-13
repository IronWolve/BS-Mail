//! The `[strip]` table: the account strip region.

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StripConfig {
    pub show: bool,
    /// Order and visibility of the segments.
    pub segments: Vec<String>,
}

impl Default for StripConfig {
    fn default() -> Self {
        Self {
            show: true,
            segments: string_list(&["account", "mailbox", "host", "connection"]),
        }
    }
}
