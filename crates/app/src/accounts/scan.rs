//! Finding the accounts on disk: every `<root>/<id>/account.toml`.
//!
//! Nothing here stops the program. A file that cannot be used becomes an
//! account that says why, so it still shows in the list and can be edited or
//! removed, and the first problem goes to the footer.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use engine::vault::{VaultView, password_name};
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
    let mut folders: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| (entry.file_name().to_string_lossy().into_owned(), entry.path()))
        .collect();
    folders.sort();
    for (name, folder) in folders {
        if !is_valid_id(&name) {
            scan.problems.push(format!(
                "the folder \"{name}\" in the accounts folder is not a usable account name \
                 (letters, digits, - and _ only); it was skipped"
            ));
            continue;
        }
        let file = folder.join(ACCOUNT_FILE);
        if !file.is_file() {
            continue;
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
    let vault_password = vault_password_of(table.as_ref(), &id, vault);
    Found { id, folder: folder.to_path_buf(), file: file.to_path_buf(), config, table, vault_password, name, address }
}

/// The vault's password for account `id` when its file says the password is
/// there and holds none itself.
fn vault_password_of(table: Option<&toml::Table>, id: &str, vault: &VaultView) -> Option<Secret> {
    let table = table?;
    let in_vault = table.get(MARKER_KEY).and_then(toml::Value::as_str) == Some(MARKER_VALUE);
    if !in_vault || text_of(table, "password").is_some() {
        return None;
    }
    vault.get(&password_name(id)).ok().flatten()
}

/// A non-empty text value of a table.
pub fn text_of(table: &toml::Table, key: &str) -> Option<String> {
    let text = table.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

#[cfg(test)]
include!("scan.tests.rs");
