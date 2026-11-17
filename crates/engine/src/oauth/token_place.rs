//! Where one account's sign-in is kept: its entries in the vault of the data
//! folder (`vault_dir` is the folder the vault and its key are in) under the
//! account's id, and the older `oauth/<id>.token` file that versions before the
//! vault wrote, which is only ever read and removed, never written.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::vault::SharedVault;

/// The place of an account's tokens. Empty until the account is read from a
/// file or given its folders (`AccountConfig::with_state_dir` and
/// `with_secrets_dir`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenPlace {
    vault_dir: PathBuf,
    id: String,
    legacy_file: PathBuf,
}

impl TokenPlace {
    /// The place of account `id`'s tokens in the vault in `vault_dir`, where an
    /// older version may still have left them in `legacy_file`.
    pub fn new(vault_dir: &Path, id: &str, legacy_file: &Path) -> Self {
        Self { vault_dir: vault_dir.to_path_buf(), id: id.to_owned(), legacy_file: legacy_file.to_path_buf() }
    }

    pub(crate) fn set_id(&mut self, id: &str) {
        id.clone_into(&mut self.id);
    }

    pub(crate) fn set_vault_dir(&mut self, dir: &Path) {
        self.vault_dir = dir.to_path_buf();
    }

    pub(crate) fn set_legacy_file(&mut self, file: &Path) {
        self.legacy_file = file.to_path_buf();
    }

    /// Whether the tokens have a vault to be kept in.
    pub fn is_set(&self) -> bool {
        !self.vault_dir.as_os_str().is_empty() && !self.id.is_empty()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn vault_dir(&self) -> &Path {
        &self.vault_dir
    }

    /// The token file an older version wrote.
    pub fn legacy_file(&self) -> &Path {
        &self.legacy_file
    }

    /// The process's one handle on the vault these tokens are in.
    pub fn vault(&self) -> Arc<SharedVault> {
        SharedVault::for_dir(&self.vault_dir)
    }

    /// What tells this account's tokens from every other's, for what is kept
    /// in memory.
    pub(super) fn key(&self) -> (PathBuf, String) {
        (self.vault_dir.clone(), self.id.clone())
    }
}

#[cfg(test)]
include!("token_place.tests.rs");
