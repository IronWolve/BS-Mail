//! The `[footer]` table: the status footer row.

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FooterConfig {
    pub show: bool,
    pub slashes: bool,
    /// Order and visibility of the segments.
    pub segments: Vec<String>,
}

impl Default for FooterConfig {
    fn default() -> Self {
        Self {
            show: true,
            slashes: true,
            segments: string_list(&[
                "name_version",
                "connection",
                "unread",
                "total",
                "last_sync",
                "help_hint",
            ]),
        }
    }
}
