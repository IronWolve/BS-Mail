//! The shape of the fixture's TOML file, as plain structs. Unknown keys are
//! rejected so a typo is reported instead of silently ignored.

use std::collections::BTreeMap;

use serde::Deserialize;
use toml::value::Datetime;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixtureFile {
    /// The moment the UI treats as the present.
    pub now: Datetime,
    pub host: HostRow,
    pub accounts: Vec<AccountRow>,
    pub folders: Vec<FolderRow>,
    pub status: StatusRow,
    pub messages: Vec<MessageRow>,
    pub open: OpenRow,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostRow {
    pub name: String,
    pub system: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AccountRow {
    pub id: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub selected: bool,
    pub connection: ConnectionText,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConnectionText {
    Online,
    Connecting,
    Offline,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FolderRow {
    pub id: i64,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub parent: Option<i64>,
    pub role: RoleText,
    pub unread: u32,
    pub total: u32,
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RoleText {
    Inbox,
    Starred,
    Sent,
    Drafts,
    Archive,
    Alerts,
    Spam,
    Trash,
    #[serde(alias = "folder")]
    Plain,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StatusRow {
    #[serde(default)]
    pub last_sync: Option<Datetime>,
    pub new_messages: u32,
    pub state: SyncText,
    #[serde(default)]
    pub queue_len: u32,
    #[serde(default)]
    pub quota_used: Option<u64>,
    #[serde(default)]
    pub quota_limit: Option<u64>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncText {
    Idle,
    Syncing,
    Failed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MessageRow {
    pub id: i64,
    pub folder: i64,
    pub uid: u32,
    pub from: String,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    pub subject: String,
    pub date: Datetime,
    /// Bytes.
    pub size: u64,
    #[serde(default)]
    pub unread: bool,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub alert: bool,
    #[serde(default)]
    pub attachments: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenRow {
    pub message: i64,
    /// Lines of spans; see the fixture file's header for the span forms.
    pub body: Vec<Vec<SpanRow>>,
}

/// A bare string is plain text; a one-entry table maps a role to its text.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum SpanRow {
    Plain(String),
    Styled(BTreeMap<String, String>),
}
