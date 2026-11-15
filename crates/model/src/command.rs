//! Commands the UI sends to the mail backend.

use std::fmt;

use zeroize::Zeroize;

use crate::address::Address;
use crate::flags::FlagChange;
use crate::ids::{AccountId, FolderId, MessageId, RequestId};

/// A secret string that never shows up in debug output or logs, and whose
/// bytes are overwritten with zeros when it is dropped.
///
/// Only this copy is wiped. A `&str` from `expose` that the caller copied
/// elsewhere is the caller's to clean up.
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

    /// Overwrites the text with zeros and leaves it empty. The drop does the
    /// same, so this is only for a caller that keeps the value around.
    pub fn wipe(&mut self) {
        wipe_text(&mut self.0);
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.wipe();
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// Overwrites the whole buffer of `text` with zeros, spare room included, and
/// leaves it empty. For text that held a secret and is not a `Secret`, such as
/// the account file's contents once they are parsed.
pub fn wipe_text(text: &mut String) {
    text.zeroize();
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
    /// Load the page of older messages that follows the `have` newest ones
    /// of a folder whose first page is already shown. The answer is an
    /// `OlderEnvelopes` whose `offset` is `have`.
    LoadMore { folder: FolderId, have: u32 },
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

#[cfg(test)]
include!("command.tests.rs");
