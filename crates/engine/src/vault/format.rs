//! The vault file, version 1:
//!
//! ```text
//! offset  size  what
//!      0     8  magic "BSMVAULT"
//!      8     1  version, 1
//!      9     1  mode: 1 key file, 2 system keychain, 3 passphrase
//!     10     2  reserved, zero
//!     12    32  salt, new at every save
//!     44    12  nonce, new at every save
//!     56     n  ciphertext, then the 16-byte tag
//! ```
//!
//! Everything before the ciphertext is the associated data of the encryption,
//! so a changed byte anywhere in the file makes it fail to open. Modes 2 and 3
//! are reserved: this version refuses them as not available.

use super::error::VaultError;

pub(super) const MAGIC: &[u8; 8] = b"BSMVAULT";
pub(super) const VERSION: u8 = 1;
pub(super) const SALT_LEN: usize = 32;
pub(super) const NONCE_LEN: usize = 12;
pub(super) const TAG_LEN: usize = 16;
pub(super) const HEADER_LEN: usize = 8 + 1 + 1 + 2 + SALT_LEN + NONCE_LEN;

/// The largest vault file read or written: one mebibyte.
pub const MAX_VAULT_BYTES: u64 = 1024 * 1024;

/// How the key that opens the vault is obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A random key in a file beside the vault.
    KeyFile,
}

impl Mode {
    pub(super) fn byte(self) -> u8 {
        match self {
            Self::KeyFile => 1,
        }
    }
}

/// The part of the file that is read before anything is decrypted.
pub(super) struct Header {
    pub mode: Mode,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0u8; HEADER_LEN];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8] = VERSION;
        bytes[9] = self.mode.byte();
        bytes[12..12 + SALT_LEN].copy_from_slice(&self.salt);
        bytes[12 + SALT_LEN..].copy_from_slice(&self.nonce);
        bytes
    }

    /// Reads the header of `file`, refusing a file that is too short to hold
    /// a header and a tag, has another magic, a version or mode this program
    /// does not have, or non-zero reserved bytes.
    pub fn parse(file: &[u8]) -> Result<Self, VaultError> {
        if file.len() < HEADER_LEN + TAG_LEN || &file[..8] != MAGIC {
            return Err(VaultError::Damaged);
        }
        if file[8] != VERSION {
            return Err(VaultError::UnknownVersion);
        }
        let mode = match file[9] {
            1 => Mode::KeyFile,
            2 | 3 => return Err(VaultError::ModeUnavailable),
            _ => return Err(VaultError::UnknownMode),
        };
        if file[10..12] != [0, 0] {
            return Err(VaultError::Damaged);
        }
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&file[12..12 + SALT_LEN]);
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&file[12 + SALT_LEN..HEADER_LEN]);
        Ok(Self { mode, salt, nonce })
    }
}

#[cfg(test)]
include!("format.tests.rs");
