//! The text of every command that can change a mailbox, built in one place.
//!
//! Six work on an explicit `UidSet` of the open mailbox, none with a sequence
//! number, `*` or a range:
//!
//! - `UID STORE <uids> +FLAGS|-FLAGS (<flags>)` for the five flags a person
//!   can change. `\Deleted` is not one of them.
//! - `UID STORE <uids> +FLAGS|-FLAGS (<tag>)` for one tag, a keyword that
//!   passed `check_tag` (a `Keyword`), and no other flag in the same command.
//! - `UID STORE <uids> +FLAGS (\Deleted)`, used only as a step inside a move
//!   by copy or a permanent delete, always followed by `UID EXPUNGE`.
//! - `UID COPY <uids> <mailbox>` and `UID MOVE <uids> <mailbox>`.
//! - `UID EXPUNGE <uids>`, which removes only the named deleted messages.
//!
//! Three work on a mailbox as a whole, for the folder actions only: `CREATE`,
//! `RENAME` and `DELETE`, each with its names already in the server's
//! encoding (printable ASCII; see `mutf7`) and quoted. The caller has checked
//! the name, and has seen to it that the mailbox is not the one open.
//!
//! A plain `EXPUNGE`, `CLOSE` or `SUBSCRIBE` is not here and is never built
//! anywhere.
//!
//! There is one command that adds mail, and it has one shape:
//! `APPEND "<sent folder>" (\Seen) {n}` followed by the literal, to save a
//! copy of a message that was just sent. Its target can only be a
//! `SentFolder`, which exists only for a folder whose role is Sent; its flag
//! is always `\Seen`; it has no date and no other flags.

use model::{Folder, FolderRole, check_tag};

use super::error::SessionError;
use super::uid_set::UidSet;

/// The folder a copy of a sent message may be appended to. The only way to
/// make one is from a folder whose role is Sent, so no other mailbox can be
/// the target of an APPEND.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentFolder {
    path: String,
}

impl SentFolder {
    /// The target for `folder`, or `None` when it is not the Sent folder.
    pub fn from_folder(folder: &Folder) -> Option<Self> {
        (folder.role == FolderRole::Sent).then(|| Self { path: folder.path.clone() })
    }

    /// The server's name for the folder.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// The flags a person can set or clear on a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreFlag {
    Seen,
    Flagged,
    Answered,
    Draft,
    /// The `$Important` keyword behind the alert marker.
    Important,
}

impl StoreFlag {
    /// The flag as the server spells it.
    pub const fn atom(self) -> &'static str {
        match self {
            Self::Seen => "\\Seen",
            Self::Flagged => "\\Flagged",
            Self::Answered => "\\Answered",
            Self::Draft => "\\Draft",
            Self::Important => "$Important",
        }
    }
}

/// A tag as the server takes it: made only from a name that passed
/// `check_tag`, so it is one plain word that cannot end the command's list or
/// start another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyword(String);

impl Keyword {
    /// The keyword for `text`, or the sentence that says why it cannot be one.
    pub fn new(text: &str) -> Result<Self, SessionError> {
        check_tag(text).map(Self).map_err(SessionError::Refused)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a flag change adds flags or removes them. Never replaces them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    Add,
    Remove,
}

impl StoreMode {
    const fn sign(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Remove => "-",
        }
    }
}

/// What a command needs from the server beyond the base protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Needs {
    Nothing,
    Move,
    UidPlus,
}

