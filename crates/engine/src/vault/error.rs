//! Why a vault could not be opened or saved, as a plain sentence for the
//! footer. No variant ever carries a secret, a key or any text of the vault.

use std::fmt;

/// What went wrong with the vault or its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    /// The vault file or its folder could not be read; holds the system's words.
    Read(String),
    /// The vault is a link or something other than a plain file.
    NotAFile,
    /// The file is larger than the vault may be.
    TooBig,
    /// Too short, wrong magic, or a header that is not one: not a vault.
    Damaged,
    /// A version of the format this program does not know.
    UnknownVersion,
    /// A mode byte this program does not know.
    UnknownMode,
    /// A mode the format has but this version cannot open yet.
    ModeUnavailable,
    /// The vault is there and its key is not.
    KeyMissing,
    /// The key does not open the vault, or the vault was changed.
    KeyMismatch,
    /// The key file cannot be used; holds what is wrong with it.
    BadKeyFile(&'static str),
    /// The decrypted text is not a table of names and texts.
    Contents,
    /// The vault or its key could not be written; holds the system's words.
    Write(String),
    /// The system gave no random numbers.
    Random,
    /// The encryption library refused the key.
    Crypto,
}

impl VaultError {
    /// Whether the vault can be set aside and a fresh one started: the file or
    /// its key is lost, not merely out of reach (an unreadable folder, a key
    /// file of the wrong length or a newer format are never overwritten).
    pub fn can_start_over(&self) -> bool {
        matches!(self, Self::Damaged | Self::KeyMissing | Self::KeyMismatch | Self::Contents)
    }

    /// Whether the error is one of writing the vault (as against opening it),
    /// so a caller that asked for nothing to be written can tell a read that
    /// found nothing usable from a write that failed.
    pub fn while_saving(&self) -> bool {
        matches!(self, Self::Write(_) | Self::Random | Self::Crypto | Self::TooBig)
    }
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const OPEN: &str = "the secrets vault could not be opened";
        match self {
            Self::Read(why) => write!(f, "the secrets vault could not be read: {why}"),
            Self::NotAFile => write!(f, "{OPEN}: the vault is a link or not a plain file"),
            Self::TooBig => write!(f, "{OPEN}: the file is larger than a vault can be"),
            Self::Damaged => write!(f, "{OPEN}: the file is damaged or is not a vault"),
            Self::UnknownVersion => write!(f, "{OPEN}: it was made by a newer version of this program"),
            Self::UnknownMode => write!(f, "{OPEN}: it is protected in a way this program does not know"),
            Self::ModeUnavailable => write!(f, "{OPEN}: that way of protecting it is not available in this version"),
            Self::KeyMissing | Self::KeyMismatch => write!(f, "{OPEN}: the key is missing or does not match"),
            Self::BadKeyFile(why) => write!(f, "{OPEN}: the key file {why}"),
            Self::Contents => write!(f, "{OPEN}: its contents are not in the form this program expects"),
            Self::Write(why) => write!(f, "the secrets vault could not be saved: {why}"),
            Self::Random => write!(f, "the secrets vault could not be saved: the system gave no random numbers"),
            Self::Crypto => write!(f, "the secrets vault could not be used: the encryption library refused the key"),
        }
    }
}

impl std::error::Error for VaultError {}

#[cfg(test)]
include!("error.tests.rs");
