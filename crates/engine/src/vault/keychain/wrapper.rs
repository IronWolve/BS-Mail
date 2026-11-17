//! The keychain as the vault uses it: a store, and the time each call may
//! take. Every method runs its call on a thread of its own (`call.rs`).
//!
//! No method is ever called while the shared handle's lock is held across the
//! wait, with the one exception of the first read of the key when nothing
//! read it at start (`shared.rs` says so): the handle's lock covers file work
//! only, so a keychain that is slow or hung never stops the mail.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use super::call::run_within;
use super::failure::KeychainFailure;
use super::key_id::KeyId;
use super::store::KeychainStore;
use super::stored_key::StoredKey;
use super::super::error::VaultError;

/// How long one call to the keychain may take: a prompt the person has to
/// answer, or a hung service, is no keychain after this.
pub(in crate::vault) const CALL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub(in crate::vault) struct Keychain {
    store: Arc<dyn KeychainStore>,
    timeout: Duration,
}

impl fmt::Debug for Keychain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keychain").field("timeout", &self.timeout).finish_non_exhaustive()
    }
}

impl Keychain {
    pub fn new(store: Arc<dyn KeychainStore>) -> Self {
        Self { store, timeout: CALL_TIMEOUT }
    }

    pub fn with_timeout(store: Arc<dyn KeychainStore>, timeout: Duration) -> Self {
        Self { store, timeout }
    }

    fn call<T: Send + 'static>(
        &self,
        work: impl FnOnce(&dyn KeychainStore) -> Result<T, KeychainFailure> + Send + 'static,
    ) -> Result<T, KeychainFailure> {
        let store = Arc::clone(&self.store);
        run_within(self.timeout, move || work(store.as_ref()))
    }

    /// The key kept for `id`.
    pub fn get(&self, id: &KeyId) -> Result<StoredKey, KeychainFailure> {
        let account = id.account();
        self.call(move |store| store.get(&account))
    }

    pub fn set(&self, id: &KeyId, key: &StoredKey) -> Result<(), KeychainFailure> {
        let (account, key) = (id.account(), key.copy());
        self.call(move |store| store.set(&account, &key))
    }

    /// Removes the entry for `id`; one that is not there is already gone.
    pub fn delete(&self, id: &KeyId) -> Result<(), KeychainFailure> {
        let account = id.account();
        self.call(move |store| store.delete(&account))
    }

    /// Whether the keychain can be used here.
    pub fn probe(&self) -> Result<(), KeychainFailure> {
        self.call(|store| store.probe())
    }

    /// Keeps `key` for `id` and reads it back: only a keychain that gives the
    /// same key back is used, because a vault that the keychain cannot open
    /// again must never be written. An entry that could not be proved is
    /// removed again (except when the keychain stopped answering, where
    /// another call would only wait again).
    pub fn store_verified(&self, id: &KeyId, key: &StoredKey) -> Result<(), VaultError> {
        self.set(id, key)?;
        match self.get(id) {
            Ok(back) if back == *key => Ok(()),
            Ok(_) | Err(KeychainFailure::NotFound) => {
                let _ = self.delete(id);
                Err(VaultError::KeychainReadBack)
            }
            Err(KeychainFailure::Unavailable) => Err(VaultError::KeychainUnavailable),
            Err(other) => {
                let _ = self.delete(id);
                Err(other.into())
            }
        }
    }
}

#[cfg(test)]
include!("wrapper.tests.rs");
