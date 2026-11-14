//! Where this run's accounts are kept, and what that allows.

use std::path::{Path, PathBuf};

use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;
use super::scan::{Scan, scan, scan_file};

/// What the Options screen says to adding or removing with `--account`.
pub const FILE_RUN: &str = "This run uses --account; add or remove accounts without it.";
/// What it says to anything about accounts while the mail is the mock's, which
/// has no accounts to change.
pub const MOCK_RUN: &str = "This run shows mock mail.";

/// The place the accounts of one run live in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// One folder per account inside this folder: `<root>/<id>/account.toml`.
    Root(PathBuf),
    /// `--account <file>`: exactly this file, and the folder it is in.
    File(PathBuf),
}

impl Place {
    /// The account file of `id`: inside its folder under a root, the file
    /// itself for `--account`. `None` for an id that is not a valid folder
    /// name.
    pub fn file_of(&self, id: &str) -> Option<PathBuf> {
        match self {
            Self::Root(root) => is_valid_id(id).then(|| root.join(id).join(ACCOUNT_FILE)),
            Self::File(file) => Some(file.clone()),
        }
    }

    /// The folder the account keeps its files in, which is where its sign-in
    /// tokens go too.
    pub fn folder_of(&self, id: &str) -> Option<PathBuf> {
        match self {
            Self::Root(root) => is_valid_id(id).then(|| root.join(id)),
            Self::File(file) => Some(folder_of_file(file)),
        }
    }

    /// Every account there is, as found on disk now.
    pub fn scan(&self) -> Scan {
        match self {
            Self::Root(root) => scan(root),
            Self::File(file) => scan_file(file),
        }
    }

    /// Why accounts cannot be added or removed in this run, if they cannot.
    pub fn refuses_changes(&self) -> Option<&'static str> {
        match self {
            Self::Root(_) => None,
            Self::File(_) => Some(FILE_RUN),
        }
    }

    /// The folder to show as the accounts folder.
    pub fn accounts_folder(&self) -> PathBuf {
        match self {
            Self::Root(root) => root.clone(),
            Self::File(file) => folder_of_file(file),
        }
    }
}

/// The folder a file is in; `.` for a bare file name.
fn folder_of_file(file: &Path) -> PathBuf {
    file.parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[cfg(test)]
include!("place.tests.rs");
