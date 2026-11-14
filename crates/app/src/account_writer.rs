//! Saving the account file the setup form made: whole or not at all, readable
//! only by the user, and without losing a file that was already there.
//!
//! The text goes into a new file beside the target first. An old file is
//! kept under a name nothing else uses (`.bak`, then `.bak1`, `.bak2`…), by a
//! hard link that fails rather than replace one; only then does the new file
//! take the target's place, in one rename, so a crash leaves the old file or
//! the new one and never half of either.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What saving did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    /// Where an account file that was already there went, if there was one.
    pub kept_old_as: Option<PathBuf>,
}

pub fn save_account(target: &Path, text: &str) -> Result<Saved, String> {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} is not a file name", target.display()))?;
    let dir = target.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir).map_err(|error| failed(dir, "make the folder", &error))?;
    let fresh = dir.join(format!("{name}.new"));
    write_private(&fresh, text)?;
    let kept = match keep_old(target, dir, name) {
        Ok(kept) => kept,
        Err(reason) => {
            let _ = fs::remove_file(&fresh);
            return Err(reason);
        }
    };
    fs::rename(&fresh, target).map_err(|error| {
        let _ = fs::remove_file(&fresh);
        failed(target, "put the new file in place", &error)
    })?;
    Ok(Saved { kept_old_as: kept })
}

/// A new file with the text in it that nobody else can read, replacing a
/// leftover of an earlier try at the same name.
fn write_private(path: &Path, text: &str) -> Result<(), String> {
    if path.is_file() {
        fs::remove_file(path).map_err(|error| failed(path, "clear an earlier try", &error))?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path).map_err(|error| failed(path, "create", &error))?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            let _ = fs::remove_file(path);
            failed(path, "write", &error)
        })
}

/// Links an existing target under a free backup name.
fn keep_old(target: &Path, dir: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    if !target.exists() {
        return Ok(None);
    }
    for attempt in 0..100u32 {
        let suffix = if attempt == 0 { "bak".to_string() } else { format!("bak{attempt}") };
        let backup = dir.join(format!("{name}.{suffix}"));
        match fs::hard_link(target, &backup) {
            Ok(()) => return Ok(Some(backup)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(failed(&backup, "keep the old file as", &error)),
        }
    }
    Err(format!("too many old copies of {name} beside it; remove some and try again"))
}

fn failed(path: &Path, what: &str, error: &std::io::Error) -> String {
    format!("could not {what} {}: {error}", path.display())
}

#[cfg(test)]
include!("account_writer.tests.rs");
