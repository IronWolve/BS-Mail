//! The operating system's own keychain, through the crate the engine manifest names for it: the
//! Credential Manager on Windows and the Secret Service (the desktop's
//! keyring, over the session bus) on Linux. Nothing for macOS is built.
//!
//! The key is kept as 64 hex characters of text under the service name the
//! program was started with and the entry name `vault-key-<key id>`: every
//! store keeps text, and a store that is given a binary value may refuse or
//! mangle it. The hex text is overwritten after use (what the operating
//! system and the crate hold inside themselves is theirs, and is not).
//!
//! Failures are told apart as `KeychainFailure` says. Every `PlatformFailure`
//! and `NoStorageAccess` is `Unavailable`: on Linux that is no session bus, no
//! service, a locked or refused collection, and on Windows a credential store
//! that did not answer. What is left is `Other`, in fixed words.
//!
//! The real keychain is never touched by a test of this program: the classes
//! of failure are tested on constructed errors, and the whole path by the fake.

use keyring::Entry;

use super::failure::KeychainFailure;
use super::hex;
use super::key_id::PROBE_ACCOUNT;
use super::store::KeychainStore;
use super::stored_key::StoredKey;

pub(super) struct SystemKeychain {
    service: String,
}

impl SystemKeychain {
    pub fn new(service: &str) -> Self {
        Self { service: service.to_string() }
    }

    fn entry(&self, account: &str) -> Result<Entry, KeychainFailure> {
        Entry::new(&self.service, account).map_err(classify)
    }
}

/// What a failure of that crate means for this program.
pub(super) fn classify(error: keyring::Error) -> KeychainFailure {
    use keyring::Error;
    match error {
        Error::NoEntry => KeychainFailure::NotFound,
        Error::PlatformFailure(_) | Error::NoStorageAccess(_) => KeychainFailure::Unavailable,
        Error::BadEncoding(mut bytes) => {
            model::wipe_bytes(&mut bytes);
            KeychainFailure::Other("the entry is not text")
        }
        Error::TooLong(..) | Error::Invalid(..) => KeychainFailure::Other("the entry's name was refused"),
        Error::Ambiguous(_) => KeychainFailure::Other("more than one entry has that name"),
        _ => KeychainFailure::Other("an error this program does not know"),
    }
}

/// The key `text` spells, or the failure that says it is not one. The text is
/// overwritten.
pub(super) fn key_of_text(text: String) -> Result<StoredKey, KeychainFailure> {
    let mut raw = text.into_bytes();
    let spelled = std::str::from_utf8(&raw).ok().and_then(hex::decode);
    model::wipe_bytes(&mut raw);
    let mut bytes = spelled.ok_or(KeychainFailure::Other("the key kept there is not in the form this program writes"))?;
    let key = StoredKey::from_slice(&bytes);
    model::wipe_bytes(&mut bytes);
    key.ok_or(KeychainFailure::Other("the key kept there is not in the form this program writes"))
}

impl KeychainStore for SystemKeychain {
    fn get(&self, account: &str) -> Result<StoredKey, KeychainFailure> {
        let text = self.entry(account)?.get_password().map_err(classify)?;
        key_of_text(text)
    }

    fn set(&self, account: &str, key: &StoredKey) -> Result<(), KeychainFailure> {
        let entry = self.entry(account)?;
        let text = hex::encode(key.bytes());
        let done = entry.set_password(&text);
        let mut raw = text.into_bytes();
        model::wipe_bytes(&mut raw);
        done.map_err(classify)
    }

    fn delete(&self, account: &str) -> Result<(), KeychainFailure> {
        match self.entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(classify(error)),
        }
    }

    fn probe(&self) -> Result<(), KeychainFailure> {
        match self.entry(PROBE_ACCOUNT)?.get_password() {
            Ok(text) => {
                // Nobody writes this entry; if there is one, what it holds is not ours to keep.
                let mut raw = text.into_bytes();
                model::wipe_bytes(&mut raw);
                Ok(())
            }
            // The keychain answered, which is all a probe asks.
            Err(keyring::Error::NoEntry | keyring::Error::BadEncoding(_)) => Ok(()),
            Err(error) => Err(classify(error)),
        }
    }
}

#[cfg(test)]
include!("real.tests.rs");
