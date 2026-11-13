//! The `[reader]` table: the reading pane region.

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReaderPosition {
    #[default]
    Below,
    Right,
    Hidden,
}

/// Which body part to show when a message has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyPreference {
    #[default]
    Plain,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReaderConfig {
    pub position: ReaderPosition,
    /// Order and visibility of the header lines.
    pub headers: Vec<String>,
    pub date_format: String,
    /// 0 wraps at the pane width, else at this column.
    pub wrap: u16,
    pub prefer: BodyPreference,
    pub highlight_body: bool,
    pub show_counter: bool,
    /// Delay before an opened message is marked read.
    pub mark_read_after_ms: u32,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            position: ReaderPosition::Below,
            headers: string_list(&["from", "to", "cc", "date", "tags"]),
            date_format: "%a, %b %d, %Y %H:%M:%S %z".to_string(),
            wrap: 0,
            prefer: BodyPreference::Plain,
            highlight_body: true,
            show_counter: true,
            mark_read_after_ms: 1500,
        }
    }
}
