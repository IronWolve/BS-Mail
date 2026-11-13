//! Identifier newtypes shared by every crate.

/// Short stable account id; also a config key, a path part and a database key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(pub String);

/// Database id of a folder; the virtual folders get ids from the same space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FolderId(pub i64);

/// Database id of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageId(pub i64);

/// Server-side message number inside one folder (valid for one uid validity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uid(pub u32);

/// Id of a conversation thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreadId(pub i64);

/// Correlates a command with the event that answers it (searches).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(pub u64);
