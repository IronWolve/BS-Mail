//! Reading and replacing the vault file: a read that refuses a link, a file
//! that is not plain or is over the size limit, and a replacement that writes
//! a temporary file beside it, flushes it and renames it over the old one, so
//! a save that fails leaves the old vault as it was.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::error::VaultError;
use super::format::MAX_VAULT_BYTES;
use super::inject::{Step, reach};
use crate::private_fs;

/// The temporary name a save writes under, beside the vault.
pub(super) fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// Opens the plain file at `path` for reading, or `None` when there is no
/// file. A link, or anything that is not a plain file, is refused by name
/// first and then again by the handle that was opened (what was opened is
/// what is read, whatever the name was swapped for in between); with `limit`,
/// a file over it is `TooBig` before any of it is read.
fn open_plain(path: &Path, limit: Option<u64>) -> Result<Option<std::fs::File>, VaultError> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(VaultError::Read(error.to_string())),
    };
    if !meta.is_file() {
        return Err(VaultError::NotAFile);
    }
    let file = std::fs::File::open(path).map_err(|error| VaultError::Read(error.to_string()))?;
    check_handle(&file, limit)?;
    Ok(Some(file))
}

/// Refuses an open file that is not a plain file, or is over `limit`: what the
/// handle is, which is what will be read, whatever its name was a moment ago.
fn check_handle(file: &std::fs::File, limit: Option<u64>) -> Result<(), VaultError> {
    let opened = file.metadata().map_err(|error| VaultError::Read(error.to_string()))?;
    if !opened.is_file() {
        return Err(VaultError::NotAFile);
    }
    if limit.is_some_and(|limit| opened.len() > limit) {
        return Err(VaultError::TooBig);
    }
    Ok(())
}

/// The vault file's bytes, or `None` when there is no file.
pub(super) fn read_limited(path: &Path) -> Result<Option<Vec<u8>>, VaultError> {
    let Some(file) = open_plain(path, Some(MAX_VAULT_BYTES))? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    file.take(MAX_VAULT_BYTES + 1).read_to_end(&mut bytes).map_err(|error| VaultError::Read(error.to_string()))?;
    if bytes.len() as u64 > MAX_VAULT_BYTES {
        return Err(VaultError::TooBig);
    }
    Ok(Some(bytes))
}

/// The first `limit` bytes of the file, or `None` when there is no file; the
/// same refusals as `read_limited` for a link or something that is not a
/// plain file. For looking at a header without reading the whole vault.
pub(super) fn read_prefix(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, VaultError> {
    let Some(file) = open_plain(path, None)? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    file.take(limit).read_to_end(&mut bytes).map_err(|error| VaultError::Read(error.to_string()))?;
    Ok(Some(bytes))
}

/// Puts `bytes` in place of the file at `path`, all or nothing. `Ok(None)`
/// means the new file is in place and its folder entry is flushed to disk;
/// `Ok(Some(why))` that it is in place but the folder could not be flushed (the
/// system's words), so a power loss could still bring the old file back: a
/// caller that is about to remove what the old file needed keeps it then.
pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<Option<String>, VaultError> {
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
    reach(Step::WriteTemp)?;
    private_fs::write_new_or_remove(&temp, bytes).map_err(fail)?;
    let renamed = reach(Step::Rename).and_then(|()| std::fs::rename(&temp, path).map_err(fail));
    if let Err(error) = renamed {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    Ok(sync_folder(folder).err())
}

/// Flushes `folder`; the system's words when it could not be.
pub(super) fn sync_folder(folder: &Path) -> Result<(), String> {
    reach(Step::SyncVault).map_err(|error| error.to_string())?;
    private_fs::sync_folder(folder).map_err(|error| error.to_string())
}

#[cfg(test)]
include!("file_io.tests.rs");
