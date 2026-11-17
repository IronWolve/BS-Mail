//! Setting a vault (or a key file) aside when a new one is started: it is
//! renamed, never deleted, and never over an earlier one. The first is
//! `vault.unreadable`, the next `vault.unreadable.1`, then `.2`, up to `.99`;
//! after that the person is told to clear some out.

use std::path::Path;

use super::error::VaultError;
use crate::private_fs;

/// The highest number a set-aside name is given.
pub(super) const MAX_NUMBER: u32 = 99;

/// The name a vault is set aside under first.
pub(super) const VAULT_ASIDE: &str = "vault.unreadable";

/// The name a key file is set aside under first.
pub(super) const KEY_ASIDE: &str = "key.unreadable";

/// The names a set-aside file may take, in order.
pub(super) fn candidates(base: &str) -> impl Iterator<Item = String> + '_ {
    std::iter::once(base.to_owned()).chain((1..=MAX_NUMBER).map(move |n| format!("{base}.{n}")))
}

/// Renames `name` in `dir` to the first free name of `base`, `base.1`, ...,
/// and says which it took; `None` when there was nothing there to set aside.
/// With `plain_only` a link or folder in the way is `NotAFile` and stays
/// where it is; without it, it is moved aside as it is (a rename never
/// follows a link and never reads what is inside).
pub(super) fn set_aside(dir: &Path, name: &str, base: &str, plain_only: bool) -> Result<Option<String>, VaultError> {
    let from = dir.join(name);
    match std::fs::symlink_metadata(&from) {
        Ok(meta) if meta.is_file() || !plain_only => {}
        Ok(_) => return Err(VaultError::NotAFile),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(VaultError::Read(error.to_string())),
    }
    for candidate in candidates(base) {
        match std::fs::symlink_metadata(dir.join(&candidate)) {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(VaultError::Read(error.to_string())),
        }
        std::fs::rename(&from, dir.join(&candidate)).map_err(|error| VaultError::Write(error.to_string()))?;
        let _ = private_fs::sync_folder(dir);
        return Ok(Some(candidate));
    }
    Err(VaultError::TooManySetAside)
}

#[cfg(test)]
include!("aside.tests.rs");
