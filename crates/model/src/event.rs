//! Events the mail backend sends to the UI.

use crate::body::Body;
use crate::command::RawPart;
use crate::envelope::Envelope;
use crate::flags::Flags;
use crate::folder::{Account, Folder};
use crate::ids::{AccountId, FolderId, MessageId, RequestId};
use crate::sync_status::{ConnectionState, Quota, SyncStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// All configured accounts, in the order the sidebar lists them.
    AccountList(Vec<Account>),
    /// The unread count of an account's Inbox changed, for the account list
    /// and the notice of new mail. Sent for every account, shown or not.
    AccountUnread {
        account: AccountId,
        unread: u32,
    },
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
    /// The raw text of a message, asked for with `Command::FetchRaw`: control
    /// characters are already taken out and the text is cut at the ceiling a
    /// body has, with a last line that says so.
    RawText {
        message: MessageId,
        part: RawPart,
        text: String,
    },
    /// The safe copy of a message's HTML part, asked for with
    /// `Command::PrepareHtml`, and how many items that load from the network
    /// (pictures, style sheets, imports) were left out of it.
    HtmlPage {
        message: MessageId,
        html: String,
        removed: u32,
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
    /// backend that never says so is taken to allow it. Sent for every
    /// account, shown or not: the composer offers only the accounts that may
    /// send.
    ChangesAllowed {
        account: AccountId,
        allowed: bool,
    },
    /// The text the composer puts under the cursor of a new message from an
    /// account, as the account's file says; empty for no signature. Sent for
    /// every account, shown or not, because the composer can write from any
    /// account that may send.
    AccountSignature {
        account: AccountId,
        text: String,
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
