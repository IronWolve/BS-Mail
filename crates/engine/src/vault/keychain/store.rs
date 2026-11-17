//! The place the vault's key is kept: the operating system's keychain, a fake
//! in memory for tests, or nothing. Entries are named by `account` (the
//! program's name is the service the implementation was made with). A call may
//! block, so the program never makes one directly: `Keychain` runs it on a
//! thread of its own with a time limit.

use super::failure::KeychainFailure;
use super::stored_key::StoredKey;

pub trait KeychainStore: Send + Sync {
    /// The key kept under `account`; `NotFound` when there is none.
    fn get(&self, account: &str) -> Result<StoredKey, KeychainFailure>;

    /// Keeps `key` under `account`, in place of what is there.
    fn set(&self, account: &str, key: &StoredKey) -> Result<(), KeychainFailure>;

    /// Removes the entry under `account`. An entry that is not there is
    /// already gone, and that is success.
    fn delete(&self, account: &str) -> Result<(), KeychainFailure>;

    /// Whether the keychain can be used at all: `Ok` when it answers (that it
    /// holds no entry of the probe's name is an answer), `Unavailable` when
    /// there is none here. Nothing is written.
    fn probe(&self) -> Result<(), KeychainFailure>;
}
