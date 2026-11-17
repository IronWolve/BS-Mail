//! Finding the secrets that are still in plain files under a data folder: the
//! passwords and client secret of account files, the token files an older
//! version wrote, and the client secrets of `oauth_clients.toml`. Nothing is
//! changed here; what is found is read into `Secret`s, so no plain `String` of
//! a token outlives the reading.

use std::fs;
use std::path::{Path, PathBuf};

use engine::{TokenRecord, token_file, token_file_in};
use model::{Secret, wipe_text};
use toml_edit::{DocumentMut, Item};

use crate::account_writer::without_secret_comments;
use crate::oauth_clients::CLIENTS_FILE;
use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;
use super::passwords::{Taken, take_passwords};

/// An account whose files still hold secrets.
pub(super) struct Candidate {
    pub id: String,
    pub file: PathBuf,
    /// The account file's own passwords and client secret, and its text without them.
    pub taken: Option<Taken>,
    /// The sign-in an older version kept in a token file.
    pub token: Option<OldTokens>,
}

/// A token file read.
pub(super) struct OldTokens {
    pub path: PathBuf,
    pub record: TokenRecord,
}

/// The client secrets in `oauth_clients.toml`, by provider, and the file's
/// text without them.
pub(super) struct ClientsFile {
    pub file: PathBuf,
    pub secrets: Vec<(String, Secret)>,
    pub text: String,
}

/// Every plain-file secret under one data folder.
#[derive(Default)]
pub(super) struct Found {
    pub accounts: Vec<Candidate>,
    pub clients: Option<ClientsFile>,
}

impl Found {
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.clients.is_none()
    }

    /// How many secrets are in plain files: each password, each client secret
    /// and each sign-in counts as one.
    pub fn count(&self) -> usize {
        let accounts: usize = self
            .accounts
            .iter()
            .map(|candidate| candidate.taken.as_ref().map_or(0, Taken::count) + usize::from(candidate.token.is_some()))
            .sum();
        accounts + self.clients.as_ref().map_or(0, |clients| clients.secrets.len())
    }
}

/// What is in plain files under the accounts folder `root` and in the data
/// folder above it. A file that is not TOML, whose id is not its folder's name
/// or that others can read is the account list's to report and is left alone.
pub(super) fn find(root: &Path) -> Found {
    Found { accounts: accounts(root), clients: root.parent().and_then(clients_file) }
}

/// The folder names under `root` that are valid account ids, in order.
pub(super) fn account_folders(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut folders: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_valid_id(name))
        .collect();
    folders.sort();
    folders
}

fn accounts(root: &Path) -> Vec<Candidate> {
    account_folders(root).into_iter().filter_map(|id| candidate(root, id)).collect()
}

fn candidate(root: &Path, id: String) -> Option<Candidate> {
    let file = root.join(&id).join(ACCOUNT_FILE);
    if !fs::symlink_metadata(&file).is_ok_and(|meta| meta.is_file()) {
        return None;
    }
    // A file the program refuses to read (others can read it) is left for its
    // owner to fix, as the account list says; the move does not mend it.
    if !private_to_owner(&file) {
        return None;
    }
    let mut text = fs::read_to_string(&file).ok()?;
    let taken = taken_from(&text, &id);
    // The file's text holds the secrets: it does not outlive this.
    wipe_text(&mut text);
    let taken = taken?;
    let token = old_tokens(&root.join(&id), &id);
    (taken.is_some() || token.is_some()).then_some(Candidate { id, file, taken, token })
}

/// The secrets of account file `text`: `None` when it is not an account file
/// of the folder `id` (not TOML, or another id), `Some(None)` when it holds no
/// secret.
fn taken_from(text: &str, id: &str) -> Option<Option<Taken>> {
    let table: toml::Table = toml::from_str(text).ok()?;
    if table.get("id").and_then(toml::Value::as_str) != Some(id) {
        return None;
    }
    Some(take_passwords(text).ok().filter(Taken::any))
}

/// The token file of an older version in `folder`, when it is a plain file
/// this program would read (owner-only, valid).
fn old_tokens(folder: &Path, id: &str) -> Option<OldTokens> {
    let path = token_file_in(folder, id);
    if !fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_file()) {
        return None;
    }
    token_file::read(&path).ok().map(|record| OldTokens { path, record })
}

#[cfg(unix)]
fn private_to_owner(file: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(file).is_ok_and(|meta| meta.permissions().mode() & 0o077 == 0)
}

#[cfg(not(unix))]
fn private_to_owner(_file: &Path) -> bool {
    true
}

/// `oauth_clients.toml` in `data`, when it holds a client secret.
fn clients_file(data: &Path) -> Option<ClientsFile> {
    let file = data.join(CLIENTS_FILE);
    if !fs::symlink_metadata(&file).is_ok_and(|meta| meta.is_file()) {
        return None;
    }
    let mut text = fs::read_to_string(&file).ok()?;
    let parsed = text.parse::<DocumentMut>();
    wipe_text(&mut text);
    let mut document = parsed.ok()?;
    let mut secrets = Vec::new();
    for (provider, item) in document.iter_mut() {
        let Some(entry) = item.as_table_like_mut() else {
            continue;
        };
        let text = entry.get("client_secret").and_then(Item::as_str).filter(|text| !text.is_empty()).map(str::to_owned);
        if let Some(text) = text {
            secrets.push((provider.get().to_owned(), Secret::new(text)));
            entry.remove("client_secret");
        }
    }
    if secrets.is_empty() {
        return None;
    }
    Some(ClientsFile { file, secrets, text: without_secret_comments(&document.to_string()) })
}

#[cfg(test)]
include!("migrate_find.tests.rs");
