//! The vault file. Two layouts share one magic; the version byte says which.
//!
//! Version 1 (key-file mode, 56 header bytes):
//!
//! ```text
//! offset  size  what
//!      0     8  magic "BSMVAULT"
//!      8     1  version, 1
//!      9     1  mode, 1 key file
//!     10     2  reserved, zero
//!     12    32  salt, new at every save
//!     44    12  nonce, new at every save
//!     56     n  ciphertext, then the 16-byte tag
//! ```
//!
//! Version 2 (passphrase mode and keychain mode, 84 header bytes): the same,
//! with 28 bytes about where the key comes from put before the per-save salt
//! and nonce. In passphrase mode (3) they are the key derivation's salt and
//! parameters; in keychain mode (2) the first 16 are the key id and the other
//! 12 are reserved and zero, so one parser reads both and the rest of the
//! header sits at the same offsets.
//!
//! ```text
//! offset  size  what
//!      0     8  magic "BSMVAULT"
//!      8     1  version, 2
//!      9     1  mode, 3 passphrase, 2 system keychain
//!     10     2  reserved, zero
//!     12    16  passphrase: key-derivation salt, new when the passphrase is set
//!               keychain: the key id, random, new when the keychain is chosen
//!     28     4  passphrase: memory cost in KiB, little-endian
//!               keychain: zero
//!     32     4  passphrase: passes (time cost), little-endian
//!               keychain: zero
//!     36     4  passphrase: lanes (parallelism), little-endian
//!               keychain: zero
//!     40    32  salt, new at every save
//!     72    12  nonce, new at every save
//!     84     n  ciphertext, then the 16-byte tag
//! ```
//!
//! Everything before the ciphertext is the associated data of the encryption,
//! so a changed byte anywhere in the file makes it fail to open. The master
//! key of a passphrase file is Argon2id over the passphrase with that salt and
//! those parameters (`kdf.rs`); the parameters are in the file so they can
//! change later, and an absurd value is refused before anything is computed
//! (`KdfParams::check`). The master key of a keychain file is 32 random bytes
//! kept in the system keychain under the key id (`keychain/`); an all-zero key
//! id and a non-zero reserved byte are refused as damage. A key-file vault is
//! always written as version 1.

use super::error::VaultError;
use super::keychain::{KEY_ID_LEN, KeyId};

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
/// The caps are 256 MiB, 4 passes and 4 lanes: well above what the program
/// writes (64 MiB, 3 passes, 1 lane) and low enough that unlocking a hostile
/// file costs a second or two, not minutes and a gibibyte.
pub(super) const MAX_M_KIB: u32 = 256 * 1024;
pub(super) const MAX_T: u32 = 4;
pub(super) const MAX_P: u32 = 4;

/// How the key that opens the vault is obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A random key in a file beside the vault.
    KeyFile,
    /// A random key kept in the system keychain, found by a key id in the file.
    Keychain,
    /// A key made from a passphrase typed at start.
    Passphrase,
}

impl Mode {
    pub(super) fn byte(self) -> u8 {
        match self {
            Self::KeyFile => 1,
            Self::Keychain => 2,
            Self::Passphrase => 3,
        }
    }
}

/// What a vault being written says about where its key comes from; the header
/// is made from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    KeyFile,
    Passphrase(KdfHeader),
    Keychain(KeyId),
}

impl Binding {
    /// The header of a vault written with this binding.
    pub fn header(self, salt: [u8; SALT_LEN], nonce: [u8; NONCE_LEN]) -> Header {
        let (mode, kdf, key_id) = match self {
            Self::KeyFile => (Mode::KeyFile, None, None),
            Self::Passphrase(kdf) => (Mode::Passphrase, Some(kdf), None),
            Self::Keychain(id) => (Mode::Keychain, None, Some(id)),
        };
        Header { mode, kdf, key_id, salt, nonce }
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
    /// `Some` exactly for a keychain vault (version 2).
    pub key_id: Option<KeyId>,
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
        if self.kdf.is_some() || self.key_id.is_some() { HEADER_LEN_V2 } else { HEADER_LEN }
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
        } else if let Some(id) = &self.key_id {
            // The 12 bytes the passphrase layout uses for its parameters stay zero.
            bytes[8] = VERSION_2;
            bytes[at..at + KEY_ID_LEN].copy_from_slice(id.bytes());
            at += KDF_SALT_LEN + 12;
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
            (VERSION_2, 2) => Mode::Keychain,
            (VERSION_2, 3) => Mode::Passphrase,
            (VERSION_1 | VERSION_2, _) => return Err(VaultError::UnknownMode),
            _ => return Err(VaultError::UnknownVersion),
        };
        if file[10..12] != [0, 0] {
            return Err(VaultError::Damaged);
        }
        if mode != Mode::KeyFile && file.len() < HEADER_LEN_V2 + TAG_LEN {
            return Err(VaultError::Damaged);
        }
        match mode {
            Mode::KeyFile => Ok(Self::rest(mode, (None, None), file, HEADER_LEN - SALT_LEN - NONCE_LEN)),
            Mode::Passphrase => {
                let mut salt = [0u8; KDF_SALT_LEN];
                salt.copy_from_slice(&file[12..12 + KDF_SALT_LEN]);
                let params = KdfParams { m_kib: le_u32(&file[28..]), t: le_u32(&file[32..]), p: le_u32(&file[36..]) };
                params.check()?;
                Ok(Self::rest(mode, (Some(KdfHeader { salt, params }), None), file, 12 + KDF_SALT_LEN + 12))
            }
            Mode::Keychain => {
                let mut id = [0u8; KEY_ID_LEN];
                id.copy_from_slice(&file[12..12 + KEY_ID_LEN]);
                let id = KeyId::from_bytes(id);
                if id.is_zero() || file[28..40].iter().any(|byte| *byte != 0) {
                    return Err(VaultError::Damaged);
                }
                Ok(Self::rest(mode, (None, Some(id)), file, 12 + KDF_SALT_LEN + 12))
            }
        }
    }

    /// The per-save salt and nonce, which start at `at`.
    fn rest(mode: Mode, bound: (Option<KdfHeader>, Option<KeyId>), file: &[u8], at: usize) -> Self {
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&file[at..at + SALT_LEN]);
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&file[at + SALT_LEN..at + SALT_LEN + NONCE_LEN]);
        Self { mode, kdf: bound.0, key_id: bound.1, salt, nonce }
    }
}

#[cfg(test)]
include!("format.tests.rs");
