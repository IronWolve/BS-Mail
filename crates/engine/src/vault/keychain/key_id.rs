//! The key id of a keychain vault: 16 random bytes made when the vault is
//! switched to the keychain, written in the vault's header and used to name
//! the keychain entry that holds the vault's key. It is not secret (it is in
//! the file in plain), it only says which entry to ask for.

use std::fmt;

use super::super::crypto::random;
use super::super::error::VaultError;
use super::hex;

/// How long a key id is.
pub const KEY_ID_LEN: usize = 16;

/// What every entry name starts with, before the hex of the key id.
const ACCOUNT_PREFIX: &str = "vault-key-";

/// The name `probe` asks the keychain about: it is never written.
pub(super) const PROBE_ACCOUNT: &str = "vault-probe";

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyId([u8; KEY_ID_LEN]);

impl KeyId {
    pub fn from_bytes(bytes: [u8; KEY_ID_LEN]) -> Self {
        Self(bytes)
    }

    /// A new random id.
    pub fn random() -> Result<Self, VaultError> {
        random::<KEY_ID_LEN>().map(Self)
    }

    pub fn bytes(&self) -> &[u8; KEY_ID_LEN] {
        &self.0
    }

    /// All zero bytes: no id this program makes, so a header that says it is
    /// damaged.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }

    pub fn hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// The name of the keychain entry that holds this vault's key.
    pub fn account(&self) -> String {
        format!("{ACCOUNT_PREFIX}{}", self.hex())
    }
}

impl fmt::Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyId({})", self.hex())
    }
}

#[cfg(test)]
include!("key_id.tests.rs");
