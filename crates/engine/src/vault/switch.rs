//! Changing how the vault is protected: from a key file to a passphrase, from
//! a passphrase to a key file, and to another passphrase. Moves to the system
//! keychain are in `switch_keychain.rs`.
//!
//! Every switch writes the whole new vault under a temporary name, flushes it
//! and renames it over the old one before anything else changes, and only
//! then removes what the old protection left (the old key file), and only when
//! the folder was flushed too: a power loss before that could bring the old
//! vault back, and its key must still be there. A failure at any step before
//! the rename leaves the old vault and its key exactly as they were; a failure
//! to remove the old key file afterwards, or to flush the folder, leaves the
//! new vault in place and is reported, not hidden. After the new vault is in
//! place the word that records the way it is protected is written
//! (`marker.rs`). The caller holds the shared handle's lock for the whole
//! switch.

use std::path::{Path, PathBuf};

use model::{MIN_PASSPHRASE_CHARS, Secret, wipe_bytes};

use super::crypto::random;
use super::error::VaultError;
use super::file_io::replace;
use super::format::{Binding, KdfHeader, Mode};
use super::inject::{Step, reach};
use super::kdf::derive;
use super::key_file::KEY_LEN;
use super::key_source::{KEY_FILE_NAME, KeySource, MasterKey, Unlocked};
use super::marker;
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
    /// The new vault is in place, but the folder could not be flushed to disk
    /// (the system's words), so a power loss could still bring the old vault
    /// back. The old key (the key file, or the keychain entry) was therefore
    /// kept; the person is told to remove it themselves once the change is
    /// safe.
    pub not_synced: Option<String>,
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
    let not_synced = vault.moved_to(KeySource::Passphrase(unlocked.clone())).save_flushed()?;
    record(vault.dir(), Mode::Passphrase);
    // The new vault is in place: only now does the old key file go, and only when it is safe.
    let key_file_left = match (vault.source(), &not_synced) {
        (KeySource::KeyFile(path), None) => remove_old_key(path),
        _ => None,
    };
    Ok((unlocked, Switched { key_file_left, keychain_left: None, not_synced }))
}

/// Writes the word that records the way the vault is now protected. The new
/// vault is in place already and the header is the truth while it can be read,
/// so a word that cannot be written is not a failure of the switch: the next
/// save writes it again.
pub(super) fn record(dir: &Path, mode: Mode) {
    let _ = marker::write(dir, mode);
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

/// `vault`, an unlocked passphrase or keychain vault, re-saved as a key-file
/// vault under a fresh random key. The key file is put in place first (an
/// unused key beside a passphrase vault opens nothing) and flushed, then the
/// vault is renamed over the old one; if either fails the new key file is
/// taken out again. `Some(why)` when the folder could not be flushed after the
/// vault was renamed (the switch stands; the caller keeps the old keychain
/// key).
pub(super) fn to_key_file(vault: &Vault) -> Result<Option<String>, VaultError> {
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
    // The vault is renamed over only once the key it will need is known to be on disk.
    if let Err(error) = flush_key(dir) {
        let _ = std::fs::remove_file(&key_path);
        return Err(error);
    }
    let not_synced = match replace(&vault.path(), &file) {
        Ok(not_synced) => not_synced,
        Err(error) => {
            let _ = std::fs::remove_file(&key_path);
            return Err(error);
        }
    };
    record(dir, Mode::KeyFile);
    Ok(not_synced)
}

/// Flushes the folder after a new key file was renamed into it.
fn flush_key(dir: &Path) -> Result<(), VaultError> {
    reach(Step::SyncKey)?;
    private_fs::sync_folder(dir).map_err(|error| VaultError::Write(error.to_string()))
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
