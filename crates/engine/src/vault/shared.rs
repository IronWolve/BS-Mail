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
//! The one thing the handle does keep is the key of a passphrase vault, once
//! the passphrase has been given (`unlock`): the key made from it, in memory
//! only, so the background refreshes of sign-in tokens keep working without
//! asking again. It is wiped by `lock`, which the program calls for its vault folder when it ends. Which way a
//! vault is protected is what its file says (`state`), so a vault that is
//! switched, restored or set aside is read the way it now is. Switching
//! (`switch_to_passphrase`, `switch_to_key_file`, `change_passphrase`) is one
//! more change under the same lock.
//!
//! Only one copy of the program holds a data folder at a time (the data lock),
//! so no other process changes the file in between.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use model::Secret;

use super::error::VaultError;
use super::file_io::read_prefix;
use super::format::{HEADER_LEN_V2, Header, Mode, TAG_LEN};
use super::key_source::{KeySource, Unlocked};
use super::store::{VAULT_FILE_NAME, Vault};
use super::switch::{self, Switched};

/// How the vault in a folder is protected, and whether it can be read now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Protection {
    pub mode: Mode,
    /// A passphrase vault whose passphrase has not been given in this run.
    pub locked: bool,
}

/// What the handle keeps between calls.
#[derive(Debug, Default)]
struct Held {
    unlocked: Option<Unlocked>,
}

