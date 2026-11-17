//! A new sign-in that an older version was in the middle of when it stopped.
//!
//! Before the vault, a browser sign-in set the account's token file aside as
//! `<name>.prev` before it wrote the new one (an empty `.prev` when there was
//! none), and the next start put things back. Nothing writes those files any
//! more; this only finishes what an older run left, so the file that comes
//! back can then be moved into the vault like any other.

use std::path::{Path, PathBuf};

/// Where an older version set the token file aside.
pub fn backup_of(token_file: &Path) -> PathBuf {
    let mut name = token_file.as_os_str().to_owned();
    name.push(".prev");
    PathBuf::from(name)
}

/// Whether an older run left a sign-in half done for this token file.
pub fn is_pending(token_file: &Path) -> bool {
    std::fs::symlink_metadata(backup_of(token_file)).is_ok()
}

/// Puts the earlier sign-in back over the token file, or removes the new token
/// file when there was no earlier one (an empty `.prev`). Does nothing when no
/// run left a `.prev`.
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

fn remove_if_there(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
include!("legacy_attempt.tests.rs");
