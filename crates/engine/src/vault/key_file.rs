//! The key file, `<data dir>/secrets/key`: 32 random bytes. It is made once,
//! with a name that must not exist yet and owner-only permissions (Unix), in a
//! folder made owner-only; a link or a file of any other length is an error
//! and is never replaced. Windows has no mode bits, so there the folder and
//! file keep the permissions the system gives them; nothing here pretends
//! otherwise.

use std::io::Read;
use std::path::Path;

use super::crypto::random;
use super::error::VaultError;
use super::key_source::MasterKey;
use crate::private_fs;

pub(super) const KEY_LEN: usize = 32;

/// The key in the file, or `None` when there is no file.
pub(super) fn read(path: &Path) -> Result<Option<MasterKey>, VaultError> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(VaultError::Read(error.to_string())),
    };
    if meta.file_type().is_symlink() {
        return Err(VaultError::BadKeyFile("is a link, not a file"));
    }
    if !meta.is_file() {
        return Err(VaultError::BadKeyFile("is not a plain file"));
    }
    let file = std::fs::File::open(path).map_err(|error| VaultError::Read(error.to_string()))?;
    check_handle(&file)?;
    let mut bytes = Vec::with_capacity(KEY_LEN + 1);
    // One byte more than a key, so a longer file is seen as longer.
    file.take(KEY_LEN as u64 + 1).read_to_end(&mut bytes).map_err(|error| VaultError::Read(error.to_string()))?;
    let key = MasterKey::from_slice(&bytes);
    model::wipe_bytes(&mut bytes);
    key.map(Some).ok_or(VaultError::BadKeyFile("is not 32 bytes long"))
}

/// What was opened, not what the name was when it was looked at, must be a
/// plain file.
fn check_handle(file: &std::fs::File) -> Result<(), VaultError> {
    if !file.metadata().map_err(|error| VaultError::Read(error.to_string()))?.is_file() {
        return Err(VaultError::BadKeyFile("is not a plain file"));
    }
    Ok(())
}

/// The key in the file, made first when there is no file.
pub(super) fn read_or_create(path: &Path) -> Result<MasterKey, VaultError> {
    if let Some(key) = read(path)? {
        return Ok(key);
    }
    if let Some(folder) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        private_fs::make_folder(folder).map_err(|error| VaultError::Write(error.to_string()))?;
    }
    let mut bytes: [u8; KEY_LEN] = random()?;
    let made = private_fs::write_new_or_remove(path, &bytes);
    let key = MasterKey::from_slice(&bytes);
    model::wipe_bytes(&mut bytes);
    match made {
        Ok(()) => {
            if let Some(folder) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
                let _ = private_fs::sync_folder(folder);
            }
            key.ok_or(VaultError::Crypto)
        }
        // Someone made the key in the meantime: theirs is the key.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read(path)?.ok_or(VaultError::KeyMissing),
        Err(error) => Err(VaultError::Write(error.to_string())),
    }
}

#[cfg(test)]
include!("key_file.tests.rs");
