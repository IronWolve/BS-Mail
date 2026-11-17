//! What went wrong in a call to the system keychain, in the three kinds the
//! program acts on differently. A failure never carries a key or any text the
//! keychain gave: its words are fixed here.

use std::fmt;

use super::super::error::VaultError;

/// Why a call to the keychain did not give what was asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeychainFailure {
    /// The keychain works and has no entry under that name: this computer or
    /// user never had the key, or it was deleted.
    NotFound,
    /// There is no keychain to talk to here, or it would not answer: no
    /// session bus, no service, access refused, locked, or no answer in time.
    Unavailable,
    /// Anything else, in fixed words.
    Other(&'static str),
}

impl fmt::Display for KeychainFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("the system keychain has no such entry"),
            Self::Unavailable => f.write_str("there is no system keychain here (or it is locked)"),
            Self::Other(why) => write!(f, "the system keychain gave an error: {why}"),
        }
    }
}

impl From<KeychainFailure> for VaultError {
    /// For opening a vault, where a key that is not there is the lost key.
    fn from(failure: KeychainFailure) -> Self {
        match failure {
            KeychainFailure::NotFound => Self::KeychainKeyMissing,
            KeychainFailure::Unavailable => Self::KeychainUnavailable,
            KeychainFailure::Other(why) => Self::KeychainOther(why),
        }
    }
}

#[cfg(test)]
include!("failure.tests.rs");
