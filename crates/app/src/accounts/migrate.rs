//! Moving the secrets that are still in plain files into the vault, once, at
//! the start of a run: the passwords and client secret of account files, the
//! sign-in tokens an older version kept in `oauth/<id>.token`, and the client
//! secrets of `oauth_clients.toml`.
//!
//! The order is what makes it safe. Everything found goes into the vault in one
//! change and the vault is saved (flushed to disk) before any file is touched;
//! an account file is then rewritten without its secrets and with
//! `secrets = "vault"`, a token file is removed, and `oauth_clients.toml` is
//! rewritten without its secrets, each through the writer any such file goes
//! through, so no backup holds a secret either. A run cut off between the two
//! steps leaves files that still hold their secrets, which the next start moves
//! again (a secret the vault already holds is not written twice). If the vault
//! cannot be saved no file is touched, and every account keeps working from its
//! file. Only after a move that went through are the old backups cleaned
//! (`migrate_strays.rs`).

use std::path::Path;

use super::migrate_files::{remove_old_tokens, rewrite_account, rewrite_clients};
use super::migrate_find::{Found, find};
use super::migrate_strays::{Strays, clean};
use super::migrate_vault::save_to_vault;

/// What the move did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Migration {
    /// How many secrets are now in the vault and out of their files.
    pub moved: usize,
    /// What to tell the user, in the order to tell it: once, in the footer.
    pub notes: Vec<String>,
}

/// Moves the plain-text secrets of the accounts under `root` and of the data
/// folder above it into the vault in `secrets`. Nothing is written when no file
/// holds one.
pub fn move_secrets_into_vault(root: &Path, secrets: &Path) -> Migration {
    let found = find(root);
    let mut migration = Migration::default();
    if !found.is_empty() {
        let total = found.count();
        if let Err(reason) = save_to_vault(secrets, &found) {
            migration.notes.push(format!(
                "{} not moved into the encrypted vault: {reason}; the files keep them for now",
                plural(total, "secret was", "secrets were")
            ));
            return migration;
        }
        take_out_of_files(&found, &mut migration);
    }
    say_what_was_left(&clean(root), &mut migration);
    migration
}

/// "1 secret", "2 secrets".
pub(super) fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// The vault holds everything in `found`: the files are cleaned now.
fn take_out_of_files(found: &Found, migration: &mut Migration) {
    let mut trouble = Vec::new();
    for candidate in &found.accounts {
        if let Some(taken) = &candidate.taken {
            match rewrite_account(candidate, taken) {
                Ok(()) => migration.moved += taken.count(),
                Err(reason) => trouble.push(format!(
                    "account {}: its secrets are in the vault, but its file could not be rewritten ({reason}); \
                     the next start tries again",
                    candidate.id
                )),
            }
        }
        if let Some(old) = &candidate.token {
            match remove_old_tokens(old) {
                Ok(()) => migration.moved += 1,
                Err(reason) => trouble.push(format!(
                    "account {}: its sign-in is in the vault, but the old token file could not be removed ({reason}); \
                     the next start tries again",
                    candidate.id
                )),
            }
        }
    }
    if let Some(clients) = &found.clients {
        match rewrite_clients(clients) {
            Ok(()) => migration.moved += clients.secrets.len(),
            Err(reason) => trouble.push(format!(
                "the client secrets are in the vault, but their file could not be rewritten ({reason}); \
                 the next start tries again"
            )),
        }
    }
    if migration.moved > 0 {
        migration.notes.push(format!("Moved {} into the encrypted vault", plural(migration.moved, "secret", "secrets")));
    }
    migration.notes.extend(trouble);
}

/// One note for the old files that could not be cleaned, with their names. A
/// file that was cleaned is not mentioned: it only lost what it should not
/// have held.
fn say_what_was_left(strays: &Strays, migration: &mut Migration) {
    if strays.left.is_empty() {
        return;
    }
    migration.notes.push(format!(
        "{} may still hold a secret and could not be cleaned: {}; delete {} if you do not need {}",
        plural(strays.left.len(), "old file", "old files"),
        strays.left.join(", "),
        if strays.left.len() == 1 { "it" } else { "them" },
        if strays.left.len() == 1 { "it" } else { "them" },
    ));
}

#[cfg(test)]
include!("migrate.tests.rs");
