//! The vault file. Two layouts share one magic; the version byte says which.
//!
//! Version 1 (key-file mode, 56 header bytes):
//!
//! ```text
//! offset  size  what
//!      0     8  magic "BSMVAULT"
//!      8     1  version, 1
//!      9     1  mode: 1 key file (2 is the reserved system keychain)
//!     10     2  reserved, zero
//!     12    32  salt, new at every save
//!     44    12  nonce, new at every save
//!     56     n  ciphertext, then the 16-byte tag
//! ```
//!
//! Version 2 (passphrase mode, 84 header bytes): the same, with the key
//! derivation's salt and parameters put before the per-save salt and nonce.
//!
//! ```text
//! offset  size  what
//!      0     8  magic "BSMVAULT"
//!      8     1  version, 2
//!      9     1  mode, 3 passphrase
//!     10     2  reserved, zero
//!     12    16  key-derivation salt, new when the passphrase is set
//!     28     4  memory cost in KiB, little-endian
//!     32     4  passes (time cost), little-endian
//!     36     4  lanes (parallelism), little-endian
//!     40    32  salt, new at every save
//!     72    12  nonce, new at every save
//!     84     n  ciphertext, then the 16-byte tag
//! ```
//!
//! Everything before the ciphertext is the associated data of the encryption,
//! so a changed byte anywhere in the file makes it fail to open. The master
//! key of a version-2 file is Argon2id over the passphrase with that salt and
//! those parameters (`kdf.rs`); the parameters are in the file so they can
//! change later, and an absurd value is refused before anything is computed
//! (`KdfParams::check`). A key-file vault is always written as version 1.

use super::error::VaultError;

pub(super) const MAGIC: &[u8; 8] = b"BSMVAULT";
pub(super) const VERSION_1: u8 = 1;
pub(super) const VERSION_2: u8 = 2;
pub(super) const SALT_LEN: usize = 32;
pub(super) const NONCE_LEN: usize = 12;
pub(super) const TAG_LEN: usize = 16;
pub(super) const KDF_SALT_LEN: usize = 16;
/// The version-1 header, which is also the shortest a vault file's header is.
pub(super) const HEADER_LEN: usize = 8 + 1 + 1 + 2 + SALT_LEN + NONCE_LEN;
/// The version-2 header: the version-1 one with the key-derivation salt and
/// its three parameters added.
pub(super) const HEADER_LEN_V2: usize = HEADER_LEN + KDF_SALT_LEN + 12;

/// The largest vault file read or written: one mebibyte.
pub const MAX_VAULT_BYTES: u64 = 1024 * 1024;

/// What the key derivation may ask of a file's parameters at most, so a
/// hostile file cannot make the program allocate or compute wildly.
pub(super) const MAX_M_KIB: u32 = 1024 * 1024;
pub(super) const MAX_T: u32 = 10;
pub(super) const MAX_P: u32 = 8;

/// How the key that opens the vault is obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A random key in a file beside the vault.
    KeyFile,
    /// A key made from a passphrase typed at start.
    Passphrase,
}

impl Mode {
    pub(super) fn byte(self) -> u8 {
        match self {
            Self::KeyFile => 1,
            Self::Passphrase => 3,
        }
    }
}

/// The three parameters of Argon2id, as the file holds them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KdfParams {
    /// Memory in KiB.
    pub m_kib: u32,
    /// Passes over the memory.
    pub t: u32,
    /// Lanes.
    pub p: u32,
}

impl KdfParams {
    /// What a new passphrase is made under: 64 MiB, three passes, one lane.
    pub const DEFAULT: Self = Self { m_kib: 64 * 1024, t: 3, p: 1 };

