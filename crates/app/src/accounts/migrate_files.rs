//! Taking the secrets out of the plain files, once the vault holds them: the
//! account file is rewritten without its secrets and with the marker, the token
//! file of an older version is removed, and `oauth_clients.toml` is rewritten
//! without its client secrets. Each goes through the writer every such file
//! goes through (atomic, owner-only, a backup with no secret in it).

use std::fs;

use crate::account_writer::{save_account, without_secret_comments};

use super::migrate_find::{Candidate, ClientsFile, OldTokens};
use super::passwords::{Taken, with_marker};

/// The account file without its secrets and with the marker. Comments that
/// mention a password or a secret go too: one that quotes the old text must not
/// stay behind.
pub(super) fn rewrite_account(candidate: &Candidate, taken: &Taken) -> Result<(), String> {
    let text = with_marker(&taken.text, true)?;
    save_account(&candidate.file, &without_secret_comments(&text)).map(|_| ())
}

/// Removes a token file that is in the vault now, and the `oauth` folder when
/// that leaves it empty.
pub(super) fn remove_old_tokens(old: &OldTokens) -> Result<(), String> {
    match fs::remove_file(&old.path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    if let Some(folder) = old.path.parent() {
        let _ = fs::remove_dir(folder);
    }
    Ok(())
}

/// `oauth_clients.toml` without its client secrets.
pub(super) fn rewrite_clients(clients: &ClientsFile) -> Result<(), String> {
    save_account(&clients.file, &clients.text).map(|_| ())
}

#[cfg(test)]
include!("migrate_files.tests.rs");
