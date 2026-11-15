//! The search that is showing in the list.

use model::RequestId;

/// What was asked for and which answer it is waiting for. An answer to an
/// earlier search, which arrives late, is not this one's and is ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSearch {
    pub query: String,
    pub request: RequestId,
    /// What the answer says it could not cover, when it says so: the list may
    /// then be missing messages.
    pub incomplete: Option<String>,
    /// The query whose hits are the rows now, once an answer has arrived.
    /// `None` while the answer is awaited: the rows are the folder's own then,
    /// and are titled so. A new search over hits keeps the old ones until its
    /// own answer replaces them.
    pub showing: Option<String>,
}
