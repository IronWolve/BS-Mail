//! What the secrets folder and the files in it must be before the program
//! uses them: the folder is a real folder, not a link, and (Unix) it is
//! owner-only, as the vault and the key file are. A folder restored from a
//! backup or a copy often comes back at 0755 and 0644; the account and token
//! files are refused when they are like that, and here the program instead
//! tightens them (0700, 0600) and says so once, or refuses with a sentence
//! when it cannot. Windows has no mode bits and nothing is done there: it
//! keeps the permissions the system gives the folder (documented in the plan).

use std::path::Path;

use super::error::VaultError;
#[cfg(unix)]
use super::key_source::KEY_FILE_NAME;
#[cfg(unix)]
use super::store::VAULT_FILE_NAME;

/// What is said once when the folder was tightened.
#[cfg(unix)]
pub(super) const TIGHTENED_FOLDER: &str = "Tightened the permissions of the secrets folder";
#[cfg(unix)]
pub(super) const TIGHTENED_VAULT: &str = "Tightened the permissions of the vault file";
#[cfg(unix)]
pub(super) const TIGHTENED_KEY: &str = "Tightened the permissions of the vault key file";

/// Checks `dir` and what is in it; the sentences for everything that was
/// tightened. A folder that is not there yet is fine.
pub(super) fn check(dir: &Path) -> Result<Vec<&'static str>, VaultError> {
    match std::fs::symlink_metadata(dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(VaultError::Read(error.to_string())),
        Ok(meta) if meta.file_type().is_symlink() => return Err(VaultError::BadFolder("is a link, not a folder")),
        Ok(meta) if !meta.is_dir() => return Err(VaultError::BadFolder("is not a folder")),
        Ok(_) => {}
    }
    tighten_all(dir)
}

#[cfg(not(unix))]
fn tighten_all(_dir: &Path) -> Result<Vec<&'static str>, VaultError> {
    Ok(Vec::new())
}

#[cfg(unix)]
fn tighten_all(dir: &Path) -> Result<Vec<&'static str>, VaultError> {
    let mut said = Vec::new();
    let parts: [(std::path::PathBuf, u32, &str, &str, bool); 3] = [
        (dir.to_path_buf(), 0o700, "the secrets folder", TIGHTENED_FOLDER, true),
        (dir.join(VAULT_FILE_NAME), 0o600, "the vault file", TIGHTENED_VAULT, false),
        (dir.join(KEY_FILE_NAME), 0o600, "the vault key file", TIGHTENED_KEY, false),
    ];
    for (path, mode, what, note, is_folder) in parts {
        if tighten(&path, mode, what, is_folder)? {
            said.push(note);
        }
    }
    Ok(said)
}

/// Sets `path` to `mode` when anyone but its owner can use it; whether it did.
/// Only a plain file (or the folder itself): a link or odd entry is left for
/// the readers, which refuse it in their own words.
#[cfg(unix)]
fn tighten(path: &Path, mode: u32, what: &'static str, is_folder: bool) -> Result<bool, VaultError> {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return Ok(false);
    };
    if !(if is_folder { meta.is_dir() } else { meta.is_file() }) || meta.permissions().mode() & 0o077 == 0 {
        return Ok(false);
    }
    let set = super::inject::reach(super::inject::Step::Tighten)
        .map_err(|_| ())
        .and_then(|()| std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|_| ()));
    set.map(|()| true).map_err(|()| VaultError::Permissions(what))
}

#[cfg(test)]
include!("permissions.tests.rs");
