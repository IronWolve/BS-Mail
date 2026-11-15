//! Events the mail backend sends to the UI.

use crate::body::Body;
use crate::envelope::Envelope;
use crate::flags::Flags;
use crate::folder::{Account, Folder};
use crate::ids::{AccountId, FolderId, MessageId, RequestId};
use crate::sync_status::{ConnectionState, Quota, SyncStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// All configured accounts, in the order the sidebar lists them.
    AccountList(Vec<Account>),
    /// The machine the engine runs on, for the account strip's host segment.
    HostInfo {
        /// Short host name, without the domain.
        host_name: String,
        /// Operating system and kernel, as one line.
        system: String,
    },
    /// The full folder tree of one account.
    FolderList {
        account: AccountId,
        folders: Vec<Folder>,
    },
    /// One page of a folder's envelopes, newest first.
    EnvelopePage {
        folder: FolderId,
        /// Position of the first envelope in the folder's full ordering.
        offset: u32,
        /// Messages in the folder.
        total: u32,
        envelopes: Vec<Envelope>,
    },
    /// A page of older messages for a folder whose first page is already
    /// listed, to go after the ones listed. `offset` is where it starts in
    /// the folder's full ordering. Asked for with `Command::LoadMore`.
    OlderEnvelopes {
        folder: FolderId,
        offset: u32,
        /// Messages in the folder.
        total: u32,
        envelopes: Vec<Envelope>,
    },
    BodyReady {
        message: MessageId,
        body: Body,
    },
    FlagsChanged {
        updates: Vec<(MessageId, Flags)>,
    },
    ConnectionState {
        account: AccountId,
        state: ConnectionState,
    },
    SyncProgress {
        account: AccountId,
        status: SyncStatus,
    },
    Quota {
        account: AccountId,
        quota: Option<Quota>,
    },
    SearchResults {
        request: RequestId,
        total: u32,
        envelopes: Vec<Envelope>,
        /// What the search could not cover, in a sentence, when it did not
        /// cover every folder it set out to (time ran out, a server refused
        /// one): the list may then be missing messages. `None`: all of it
        /// ran.
        incomplete: Option<String>,
    },
    /// Whether the account lets the program change mail on the server. An
    /// account that does not shows a note instead of trying and failing. A
    /// backend that never says so is taken to allow it.
    ChangesAllowed {
        account: AccountId,
        allowed: bool,
    },
    /// A message the user sent has been accepted by the server for delivery.
    MessageSent {
        account: AccountId,
    },
    /// An operation failed; the footer shows `reason`.
    OpFailed {
        account: Option<AccountId>,
        operation: String,
        reason: String,
    },
}
