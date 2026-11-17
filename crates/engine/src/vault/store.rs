//! The vault: opened from a folder with a way to get its key, read and
//! changed in memory, and saved whole.

use std::fmt;
use std::path::{Path, PathBuf};

use model::{Secret, wipe_bytes};

use super::crypto;
use super::entries::Entries;
use super::error::VaultError;
use super::file_io::{read_limited, replace};
use super::format::{Header, KdfHeader, MAX_VAULT_BYTES, Mode};
use super::kdf::derive;
use super::key_source::{KeySource, MasterKey, Unlocked};

/// The vault's file name inside its folder.
pub const VAULT_FILE_NAME: &str = "vault";

/// The folder the vault and its key are in, inside the data folder (and beside
/// the file for `--account <file>`).
pub const SECRETS_FOLDER_NAME: &str = "secrets";

/// The name an unreadable vault is set aside under when a fresh one is
/// started.
const UNREADABLE_NAME: &str = "vault.unreadable";

/// The secrets of one data folder. Its debug output shows no entry.
pub struct Vault {
    dir: PathBuf,
    source: KeySource,
    entries: Entries,
}

impl fmt::Debug for Vault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vault").field("dir", &self.dir).field("entries", &self.entries).finish_non_exhaustive()
    }
}

impl Vault {
    /// The vault in `dir`. With no vault file there is an empty one, and
    /// nothing is written (not even the key) until the first `save`. A vault
    /// that is there must open with the key `source` gives, or this fails
    /// with why.
    pub fn open_or_create(dir: &Path, source: KeySource) -> Result<Self, VaultError> {
        let path = dir.join(VAULT_FILE_NAME);
        let Some(bytes) = read_limited(&path)? else {
            return Ok(Self { dir: dir.to_path_buf(), source, entries: Entries::default() });
        };
        let header = Header::parse(&bytes)?;
        let master = source.key_for(&header)?;
        // A key held for this very file that fails to open it means the file changed.
        let entries = read_entries(&master, &header, &bytes, VaultError::Damaged)?;
        Ok(Self { dir: dir.to_path_buf(), source, entries })
    }

    /// The passphrase vault in `dir`, opened with `passphrase`: the key is
    /// made from it under the settings the file names (this takes about a
    /// second). A passphrase that does not open it is `WrongPassphrase`; a
    /// folder with no vault or a vault in another mode is an error too, since
    /// there is nothing a passphrase could unlock.
    pub(super) fn open_with_passphrase(dir: &Path, passphrase: &Secret) -> Result<Self, VaultError> {
        let bytes = read_limited(&dir.join(VAULT_FILE_NAME))?.ok_or(VaultError::WrongMode)?;
        let header = Header::parse(&bytes)?;
        let Some(kdf) = header.kdf else {
            return Err(VaultError::WrongMode);
        };
        let master = derive(passphrase, &kdf)?;
        let entries = read_entries(&master, &header, &bytes, VaultError::WrongPassphrase)?;
        let source = KeySource::Passphrase(Unlocked::new(master, kdf));
        Ok(Self { dir: dir.to_path_buf(), source, entries })
    }

    /// Sets the vault file in `dir` aside as `vault.unreadable` (replacing an
    /// earlier one) and returns an empty vault. For a vault whose key is lost
    /// or that is damaged, when the user is giving its secrets again. Only a
    /// plain file is moved; the key is kept, and made on the first save when
    /// it is gone.
    pub fn start_over(dir: &Path, source: KeySource) -> Result<Self, VaultError> {
        let path = dir.join(VAULT_FILE_NAME);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() => std::fs::rename(&path, dir.join(UNREADABLE_NAME))
                .map_err(|error| VaultError::Write(error.to_string()))?,
            Ok(_) => return Err(VaultError::NotAFile),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(VaultError::Read(error.to_string())),
        }
        Ok(Self { dir: dir.to_path_buf(), source, entries: Entries::default() })
    }

    /// A copy of the secret kept under `name`.
    pub fn get(&self, name: &str) -> Option<Secret> {
        self.entries.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains(name)
    }

    pub fn set(&mut self, name: &str, value: Secret) {
        self.entries.set(name, value);
    }

    /// Whether there was an entry to remove.
    pub fn remove(&mut self, name: &str) -> bool {
        self.entries.remove(name)
    }

    /// Removes every entry whose name starts with `prefix`; how many went.
    pub fn remove_prefix(&mut self, prefix: &str) -> usize {
        self.entries.remove_prefix(prefix)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.len() == 0
    }

    /// The folder the vault and its key are in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The vault file's place.
    pub fn path(&self) -> PathBuf {
        self.dir.join(VAULT_FILE_NAME)
    }

    /// The key this vault was opened (or is to be saved) with.
    pub(super) fn source(&self) -> &KeySource {
        &self.source
    }

    /// The passphrase key held, when this vault is a passphrase vault.
    pub(super) fn unlocked(&self) -> Option<Unlocked> {
        match &self.source {
            KeySource::Passphrase(unlocked) => Some(unlocked.clone()),
            KeySource::KeyFile(_) => None,
        }
    }

    /// The same entries, to be saved with `source` from now on (nothing is
    /// written).
    pub(super) fn moved_to(&self, source: KeySource) -> Self {
        Self { dir: self.dir.clone(), source, entries: self.entries.clone() }
    }

    /// Writes the vault: the key is made first if this is the first save of a
    /// key-file vault, then a new file under a temporary name is flushed and
    /// renamed over the old one. A save that fails leaves the old vault as it
    /// was.
    pub fn save(&self) -> Result<(), VaultError> {
        let master = self.source.key_for_saving()?;
        let file = self.sealed(&master, self.source.kdf())?;
        replace(&self.path(), &file)
    }

    /// The whole file these entries make under `master` (and `kdf` for a
    /// passphrase vault), not yet written.
    pub(super) fn sealed(&self, master: &MasterKey, kdf: Option<&KdfHeader>) -> Result<Vec<u8>, VaultError> {
        let mut plain = self.entries.to_plaintext()?;
        if plain.len() as u64 > MAX_VAULT_BYTES {
            wipe_bytes(&mut plain);
            return Err(VaultError::TooBig);
        }
        crypto::seal(master, kdf, plain)
    }

    /// Which way the vault is protected.
    pub fn mode(&self) -> Mode {
        self.source.mode()
    }
}

/// The entries in `bytes`, whose header is `header`, opened with `master`. A
/// key that does not open a key-file vault is the lost or wrong key; for a
/// passphrase vault the caller says what that means (`on_mismatch`).
fn read_entries(master: &MasterKey, header: &Header, bytes: &[u8], on_mismatch: VaultError) -> Result<Entries, VaultError> {
    let mut plain = match crypto::open(master, header, bytes) {
        Err(VaultError::KeyMismatch) if header.mode == Mode::Passphrase => return Err(on_mismatch),
        other => other?,
    };
    let entries = Entries::from_plaintext(&plain);
    wipe_bytes(&mut plain);
    entries
}

#[cfg(test)]
include!("store.tests.rs");
