//! One running copy per data folder: a lock on the `lock` file inside it,
//! held until the run ends. The lock belongs to the operating system, so a
//! crash lets go of it and a copied `lock` file is harmless.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{ErrorKind, Read, Seek, Write};
use std::path::Path;

use chrono::{Local, TimeZone};

use crate::paths::LOCK_FILE;
use crate::private_dirs;

/// The lock on a data folder, held for as long as this value lives.
#[derive(Debug)]
pub struct DataLock {
    _file: File,
}

/// Why a copy may not use the data folder.
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    /// Another copy already holds the folder; `holder` says who, as far as
    /// the lock file could tell.
    Busy { holder: String },
    /// The folder can be written but no lock can be had on it (the lock file
    /// cannot be opened, or the file system has no locks), so nothing would
    /// keep two copies from writing the same files at once.
    Unlockable { reason: String },
}

/// Takes the lock on `folder`, making the folder if it is not there yet.
///
/// `Ok(None)` means the folder cannot be written at all (it cannot be made,
/// or the lock file may not be created in it): nothing can then be damaged by
/// two copies, and the run goes on without a lock, as before. A lock that is
/// held elsewhere refuses, and so does a folder that can be written but
/// cannot be locked: failing open there would let two copies race on the
/// account and token files.
pub fn take(folder: &Path) -> Result<Option<DataLock>, Refused> {
    if private_dirs::make(folder).is_err() {
        return Ok(None);
    }
    let path = folder.join(LOCK_FILE);
    let opened = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path);
    let mut file = match opened {
        Ok(file) => file,
        Err(error) if matches!(error.kind(), ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem) => {
            return Ok(None);
        }
        Err(error) => {
            return Err(Refused::Unlockable { reason: format!("the lock file {} cannot be opened: {error}", path.display()) });
        }
    };
    match file.try_lock() {
        Ok(()) => {
            write_holder(&mut file);
            Ok(Some(DataLock { _file: file }))
        }
        Err(TryLockError::WouldBlock) => Err(Refused::Busy { holder: read_holder(&path) }),
        Err(TryLockError::Error(error)) => {
            Err(Refused::Unlockable { reason: format!("this file system cannot lock the folder: {error}") })
        }
    }
}

/// Records this process in the lock file, for the message a second copy gets.
/// Best effort: the lock is what counts.
fn write_holder(file: &mut File) {
    let started = Local::now().timestamp();
    let text = format!("pid {}\nstarted {started}\n", std::process::id());
    let _ = file.set_len(0);
    let _ = file.rewind();
    let _ = file.write_all(text.as_bytes());
    let _ = file.flush();
}

/// Who holds the folder, in words; empty when the lock file cannot be read
/// (some systems keep a locked file from being read by anyone else).
fn read_holder(path: &Path) -> String {
    let mut text = String::new();
    let read = File::open(path).and_then(|mut file| file.read_to_string(&mut text));
    if read.is_err() {
        return String::new();
    }
    describe(&text)
}

/// "process 1234, started 21:58" from the two lines written above.
fn describe(text: &str) -> String {
    let field = |name: &str| {
        text.lines().find_map(|line| line.strip_prefix(name)?.trim().parse::<i64>().ok())
    };
    let (Some(pid), started) = (field("pid "), field("started ")) else {
        return String::new();
    };
    let at = started.and_then(|secs| Local.timestamp_opt(secs, 0).single());
    match at {
        Some(time) => format!("process {pid}, started {}", time.format("%Y-%m-%d %H:%M")),
        None => format!("process {pid}"),
    }
}

#[cfg(test)]
include!("data_lock.tests.rs");
