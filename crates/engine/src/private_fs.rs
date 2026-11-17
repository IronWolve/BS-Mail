//! Files and folders only their owner can use: the folder is made with
//! owner-only permissions where the system has them, and a new file is
//! written whole, to a name that does not exist yet, then flushed to disk.
//! Callers write to a temporary name and rename it into place.

use std::io::Write;
use std::path::Path;

/// Makes `dir` and any missing parents.
#[cfg(unix)]
pub(crate) fn make_folder(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

#[cfg(not(unix))]
pub(crate) fn make_folder(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(unix)]
fn open_new(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn open_new(path: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(path)
}

/// Writes `bytes` to a new file at `path`; an existing file is an error.
pub(crate) fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = open_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// As `write_new`, and a file this call made but could not finish writing is
/// removed again, so a failed write leaves no half file behind. A file that
/// was there already is an error and is left as it is.
pub(crate) fn write_new_or_remove(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = open_new(path)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    if written.is_err() {
        drop(file);
        let _ = std::fs::remove_file(path);
    }
    written
}

/// Flushes the folder's own entry list, so a file renamed into it a moment
/// ago is still there after a power loss. Folders cannot be opened for this on
/// every system; where they cannot, there is nothing to do.
#[cfg(unix)]
pub(crate) fn sync_folder(dir: &Path) -> std::io::Result<()> {
    std::fs::File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
pub(crate) fn sync_folder(_dir: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
include!("private_fs.tests.rs");
