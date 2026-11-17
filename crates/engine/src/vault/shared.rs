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
//! the passphrase has been given (`unlock`), and the key of a keychain vault,
//! once it has been read from the system keychain (`unlock_keychain`, at
//! start): in memory only, so the background refreshes of sign-in tokens keep
//! working without asking again. They are wiped by `lock`, which the program
//! calls for its vault folder when it ends. A keychain that failed is
//! remembered too, until `lock` (the program tries again at its next start),
//! so a keychain that is missing or hung costs one wait, not one for every
//! save. Which way a vault is protected is what its file says (`peek`), so a
//! vault that is switched, restored or set aside is read the way it now is.
//! Switching (`switch_to_passphrase`, `switch_to_key_file`,
//! `change_passphrase`, `switch_to_keychain`) is one more change under the
//! same lock. The lock covers file work: the calls to the keychain, which can
//! wait, are made before it is taken and after it is let go (`shared_keychain.rs`).
//!
//! Two copies of the program can still share one secrets folder: a run with
//! `--account <file>` takes no data lock and uses `secrets/` beside the file.
//! So every read, change and save also holds an operating-system lock on
//! `secrets/lock` (`file_lock.rs`), taken after the handle's own lock and held
//! for the file work only, so two processes never overwrite each other's
//! tokens or passwords and never meet in `vault.tmp`; a file system that cannot
//! lock refuses a change with a sentence. A copy that holds no lock at all can
//! only read, and a read always sees a whole vault (a save is a rename).
//!
//! Which way a vault is protected is the vault's own header while it can be
//! read, else the word in `secrets/protection` (`marker.rs`), else key file
//! for a folder that has neither. A passphrase or keychain vault that is
//! missing or damaged is therefore never taken for a fresh folder, and only an
//! explicit, confirmed start-over (`shared_start_over.rs`) sets one aside.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use model::Secret;

use super::error::VaultError;
use super::file_io::read_prefix;
use super::file_lock::FileLock;
use super::format::{HEADER_LEN_V2, Header, Mode, TAG_LEN};
use super::keychain::{Keychain, KeyId};
use super::key_source::{KeySource, KeychainKey, Unlocked};
use super::marker::{self, Marker};
use super::store::{VAULT_FILE_NAME, Vault};
use super::switch::record;

/// How the vault in a folder is protected, and whether it can be read now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Protection {
    pub mode: Mode,
    /// A passphrase vault whose passphrase has not been given in this run.
    pub locked: bool,
}

/// What the handle knows about the key of a keychain vault.
#[derive(Debug, Default)]
pub(super) enum KeychainHold {
    #[default]
    Nothing,
    /// The key read from the keychain, for this key id.
    Key(KeychainKey),
    /// Reading the key of this key id failed; the failure stands until `lock`.
    Failed(KeyId, VaultError),
}

/// What the handle keeps between calls.
#[derive(Debug, Default)]
pub(super) struct Held {
    pub(super) unlocked: Option<Unlocked>,
    pub(super) keychain: KeychainHold,
}

