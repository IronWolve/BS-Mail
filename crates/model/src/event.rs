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
        /// Short host name, such as `hydra`.
        host_name: String,
        /// Operating system and kernel, such as `Linux 6.8.12`.
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
    },
    /// An operation failed; the footer shows `reason`.
    OpFailed {
        account: Option<AccountId>,
        operation: String,
        reason: String,
    },
}
