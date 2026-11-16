//! The folders an account file names by hand: which mailbox on the server is
//! the Sent, Drafts, Trash, Archive and Spam folder.
//!
//! Without them the roles are guessed from the server's SPECIAL-USE marks
//! and from common names (`imap_session/folder_roles.rs`), and a server with
//! odd or translated names gets no Trash, so a delete is refused. The file
//! says which mailbox it is with a `folders` table (or a `[folders]` section)
//! of paths on the server, as the sidebar shows them or as the server writes
//! them, with the server's own separator for a folder inside another:
//!
//! ```toml
//! folders = { trash = "Gelöschte Elemente", sent = "INBOX/Sent", spam = "Junk" }
//! ```
//!
//! A role that is named wins over the guess, and the guess for a role that is
//! named never lands on another folder. Roles that are left out (or empty)
//! are guessed as before. A name that is not on the server is said once, in
//! the footer, when the account first connects.

use model::FolderRole;
use serde::Deserialize;

use crate::account_config::ConfigError;

/// The longest path accepted; real paths are far shorter.
const MAX_PATH: usize = 300;

/// The five roles a file can name, each a path on the server.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FolderPaths {
    pub sent: Option<String>,
    pub drafts: Option<String>,
    pub trash: Option<String>,
    pub archive: Option<String>,
    pub spam: Option<String>,
}

impl FolderPaths {
    /// The roles the file names, in a fixed order, with their paths.
    pub fn named(&self) -> Vec<(FolderRole, &str)> {
        [
            (FolderRole::Sent, &self.sent),
            (FolderRole::Drafts, &self.drafts),
            (FolderRole::Trash, &self.trash),
            (FolderRole::Archive, &self.archive),
            (FolderRole::Spam, &self.spam),
        ]
        .into_iter()
        .filter_map(|(role, path)| path.as_deref().map(|path| (role, path)))
        .collect()
    }

    /// The table with each path trimmed and an empty one dropped (it means
    /// "guess"), or the key that holds something that cannot be a path.
    pub(crate) fn checked(self) -> Result<Self, ConfigError> {
        let clean = |key: &str, value: Option<String>| -> Result<Option<String>, ConfigError> {
            let Some(text) = value else {
                return Ok(None);
            };
            let path = text.trim();
            if path.is_empty() {
                return Ok(None);
            }
            if path.chars().count() > MAX_PATH || path.chars().any(char::is_control) {
                return Err(ConfigError::Invalid(format!(
                    "`folders.{key}` must be a folder's path on the server: no line breaks or other control characters, at most {MAX_PATH} characters"
                )));
            }
            Ok(Some(path.to_owned()))
        };
        Ok(Self {
            sent: clean("sent", self.sent)?,
            drafts: clean("drafts", self.drafts)?,
            trash: clean("trash", self.trash)?,
            archive: clean("archive", self.archive)?,
            spam: clean("spam", self.spam)?,
        })
    }
}

/// The key of the account file for a role, for the sentences that tell a
/// person which key to set.
pub fn key_of(role: FolderRole) -> &'static str {
    match role {
        FolderRole::Sent => "sent",
        FolderRole::Drafts => "drafts",
        FolderRole::Trash => "trash",
        FolderRole::Archive => "archive",
        FolderRole::Spam => "spam",
        _ => "",
    }
}

#[cfg(test)]
include!("account_folders.tests.rs");