    /// Refuses zeros, values over the caps, and a memory too small for the
    /// lanes (Argon2 needs at least 8 KiB for each).
    pub fn check(self) -> Result<(), VaultError> {
        let in_range = (1..=MAX_M_KIB).contains(&self.m_kib)
            && (1..=MAX_T).contains(&self.t)
            && (1..=MAX_P).contains(&self.p)
            && self.m_kib >= self.p * 8;
        if in_range { Ok(()) } else { Err(VaultError::KdfRefused) }
    }
}

/// What a passphrase vault says about how its key was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KdfHeader {
    pub salt: [u8; KDF_SALT_LEN],
    pub params: KdfParams,
}

/// The part of the file that is read before anything is decrypted.
#[derive(Debug)]
pub(super) struct Header {
    pub mode: Mode,
    /// `Some` exactly for a passphrase vault (version 2).
    pub kdf: Option<KdfHeader>,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

fn le_u32(bytes: &[u8]) -> u32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&bytes[..4]);
    u32::from_le_bytes(word)
}

impl Header {
    /// How many bytes the header takes in the file.
    pub fn len(&self) -> usize {
        if self.kdf.is_some() { HEADER_LEN_V2 } else { HEADER_LEN }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; self.len()];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[9] = self.mode.byte();
        let mut at = 12;
        bytes[8] = VERSION_1;
        if let Some(kdf) = &self.kdf {
            bytes[8] = VERSION_2;
            bytes[at..at + KDF_SALT_LEN].copy_from_slice(&kdf.salt);
            at += KDF_SALT_LEN;
            for value in [kdf.params.m_kib, kdf.params.t, kdf.params.p] {
                bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
                at += 4;
            }
        }
        bytes[at..at + SALT_LEN].copy_from_slice(&self.salt);
        bytes[at + SALT_LEN..at + SALT_LEN + NONCE_LEN].copy_from_slice(&self.nonce);
        bytes
    }

    /// Reads the header of `file`, refusing a file that is too short to hold
    /// a header and a tag, has another magic, a version or mode this program
    /// does not have (or a mode of the other version), non-zero reserved
    /// bytes, or key-derivation parameters out of range.
    pub fn parse(file: &[u8]) -> Result<Self, VaultError> {
        if file.len() < HEADER_LEN + TAG_LEN || &file[..8] != MAGIC {
            return Err(VaultError::Damaged);
        }
        let mode = match (file[8], file[9]) {
            (VERSION_1, 1) => Mode::KeyFile,
            (VERSION_2, 3) => Mode::Passphrase,
            (VERSION_1 | VERSION_2, 2) => return Err(VaultError::ModeUnavailable),
            (VERSION_1 | VERSION_2, _) => return Err(VaultError::UnknownMode),
            _ => return Err(VaultError::UnknownVersion),
        };
        if file[10..12] != [0, 0] {
            return Err(VaultError::Damaged);
        }
        match mode {
            Mode::KeyFile => Ok(Self::rest(mode, None, file, HEADER_LEN - SALT_LEN - NONCE_LEN)),
            Mode::Passphrase => {
                if file.len() < HEADER_LEN_V2 + TAG_LEN {
                    return Err(VaultError::Damaged);
                }
                let mut salt = [0u8; KDF_SALT_LEN];
                salt.copy_from_slice(&file[12..12 + KDF_SALT_LEN]);
                let params = KdfParams { m_kib: le_u32(&file[28..]), t: le_u32(&file[32..]), p: le_u32(&file[36..]) };
                params.check()?;
                Ok(Self::rest(mode, Some(KdfHeader { salt, params }), file, 12 + KDF_SALT_LEN + 12))
            }
        }
    }

    /// The per-save salt and nonce, which start at `at`.
    fn rest(mode: Mode, kdf: Option<KdfHeader>, file: &[u8], at: usize) -> Self {
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&file[at..at + SALT_LEN]);
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&file[at + SALT_LEN..at + SALT_LEN + NONCE_LEN]);
        Self { mode, kdf, salt, nonce }
    }
}

#[cfg(test)]
include!("format.tests.rs");
