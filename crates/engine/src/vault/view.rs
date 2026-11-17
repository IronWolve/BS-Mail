//! A vault as an account file's reader sees it: opened once, asked for
//! secrets by name, and never the reason an account is refused except by the
//! sentence it gives for that one account.

use std::path::{Path, PathBuf};

use model::Secret;

use super::error::VaultError;
use super::shared::SharedVault;
use super::store::Vault;

/// The vault, or why it could not be opened, or no vault at all.
#[derive(Debug)]
pub struct VaultView {
    state: State,
    /// The folder it was read from; `None` for no vault at all.
    dir: Option<PathBuf>,
    /// Set when the vault could not be opened and a save of a secret given
    /// again would not set it aside by itself (it is not a key-file vault, or
    /// the error is one only the explicit "Start the vault over" deals with):
    /// the sentence for the account then says that way out.
    explicit_only: bool,
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
        Self { state: State::Nothing, dir: None, explicit_only: false }
    }

    /// The vault in `dir`, opened with its key file. Nothing is written, and a
    /// folder with no vault is a vault with no secrets.
    pub fn open(dir: &Path) -> Self {
        let shared = SharedVault::for_dir(dir);
        match shared.load() {
            Ok(vault) => Self::from_vault(vault),
            Err(error) => {
                let explicit_only = error.offers_start_over() && !(error.saves_start_over() && shared.starts_over_on_save());
                Self { state: State::Failed(error), dir: Some(dir.to_path_buf()), explicit_only }
            }
        }
    }

    pub fn from_vault(vault: Vault) -> Self {
        let dir = Some(vault.dir().to_path_buf());
        Self { state: State::Ready(vault), dir, explicit_only: false }
    }

    /// The folder the vault is in, when there is one: where the sign-in
    /// tokens of the accounts read through this view are kept too.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Whether the vault could not be opened and only the explicit "Start the
    /// vault over" in Options is the way out (a save of a password given again
    /// would fail the same way).
    pub fn needs_explicit_start_over(&self) -> bool {
        self.explicit_only
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
