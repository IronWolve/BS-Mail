//! The `[list]` table and `[list.widths]`: the message list region.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::defaults::string_list;

/// How the From column shows an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FromStyle {
    #[default]
    NameAddress,
    Name,
    Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateStyle {
    #[default]
    Relative,
    Fixed,
}

/// How the Size column shows a byte count: `42K`, `42 KiB` or plain bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeStyle {
    #[default]
    Short,
    Iec,
    Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SortSetting {
    pub column: String,
    pub descending: bool,
}

impl Default for SortSetting {
    fn default() -> Self {
        Self {
            column: "date".to_string(),
            descending: true,
        }
    }
}

/// Default column widths in cells; the flex column has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListWidths {
    pub num: u16,
    pub flag: u16,
    pub star: u16,
    pub from: u16,
    pub date: u16,
    pub size: u16,
}

impl Default for ListWidths {
    fn default() -> Self {
        Self {
            num: 3,
            flag: 3,
            star: 3,
            from: 33,
            date: 15,
            size: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListConfig {
    /// Order and visibility of the columns.
    pub columns: Vec<String>,
    /// The column that takes the leftover width.
    pub flex_column: String,
    pub from_format: FromStyle,
    pub date_format: DateStyle,
    pub date_fixed: String,
    pub date_today: String,
    pub date_yesterday: String,
    pub date_older: String,
    /// Used when the date column is narrowed.
    pub date_short: String,
    pub size_format: SizeStyle,
    pub show_hints: bool,
    /// Conversations: the loaded rows are grouped by the ids in their
    /// headers, a thread shows as one row and opens with Right. The `T` key
    /// turns it on or off for the session. Only loaded mail is grouped, and
    /// the list's title says so.
    pub threaded: bool,
    // Tables come last so the settings serialize with plain values first.
    /// Subject tag to theme role name.
    pub subject_tags: BTreeMap<String, String>,
    /// The sort of a kind of folder the user has not chosen one for: a column
    /// (`date`, `from`, `subject`, `size` or `flag`) and a direction. The
    /// default, `date` with `descending`, is the folder's own order, newest
    /// arrival first.
    pub default_sort: SortSetting,
    pub widths: ListWidths,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            columns: string_list(&["num", "flag", "star", "from", "subject", "date", "size"]),
            flex_column: "subject".to_string(),
            from_format: FromStyle::NameAddress,
            date_format: DateStyle::Relative,
            date_fixed: "%y-%m-%d %H:%M".to_string(),
            date_today: "Today %H:%M".to_string(),
            date_yesterday: "Yesterday %H:%M".to_string(),
            date_older: "%b %d %H:%M".to_string(),
            date_short: "%m-%d %H:%M".to_string(),
            size_format: SizeStyle::Short,
            show_hints: true,
            threaded: false,
            subject_tags: default_subject_tags(),
            default_sort: SortSetting::default(),
            widths: ListWidths::default(),
        }
    }
}

fn default_subject_tags() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("Project".to_string(), "subject.tag.project".to_string()),
        ("PR".to_string(), "subject.tag.default".to_string()),
        ("ALERT".to_string(), "subject.tag.alert".to_string()),
    ])
}

#[cfg(test)]
include!("list.tests.rs");
