//! A lock on the secrets folder that every copy of the program respects:
//! `secrets/lock`, an owner-only file the operating system locks. The shared
//! handle (`shared.rs`) keeps changes of one process apart; this keeps the
//! changes of two processes apart (two runs on one folder, `--account <file>`
//! runs take no data lock), so one never overwrites a token or password the
//! other has just saved. It is held for the file work of one read, change and
//! save only, never across a network call or a keychain wait, and the
//! operating system lets go of it when the process ends, whatever way.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::time::{Duration, Instant};

use super::error::VaultError;
use super::inject::lock_fault;
use super::store::VAULT_FILE_NAME;
use crate::private_fs;

/// The lock file's name inside the secrets folder.
pub const LOCK_NAME: &str = "lock";

/// How long a change waits for another copy to finish its own: far longer
/// than the file work it guards. The same in every build, so a slow disk
/// under a test cannot make a busy folder look stuck.
const WAIT: Duration = Duration::from_secs(10);

#[cfg(test)]
thread_local! {
    static SHORT_WAIT: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// For a test of the refusal only: while `run` runs, changes made on THIS thread wait no longer than
/// `wait`, so the refusal comes quickly; every other thread and test keeps the full wait.
#[cfg(test)]
pub(super) fn with_wait<T>(wait: Duration, run: impl FnOnce() -> T) -> T {
    let before = SHORT_WAIT.replace(Some(wait));
    let result = run();
    SHORT_WAIT.set(before);
    result
}

/// The wait a change gives another copy on this thread now.
fn wait_limit() -> Duration {
    #[cfg(test)]
    if let Some(short) = SHORT_WAIT.get() {
        return short;
    }
    WAIT
}

/// How often a waiting change looks again.
const STEP: Duration = Duration::from_millis(10);

/// The lock, held for as long as this value lives.
#[derive(Debug)]
pub(super) struct FileLock {
    _file: File,
}

#[cfg(unix)]
fn open(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn open(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)
}

fn cannot(why: impl std::fmt::Display) -> VaultError {
    VaultError::NoLock(why.to_string())
}

/// Whether there is nothing in `dir` yet that a lock would protect: no folder,
/// or a folder with neither a lock file nor a vault. (A change that then does
/// save takes the lock first, and checks that nobody made the vault meanwhile.)
fn nothing_to_protect(dir: &Path) -> bool {
    std::fs::symlink_metadata(dir).is_err()
        || (std::fs::symlink_metadata(dir.join(LOCK_NAME)).is_err() && std::fs::symlink_metadata(dir.join(VAULT_FILE_NAME)).is_err())
}

/// Takes the lock on the secrets folder `dir`. With `create` the folder (and
/// the lock file) are made when they are not there; without it, a folder that
/// has no vault in it yet has nothing to protect and there is nothing to lock
/// (`Ok(None)`: so a change that edits nothing writes nothing), and neither
/// does a folder nothing can be written to (a change could not be saved there
/// anyway). A file system that has no locks
/// is `NoLock`, and a lock another copy keeps past `WAIT` is `LockBusy`.
pub(super) fn acquire(dir: &Path, create: bool) -> Result<Option<FileLock>, VaultError> {
    if !create && nothing_to_protect(dir) {
        return Ok(None);
    }
    if create {
        private_fs::make_folder(dir).map_err(|error| VaultError::Write(error.to_string()))?;
    }
    let path = dir.join(LOCK_NAME);
    if std::fs::symlink_metadata(&path).is_ok_and(|found| !found.is_file()) {
        // A link would send the lock, and anything written, to whatever it points at.
        return Err(cannot("the lock file is a link or not a plain file"));
    }
    let file = match open(&path) {
        Ok(file) => file,
        Err(error) if !create && matches!(error.kind(), std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem) => {
            return Ok(None);
        }
        Err(error) => return Err(cannot(format!("the lock file cannot be opened: {error}"))),
    };
    if !file.metadata().is_ok_and(|meta| meta.is_file()) {
        return Err(cannot("the lock file is not a plain file"));
    }
    wait_for(file).map(Some)
}

/// Locks `file`, waiting a little for another copy to let go.
fn wait_for(file: File) -> Result<FileLock, VaultError> {
    let started = Instant::now();
    let limit = wait_limit();
    loop {
        if let Some(error) = lock_fault() {
            return Err(cannot(error));
        }
        match file.try_lock() {
            Ok(()) => return Ok(FileLock { _file: file }),
            Err(TryLockError::WouldBlock) if started.elapsed() < limit => std::thread::sleep(STEP),
            Err(TryLockError::WouldBlock) => return Err(VaultError::LockBusy),
            Err(TryLockError::Error(error)) => return Err(cannot(error)),
        }
    }
}

#[cfg(test)]
include!("file_lock.tests.rs");
