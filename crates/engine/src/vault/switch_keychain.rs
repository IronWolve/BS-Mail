//! Moving the vault to the system keychain. The key is made and kept in the
//! keychain first (and read back, `keychain::Keychain::store_verified`), so a
//! vault that the keychain cannot open again is never written; the new vault
//! is then written and renamed over the old one under the handle's lock, and
//! only after that does the old key material go (the old key file here; an old
//! keychain key, when the vault was a keychain vault, is removed by the caller
//! once the lock is let go). A failure at any step leaves the old vault and its
//! key as they were; the caller removes the entry it made.

use super::error::VaultError;
use super::format::Mode;
use super::key_source::{KeySource, KeychainKey, MasterKey};
use super::keychain::{KeyId, StoredKey};
use super::store::Vault;
use super::switch::{Switched, record, remove_old_key};

/// A new key and the key id it is kept under, not yet in any vault.
pub(super) struct FreshKey {
    pub id: KeyId,
    pub key: StoredKey,
}

impl FreshKey {
    pub fn make() -> Result<Self, VaultError> {
        Ok(Self { id: KeyId::random()?, key: StoredKey::random()? })
    }

    fn for_vault(&self) -> Result<KeychainKey, VaultError> {
        let master = MasterKey::from_slice(self.key.bytes()).ok_or(VaultError::Crypto)?;
        Ok(KeychainKey::new(master, self.id))
    }
}

/// `vault` re-saved as a keychain vault under `fresh`, whose key is in the
/// keychain already. The key it now has is returned, for the shared handle to
/// hold. The old key file goes only after the new vault is in place.
pub(super) fn to_keychain(vault: &Vault, fresh: &FreshKey) -> Result<(KeychainKey, Switched), VaultError> {
    let held = fresh.for_vault()?;
    let not_synced = vault.moved_to(KeySource::Keychain(held.clone())).save_flushed()?;
    record(vault.dir(), Mode::Keychain);
    // The old key file goes only after the new vault is in place and the folder is flushed.
    let key_file_left = match (vault.source(), &not_synced) {
        (KeySource::KeyFile(path), None) => remove_old_key(path),
        _ => None,
    };
    Ok((held, Switched { key_file_left, keychain_left: None, not_synced }))
}

#[cfg(test)]
include!("switch_keychain.tests.rs");
