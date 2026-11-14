//! The client ids this program signs in with, kept once for every account in
//! `oauth_clients.toml` in the data folder:
//!
//! ```toml
//! [google]
//! client_id = "..."
//! client_secret = "..."   # only if the provider gave one
//!
//! [microsoft]
//! client_id = "..."
//! ```
//!
//! A client id says which program is asking, so the program needs one per
//! provider, not one per account. The form fills it in for a provider that has
//! an entry; a sign-in that worked with one typed into the form saves it here
//! when the provider has none yet. A provider's entry is never replaced by the
//! program, and the file is written whole or not at all, readable by its owner
//! only.

use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, value};
use ui::SavedClient;

use crate::account_writer::save_account;

pub const CLIENTS_FILE: &str = "oauth_clients.toml";

/// The longest client id or secret taken; real ones are far shorter.
const MAX_TEXT: usize = 512;

/// The file of client ids in one data folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientStore {
    file: PathBuf,
}

impl ClientStore {
    pub fn in_folder(folder: &Path) -> Self {
        Self { file: folder.join(CLIENTS_FILE) }
    }

    /// The client ids saved. A missing or unreadable file, and an entry that
    /// is not a usable client id, count as nothing saved.
    pub fn load(&self) -> Vec<SavedClient> {
        let Ok(text) = fs::read_to_string(&self.file) else {
            return Vec::new();
        };
        let Ok(table) = text.parse::<toml::Table>() else {
            return Vec::new();
        };
        table
            .iter()
            .filter_map(|(provider, entry)| {
                let entry = entry.as_table()?;
                let client_id = entry.get("client_id")?.as_str().filter(|text| usable(text))?;
                let client_secret = entry.get("client_secret").and_then(toml::Value::as_str).filter(|text| usable(text));
                Some(SavedClient {
                    provider: provider.clone(),
                    client_id: client_id.to_string(),
                    client_secret: client_secret.unwrap_or_default().to_string(),
                })
            })
            .collect()
    }

    /// Saves the client id for `provider` unless the provider has one
    /// already. `Ok(true)` when the file was written, `Ok(false)` when there
    /// was nothing to do. A file that is not TOML is left alone and said so.
    pub fn remember(&self, provider: &str, client_id: &str, client_secret: Option<&str>) -> Result<bool, String> {
        let secret = client_secret.filter(|text| !text.is_empty());
        let fits = !provider.is_empty()
            && provider.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
            && usable(client_id)
            && secret.is_none_or(usable);
        if !fits {
            return Ok(false);
        }
        let old = match fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(format!("could not read {}: {error}", self.file.display())),
        };
        let mut document: DocumentMut = old
            .parse()
            .map_err(|_| format!("{} is not valid TOML, so the client id was not saved in it", self.file.display()))?;
        if document.get(provider).and_then(|entry| entry.get("client_id")).and_then(|id| id.as_str()).is_some_and(usable) {
            return Ok(false);
        }
        let entry = document.entry(provider).or_insert_with(toml_edit::table);
        let table = entry
            .as_table_like_mut()
            .ok_or_else(|| format!("{} has \"{provider}\" set to something other than a table", self.file.display()))?;
        table.insert("client_id", value(client_id));
        if let Some(secret) = secret {
            table.insert("client_secret", value(secret));
        }
        save_account(&self.file, &document.to_string())?;
        Ok(true)
    }
}

/// One run of printable characters of a sensible length.
fn usable(text: &str) -> bool {
    !text.is_empty() && text.len() <= MAX_TEXT && !text.chars().any(|ch| ch.is_whitespace() || ch.is_control())
}

#[cfg(test)]
include!("oauth_clients.tests.rs");