/// One changing command, ready to be written.
pub(super) enum WriteCommand<'a> {
    Store {
        set: &'a UidSet,
        mode: StoreMode,
        flags: &'a [StoreFlag],
    },
    /// One tag, alone, on exactly these messages.
    StoreKeyword {
        set: &'a UidSet,
        mode: StoreMode,
        keyword: &'a Keyword,
    },
    /// A new mailbox; `path` is in the server's encoding.
    CreateMailbox { path: &'a str },
    RenameMailbox { from: &'a str, to: &'a str },
    DeleteMailbox { path: &'a str },
    /// `\Deleted` on exactly these messages; only as a step before expunging.
    MarkDeleted { set: &'a UidSet },
    Copy { set: &'a UidSet, to: &'a str },
    Move { set: &'a UidSet, to: &'a str },
    Expunge { set: &'a UidSet },
    /// Saves a sent message: the line only, the literal follows separately.
    Append { target: &'a SentFolder, size: usize },
}

impl WriteCommand<'_> {
    /// The command line without its tag.
    pub(super) fn line(&self) -> Result<String, SessionError> {
        match self {
            Self::Store { set, mode, flags } => {
                if flags.is_empty() {
                    return Err(SessionError::Refused("no flags to change".to_owned()));
                }
                let atoms: Vec<&str> = flags.iter().map(|flag| flag.atom()).collect();
                Ok(format!("UID STORE {set} {}FLAGS ({})", mode.sign(), atoms.join(" ")))
            }
            Self::StoreKeyword { set, mode, keyword } => {
                Ok(format!("UID STORE {set} {}FLAGS ({})", mode.sign(), keyword.as_str()))
            }
            Self::CreateMailbox { path } => Ok(format!("CREATE {}", quote_encoded(path)?)),
            Self::RenameMailbox { from, to } => Ok(format!("RENAME {} {}", quote_encoded(from)?, quote_encoded(to)?)),
            Self::DeleteMailbox { path } => Ok(format!("DELETE {}", quote_encoded(path)?)),
            Self::MarkDeleted { set } => Ok(format!("UID STORE {set} +FLAGS (\\Deleted)")),
            Self::Copy { set, to } => Ok(format!("UID COPY {set} {}", quote_mailbox(to)?)),
            Self::Move { set, to } => Ok(format!("UID MOVE {set} {}", quote_mailbox(to)?)),
            Self::Expunge { set } => Ok(format!("UID EXPUNGE {set}")),
            Self::Append { target, size } => {
                if *size == 0 {
                    return Err(SessionError::Refused("the copy to save is empty".to_owned()));
                }
                Ok(format!("APPEND {} (\\Seen) {{{size}}}", quote_mailbox(target.path())?))
            }
        }
    }

    /// True for a command that acts on a mailbox as a whole and so needs no
    /// mailbox open for changes.
    pub(super) fn is_about_a_mailbox(&self) -> bool {
        matches!(self, Self::CreateMailbox { .. } | Self::RenameMailbox { .. } | Self::DeleteMailbox { .. })
    }

    /// True for a command that is followed by a literal, which `send_change`
    /// cannot carry.
    pub(super) fn has_literal(&self) -> bool {
        matches!(self, Self::Append { .. })
    }

    /// The extension the server must offer for this command to be safe.
    pub(super) fn needs(&self) -> Needs {
        match self {
            Self::Move { .. } => Needs::Move,
            Self::Expunge { .. } => Needs::UidPlus,
            _ => Needs::Nothing,
        }
    }
}

/// A mailbox name as a quoted string. Names with line breaks or other
/// control characters are refused rather than sent.
fn quote_mailbox(name: &str) -> Result<String, SessionError> {
    if name.is_empty() || name.chars().any(char::is_control) {
        return Err(SessionError::Refused(
            "the destination folder has a name that cannot be sent".to_owned(),
        ));
    }
    Ok(format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\"")))
}

/// A mailbox name already in the server's encoding, as a quoted string: only
/// printable ASCII goes out, so a name that was not encoded is refused rather
/// than sent as bytes the server would read as something else.
fn quote_encoded(name: &str) -> Result<String, SessionError> {
    if !name.chars().all(|ch| ch.is_ascii_graphic() || ch == ' ') {
        return Err(SessionError::Refused("the folder has a name that cannot be sent".to_owned()));
    }
    quote_mailbox(name)
}

#[cfg(test)]
include!("write_command.tests.rs");
