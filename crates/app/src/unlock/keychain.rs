//! Reading the key of a keychain vault at start, before any account is read:
//! the key is in the system keychain, so there is no prompt and nothing for
//! the person to type. The shared handle keeps the key for the run, or the
//! failure (a keychain that is not there, or has no key for this vault), so
//! the accounts that need a secret say why and the program runs on.

use engine::vault::{SharedVault, VaultError};

use crate::cli::Options;

use super::target::vault_folder;

/// Reads the key of this run's vault from the system keychain when the vault is
/// a keychain vault; a vault protected another way, or no vault, is left alone.
/// The failure, if any, is kept by the handle; it is returned only so a caller
/// that wants it can look. Nothing is ever set aside or deleted here.
pub fn open_keychain_vault(options: &Options) -> Result<(), VaultError> {
    match vault_folder(options) {
        Some(folder) => SharedVault::for_dir(&folder).unlock_keychain(),
        None => Ok(()),
    }
}

#[cfg(test)]
include!("keychain.tests.rs");
