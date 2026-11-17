//! A vault as an account file's reader sees it: opened once, asked for
//! secrets by name, and never the reason an account is refused except by the
//! sentence it gives for that one account.

use std::path::Path;

use model::Secret;

use super::error::VaultError;
use super::key_source::KeySource;
use super::store::Vault;

/// The vault, or why it could not be opened, or no vault at all.
#[derive(Debug)]
pub struct VaultView {
    state: State,
}

#[derive(Debug)]
enum State {
    /// Nothing to read secrets from: every name is absent.
    Nothing,
    Ready(Vault),
    Failed(VaultError),
}

impl VaultView {
    /// No vault: every secret asked for is absent.
    pub fn none() -> Self {
        Self { state: State::Nothing }
    }

    /// The vault in `dir`, opened with its key file. Nothing is written, and a
    /// folder with no vault is a vault with no secrets.
    pub fn open(dir: &Path) -> Self {
        match Vault::open_or_create(dir, KeySource::key_file_in(dir)) {
            Ok(vault) => Self::from_vault(vault),
            Err(error) => Self { state: State::Failed(error) },
        }
    }

    pub fn from_vault(vault: Vault) -> Self {
        Self { state: State::Ready(vault) }
    }

    /// The secret kept under `name`; `Ok(None)` when it is not there, and the
    /// reason when the vault could not be opened.
    pub fn get(&self, name: &str) -> Result<Option<Secret>, VaultError> {
        match &self.state {
            State::Nothing => Ok(None),
            State::Ready(vault) => Ok(vault.get(name)),
            State::Failed(error) => Err(error.clone()),
        }
    }
}

#[cfg(test)]
include!("view.tests.rs");
