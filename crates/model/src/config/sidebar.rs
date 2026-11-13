//! The `[sidebar]` table: the folders, accounts and status region.

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SidebarConfig {
    pub show: bool,
    /// Order and visibility of the sections.
    pub sections: Vec<String>,
    /// Order and visibility of the status panel's lines.
    pub status_lines: Vec<String>,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            show: true,
            sections: string_list(&["folders", "accounts", "status"]),
            status_lines: string_list(&[
                "last_sync",
                "new_messages",
                "sync_state",
                "storage",
                "gauge",
            ]),
        }
    }
}
