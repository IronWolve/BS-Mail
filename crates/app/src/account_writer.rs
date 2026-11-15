//! Saving the account file the setup form made: whole or not at all, readable
//! only by the user, and without losing a file that was already there.
//!
//! The text goes into a new file beside the target first. An old file is
//! kept as one backup, `.bak`, with its passwords and secrets left out: a
//! backup is for getting the settings back, and an old password, often reused
//! elsewhere, must not stay on disk for ever. Each save replaces the backup of
//! the one before. Only then does the new file take the target's place, in one
//! rename, so a crash leaves the old file or the new one and never half of
//! either.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::private_dirs;

/// The keys of the account file that hold a secret.
const SECRET_KEYS: [&str; 3] = ["password", "smtp_password", "oauth_client_secret"];

/// What saving did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    /// Where the backup of an account file that was already there is, if
    /// there was one.
    pub kept_old_as: Option<PathBuf>,
}

pub fn save_account(target: &Path, text: &str) -> Result<Saved, String> {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} is not a file name", target.display()))?;
    let dir = target.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    private_dirs::make(dir).map_err(|error| failed(dir, "make the folder", &error))?;
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

/// Keeps what the old target held, without its secrets, as `<name>.bak`,
/// replacing the backup of the save before.
fn keep_old(target: &Path, dir: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    if !target.exists() {
        return Ok(None);
    }
    let old = fs::read_to_string(target).map_err(|error| failed(target, "read the old file to keep a copy of", &error))?;
    let backup = dir.join(format!("{name}.bak"));
    let fresh = dir.join(format!("{name}.bak.new"));
    write_private(&fresh, &without_secrets(&old))?;
    fs::rename(&fresh, &backup).map_err(|error| {
        let _ = fs::remove_file(&fresh);
        failed(&backup, "keep the old file as", &error)
    })?;
    Ok(Some(backup))
}

/// The text without the lines that hold a secret: a password, an app
/// password or a client secret, and any comment that mentions one.
fn without_secrets(text: &str) -> String {
    text.split_inclusive('\n')
        .filter(|line| {
            let key = line.split('=').next().unwrap_or("").trim();
            let comment = line.trim_start().starts_with('#');
            let mentions = |word: &str| line.to_ascii_lowercase().contains(word);
            !SECRET_KEYS.contains(&key) && !(comment && (mentions("password") || mentions("secret")))
        })
        .collect()
}

fn failed(path: &Path, what: &str, error: &std::io::Error) -> String {
    format!("could not {what} {}: {error}", path.display())
}

#[cfg(test)]
include!("account_writer.tests.rs");
