//! Commands the UI sends to the mail backend.

use std::fmt;

use zeroize::Zeroize;

use crate::address::Address;
use crate::attachments::OutgoingAttachment;
use crate::body::AttachmentInfo;
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
    /// Files that go with the message, after its text.
    pub attachments: Vec<OutgoingAttachment>,
}

/// Which raw text of a message to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawPart {
    /// The header block as received, up to the blank line.
    Headers,
    /// The whole message as received: the headers and every part.
    Source,
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
    /// Read the raw text of a message (its headers, or all of it), for the
    /// reader to show as it is. The answer is a `RawText`.
    FetchRaw { message: MessageId, part: RawPart },
    /// Make a safe copy of a message's HTML part, with scripts, frames and
    /// everything that loads from the network left out, for the program to
    /// open in the browser. The answer is an `HtmlPage`; a message with no
    /// HTML part is answered with a failure.
    PrepareHtml { message: MessageId },
    /// Download one attachment of a message and decode it to the file it
    /// holds, for the program to save, open or send on. The attachment is
    /// the one the reader listed (its `section` names it on the server).
    /// The answer is an `AttachmentData`, or a failure named `save attachment`
    /// that says in words why not (it is larger than `MAX_FETCH_BYTES`, the
    /// message is gone, the server lays it out differently).
    FetchAttachment { message: MessageId, attachment: AttachmentInfo },
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
    /// Add a tag (an IMAP keyword) to the messages, or take it off them. The
    /// answer is a `TagsChanged`. A tag the engine cannot send (see
    /// `check_tag`) is refused with a failure.
    SetTag {
        messages: Vec<MessageId>,
        tag: String,
        on: bool,
    },
    /// Make a folder named `name`, inside `parent` or at the top when there is
    /// none. The folder list is sent again once the server has made it.
    CreateFolder {
        account: AccountId,
        parent: Option<FolderId>,
        name: String,
    },
    /// Give a folder another name (it stays where it is in the tree). The
    /// special folders are refused.
    RenameFolder { folder: FolderId, name: String },
    /// Remove a folder and the messages in it for good. The special folders
    /// and a folder that holds other folders are refused.
    DeleteFolder { folder: FolderId },
    /// Mark every unread message of a folder read.
    MarkAllRead { folder: FolderId },
    /// Remove every message of a Trash or Spam folder for good. Any other
    /// folder is refused.
    EmptyFolder { folder: FolderId },
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
