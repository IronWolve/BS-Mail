//! The vault: opened from a folder with a way to get its key, read and
//! changed in memory, and saved whole.

use std::fmt;
use std::path::{Path, PathBuf};

use model::{Secret, wipe_bytes};

use super::aside::{VAULT_ASIDE, set_aside};
use super::crypto;
use super::entries::Entries;
use super::error::VaultError;
use super::file_io::{read_limited, replace};
use super::format::{Binding, Header, MAX_VAULT_BYTES, Mode};
use super::kdf::derive;
use super::keychain::KeyId;
use super::key_source::{KeySource, MasterKey, Unlocked};

/// The vault's file name inside its folder.
pub const VAULT_FILE_NAME: &str = "vault";

/// The folder the vault and its key are in, inside the data folder (and beside
/// the file for `--account <file>`).
pub const SECRETS_FOLDER_NAME: &str = "secrets";

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

    /// Sets the vault file in `dir` aside as `vault.unreadable` (or, when that
    /// name is taken, `vault.unreadable.1`, `.2`, ... up to `.99`: an earlier
    /// set-aside vault is never replaced) and returns an empty vault. For a
    /// vault whose key is lost or that is damaged, when the user is giving its
    /// secrets again. Only a plain file is moved; the key is kept, and made on
    /// the first save when it is gone.
    pub fn start_over(dir: &Path, source: KeySource) -> Result<Self, VaultError> {
        set_aside(dir, VAULT_FILE_NAME, VAULT_ASIDE, true)?;
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
            KeySource::KeyFile(_) | KeySource::Keychain(_) => None,
        }
    }

    /// The key id of the keychain key this vault is held open with, when it
    /// is a keychain vault.
    pub(super) fn keychain_id(&self) -> Option<KeyId> {
        match &self.source {
            KeySource::Keychain(held) => Some(held.id()),
            KeySource::KeyFile(_) | KeySource::Passphrase(_) => None,
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
        self.save_flushed().map(|_| ())
    }

    /// As `save`, and says whether the folder could be flushed after the
    /// rename: `Some(why)` when it could not, so the new vault is in place but
    /// a power loss could still bring the old one back (see `file_io::replace`).
    pub(super) fn save_flushed(&self) -> Result<Option<String>, VaultError> {
        let master = self.source.key_for_saving()?;
        let file = self.sealed(&master, &self.source.binding())?;
        replace(&self.path(), &file)
    }

    /// The whole file these entries make under `master`, whose header says
    /// what `binding` does, not yet written.
    pub(super) fn sealed(&self, master: &MasterKey, binding: &Binding) -> Result<Vec<u8>, VaultError> {
        let mut plain = self.entries.to_plaintext()?;
        if plain.len() as u64 > MAX_VAULT_BYTES {
            wipe_bytes(&mut plain);
            return Err(VaultError::TooBig);
        }
        crypto::seal(master, binding, plain)
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
