//! The vault's master key as the keychain exchanges it: 32 bytes, overwritten
//! when dropped, never shown in debug output. Cloning is not offered; a copy
//! is made on purpose (`copy`) so each one is visible in the code.

use std::fmt;

use super::super::crypto::random;
use super::super::error::VaultError;
use super::super::key_file::KEY_LEN;

pub struct StoredKey([u8; KEY_LEN]);

impl StoredKey {
    pub fn new(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// `None` unless `bytes` is exactly a key long.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let key: [u8; KEY_LEN] = bytes.try_into().ok()?;
        Some(Self(key))
    }

    /// A new random key.
    pub fn random() -> Result<Self, VaultError> {
        random::<KEY_LEN>().map(Self)
    }

    pub fn bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    /// Another copy of the key, for a call that runs on a thread of its own.
    pub fn copy(&self) -> Self {
        Self(self.0)
    }
}

impl Drop for StoredKey {
    fn drop(&mut self) {
        model::wipe_bytes(&mut self.0);
    }
}

impl fmt::Debug for StoredKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StoredKey(<redacted>)")
    }
}

impl PartialEq for StoredKey {
    fn eq(&self, other: &Self) -> bool {
        // Every byte is looked at whatever the first difference is.
        self.0.iter().zip(&other.0).fold(0u8, |differ, (a, b)| differ | (a ^ b)) == 0
    }
}

impl Eq for StoredKey {}

#[cfg(test)]
include!("stored_key.tests.rs");
