//! Deleting an account's folder.
//!
//! The Options screen promises that removing an account deletes its whole
//! folder, saved sign-in and backups of the account file included, and asks
//! first. This is that deletion. It only ever touches one exact path: the
//! folder named by a valid id directly inside the accounts folder, checked to
//! be a real folder and not a link. Nothing is matched by pattern, glob or
//! age.

use std::fs;
use std::path::{Path, PathBuf};

use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;

/// The one folder that holds account `id` under `root`, when it is a real
/// folder there; otherwise why it is not.
pub fn checked_folder(root: &Path, id: &str) -> Result<PathBuf, String> {
    if !is_valid_id(id) {
        return Err(format!("\"{id}\" is not the id of an account"));
    }
    let folder = root.join(id);
    let meta = fs::symlink_metadata(&folder).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => format!("there is no folder for the account \"{id}\""),
        _ => format!("could not look at the folder of the account \"{id}\": {error}"),
    })?;
    if meta.file_type().is_symlink() {
        return Err(format!(
            "the folder of the account \"{id}\" is a link, not a folder; remove the link by hand if you want it gone"
        ));
    }
    if !meta.is_dir() {
        return Err(format!("\"{id}\" in the accounts folder is not a folder"));
    }
    Ok(folder)
}

/// Deletes the folder of account `id` and everything in it.
pub fn remove(root: &Path, id: &str) -> Result<(), String> {
    let folder = checked_folder(root, id)?;
    fs::remove_dir_all(&folder).map_err(|error| format!("could not delete the folder {}: {error}", folder.display()))
}

/// Deletes the folder of `id` only when no account file was ever saved in it:
/// what an abandoned attempt leaves behind, such as a saved sign-in. Returns
/// whether anything was deleted. A folder with an account file is never
/// touched.
pub fn remove_unsaved(root: &Path, id: &str) -> Result<bool, String> {
    let folder = match checked_folder(root, id) {
        Ok(folder) => folder,
        Err(_) if !root.join(id).exists() => return Ok(false),
        Err(reason) => return Err(reason),
    };
    if folder.join(ACCOUNT_FILE).exists() {
        return Ok(false);
    }
    fs::remove_dir_all(&folder).map_err(|error| format!("could not delete the folder {}: {error}", folder.display()))?;
    Ok(true)
}

#[cfg(test)]
include!("remove.tests.rs");
