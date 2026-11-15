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
/// without any comment that mentions one, whether it has a line of its own or
/// follows a setting on the same line; `None` when the text is not TOML, or
/// when what is left is not (nothing is kept that cannot be told to be clean).
fn without_secrets(text: &str) -> Option<String> {
    let mut document: DocumentMut = text.parse().ok()?;
    strip_secrets(document.as_table_mut());
    let redacted: String = without_secret_trailing_comments(&document.to_string())
        .split_inclusive('\n')
        .filter(|line| !(line.trim_start().starts_with('#') && mentions_a_secret(line)))
        .collect();
    redacted.parse::<DocumentMut>().ok()?;
    Some(redacted)
}

/// Where the scan of a TOML text is: in plain text, or inside one of the four
/// kinds of string, where a `#` is text and not a comment.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scan {
    Plain,
    Basic,
    Literal,
    MultiBasic,
    MultiLiteral,
}

/// `text` without the comments that follow a setting on its line and mention
/// a password or a secret. Only the comment goes, and the spaces before it;
/// the setting and the rest of the file stay. A comment on a line of its own
/// is left for the caller, and a `#` inside a string is never a comment. `text`
/// is TOML that already parsed.
fn without_secret_trailing_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut state, mut at, mut content) = (Scan::Plain, 0, false);
    while at < chars.len() {
        let ch = chars[at];
        let run = chars[at..].iter().take_while(|next| **next == ch).count();
        match (state, ch) {
            (Scan::Plain, '#') => {
                let end = chars[at..].iter().position(|c| *c == '\n' || *c == '\r').map_or(chars.len(), |n| at + n);
                let comment: String = chars[at..end].iter().collect();
                if content && mentions_a_secret(&comment) {
                    out.truncate(out.trim_end_matches([' ', '\t']).len());
                } else {
                    out.push_str(&comment);
                }
                at = end;
                continue;
            }
            (Scan::Plain, '\n') => content = false,
            (Scan::Plain, '"' | '\'') => {
                let multi = run >= 3;
                state = match (ch, multi) {
                    ('"', true) => Scan::MultiBasic,
                    ('"', false) => Scan::Basic,
                    (_, true) => Scan::MultiLiteral,
                    _ => Scan::Literal,
                };
                let take = if multi { 3 } else { 1 };
                out.extend(&chars[at..at + take]);
                (at, content) = (at + take, true);
                continue;
            }
            (Scan::Basic | Scan::MultiBasic, '\\') if at + 1 < chars.len() => {
                out.extend(&chars[at..at + 2]);
                at += 2;
                continue;
            }
            (Scan::Basic, '"') | (Scan::Literal, '\'') => state = Scan::Plain,
            (Scan::MultiBasic, '"') | (Scan::MultiLiteral, '\'') if run >= 3 => {
                // Up to two quotes may sit just before the closing three.
                out.extend(&chars[at..at + run]);
                (state, at, content) = (Scan::Plain, at + run, true);
                continue;
            }
            (Scan::MultiBasic, '"') | (Scan::MultiLiteral, '\'') => {
                out.extend(&chars[at..at + run]);
                at += run;
                continue;
            }
            _ => {}
        }
        content |= !ch.is_whitespace();
        out.push(ch);
        at += 1;
    }
    out
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
