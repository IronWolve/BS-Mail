//! The `[keybar]` table: the row of keycap buttons.

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBarConfig {
    pub show: bool,
    /// Action names, in order; each shows its first bound key.
    pub buttons: Vec<String>,
}

impl Default for KeyBarConfig {
    fn default() -> Self {
        Self {
            show: true,
            buttons: string_list(&[
                "compose",
                "reply",
                "reply_all",
                "forward",
                "delete",
                "next_unread",
                "search",
                "move",
                "sync",
                "help",
                "quit",
            ]),
        }
    }
}

#[cfg(test)]
include!("key_bar.tests.rs");
