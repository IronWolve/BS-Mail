//! The one handle every part of the program uses to change the vault.
//!
//! Several accounts refresh their sign-in at the same moment, and the account
//! form saves passwords while they do. Each of those is a read of the whole
//! vault, a change, and a save of the whole vault, so two of them at once
//! would lose the entry of whichever saved first. This handle is the one gate:
//! there is exactly one per vault folder in the process (`for_dir`), a change
//! holds its lock from the moment it reads the vault from disk until the save
//! is renamed into place, and the handle keeps no copy of the vault, so what it
//! reads under the lock is always what the last save wrote. The lock is held
//! for file work only, never across a network call: the closures given to
//! `update` are short and do no waiting.
//!
//! Only one copy of the program holds a data folder at a time (the data lock),
//! so no other process changes the file in between.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::error::VaultError;
use super::key_source::KeySource;
use super::store::Vault;

/// One vault folder's gate.
#[derive(Debug)]
pub struct SharedVault {
    dir: PathBuf,
    gate: Mutex<()>,
}

/// The handles made so far, by the folder they are for.
static HANDLES: Mutex<Vec<(PathBuf, Arc<SharedVault>)>> = Mutex::new(Vec::new());

/// The same folder under any spelling (`a/../a`, a link in the way) is one
/// key: the folder that holds the vault is named by its real place when it
/// exists yet, and as given when it does not.
fn key_of(dir: &Path) -> PathBuf {
    let parent = dir.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let real = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    match dir.file_name() {
        Some(name) => real.join(name),
        None => real,
    }
}

impl SharedVault {
    /// The handle for the vault in `dir`: the same one every time, from any
    /// thread.
    pub fn for_dir(dir: &Path) -> Arc<Self> {
        let key = key_of(dir);
        let mut all = HANDLES.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, found)) = all.iter().find(|(known, _)| *known == key) {
            return Arc::clone(found);
        }
        let made = Arc::new(Self { dir: dir.to_path_buf(), gate: Mutex::new(()) });
        all.push((key, Arc::clone(&made)));
        made
    }

    /// The folder the vault and its key are in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn hold(&self) -> MutexGuard<'_, ()> {
        self.gate.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn source(&self) -> KeySource {
        KeySource::key_file_in(&self.dir)
    }

    /// The vault as it is on disk now, read under the lock (so never in the
    /// middle of another change). A folder with no vault is an empty vault.
    pub fn load(&self) -> Result<Vault, VaultError> {
        let _held = self.hold();
        Vault::open_or_create(&self.dir, self.source())
    }

    /// Looks at the vault as it is on disk now; changes nothing.
    pub fn read<T>(&self, look: impl FnOnce(&Vault) -> T) -> Result<T, VaultError> {
        self.load().map(|vault| look(&vault))
    }

    /// Reads the vault from disk under the lock, lets `change` edit it, and
    /// saves it when `change` says (the `bool`) that it edited anything. The
    /// lock is held until the save is on disk. A vault that cannot be opened
    /// is an error and is never replaced; a save that fails leaves the old
    /// vault as it was.
    pub fn update<T>(&self, change: impl FnOnce(&mut Vault) -> (T, bool)) -> Result<T, VaultError> {
        let _held = self.hold();
        let mut vault = Vault::open_or_create(&self.dir, self.source())?;
        let (result, changed) = change(&mut vault);
        if changed {
            vault.save()?;
        }
        Ok(result)
    }

    /// As `update`, for a user who is giving a secret now: a vault whose file
    /// or key is lost (see `VaultError::can_start_over`) is set aside as
    /// `vault.unreadable` and a new one started. `start` says whether `change`
    /// then runs on an empty vault; when it is false the lost vault is left
    /// where it is and the error is returned.
    pub fn update_or_start_over<T>(
        &self,
        start: bool,
        change: impl FnOnce(&mut Vault) -> (T, bool),
    ) -> Result<T, VaultError> {
        let _held = self.hold();
        let mut vault = match Vault::open_or_create(&self.dir, self.source()) {
            Ok(vault) => vault,
            Err(error) if start && error.can_start_over() => Vault::start_over(&self.dir, self.source())?,
            Err(error) => return Err(error),
        };
        let (result, changed) = change(&mut vault);
        if changed {
            vault.save()?;
        }
        Ok(result)
    }
}

#[cfg(test)]
include!("shared.tests.rs");
