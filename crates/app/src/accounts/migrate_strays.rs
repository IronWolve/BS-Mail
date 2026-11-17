//! Old files in an account's folder that may still hold a secret after the
//! move: backups of the account file (`account.toml.bak`, `.bak1`, `.bak2`...,
//! which the first builds made as whole copies, password included) and files
//! named like a token file (`*.token*`: a set-aside `.prev`, a temporary name
//! a cut-off write left). The move must not leave a secret behind in any of
//! them.
//!
//! - A backup of the account file that reads as TOML is rewritten without its
//!   password and secret entries (and the comments that mention them), the way
//!   the writer does for every backup; one that holds none is left as it is.
//! - A file named like a token file that holds a sign-in is removed: it has
//!   nothing else in it, and the account's own sign-in is in the vault.
//! - What cannot be read as TOML is not guessed at: it is left, and named once.
//!
//! The account's own token file (`oauth/<id>.token`) is not looked at here:
//! the move takes it into the vault, and one it could not take is the account
//! list's to report.

use std::fs;
use std::io::Read;
use std::path::Path;

use model::wipe_text;

use crate::account_writer::{replace_without_backup, without_secrets};
use crate::paths::ACCOUNT_FILE;

use super::migrate_find::account_folders;

/// The largest file looked into.
const MAX_BYTES: u64 = 1024 * 1024;

/// The folder of tokens beside the account file.
const OAUTH_FOLDER: &str = "oauth";

/// What was done with the old files under one accounts folder.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Strays {
    /// How many files were cleaned or removed.
    pub cleaned: usize,
    /// The files that could not be cleaned, as `<id>/<name>`.
    pub left: Vec<String>,
}

/// What happened to one file.
enum Outcome {
    /// It held no secret.
    Nothing,
    Cleaned,
    /// It may hold one and could not be cleaned.
    Left,
}

/// Cleans every old file of the account folders under `root`.
pub(super) fn clean(root: &Path) -> Strays {
    let mut strays = Strays::default();
    for id in account_folders(root) {
        let folder = root.join(&id);
        for (dir, in_oauth) in [(folder.clone(), false), (folder.join(OAUTH_FOLDER), true)] {
            clean_folder(&dir, &id, in_oauth, &mut strays);
        }
        // A folder the cleaning emptied goes; one with anything else stays.
        let _ = fs::remove_dir(folder.join(OAUTH_FOLDER));
    }
    strays
}

fn clean_folder(dir: &Path, id: &str, in_oauth: bool, strays: &mut Strays) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut names: Vec<String> = entries.filter_map(Result::ok).filter_map(|entry| entry.file_name().into_string().ok()).collect();
    names.sort();
    for name in names {
        let path = dir.join(&name);
        if !fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_file()) {
            continue;
        }
        let own_tokens = in_oauth && name == format!("{id}.token");
        let outcome = if !in_oauth && name.starts_with(&format!("{ACCOUNT_FILE}.bak")) {
            clean_backup(&path)
        } else if name.contains(".token") && !own_tokens {
            clean_tokens(&path)
        } else {
            continue;
        };
        match outcome {
            Outcome::Nothing => {}
            Outcome::Cleaned => strays.cleaned += 1,
            Outcome::Left => strays.left.push(format!("{id}/{}{name}", if in_oauth { "oauth/" } else { "" })),
        }
    }
}

/// The file's text, when it is small enough to look into and is text.
fn read_small(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    String::from_utf8(bytes).ok()
}

/// Looks into the file at `path` with `inspect`, and wipes the text read.
fn with_text(path: &Path, inspect: impl FnOnce(&str) -> Outcome) -> Outcome {
    if fs::metadata(path).is_ok_and(|meta| meta.len() == 0) {
        return Outcome::Nothing;
    }
    let Some(mut text) = read_small(path) else {
        return Outcome::Left;
    };
    let outcome = inspect(&text);
    wipe_text(&mut text);
    outcome
}

fn clean_backup(path: &Path) -> Outcome {
    with_text(path, |text| {
        let Some(cleaned) = without_secrets(text) else {
            return Outcome::Left;
        };
        if cleaned == text {
            return Outcome::Nothing;
        }
        match replace_without_backup(path, &cleaned) {
            Ok(()) => Outcome::Cleaned,
            Err(_) => Outcome::Left,
        }
    })
}

fn clean_tokens(path: &Path) -> Outcome {
    with_text(path, |text| {
        let Ok(table) = toml::from_str::<toml::Table>(text) else {
            return Outcome::Left;
        };
        let holds = ["access_token", "refresh_token"]
            .iter()
            .any(|key| table.get(*key).and_then(toml::Value::as_str).is_some_and(|token| !token.is_empty()));
        if !holds {
            return Outcome::Nothing;
        }
        match fs::remove_file(path) {
            Ok(()) => Outcome::Cleaned,
            Err(_) => Outcome::Left,
        }
    })
}

#[cfg(test)]
include!("migrate_strays.tests.rs");
