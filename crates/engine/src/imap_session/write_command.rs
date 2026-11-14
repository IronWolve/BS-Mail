//! The text of every command that can change a mailbox, built in one place.
//!
//! There are five, all on an explicit `UidSet` of the open mailbox and none
//! with a sequence number, `*` or a range:
//!
//! - `UID STORE <uids> +FLAGS|-FLAGS (<flags>)` for the five flags a person
//!   can change. `\Deleted` is not one of them.
//! - `UID STORE <uids> +FLAGS (\Deleted)`, used only as a step inside a move
//!   by copy or a permanent delete, always followed by `UID EXPUNGE`.
//! - `UID COPY <uids> <mailbox>` and `UID MOVE <uids> <mailbox>`.
//! - `UID EXPUNGE <uids>`, which removes only the named deleted messages.
//!
//! A plain `EXPUNGE`, `CLOSE`, `CREATE`, `DELETE`, `RENAME` or `SUBSCRIBE` is
//! not here and is never built anywhere.
//!
//! There is one command that adds mail, and it has one shape:
//! `APPEND "<sent folder>" (\Seen) {n}` followed by the literal, to save a
//! copy of a message that was just sent. Its target can only be a
//! `SentFolder`, which exists only for a folder whose role is Sent; its flag
//! is always `\Seen`; it has no date and no other flags.

use model::{Folder, FolderRole};

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

#[cfg(test)]
include!("write_command.tests.rs");
