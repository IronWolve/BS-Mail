//! Where the key that opens the vault comes from: a random key in a file
//! beside the vault, the key made from a typed passphrase and held in memory,
//! or a random key read from the system keychain (by the key id the vault's
//! header names) and held in memory.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::VaultError;
use super::format::{Binding, Header, KdfHeader, Mode};
use super::key_file::{self, KEY_LEN};
use super::keychain::KeyId;

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

/// The random key read from the system keychain for a vault, with the key id
/// it was kept under. Held in memory for as long as the vault stays open and
/// wiped when the last copy goes. A copy shares the one key.
#[derive(Clone)]
pub struct KeychainKey {
    key: Arc<MasterKey>,
    id: KeyId,
}

impl KeychainKey {
    pub(super) fn new(key: MasterKey, id: KeyId) -> Self {
        Self { key: Arc::new(key), id }
    }

    pub(super) fn id(&self) -> KeyId {
        self.id
    }
}

impl fmt::Debug for KeychainKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KeychainKey(<redacted>)")
    }
}

impl PartialEq for KeychainKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.key.bytes() == other.key.bytes()
    }
}

impl Eq for KeychainKey {}

/// How the key is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// A random key in this file.
    KeyFile(PathBuf),
    /// The key made from the passphrase, already computed.
    Passphrase(Unlocked),
    /// The random key the system keychain gave, already read.
    Keychain(KeychainKey),
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
            Self::Keychain(_) => Mode::Keychain,
        }
    }

    /// What the header of a vault saved with this key says.
    pub(super) fn binding(&self) -> Binding {
        match self {
            Self::KeyFile(_) => Binding::KeyFile,
            Self::Passphrase(unlocked) => Binding::Passphrase(*unlocked.kdf()),
            Self::Keychain(held) => Binding::Keychain(held.id),
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
            (Self::Keychain(held), Mode::Keychain) if header.key_id == Some(held.id) => Ok(Arc::clone(&held.key)),
            // The key held is for another vault's id: this vault's key is not what was read.
            (Self::Keychain(_), Mode::Keychain) => Err(VaultError::KeychainKeyMissing),
            // A key for one way of protecting meeting a vault protected another way.
            (Self::Passphrase(_) | Self::Keychain(_), _) | (Self::KeyFile(_), Mode::Keychain) => Err(VaultError::WrongMode),
        }
    }

    /// The key to save a vault with, made first when this is the first save
    /// of a key-file vault.
    pub(super) fn key_for_saving(&self) -> Result<Arc<MasterKey>, VaultError> {
        match self {
            Self::KeyFile(path) => key_file::read_or_create(path).map(Arc::new),
            Self::Passphrase(unlocked) => Ok(Arc::clone(&unlocked.key)),
            Self::Keychain(held) => Ok(Arc::clone(&held.key)),
        }
    }
}

#[cfg(test)]
include!("key_source.tests.rs");
