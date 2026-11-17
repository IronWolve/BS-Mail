//! The store of a system with no keychain this program can use (any system
//! but Linux and Windows, and a run that was never told its service name):
//! every call says so, and nothing is kept.

use super::failure::KeychainFailure;
use super::store::KeychainStore;
use super::stored_key::StoredKey;

#[derive(Debug, Clone, Copy, Default)]
pub struct Unavailable;

impl KeychainStore for Unavailable {
    fn get(&self, _account: &str) -> Result<StoredKey, KeychainFailure> {
        Err(KeychainFailure::Unavailable)
    }

    fn set(&self, _account: &str, _key: &StoredKey) -> Result<(), KeychainFailure> {
        Err(KeychainFailure::Unavailable)
    }

    fn delete(&self, _account: &str) -> Result<(), KeychainFailure> {
        Err(KeychainFailure::Unavailable)
    }

    fn probe(&self) -> Result<(), KeychainFailure> {
        Err(KeychainFailure::Unavailable)
    }
}

#[cfg(test)]
include!("unavailable.tests.rs");
