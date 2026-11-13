//! Connection state, sync state, quota and the status block the sidebar shows.

/// State of one account's server connection (`IMAP: Online` in the strip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    Online,
    Connecting,
    #[default]
    Offline,
}

/// What the sync machinery is doing (`Sync:` line in the status panel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncState {
    #[default]
    Idle,
    Syncing,
    Failed,
}

/// Mailbox storage on the server, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quota {
    pub used: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncStatus {
    /// UTC seconds since the Unix epoch; `None` before the first sync.
    pub last_sync: Option<i64>,
    pub new_messages: u32,
    pub state: SyncState,
    /// Operations waiting to be replayed on the server.
    pub queue_len: u32,
    pub quota: Option<Quota>,
}
