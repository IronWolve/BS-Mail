//! The vault: opened from a folder with a way to get its key, read and
//! changed in memory, and saved whole.

use std::fmt;
use std::path::{Path, PathBuf};

use model::{Secret, wipe_bytes};

use super::crypto;
use super::entries::Entries;
use super::error::VaultError;
use super::file_io::{read_limited, replace};
use super::format::{Header, MAX_VAULT_BYTES};
use super::key_source::KeySource;

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
        let master = source.existing_key()?;
        let mut plain = crypto::open(&master, &header, &bytes)?;
        let entries = Entries::from_plaintext(&plain);
        wipe_bytes(&mut plain);
        Ok(Self { dir: dir.to_path_buf(), source, entries: entries? })
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

    /// Writes the vault: the key is made first if this is the first save, then
    /// a new file under a temporary name is flushed and renamed over the old
    /// one. A save that fails leaves the old vault as it was.
    pub fn save(&self) -> Result<(), VaultError> {
        let master = self.source.key_for_saving()?;
        let mut plain = self.entries.to_plaintext()?;
        if plain.len() as u64 > MAX_VAULT_BYTES {
            wipe_bytes(&mut plain);
            return Err(VaultError::TooBig);
        }
        let file = crypto::seal(&master, self.source.mode(), plain)?;
        replace(&self.path(), &file)
    }
}

#[cfg(test)]
include!("store.tests.rs");
