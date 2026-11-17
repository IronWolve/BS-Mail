//! The word that records how the folder's vault is protected, outside the
//! vault itself: `secrets/protection`, one word, `keyfile`, `passphrase` or
//! `keychain`, owner-only.
//!
//! The vault's own header is the truth while it can be read. The word is for
//! when it cannot: a passphrase or keychain vault that was deleted, cut short
//! or damaged must never be taken for "no vault yet" and quietly replaced by a
//! key-file vault with its key beside the data. It is written whenever a vault
//! is saved or switched, after the new vault is in place, and when the header
//! and the word disagree the header wins and the word is repaired. A folder
//! that has neither a readable header nor a word is a fresh folder: key file.

use std::io::Read;
use std::path::{Path, PathBuf};

use super::error::VaultError;
use super::file_io::temp_path;
use super::format::Mode;
use super::inject::{Step, reach};
use crate::private_fs;

/// The word's file name inside the secrets folder.
pub const MARKER_NAME: &str = "protection";

/// More than any word is long.
const MAX_BYTES: u64 = 64;

/// What the word file says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marker {
    /// There is no word file.
    Missing,
    /// A word this program wrote.
    Valid(Mode),
    /// There is a file and it is not one of the three words (or is a link, too
    /// long, or cannot be read): damage, so the way is not known.
    Garbled,
}

/// The word for a way of protecting the vault.
pub(super) fn word(mode: Mode) -> &'static str {
    match mode {
        Mode::KeyFile => "keyfile",
        Mode::Passphrase => "passphrase",
        Mode::Keychain => "keychain",
    }
}

fn path_in(dir: &Path) -> PathBuf {
    dir.join(MARKER_NAME)
}

/// What the word file in `dir` says.
pub(super) fn read(dir: &Path) -> Marker {
    let path = path_in(dir);
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Marker::Missing,
        Ok(meta) if meta.is_file() => {}
        _ => return Marker::Garbled,
    }
    let Ok(file) = std::fs::File::open(&path) else {
        return Marker::Garbled;
    };
    // The handle, not the name: what was opened is a plain file of a sane size.
    if !file.metadata().is_ok_and(|meta| meta.is_file() && meta.len() <= MAX_BYTES) {
        return Marker::Garbled;
    }
    let mut text = String::new();
    if file.take(MAX_BYTES).read_to_string(&mut text).is_err() {
        return Marker::Garbled;
    }
    [Mode::KeyFile, Mode::Passphrase, Mode::Keychain]
        .into_iter()
        .find(|mode| text.trim() == word(*mode))
        .map_or(Marker::Garbled, Marker::Valid)
}

/// Records `mode` in `dir`: a new file under a temporary name is written and
/// flushed, renamed over the old one, and the folder is flushed. Nothing is
/// written when the file already says it.
pub(super) fn write(dir: &Path, mode: Mode) -> Result<(), VaultError> {
    if read(dir) == Marker::Valid(mode) {
        return Ok(());
    }
    let fail = |error: std::io::Error| VaultError::Write(error.to_string());
    private_fs::make_folder(dir).map_err(fail)?;
    let path = path_in(dir);
    let temp = temp_path(&path);
    if std::fs::symlink_metadata(&temp).is_ok_and(|meta| !meta.is_dir()) {
        std::fs::remove_file(&temp).map_err(fail)?;
    }
    reach(Step::WriteMarker)?;
    private_fs::write_new_or_remove(&temp, format!("{}\n", word(mode)).as_bytes()).map_err(fail)?;
    if let Err(error) = std::fs::rename(&temp, &path) {
        let _ = std::fs::remove_file(&temp);
        return Err(fail(error));
    }
    private_fs::sync_folder(dir).map_err(fail)
}

#[cfg(test)]
include!("marker.tests.rs");