/// One vault folder's gate.
#[derive(Debug)]
pub struct SharedVault {
    dir: PathBuf,
    gate: Mutex<Held>,
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
        let made = Arc::new(Self { dir: dir.to_path_buf(), gate: Mutex::new(Held::default()) });
        all.push((key, Arc::clone(&made)));
        made
    }

    /// The folder the vault and its key are in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn hold(&self) -> MutexGuard<'_, Held> {
        self.gate.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The header of the vault file as it is now; `None` when there is no
    /// file or it cannot be read as a vault (opening it then says why).
    fn peek(&self) -> Option<Header> {
        let bytes = read_prefix(&self.dir.join(VAULT_FILE_NAME), (HEADER_LEN_V2 + TAG_LEN) as u64).ok()??;
        Header::parse(&bytes).ok()
    }

    /// The key for the vault as the file says it is protected now: the key
    /// file, or the passphrase key held, or `Locked`.
    fn source(&self, held: &Held) -> Result<KeySource, VaultError> {
        match self.peek() {
            Some(Header { kdf: Some(kdf), .. }) => match &held.unlocked {
                Some(unlocked) if *unlocked.kdf() == kdf => Ok(KeySource::Passphrase(unlocked.clone())),
                _ => Err(VaultError::Locked),
            },
            _ => Ok(KeySource::key_file_in(&self.dir)),
        }
    }

    /// How the vault is protected now and whether it is locked. A folder with
    /// no vault, and a file that cannot be read as one, count as a key file
    /// (what the next save makes; opening says what is wrong).
    pub fn protection(&self) -> Protection {
        let held = self.hold();
        match self.peek() {
            Some(Header { mode: Mode::Passphrase, kdf, .. }) => {
                let open = matches!((&held.unlocked, &kdf), (Some(unlocked), Some(kdf)) if unlocked.kdf() == kdf);
                Protection { mode: Mode::Passphrase, locked: !open }
            }
            _ => Protection { mode: Mode::KeyFile, locked: false },
        }
    }

    /// The vault as it is on disk now, read under the lock (so never in the
    /// middle of another change). A folder with no vault is an empty vault; a
    /// locked one is `Locked`.
    pub fn load(&self) -> Result<Vault, VaultError> {
        let held = self.hold();
        Vault::open_or_create(&self.dir, self.source(&held)?)
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
        let held = self.hold();
        let mut vault = Vault::open_or_create(&self.dir, self.source(&held)?)?;
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
    /// where it is and the error is returned. A locked vault or a wrong
    /// passphrase is never started over: only the prompt's own "forgot the
    /// passphrase" does that (`forget_passphrase`).
    pub fn update_or_start_over<T>(
        &self,
        start: bool,
        change: impl FnOnce(&mut Vault) -> (T, bool),
    ) -> Result<T, VaultError> {
        let held = self.hold();
        let source = self.source(&held)?;
        let mut vault = match Vault::open_or_create(&self.dir, source.clone()) {
            Ok(vault) => vault,
            Err(error) if start && error.can_start_over() && source.mode() == Mode::KeyFile => {
                Vault::start_over(&self.dir, source)?
            }
            Err(error) => return Err(error),
        };
        let (result, changed) = change(&mut vault);
        if changed {
            vault.save()?;
        }
        Ok(result)
    }

    /// Gives the passphrase of a passphrase vault: the key is made from it
    /// (about a second) and, when it opens the vault, kept in memory until
    /// `lock`. A passphrase that does not open it is `WrongPassphrase` and
    /// nothing is kept. A vault that is not protected by a passphrase, or is
    /// open already, is left as it is.
    pub fn unlock(&self, passphrase: &Secret) -> Result<(), VaultError> {
        let mut held = self.hold();
        match self.peek() {
            Some(Header { kdf: Some(kdf), .. }) => {
                if held.unlocked.as_ref().is_some_and(|unlocked| *unlocked.kdf() == kdf) {
                    return Ok(());
                }
                held.unlocked = Vault::open_with_passphrase(&self.dir, passphrase)?.unlocked();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Wipes the passphrase key held, if there is one. The vault is locked
    /// again; secrets are unavailable until `unlock`.
    pub fn lock(&self) {
        self.hold().unlocked = None;
    }

    /// For a passphrase that is forgotten: sets the vault file aside as
    /// `vault.unreadable` (nothing is deleted) so the folder has no vault, and
    /// drops any key held. The next save makes a new, empty key-file vault.
    /// Only a passphrase vault is set aside.
    pub fn forget_passphrase(&self) -> Result<(), VaultError> {
        let mut held = self.hold();
        if self.peek().is_none_or(|header| header.mode != Mode::Passphrase) {
            return Err(VaultError::WrongMode);
        }
        Vault::start_over(&self.dir, KeySource::key_file_in(&self.dir))?;
        held.unlocked = None;
        Ok(())
    }

    /// Protects the vault with `passphrase` from now on: a key-file vault (or
    /// none yet) becomes a passphrase vault, a passphrase vault that is open
    /// gets a new passphrase and a new key-derivation salt. Atomic: see
    /// `switch.rs`. On an error the old vault and key are as they were.
    pub fn switch_to_passphrase(&self, passphrase: &Secret) -> Result<Switched, VaultError> {
        let mut held = self.hold();
        let vault = Vault::open_or_create(&self.dir, self.source(&held)?)?;
        let (unlocked, switched) = switch::to_passphrase(&vault, passphrase)?;
        held.unlocked = Some(unlocked);
        Ok(switched)
    }

    /// As `switch_to_passphrase`, for a vault that is a passphrase vault
    /// already: `WrongMode` for any other.
    pub fn change_passphrase(&self, passphrase: &Secret) -> Result<Switched, VaultError> {
        if self.protection().mode != Mode::Passphrase {
            return Err(VaultError::WrongMode);
        }
        self.switch_to_passphrase(passphrase)
    }

    /// Protects the vault with a new random key file instead of a passphrase;
    /// the vault must be open (`Locked` otherwise). A vault that is a key-file
    /// vault already is left as it is. The passphrase key is dropped.
    pub fn switch_to_key_file(&self) -> Result<(), VaultError> {
        let mut held = self.hold();
        if self.peek().is_none_or(|header| header.mode != Mode::Passphrase) {
            return Ok(());
        }
        let vault = Vault::open_or_create(&self.dir, self.source(&held)?)?;
        switch::to_key_file(&vault)?;
        held.unlocked = None;
        Ok(())
    }
}

#[cfg(test)]
include!("shared.tests.rs");
