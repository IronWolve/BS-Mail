//! The earlier sign-in, kept aside while a new one is tried.
//!
//! A new browser sign-in writes its tokens to the account's token file before
//! the mail server has accepted the account and before the account file is
//! saved. So that giving up (or a failed try, or a crash) can put things back,
//! the first write of an attempt moves the old token file to `<name>.prev`;
//! when there was none, an empty `.prev` says so. Nothing else is needed to
//! know that an attempt is in flight: if `.prev` exists, the token file is
//! not yet the account's own.
//!
//! - `keep` runs just before the new tokens are written. The first call of an
//!   attempt keeps; later ones leave the original in place.
//! - `restore` puts the old file back, or removes the new one when there was
//!   none. It does nothing when no attempt left a `.prev`.
//! - `discard` runs once the account is saved: the new tokens are the account's.

use std::path::{Path, PathBuf};

use crate::private_fs::{make_folder, write_new};

/// Where the earlier token file waits: the token file's name plus `.prev`.
pub fn backup_of(token_file: &Path) -> PathBuf {
    let mut name = token_file.as_os_str().to_owned();
    name.push(".prev");
    PathBuf::from(name)
}

/// Whether an attempt has replaced this token file and not yet finished.
pub fn is_pending(token_file: &Path) -> bool {
    std::fs::symlink_metadata(backup_of(token_file)).is_ok()
}

/// Sets the token file aside before a new sign-in replaces it. The old file is
/// moved, not copied, so no second copy of a secret is made; the caller holds
/// the token lock, so nobody reads the file in between. A `.prev` that is
/// already there belongs to an earlier try of the same attempt and is kept.
pub fn keep(token_file: &Path) -> Result<(), String> {
    let backup = backup_of(token_file);
    if is_pending(token_file) {
        return Ok(());
    }
    let fail = |what: &str, error: &dyn std::fmt::Display| {
        format!("cannot keep the earlier sign-in of {} aside ({what}): {error}", token_file.display())
    };
    match std::fs::rename(token_file, &backup) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let dir = token_file.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
            make_folder(dir).map_err(|e| fail("the folder", &e))?;
            write_new(&backup, b"").map_err(|e| fail("the marker", &e))
        }
        Err(error) => Err(fail("moving it", &error)),
    }
}

/// Puts the earlier sign-in back over the token file, or removes the new
/// token file when there was no earlier one. Does nothing when no attempt left
/// a `.prev`.
pub fn restore(token_file: &Path) -> Result<(), String> {
    let backup = backup_of(token_file);
    let size = match std::fs::symlink_metadata(&backup) {
        Ok(meta) => meta.len(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot look at {}: {error}", backup.display())),
    };
    let put_back = if size == 0 {
        // There was no earlier sign-in: the new one goes, and then the marker.
        remove_if_there(token_file).and_then(|()| remove_if_there(&backup))
    } else {
        std::fs::rename(&backup, token_file)
    };
    put_back.map_err(|error| format!("cannot put the earlier sign-in back at {}: {error}", token_file.display()))
}

/// The new tokens are the account's own: the earlier ones are not needed.
pub fn discard(token_file: &Path) {
    let _ = remove_if_there(&backup_of(token_file));
}

fn remove_if_there(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
include!("previous_token.tests.rs");
