//! Writing a file so that a crash leaves the old one or the new one, never
//! half of either: the text goes to a temporary file in the same folder and
//! the temporary file is renamed over the real one.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Replaces `path` with `text`. The folder is made when it is not there. A
/// file that is a link is written through, so the link stays a link. The new
/// file keeps the old one's permissions. No temporary file is left behind,
/// whether the write works or not.
pub(super) fn write(path: &Path, text: &str) -> io::Result<()> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let folder = target.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    crate::private_dirs::make(folder)?;
    let temporary = temporary_beside(&target);
    let result = write_then_rename(&temporary, &target, text);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_then_rename(temporary: &Path, target: &Path, text: &str) -> io::Result<()> {
    let mut file = File::create(temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    if let Ok(meta) = fs::metadata(target) {
        fs::set_permissions(temporary, meta.permissions())?;
    }
    fs::rename(temporary, target)
}

/// A name for the temporary file that is its own: the real name, then this
/// process's number.
fn temporary_beside(target: &Path) -> PathBuf {
    let name = target.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    target.with_file_name(format!(".{name}.{}.tmp", std::process::id()))
}

#[cfg(test)]
include!("atomic.tests.rs");
