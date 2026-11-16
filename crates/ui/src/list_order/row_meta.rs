//! What each row of the list is: a message on its own, the row that stands
//! for a thread of several messages, or one message of an open thread.

use model::MessageId;

/// The part a row plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// One message, in no thread (or threads are off).
    Single,
    /// The row of a thread: the newest message, with the count and the marks
    /// of the whole thread. `open` is whether its messages are listed below.
    Head { open: bool },
    /// One message of the open thread above it, indented.
    Child,
}

/// One row's kind, and the thread it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowMeta {
    pub kind: RowKind,
    /// How many messages the thread has; 1 for a message on its own.
    pub count: usize,
    /// The thread in the page's `threads`; meaningless for a single message.
    pub thread: usize,
}

impl RowMeta {
    pub const SINGLE: RowMeta = RowMeta { kind: RowKind::Single, count: 1, thread: 0 };
}

/// The messages of one thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadInfo {
    /// Every loaded message of the thread, the oldest first.
    pub members: Vec<MessageId>,
    /// The message whose row stands for the thread: the newest.
    pub head: MessageId,
}
