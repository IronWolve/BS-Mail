//! Wipes the passphrase key from memory when a run ends, however it ends.

use std::path::PathBuf;

use engine::vault::SharedVault;

/// Locks the vault in its folder when dropped: the key made from the
/// passphrase, kept in memory for the run, is overwritten. Harmless for a vault
/// that holds none.
#[derive(Debug)]
pub struct LockOnExit(Option<PathBuf>);

impl LockOnExit {
    /// For the vault in `folder`; `None` (mock mail) guards nothing.
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self(folder)
    }
}

impl Drop for LockOnExit {
    fn drop(&mut self) {
        if let Some(folder) = &self.0 {
            SharedVault::for_dir(folder).lock();
        }
    }
}

#[cfg(test)]
include!("guard.tests.rs");