/// One vault folder's gate.
#[derive(Debug)]
pub struct SharedVault {
    pub(super) dir: PathBuf,
    pub(super) gate: Mutex<Held>,
    /// The keychain this folder's vault uses, when it was given one (tests);
    /// else the program's own (`keychain::default_store`). Its own lock, so a
    /// call that waits never holds the gate.
    pub(super) keychain: Mutex<Option<Keychain>>,
    /// What was said about the folder and not yet taken (`take_notes`), such
    /// as that its permissions were tightened.
    pub(super) notes: Mutex<Vec<String>>,
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
        let made = Arc::new(Self {
            dir: dir.to_path_buf(),
            gate: Mutex::new(Held::default()),
            keychain: Mutex::new(None),
            notes: Mutex::new(Vec::new()),
        });
        all.push((key, Arc::clone(&made)));
        made
    }

    /// The folder the vault and its key are in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub(super) fn hold(&self) -> MutexGuard<'_, Held> {
        self.gate.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The header of the vault file as it is now: `Ok(None)` when there is no
    /// file, and why not when the file cannot be read as a vault.
    pub(super) fn header(&self) -> Result<Option<Header>, VaultError> {
        let path = self.dir.join(VAULT_FILE_NAME);
        let Some(bytes) = read_prefix(&path, (HEADER_LEN_V2 + TAG_LEN) as u64)? else {
            return Ok(None);
        };
        Header::parse(&bytes).map(Some)
    }

    /// The header of the vault file as it is now; `None` when there is no
    /// file or it cannot be read as a vault (opening it then says why).
    pub(super) fn peek(&self) -> Option<Header> {
        self.header().ok().flatten()
    }

    /// The way the word in the folder says the vault is protected, for when
    /// the vault's own header cannot be read: a folder with no word is fresh
    /// (key file), and a word that is damaged is not known, so it counts as
    /// protected (a passphrase) and nothing is started over unasked.
    pub(super) fn marked_mode(&self) -> Mode {
        match marker::read(&self.dir) {
            Marker::Missing => Mode::KeyFile,
            Marker::Valid(mode) => mode,
            Marker::Garbled => Mode::Passphrase,
        }
    }

    /// How the vault is protected: what its header says when that can be
    /// read, else what the folder's word says (`marked_mode`).
    pub(super) fn effective_mode(&self) -> Mode {
        self.peek().map_or_else(|| self.marked_mode(), |header| header.mode)
    }

    /// The key for the vault as the file says it is protected now: the key
    /// file, or the passphrase key held, or the keychain key held (read now
    /// if nothing read it yet, or its failure), or `Locked`. A passphrase or
    /// keychain vault whose file is missing or unreadable has no key to find:
    /// that is `Missing` (or what is wrong with the file), never a key file.
    pub(super) fn source(&self, held: &mut Held) -> Result<KeySource, VaultError> {
        let header = match self.header() {
            Ok(Some(header)) => return self.source_for(held, &header),
            Ok(None) => None,
            Err(error) => Some(error),
        };
        match self.marked_mode() {
            Mode::KeyFile => Ok(KeySource::key_file_in(&self.dir)),
            Mode::Passphrase | Mode::Keychain => Err(header.unwrap_or(VaultError::Missing)),
        }
    }

    fn source_for(&self, held: &mut Held, header: &Header) -> Result<KeySource, VaultError> {
        match header {
            Header { kdf: Some(kdf), .. } => match &held.unlocked {
                Some(unlocked) if unlocked.kdf() == kdf => Ok(KeySource::Passphrase(unlocked.clone())),
                _ => Err(VaultError::Locked),
            },
            Header { key_id: Some(id), .. } => self.keychain_source(held, *id),
            _ => Ok(KeySource::key_file_in(&self.dir)),
        }
    }

    /// How the vault is protected now and whether it is locked. A folder with
    /// no vault and no word is a key file (what the next save makes). A
    /// passphrase or keychain vault that cannot be read is told by the folder's
    /// word, and counts as locked (opening it says what is wrong).
    pub fn protection(&self) -> Protection {
        let held = self.hold();
        match self.header() {
            Ok(Some(Header { mode: Mode::Passphrase, kdf, .. })) => {
                let open = matches!((&held.unlocked, &kdf), (Some(unlocked), Some(kdf)) if unlocked.kdf() == kdf);
                Protection { mode: Mode::Passphrase, locked: !open }
            }
            Ok(Some(Header { mode: Mode::Keychain, key_id, .. })) => {
                let open = matches!((&held.keychain, key_id), (KeychainHold::Key(key), Some(id)) if key.id() == id);
                Protection { mode: Mode::Keychain, locked: !open }
            }
            Ok(Some(_)) => Protection { mode: Mode::KeyFile, locked: false },
            Ok(None) | Err(_) => match self.marked_mode() {
                Mode::KeyFile => Protection { mode: Mode::KeyFile, locked: false },
                mode => Protection { mode, locked: true },
            },
        }
    }

    /// The vault as it is on disk now, read under the lock (so never in the
    /// middle of another change). A folder with no vault is an empty vault; a
    /// locked one is `Locked`.
    pub fn load(&self) -> Result<Vault, VaultError> {
        let mut held = self.hold();
        self.open_folder()?;
        let source = self.source(&mut held)?;
        Vault::open_or_create(&self.dir, source)
    }

    /// Looks at the vault as it is on disk now; changes nothing.
    pub fn read<T>(&self, look: impl FnOnce(&Vault) -> T) -> Result<T, VaultError> {
        self.load().map(|vault| look(&vault))
    }

    /// Reads the vault from disk under the lock, lets `change` edit it, and
    /// saves it when `change` says (the `bool`) that it edited anything. Both
    /// locks (this handle's and the folder's) are held until the save is on
    /// disk. A vault that cannot be opened is an error and is never replaced;
    /// a save that fails leaves the old vault as it was.
    pub fn update<T>(&self, change: impl FnOnce(&mut Vault) -> (T, bool)) -> Result<T, VaultError> {
        let mut held = self.hold();
        let (mut lock, source) = self.begin(&mut held, false)?;
        let mut vault = Vault::open_or_create(&self.dir, source?)?;
        let (result, changed) = change(&mut vault);
        if changed {
            self.save_locked(&vault, &mut lock)?;
        }
        Ok(result)
    }

    /// As `update`, for a user who is giving a secret now. A key-file vault
    /// whose file or key is lost (see `VaultError::can_start_over`) is set
    /// aside as `vault.unreadable` (never over an earlier one: `.1`, `.2`...)
    /// and a new one started, and then `change` runs on an empty vault; with
    /// `start` false it is left where it is and the error is returned. **Only
    /// a key-file vault is ever started over by a save.** A passphrase vault
    /// (locked, wrong passphrase, missing or damaged) and a keychain vault (key
    /// not in the keychain, keychain out of reach, missing or damaged) fail
    /// with their own sentence and nothing is set aside: the prompt's own
    /// "forgot the passphrase" (`forget_passphrase`) and the explicit, confirmed
    /// "Start the vault over" (`start_vault_over`) are the only ways out. A
    /// vault that is started over is an empty key-file vault from then on.
    pub fn update_or_start_over<T>(
        &self,
        start: bool,
        change: impl FnOnce(&mut Vault) -> (T, bool),
    ) -> Result<T, VaultError> {
        let mut held = self.hold();
        let (mut lock, source) = self.begin(&mut held, false)?;
        let mut vault = self.open_or_start_over(&mut held, source, start)?;
        let (result, changed) = change(&mut vault);
        if changed {
            self.save_locked(&vault, &mut lock)?;
        }
        Ok(result)
    }

    fn open_or_start_over(
        &self,
        held: &mut Held,
        source: Result<KeySource, VaultError>,
        start: bool,
    ) -> Result<Vault, VaultError> {
        match source.and_then(|source| Vault::open_or_create(&self.dir, source)) {
            Err(error) if start && error.can_start_over() && self.effective_mode() == Mode::KeyFile => {
                held.keychain = KeychainHold::Nothing;
                Vault::start_over(&self.dir, KeySource::key_file_in(&self.dir))
            }
            other => other,
        }
    }

    /// Gives the passphrase of a passphrase vault: the key is made from it
    /// (about a second) and, when it opens the vault, kept in memory until
    /// `lock`. A passphrase that does not open it is `WrongPassphrase` and
    /// nothing is kept. A vault that is not protected by a passphrase, or is
    /// open already, is left as it is; a passphrase vault that is missing or
    /// damaged has nothing to open (`passphrase_vault_lost`).
    pub fn unlock(&self, passphrase: &Secret) -> Result<(), VaultError> {
        let mut held = self.hold();
        self.open_folder()?;
        match self.peek() {
            Some(Header { kdf: Some(kdf), .. }) => {
                if held.unlocked.as_ref().is_some_and(|unlocked| *unlocked.kdf() == kdf) {
                    return Ok(());
                }
                held.unlocked = Vault::open_with_passphrase(&self.dir, passphrase)?.unlocked();
                self.repair_marker(Mode::Passphrase);
                Ok(())
            }
            _ => self.passphrase_vault_lost().map_or(Ok(()), Err),
        }
    }

    /// Wipes the passphrase key and the keychain key held, if there are any,
    /// and forgets a keychain failure. The vault is locked again; secrets are
    /// unavailable until `unlock` (a keychain vault reads its key again).
    pub fn lock(&self) {
        let mut held = self.hold();
        held.unlocked = None;
        held.keychain = KeychainHold::Nothing;
    }

    /// Writes `vault` while the folder's lock is held (taken now, with the
    /// folder, when the change began in a folder that was not there yet: then
    /// the vault must still not be there, or another copy made it first), and
    /// records how it is protected.
    pub(super) fn save_locked(&self, vault: &Vault, lock: &mut Option<FileLock>) -> Result<(), VaultError> {
        if lock.is_none() {
            *lock = super::file_lock::acquire(&self.dir, true)?;
            if self.dir.join(VAULT_FILE_NAME).exists() {
                return Err(VaultError::LockBusy);
            }
        }
        vault.save()?;
        record(&self.dir, vault.mode());
        Ok(())
    }
}

#[cfg(test)]
include!("shared.tests.rs");
