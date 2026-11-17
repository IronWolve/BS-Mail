//! Writing an account's passwords into the vault, and taking them out again.
//!
//! The vault is opened afresh for each change and saved whole, so nothing here
//! keeps a copy that could go stale, and a save that fails changes nothing.

use std::path::Path;

use engine::vault::{KeySource, Vault, VaultError, account_prefix, password_name, smtp_password_name};
use model::Secret;

/// Puts the passwords (the mailbox one and the separate sending one, when
/// there is one) into the vault in `dir` under account `id` and says whether the vault now holds any password of that account (so the
/// file has to say `secrets = "vault"`).
///
/// A new account (`fresh`) starts with none of an earlier account's entries of
/// the same id. An edit with no password given takes the old one out (the
/// account no longer uses one). A password that is already there is not
/// written again. A vault that is lost (its key is gone) or damaged is set
/// aside and a new one started, since the user is giving a password now and
/// the other accounts ask for theirs in their own words. Nothing is changed
/// when the vault cannot be saved, and the error is a sentence for the form.
pub fn put(dir: &Path, id: &str, fresh: bool, password: Option<Secret>, smtp_password: Option<Secret>) -> Result<bool, String> {
    let given = password.is_some() || smtp_password.is_some();
    let source = KeySource::key_file_in(dir);
    let mut vault = match Vault::open_or_create(dir, source.clone()) {
        Ok(vault) => vault,
        // Nothing to write and nothing readable: the account has no entry to keep.
        Err(_) if !given => return Ok(false),
        Err(error) if error.can_start_over() => Vault::start_over(dir, source).map_err(|error| error.to_string())?,
        Err(error) => return Err(error.to_string()),
    };
    let mut changed = fresh && vault.remove_prefix(&account_prefix(id)) > 0;
    let (mailbox, sending) = (password_name(id), smtp_password_name(id));
    match password {
        Some(password) => changed |= put_entry(&mut vault, &mailbox, password),
        None => changed |= vault.remove(&mailbox),
    }
    if let Some(password) = smtp_password {
        changed |= put_entry(&mut vault, &sending, password);
    }
    if changed {
        vault.save().map_err(|error| error.to_string())?;
    }
    Ok(vault.contains(&mailbox) || vault.contains(&sending))
}

/// Sets an entry unless it already holds this text; whether it changed.
fn put_entry(vault: &mut Vault, name: &str, password: Secret) -> bool {
    if vault.get(name).is_some_and(|old| old == password) {
        return false;
    }
    vault.set(name, password);
    true
}

/// Takes every entry of account `id` out of the vault in `dir`. A folder with
/// no vault, or one whose entries cannot be read at all (its key is lost), has
/// nothing to take out.
pub fn forget(dir: &Path, id: &str) -> Result<(), String> {
    let vault = Vault::open_or_create(dir, KeySource::key_file_in(dir));
    let mut vault = match vault {
        Ok(vault) => vault,
        Err(error) if error.can_start_over() => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if vault.remove_prefix(&account_prefix(id)) == 0 {
        return Ok(());
    }
    vault.save().map_err(|error: VaultError| error.to_string())
}

#[cfg(test)]
include!("vault_store.tests.rs");
