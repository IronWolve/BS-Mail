//! Changing how the vault is protected: from a key file to a passphrase, from
//! a passphrase to a key file, and to another passphrase. Moves to the system
//! keychain are in `switch_keychain.rs`.
//!
//! Every switch writes the whole new vault under a temporary name, flushes it
//! and renames it over the old one before anything else changes, and only
//! then removes what the old protection left (the old key file). A failure at
//! any step before the rename leaves the old vault and its key exactly as they
//! were; a failure to remove the old key file afterwards leaves the new vault
//! in place and is reported, not hidden. The caller holds the shared handle's
//! lock for the whole switch.

use std::path::{Path, PathBuf};

use model::{MIN_PASSPHRASE_CHARS, Secret, wipe_bytes};

use super::crypto::random;
use super::error::VaultError;
use super::file_io::replace;
use super::format::{Binding, KdfHeader};
use super::inject::{Step, reach};
use super::kdf::derive;
use super::key_file::KEY_LEN;
use super::key_source::{KEY_FILE_NAME, KeySource, MasterKey, Unlocked};
use super::store::Vault;
use crate::private_fs;

/// What a switch left behind.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Switched {
    /// The new vault is in place, but the key file the old one used could not
    /// be removed (the system's words). It opens nothing any more; the person
    /// is told to delete it.
    pub key_file_left: Option<String>,
    /// The new vault is in place, but the key the old keychain vault used
    /// could not be removed from the system keychain (the keychain's words).
    /// It opens nothing any more; the person is told to delete it there.
    pub keychain_left: Option<String>,
}

/// The name a new key file is written under before it is put in place.
fn new_key_path(dir: &Path) -> PathBuf {
    dir.join(format!("{KEY_FILE_NAME}.new"))
}

/// `vault` re-saved as a passphrase vault under `passphrase`, with a new
/// key-derivation salt. The key it now has is returned, for the shared handle
/// to hold.
pub(super) fn to_passphrase(vault: &Vault, passphrase: &Secret) -> Result<(Unlocked, Switched), VaultError> {
    if passphrase.expose().chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(VaultError::PassphraseTooShort);
    }
    reach(Step::Derive)?;
    let kdf = KdfHeader::fresh()?;
    let unlocked = Unlocked::new(derive(passphrase, &kdf)?, kdf);
    vault.moved_to(KeySource::Passphrase(unlocked.clone())).save()?;
    // The new vault is in place: only now does the old key file go.
    let key_file_left = match vault.source() {
        KeySource::KeyFile(path) => remove_old_key(path),
        KeySource::Passphrase(_) | KeySource::Keychain(_) => None,
    };
    Ok((unlocked, Switched { key_file_left, keychain_left: None }))
}

/// Takes the old key file out; the system's words when it could not be.
pub(super) fn remove_old_key(path: &Path) -> Option<String> {
    if let Err(error) = reach(Step::RemoveKey) {
        return Some(error.to_string());
    }
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => Some(error.to_string()),
        Ok(_) => {
            let gone = std::fs::remove_file(path).err().map(|error| error.to_string());
            if let Some(folder) = path.parent() {
                let _ = private_fs::sync_folder(folder);
            }
            gone
        }
    }
}

/// `vault`, an unlocked passphrase vault, re-saved as a key-file vault under a
/// fresh random key. The key file is put in place first (an unused key beside
/// a passphrase vault opens nothing), then the vault is renamed over the old
/// one; if that fails the new key file is taken out again.
pub(super) fn to_key_file(vault: &Vault) -> Result<(), VaultError> {
    let dir = vault.dir();
    let (temp, key_path) = (new_key_path(dir), dir.join(KEY_FILE_NAME));
    let fail = |error: std::io::Error| VaultError::Write(error.to_string());
    reach(Step::WriteKey)?;
    private_fs::make_folder(dir).map_err(fail)?;
    // A leftover of an earlier try holds only a key that was never used.
    if std::fs::symlink_metadata(&temp).is_ok_and(|meta| !meta.is_dir()) {
        std::fs::remove_file(&temp).map_err(fail)?;
    }
    let master = write_new_key(&temp)?;
    let file = vault.sealed(&master, &Binding::KeyFile).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    let placed = reach(Step::RenameKey).and_then(|()| std::fs::rename(&temp, &key_path).map_err(fail));
    if let Err(error) = placed {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    let _ = private_fs::sync_folder(dir);
    if let Err(error) = replace(&vault.path(), &file) {
        let _ = std::fs::remove_file(&key_path);
        return Err(error);
    }
    Ok(())
}

/// Writes a new random key to `path` (which must not exist) and returns it.
fn write_new_key(path: &Path) -> Result<MasterKey, VaultError> {
    let mut bytes: [u8; KEY_LEN] = random()?;
    let written = private_fs::write_new_or_remove(path, &bytes);
    let key = MasterKey::from_slice(&bytes);
    wipe_bytes(&mut bytes);
    written.map_err(|error| VaultError::Write(error.to_string()))?;
    key.ok_or(VaultError::Crypto)
}

#[cfg(test)]
include!("switch.tests.rs");
