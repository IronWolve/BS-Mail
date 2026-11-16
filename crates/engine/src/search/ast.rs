//! What a query means, once it is read: a tree of terms joined by and, or
//! and not.

/// A search as a tree. `And` and `Or` hold at least two parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Term(Term),
}

/// One thing a message can be tested for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A bare word or phrase: anywhere in the subject, sender or text.
    Any(String),
    From(String),
    To(String),
    Cc(String),
    Subject(String),
    Body(String),
    /// An IMAP keyword on the message.
    Tag(String),
    Is(Status),
    /// Received before this day.
    Before(Date),
    /// Received on or after this day.
    After(Date),
    /// Bigger than this many bytes.
    Larger(u32),
    /// Smaller than this many bytes.
    Smaller(u32),
    /// Limits which folders are searched; never sent to the server.
    Folder(String),
}

/// The flag states `is:` knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Unread,
    Read,
    Flagged,
    Answered,
    Draft,
    /// The alert marker, the `$Important` keyword.
    Alert,
}

/// A calendar day, already checked to exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// A query split into where to look and what to look for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// Texts from `folder:`; a folder is searched only if its path or name
    /// contains every one of them. Empty means the usual set of folders.
    pub folders: Vec<String>,
    /// What a message must satisfy; `None` means every message.
    pub expr: Option<Expr>,
}
