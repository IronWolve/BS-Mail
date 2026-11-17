//! Reading and replacing the vault file: a read that refuses a link, a file
//! that is not plain or is over the size limit, and a replacement that writes
//! a temporary file beside it, flushes it and renames it over the old one, so
//! a save that fails leaves the old vault as it was.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::error::VaultError;
use super::format::MAX_VAULT_BYTES;
use crate::private_fs;

/// The temporary name a save writes under, beside the vault.
pub(super) fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// The vault file's bytes, or `None` when there is no file.
pub(super) fn read_limited(path: &Path) -> Result<Option<Vec<u8>>, VaultError> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(VaultError::Read(error.to_string())),
    };
    if !meta.is_file() {
        return Err(VaultError::NotAFile);
    }
    if meta.len() > MAX_VAULT_BYTES {
        return Err(VaultError::TooBig);
    }
    let file = std::fs::File::open(path).map_err(|error| VaultError::Read(error.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_VAULT_BYTES + 1).read_to_end(&mut bytes).map_err(|error| VaultError::Read(error.to_string()))?;
    if bytes.len() as u64 > MAX_VAULT_BYTES {
        return Err(VaultError::TooBig);
    }
    Ok(Some(bytes))
}

/// Puts `bytes` in place of the file at `path`, all or nothing.
pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), VaultError> {
    let fail = |error: std::io::Error| VaultError::Write(error.to_string());
    if bytes.len() as u64 > MAX_VAULT_BYTES {
        return Err(VaultError::TooBig);
    }
    let folder = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    private_fs::make_folder(folder).map_err(fail)?;
    let temp = temp_path(path);
    // A leftover of a save that was cut off: its text is only ciphertext, and
    // it is in the way of the new one.
    if std::fs::symlink_metadata(&temp).is_ok_and(|meta| !meta.is_dir()) {
        std::fs::remove_file(&temp).map_err(fail)?;
    }
    private_fs::write_new_or_remove(&temp, bytes).map_err(fail)?;
    if let Err(error) = std::fs::rename(&temp, path) {
        let _ = std::fs::remove_file(&temp);
        return Err(fail(error));
    }
    let _ = private_fs::sync_folder(folder);
    Ok(())
}

#[cfg(test)]
include!("file_io.tests.rs");
