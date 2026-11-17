//! Moving the passwords of account files that still hold them in plain text
//! into the vault, once, at the start of a run.
//!
//! The order is what makes it safe. The passwords go into the vault first and
//! the vault is saved (flushed to disk) before any account file is touched; an
//! account file is then rewritten without them and with `secrets = "vault"`,
//! through the same writer as any save, so its backup holds no password either.
//! A run cut off between the two steps leaves a file that still holds its
//! password, which the next start moves again (a password the vault already
//! holds is not written twice). If the vault cannot be saved no file is
//! touched, and every account keeps working from its file.

use std::fs;
use std::path::{Path, PathBuf};

use model::Secret;

use crate::account_writer::{save_account, without_password_comments};
use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;
use super::passwords::{Taken, take_passwords, with_marker};
use engine::vault::{KeySource, Vault, password_name, smtp_password_name};

/// What the move did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Migration {
    /// How many passwords are now in the vault and out of their files.
    pub moved: usize,
    /// What to tell the user, in the order to tell it: once, in the footer.
    pub notes: Vec<String>,
}

/// An account file that still holds a password.
struct Candidate {
    id: String,
    file: PathBuf,
    taken: Taken,
}

/// Moves the plain-text passwords of the accounts under `root` into the vault
/// in `secrets`. Nothing is written when no file holds one.
pub fn move_passwords_into_vault(root: &Path, secrets: &Path) -> Migration {
    let candidates = candidates(root);
    let mut migration = Migration::default();
    if candidates.is_empty() {
        return migration;
    }
    let total: usize = candidates.iter().map(|candidate| candidate.taken.count()).sum();
    if let Err(reason) = save_to_vault(secrets, &candidates) {
        migration.notes.push(format!(
            "{} not moved into the encrypted vault: {reason}; the account files keep them for now",
            plural(total, "password was", "passwords were")
        ));
        return migration;
    }
    let mut trouble = Vec::new();
    for candidate in candidates {
        let count = candidate.taken.count();
        match rewrite(&candidate) {
            Ok(()) => migration.moved += count,
            Err(reason) => trouble.push(format!(
                "account {}: its password is in the vault, but its file could not be rewritten ({reason}); \
                 the next start tries again",
                candidate.id
            )),
        }
    }
    if migration.moved > 0 {
        migration.notes.push(format!("Moved {} into the encrypted vault", plural(migration.moved, "password", "passwords")));
    }
    migration.notes.extend(trouble);
    migration
}

/// "1 password", "2 passwords".
fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// The account files under `root` that hold a plain-text password and are the
/// account their folder says. A file that is not TOML, or whose id is not its
/// folder's name, is the account list's to report and is left alone.
fn candidates(root: &Path) -> Vec<Candidate> {
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
    folders.into_iter().filter_map(|id| candidate(root, id)).collect()
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
    let text = fs::read_to_string(&file).ok()?;
    let table: toml::Table = toml::from_str(&text).ok()?;
    if table.get("id").and_then(toml::Value::as_str) != Some(id.as_str()) {
        return None;
    }
    let taken = take_passwords(&text).ok()?;
    taken.any().then_some(Candidate { id, file, taken })
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

/// Writes every candidate's passwords into the vault and saves it. Passwords
/// the vault already holds are left alone, and a vault that already holds all
/// of them is not written again.
fn save_to_vault(secrets: &Path, candidates: &[Candidate]) -> Result<(), String> {
    let mut vault = Vault::open_or_create(secrets, KeySource::key_file_in(secrets)).map_err(|error| error.to_string())?;
    let mut changed = false;
    for candidate in candidates {
        for (name, password) in [
            (password_name(&candidate.id), &candidate.taken.password),
            (smtp_password_name(&candidate.id), &candidate.taken.smtp_password),
        ] {
            if let Some(password) = password {
                changed |= put(&mut vault, &name, password);
            }
        }
    }
    if changed {
        vault.save().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn put(vault: &mut Vault, name: &str, password: &Secret) -> bool {
    if vault.get(name).is_some_and(|old| old == *password) {
        return false;
    }
    vault.set(name, password.clone());
    true
}

/// The account file without its passwords and with the marker, through the
/// writer every account file goes through.
fn rewrite(candidate: &Candidate) -> Result<(), String> {
    let text = with_marker(&candidate.taken.text, true)?;
    save_account(&candidate.file, &without_password_comments(&text)).map(|_| ())
}

#[cfg(test)]
include!("migrate.tests.rs");
