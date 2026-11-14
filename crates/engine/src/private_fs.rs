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
