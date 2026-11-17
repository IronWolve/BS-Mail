//! The shared handle's dealings with the system keychain: the store it uses,
//! the probe, reading the key at start, moving the vault to the keychain, and
//! removing a key that the vault no longer uses.
//!
//! **Where the lock is held.** Every call to the keychain can wait (a prompt,
//! a service that hangs; each call is cut off after `CALL_TIMEOUT`), so the
//! handle's lock is never held across one on purpose: `unlock_keychain` and
//! `switch_to_keychain` look at the file under the lock, let it go, call the
//! keychain, and take it again for the file work; the old key of a vault that
//! was left is removed after the lock is let go. The folder's own lock, which
//! keeps two copies of the program apart (`file_lock.rs`), is never held across
//! a keychain call either: a change reads a keychain vault's key first, with
//! nothing held, and only then takes it (`begin`). The one exception is a plain
//! read (`load`) of a keychain vault whose key nothing read at start
//! (`keychain_source`): it reads under the handle's lock, once, and the answer
//! (the key or the failure) is kept until `lock`, so it can cost one wait and
//! never one per save. The program reads the key at start, before any account,
//! so that exception is not met in a run.

use std::sync::Arc;
use std::time::Duration;

use super::error::VaultError;
use super::format::Header;
use super::key_source::{KeySource, KeychainKey, MasterKey};
use super::keychain::{Keychain, KeyId, KeychainStore, default_store};
use super::shared::{Held, KeychainHold, SharedVault};
use super::store::Vault;
use super::switch::Switched;
use super::switch_keychain::{FreshKey, to_keychain};
use crate::vault::format::Mode;

impl SharedVault {
    /// The keychain this vault folder uses: the one it was given, else the
    /// program's own.
    pub(super) fn keychain(&self) -> Keychain {
        let given = self.keychain.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        given.unwrap_or_else(|| Keychain::new(default_store()))
    }

