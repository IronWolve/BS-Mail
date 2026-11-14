//! Commands the UI sends to the mail backend.

use std::fmt;

use crate::address::Address;
use crate::flags::FlagChange;
use crate::ids::{AccountId, FolderId, MessageId, RequestId};

/// A secret string that never shows up in debug output or logs.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The secret text; call only where it is actually used.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// A message ready to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMessage {
    pub account: AccountId,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: String,
    pub body: String,
    /// The message this one answers, when it is a reply.
    pub in_reply_to: Option<MessageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Load the newest page of a folder's envelopes.
    OpenFolder { folder: FolderId },
    /// Load and render one message body.
    FetchBody { message: MessageId },
    /// Change flags on one or more messages.
    SetFlags {
        messages: Vec<MessageId>,
        change: FlagChange,
    },
    Move {
        messages: Vec<MessageId>,
        to: FolderId,
    },
    Delete { messages: Vec<MessageId> },
    Send { message: OutgoingMessage },
    Search {
        request: RequestId,
        account: AccountId,
        query: String,
    },
    /// Sync one account now.
    SyncNow { account: AccountId },
    /// The user switched to this account. A backend that serves several
    /// accounts sends again what the screen needs about it (folders,
    /// connection, sync status, quota, whether changes are allowed); one that
    /// serves a single account has nothing to add.
    SelectAccount { account: AccountId },
    /// Open the credential vault.
    Unlock { passphrase: Secret },
}
