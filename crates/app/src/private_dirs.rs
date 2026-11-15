//! Making folders only their owner can open. The data folder lists the
//! accounts and when their files change, and holds the sign-in tokens, so a
//! folder this program makes is private from the start. A folder that was
//! there already keeps the permissions it has: it is not this program's to
//! change.

use std::io;
use std::path::Path;

/// Makes `dir` and any missing parents, each with owner-only permissions where
/// the system has them.
#[cfg(unix)]
pub fn make(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

#[cfg(not(unix))]
pub fn make(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(test)]
include!("private_dirs.tests.rs");