    /// Makes this folder's vault use `store` as its keychain, in place of the
    /// system's: for tests, which never touch the real keychain.
    pub fn use_keychain(&self, store: Arc<dyn KeychainStore>) {
        *self.keychain.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Keychain::new(store));
    }

    /// As `use_keychain`, with a time limit for each call of its own.
    pub fn use_keychain_with_timeout(&self, store: Arc<dyn KeychainStore>, timeout: Duration) {
        *self.keychain.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(Keychain::with_timeout(store, timeout));
    }

    /// Whether the system keychain can be used here: `Ok`, or why not
    /// (`KeychainUnavailable` for none). Nothing is written. Waits at most one
    /// call's time limit and holds no lock while it does.
    pub fn keychain_probe(&self) -> Result<(), VaultError> {
        self.keychain().probe().map_err(VaultError::from)
    }

    /// The key of a keychain vault, read from the keychain now and kept until
    /// `lock` (or its failure, kept the same way): what the program does at
    /// start, so no account waits for it later. A vault that is not a keychain
    /// vault, or whose key is held already, is left as it is and this is
    /// `Ok`. The wait holds no lock.
    pub fn unlock_keychain(&self) -> Result<(), VaultError> {
        let id = {
            let held = self.hold();
            match self.peek() {
                Some(Header { key_id: Some(id), .. }) if !holds_key(&held, id) => id,
                _ => return Ok(()),
            }
        };
        let read = self.keychain().get(&id).map_err(VaultError::from).and_then(|stored| held_key(&stored, id));
        let mut held = self.hold();
        // Only a vault that is still the one asked about takes the answer.
        if self.peek().is_some_and(|header| header.key_id == Some(id)) {
            held.keychain = match &read {
                Ok(key) => KeychainHold::Key(key.clone()),
                Err(error) => KeychainHold::Failed(id, error.clone()),
            };
        }
        drop(held);
        if read.is_ok() {
            self.repair_marker(Mode::Keychain);
        }
        read.map(|_| ())
    }

    /// The key source of a keychain vault with key id `id`: the key held, the
    /// failure kept, or, when nothing read it yet, the keychain asked now.
    pub(super) fn keychain_source(&self, held: &mut Held, id: KeyId) -> Result<KeySource, VaultError> {
        match &held.keychain {
            KeychainHold::Key(key) if key.id() == id => return Ok(KeySource::Keychain(key.clone())),
            KeychainHold::Failed(failed, error) if *failed == id => return Err(error.clone()),
            _ => {}
        }
        let read = self.keychain().get(&id).map_err(VaultError::from).and_then(|stored| held_key(&stored, id));
        match read {
            Ok(key) => {
                held.keychain = KeychainHold::Key(key.clone());
                Ok(KeySource::Keychain(key))
            }
            Err(error) => {
                held.keychain = KeychainHold::Failed(id, error.clone());
                Err(error)
            }
        }
    }

    /// Protects the vault with a new random key kept in the system keychain
    /// instead of a key file or a passphrase. The vault must be open (`Locked`
    /// or the keychain's own failure otherwise); a keychain vault already is
    /// left as it is. In order: the vault is looked at under the lock; the key
    /// is stored in the keychain and read back, with no lock held (a keychain
    /// that does not give it back is `KeychainReadBack` and nothing is
    /// written); the new vault is written under the lock; then the old key
    /// file, if there was one, is removed. A failure at any step leaves the
    /// old vault and key as they were, and the entry made in the keychain is
    /// removed again.
    pub fn switch_to_keychain(&self) -> Result<Switched, VaultError> {
        {
            let mut held = self.hold();
            if self.peek().is_some_and(|header| header.mode == Mode::Keychain) {
                return Ok(Switched::default());
            }
            let source = self.source(&mut held)?;
            Vault::open_or_create(&self.dir, source)?;
        }
        let keychain = self.keychain();
        let fresh = FreshKey::make()?;
        keychain.store_verified(&fresh.id, &fresh.key)?;
        let done = {
            let mut held = self.hold();
            self.write_keychain_vault(&mut held, &fresh)
        };
        match done {
            Ok(switched) => Ok(switched),
            Err(error) => {
                // The vault was not written: the entry opens nothing, take it out again.
                let _ = keychain.delete(&fresh.id);
                Err(error)
            }
        }
    }

    /// The file work of `switch_to_keychain`, under the lock.
    fn write_keychain_vault(&self, held: &mut Held, fresh: &FreshKey) -> Result<Switched, VaultError> {
        let (_lock, source) = self.begin(held, true)?;
        let vault = Vault::open_or_create(&self.dir, source?)?;
        let (key, switched) = to_keychain(&vault, fresh)?;
        held.unlocked = None;
        held.keychain = KeychainHold::Key(key);
        Ok(switched)
    }

    /// Removes the key `old` from the keychain once the vault no longer uses
    /// it; a failure is said in `switched`, not hidden (the new vault stands).
    pub(super) fn remove_old_keychain_key(&self, old: Option<KeyId>, switched: &mut Switched) {
        let Some(id) = old else {
            return;
        };
        if switched.not_synced.is_some() {
            // The new vault could not be flushed to disk: a power loss could bring the old one
            // back, and its key must still be there.
            return;
        }
        if let Err(failure) = self.keychain().delete(&id) {
            switched.keychain_left = Some(failure.to_string());
        }
    }
}

/// Whether `held` holds the key for `id`.
fn holds_key(held: &Held, id: KeyId) -> bool {
    matches!(&held.keychain, KeychainHold::Key(key) if key.id() == id)
}

/// The key the keychain gave, made ready to open the vault.
fn held_key(stored: &super::keychain::StoredKey, id: KeyId) -> Result<KeychainKey, VaultError> {
    MasterKey::from_slice(stored.bytes()).map(|master| KeychainKey::new(master, id)).ok_or(VaultError::Crypto)
}

#[cfg(test)]
include!("shared_keychain.tests.rs");
