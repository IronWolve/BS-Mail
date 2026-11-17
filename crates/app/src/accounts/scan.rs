//! Finding the accounts on disk: every `<root>/<id>/account.toml`.
//!
//! Nothing here stops the program. A file that cannot be used becomes an
//! account that says why, so it still shows in the list and can be edited or
//! removed, and the first problem goes to the footer.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use engine::vault::{VaultView, client_secret_name, password_name};
use engine::{AccountConfig, MARKER_KEY, MARKER_VALUE};
use model::Secret;

use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;
use super::place::Place;
use super::reason::reason_of;

/// One account folder, read.
pub struct Found {
    /// The folder's name, which is the account's id.
    pub id: String,
    pub folder: PathBuf,
    pub file: PathBuf,
    /// The checked account, or why it cannot be used.
    pub config: Result<AccountConfig, String>,
    /// The file as a TOML table, when it could be read as one. It can hold a
    /// password, so it is never printed.
    pub table: Option<toml::Table>,
    /// The password the vault holds for an account whose file says so and has
    /// none of its own; it is what the form starts with when the account is
    /// edited.
    pub vault_password: Option<Secret>,
    /// The same for the OAuth2 client secret (`oauth_client_secret` in an older
    /// file): what the form starts with when the account is edited.
    pub vault_client_secret: Option<Secret>,
    /// For the account list: from the file when it could be read, else the id.
    pub name: String,
    pub address: String,
}

impl fmt::Debug for Found {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Found")
            .field("id", &self.id)
            .field("file", &self.file)
            .field("usable", &self.config.is_ok())
            .finish_non_exhaustive()
    }
}

/// Every account found, in the order of their folders' names, and what was
/// wrong along the way, one line each.
#[derive(Debug, Default)]
pub struct Scan {
    pub found: Vec<Found>,
    pub problems: Vec<String>,
}

impl Scan {
    /// Whether some account other than `except` already uses `address`, on
    /// any spelling of the capitals.
    pub fn has_address(&self, address: &str, except: Option<&str>) -> bool {
        let wanted = address.trim().to_lowercase();
        self.found
            .iter()
            .filter(|found| Some(found.id.as_str()) != except)
            .any(|found| found.address.trim().to_lowercase() == wanted)
    }
}

/// The accounts under `root`. A root that is not there has none.
pub fn scan(root: &Path) -> Scan {
    let vault = VaultView::open(&Place::Root(root.to_path_buf()).secrets_folder());
    let mut scan = Scan::default();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return scan,
        Err(error) => {
            scan.problems.push(format!("the accounts folder {} could not be read: {error}", root.display()));
            return scan;
        }
    };
    // What each entry is, without following a link: a link to a folder is listed so it can be
    // reported, a link to anything else (or to nothing) is not a folder.
    let mut folders: Vec<(String, PathBuf, bool)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let link = entry.file_type().ok()?.is_symlink();
            let wanted = if link { entry.path().is_dir() } else { entry.file_type().ok()?.is_dir() };
            wanted.then(|| (entry.file_name().to_string_lossy().into_owned(), entry.path(), link))
        })
        .collect();
    folders.sort();
    for (name, folder, link) in folders {
        if link {
            scan.problems.push(format!("the folder \"{name}\" in the accounts folder is a link, not a folder; it was skipped"));
            continue;
        }
        if !is_valid_id(&name) {
            scan.problems.push(format!(
                "the folder \"{name}\" in the accounts folder is not a usable account name \
                 (letters, digits, - and _ only); it was skipped"
            ));
            continue;
        }
        let file = folder.join(ACCOUNT_FILE);
        match fs::symlink_metadata(&file) {
            Ok(meta) if meta.file_type().is_symlink() => {
                scan.problems.push(format!("account {name}: {ACCOUNT_FILE} is a link, not a file; it was skipped"));
                continue;
            }
            Ok(meta) if meta.is_file() => {}
            _ => continue,
        }
        let found = read_account(&file, &folder, Some(&name), &vault);
        if let Err(reason) = &found.config {
            scan.problems.push(format!("account {name}: {reason}"));
        }
        scan.found.push(found);
    }
    scan
}

/// The one account of `--account <file>`, when the file is there.
pub fn scan_file(file: &Path) -> Scan {
    let mut scan = Scan::default();
    if !file.is_file() {
        return scan;
    }
    let folder = file.parent().filter(|dir| !dir.as_os_str().is_empty()).map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let vault = VaultView::open(&Place::File(file.to_path_buf()).secrets_folder());
    let found = read_account(file, &folder, None, &vault);
    if let Err(reason) = &found.config {
        scan.problems.push(format!("account {}: {reason}", found.id));
    }
    scan.found.push(found);
    scan
}

/// Reads one account file. `folder_id` is the folder's name where the id must
/// be it; `None` takes the id from the file.
fn read_account(file: &Path, folder: &Path, folder_id: Option<&str>, vault: &VaultView) -> Found {
    let table = fs::read_to_string(file).ok().and_then(|text| toml::from_str::<toml::Table>(&text).ok());
    let mut config = AccountConfig::from_file_with_vault(file, vault).map_err(|error| reason_of(&error));
    if let (Ok(checked), Some(id)) = (&config, folder_id)
        && checked.id != id
    {
        config = Err(format!(
            "the id in the file (\"{}\") is not the name of its folder (\"{id}\"); \
             make them the same or the account will not start",
            checked.id
        ));
    }
    let id = folder_id
        .map(str::to_string)
        .or_else(|| config.as_ref().ok().map(|checked| checked.id.clone()))
        .or_else(|| table.as_ref().and_then(|table| text_of(table, "id")))
        .unwrap_or_else(|| "account".to_string());
    let (name, address) = match &config {
        Ok(checked) => (checked.name.clone(), checked.address.clone()),
        Err(_) => (
            table.as_ref().and_then(|table| text_of(table, "name")).unwrap_or_else(|| id.clone()),
            table.as_ref().and_then(|table| text_of(table, "address")).unwrap_or_else(|| id.clone()),
        ),
    };
    let vault_password = vault_secret_of(table.as_ref(), vault, "password", &password_name(&id));
    let vault_client_secret = vault_secret_of(table.as_ref(), vault, "oauth_client_secret", &client_secret_name(&id));
    Found {
        id,
        folder: folder.to_path_buf(),
        file: file.to_path_buf(),
        config,
        table,
        vault_password,
        vault_client_secret,
        name,
        address,
    }
}

/// The vault's entry `name` for an account whose file says its secrets are
/// there and holds none under `key` itself.
fn vault_secret_of(table: Option<&toml::Table>, vault: &VaultView, key: &str, name: &str) -> Option<Secret> {
    let table = table?;
    let in_vault = table.get(MARKER_KEY).and_then(toml::Value::as_str) == Some(MARKER_VALUE);
    if !in_vault || text_of(table, key).is_some() {
        return None;
    }
    vault.get(name).ok().flatten()
}

/// A non-empty text value of a table.
pub fn text_of(table: &toml::Table, key: &str) -> Option<String> {
    let text = table.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

#[cfg(test)]
include!("scan.tests.rs");
