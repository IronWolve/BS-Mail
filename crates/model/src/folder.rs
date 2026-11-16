//! Accounts and the folder tree the sidebar shows.

use crate::ids::{AccountId, FolderId};
use crate::sync_status::ConnectionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub address: String,
    pub state: ConnectionState,
    /// The unread count of the account's Inbox, once it is known: `None` until
    /// the first look at the server has answered.
    pub unread: Option<u32>,
    /// The account whose mailbox the UI shows first. The backend names one at
    /// start-up; after that the UI owns the live selection.
    pub selected: bool,
}

/// What a folder is for; drives its icon and its place in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FolderRole {
    Inbox,
    /// Virtual: a saved query for the flagged flag, not a server mailbox.
    Starred,
    Sent,
    Drafts,
    Archive,
    /// A query for the alert marker unless the account has a real folder.
    Alerts,
    Spam,
    Trash,
    #[default]
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: FolderId,
    pub account: AccountId,
    /// Full server path, with the server's delimiter.
    pub path: String,
    /// Last path segment, as shown.
    pub name: String,
    pub parent: Option<FolderId>,
    pub role: FolderRole,
    pub unread: u32,
    pub total: u32,
    pub expanded: bool,
}
