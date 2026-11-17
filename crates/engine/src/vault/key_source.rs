//! Where the key that opens the vault comes from. Only the key file exists in
//! this version; the system keychain and a typed passphrase are the modes the
//! file format already reserves, and each is one more variant here.

use std::fmt;
use std::path::{Path, PathBuf};

use super::error::VaultError;
use super::format::Mode;
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

/// How the key is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// A random key in this file.
    KeyFile(PathBuf),
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
        }
    }

    /// The key of a vault that exists, which must already be there.
    pub(super) fn existing_key(&self) -> Result<MasterKey, VaultError> {
        match self {
            Self::KeyFile(path) => key_file::read(path)?.ok_or(VaultError::KeyMissing),
        }
    }

    /// The key to save a vault with, made first when this is the first save.
    pub(super) fn key_for_saving(&self) -> Result<MasterKey, VaultError> {
        match self {
            Self::KeyFile(path) => key_file::read_or_create(path),
        }
    }
}

#[cfg(test)]
include!("key_source.tests.rs");
