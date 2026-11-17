//! Where the key that opens the vault comes from: a random key in a file
//! beside the vault, or the key made from a typed passphrase and held in
//! memory. The system keychain is the mode the file format reserves for a
//! later version, and is one more variant here when it is built.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::VaultError;
use super::format::{Header, KdfHeader, Mode};
use super::key_file::{self, KEY_LEN};

/// The key file's name inside the vault's folder.
pub const KEY_FILE_NAME: &str = "key";

/// The 32-byte key the vault's own keys are made from. Overwritten when
/// dropped; its debug output shows nothing.
pub(super) struct MasterKey([u8; KEY_LEN]);

impl MasterKey {
    /// `None` unless `bytes` is exactly a key long.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let key: [u8; KEY_LEN] = bytes.try_into().ok()?;
        Some(Self(key))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        model::wipe_bytes(&mut self.0);
    }
}

impl fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MasterKey(<redacted>)")
    }
}

/// The key made from a passphrase, with the key-derivation settings it was
/// made under. It is held in memory for as long as the vault stays unlocked
/// and wiped when the last copy goes. A copy shares the one key.
#[derive(Clone)]
pub struct Unlocked {
    key: Arc<MasterKey>,
    kdf: KdfHeader,
}

impl Unlocked {
    pub(super) fn new(key: MasterKey, kdf: KdfHeader) -> Self {
        Self { key: Arc::new(key), kdf }
    }

    pub(super) fn kdf(&self) -> &KdfHeader {
        &self.kdf
    }
}

impl fmt::Debug for Unlocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Unlocked(<redacted>)")
    }
}

impl PartialEq for Unlocked {
    fn eq(&self, other: &Self) -> bool {
        self.kdf == other.kdf && self.key.bytes() == other.key.bytes()
    }
}

impl Eq for Unlocked {}

/// How the key is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// A random key in this file.
    KeyFile(PathBuf),
    /// The key made from the passphrase, already computed.
    Passphrase(Unlocked),
}

impl KeySource {
    /// The key file in the vault's own folder `dir`.
    pub fn key_file_in(dir: &Path) -> Self {
        Self::KeyFile(dir.join(KEY_FILE_NAME))
    }

    /// The mode byte a vault made with this key carries.
    pub fn mode(&self) -> Mode {
        match self {
            Self::KeyFile(_) => Mode::KeyFile,
            Self::Passphrase(_) => Mode::Passphrase,
        }
    }

    /// The key-derivation settings a vault saved with this key names; `None`
    /// for a key file.
    pub(super) fn kdf(&self) -> Option<&KdfHeader> {
        match self {
            Self::KeyFile(_) => None,
            Self::Passphrase(unlocked) => Some(unlocked.kdf()),
        }
    }

    /// The key of a vault that exists, whose header is `header`: the key file
    /// must be there, and a passphrase key must be the one for this file. A
    /// vault in the other mode is not opened with this key.
    pub(super) fn key_for(&self, header: &Header) -> Result<Arc<MasterKey>, VaultError> {
        match (self, header.mode) {
            (Self::KeyFile(path), Mode::KeyFile) => key_file::read(path)?.map(Arc::new).ok_or(VaultError::KeyMissing),
            // A passphrase vault and no passphrase held: it is locked.
            (Self::KeyFile(_), Mode::Passphrase) => Err(VaultError::Locked),
            (Self::Passphrase(unlocked), Mode::Passphrase) if header.kdf.as_ref() == Some(unlocked.kdf()) => {
                Ok(Arc::clone(&unlocked.key))
            }
            (Self::Passphrase(_), Mode::Passphrase) => Err(VaultError::Locked),
            (Self::Passphrase(_), Mode::KeyFile) => Err(VaultError::WrongMode),
        }
    }

    /// The key to save a vault with, made first when this is the first save
    /// of a key-file vault.
    pub(super) fn key_for_saving(&self) -> Result<Arc<MasterKey>, VaultError> {
        match self {
            Self::KeyFile(path) => key_file::read_or_create(path).map(Arc::new),
            Self::Passphrase(unlocked) => Ok(Arc::clone(&unlocked.key)),
        }
    }
}

#[cfg(test)]
include!("key_source.tests.rs");
