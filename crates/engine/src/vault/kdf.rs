//! Making the master key of a passphrase vault: Argon2id over the typed
//! passphrase, with the salt and the parameters the file names (`format.rs`).
//! It is computed once per unlock (and once per change of passphrase) and the
//! 32 bytes it gives are kept in memory only, in a `MasterKey`, which is
//! overwritten when dropped. The memory Argon2 works in is overwritten too.
//!
//! The passphrase is used as the bytes it was typed as; no Unicode
//! normalisation is applied, so the same letters typed in two forms of the
//! same accented character are two passphrases.

use argon2::{Algorithm, Argon2, Block, Params, Version};
use model::Secret;

use super::crypto::random;
use super::error::VaultError;
use super::format::{KDF_SALT_LEN, KdfHeader, KdfParams};
use super::key_file::KEY_LEN;
use super::key_source::MasterKey;

/// What the unit tests of this crate make a new passphrase under, so a test
/// that sets a passphrase does not spend 64 MiB and a second on it. The
/// default settings themselves are checked by a test of their own.
#[cfg(test)]
const TEST_PARAMS: KdfParams = KdfParams { m_kib: 256, t: 1, p: 1 };

impl KdfHeader {
    /// A header for a new passphrase: a fresh random salt and the default
    /// parameters.
    pub fn fresh() -> Result<Self, VaultError> {
        #[cfg(test)]
        let params = TEST_PARAMS;
        #[cfg(not(test))]
        let params = KdfParams::DEFAULT;
        Ok(Self { salt: random::<KDF_SALT_LEN>()?, params })
    }
}

/// The master key for `passphrase` under `kdf`. Refuses parameters out of
/// range before it allocates anything.
pub(super) fn derive(passphrase: &Secret, kdf: &KdfHeader) -> Result<MasterKey, VaultError> {
    kdf.params.check()?;
    let params = Params::new(kdf.params.m_kib, kdf.params.t, kdf.params.p, Some(KEY_LEN))
        .map_err(|_| VaultError::KdfRefused)?;
    let block_count = params.block_count();
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut blocks = vec![Block::new(); block_count];
    let mut out = [0u8; KEY_LEN];
    let done = argon.hash_password_into_with_memory(passphrase.expose().as_bytes(), &kdf.salt, &mut out, &mut blocks[..]);
    // The memory held values derived from the passphrase: overwrite it before it is freed.
    blocks.iter_mut().for_each(|block| *block = Block::new());
    std::hint::black_box(&blocks);
    let key = MasterKey::from_slice(&out);
    model::wipe_bytes(&mut out);
    done.map_err(|_| VaultError::Crypto)?;
    key.ok_or(VaultError::Crypto)
}

#[cfg(test)]
include!("kdf.tests.rs");
