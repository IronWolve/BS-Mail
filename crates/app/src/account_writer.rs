//! Saving the account file the setup form made: whole or not at all, readable
//! only by the user, and without losing a file that was already there.
//!
//! The text goes into a new file beside the target first. An old file is
//! kept as one backup, `.bak`, with its passwords and secrets left out: a
//! backup is for getting the settings back, and an old password, often reused
//! elsewhere, must not stay on disk for ever. The old text is read as TOML and
//! every entry whose key names a password or a secret is removed at any depth,
//! however it is written (quoted key, value over several lines); a file that
//! is not TOML cannot be redacted with certainty, so no backup is kept of it.
//! Each save replaces the backup of the one before. Only then does the new
//! file take the target's place, in one rename, so a crash leaves the old file
//! or the new one and never half of either.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, TableLike, Value};

use crate::private_dirs;

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
    let Some(kept) = without_secrets(&old) else {
        // Not TOML, so what is a secret in it cannot be told: no copy.
        return Ok(None);
    };
    let backup = dir.join(format!("{name}.bak"));
    let fresh = dir.join(format!("{name}.bak.new"));
    write_private(&fresh, &kept)?;
    fs::rename(&fresh, &backup).map_err(|error| {
        let _ = fs::remove_file(&fresh);
        failed(&backup, "keep the old file as", &error)
    })?;
    Ok(Some(backup))
}

/// The text without the entries that hold a secret (a password, an app
/// password, a client secret: any key that names one, at any depth) and
/// without any comment that mentions one; `None` when the text is not TOML.
fn without_secrets(text: &str) -> Option<String> {
    let mut document: DocumentMut = text.parse().ok()?;
    strip_secrets(document.as_table_mut());
    let redacted: String = document
        .to_string()
        .split_inclusive('\n')
        .filter(|line| !(line.trim_start().starts_with('#') && mentions_a_secret(line)))
        .collect();
    Some(redacted)
}

/// Removes the entries of `table` that hold a secret, and looks inside the
/// others.
fn strip_secrets(table: &mut dyn TableLike) {
    let keys: Vec<String> = table.iter().map(|(key, _)| key.to_owned()).collect();
    for key in keys {
        if mentions_a_secret(&key) {
            table.remove(&key);
            continue;
        }
        match table.get_mut(&key) {
            Some(Item::Table(inner)) => strip_secrets(inner),
            Some(Item::Value(Value::InlineTable(inner))) => strip_secrets(inner),
            Some(Item::ArrayOfTables(tables)) => tables.iter_mut().for_each(|inner| strip_secrets(inner)),
            _ => {}
        }
    }
}

/// Whether `text` names a password or a secret, in any letter case.
fn mentions_a_secret(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("password") || lower.contains("secret")
}

fn failed(path: &Path, what: &str, error: &std::io::Error) -> String {
    format!("could not {what} {}: {error}", path.display())
}

#[cfg(test)]
include!("account_writer.tests.rs");
