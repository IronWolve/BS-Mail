//! Writing an account's secrets into the vault, and taking them out again.
//!
//! The secrets are the mailbox password, the separate sending password and the
//! OAuth2 client secret. Every change goes through the process's one handle on
//! the vault (`engine::vault::SharedVault`), which reads the vault from disk
//! under its lock, so a save of the form and a refresh of a sign-in at the same
//! moment never lose each other's entries, and a save that fails changes
//! nothing.

use std::path::Path;

use engine::vault::{
    SharedVault, VaultError, account_prefix, client_secret_name, password_name, provider_client_secret_name,
    smtp_password_name,
};
use model::Secret;

/// Puts the secrets (the mailbox password, the separate sending password and
/// the OAuth2 client secret, when there are any) into the vault in `dir` under
/// account `id` and says whether the vault now holds any of them for that
/// account (so the file has to say `secrets = "vault"`).
///
/// A new account (`fresh`) starts with none of an earlier account's passwords
/// or client secret of the same id; its sign-in entries are left alone, because
/// a browser sign-in saves its tokens before the account is saved, and those
/// are this account's own. An edit that gives no password or client secret
/// takes the old one out (the account no longer uses one). A secret that is
/// already there is not written again. A vault that is lost (its key is gone) or damaged is set
/// aside and a new one started, since the user is giving a secret now and the
/// other accounts ask for theirs in their own words. Nothing is changed when
/// the vault cannot be saved, and the error is a sentence for the form.
pub fn put(
    dir: &Path,
    id: &str,
    fresh: bool,
    password: Option<Secret>,
    smtp_password: Option<Secret>,
    client_secret: Option<Secret>,
) -> Result<bool, String> {
    let given = password.is_some() || smtp_password.is_some() || client_secret.is_some();
    let (mailbox, sending, client) = (password_name(id), smtp_password_name(id), client_secret_name(id));
    let outcome = SharedVault::for_dir(dir).update_or_start_over(given, |vault| {
        let mut changed = false;
        match password {
            Some(password) => changed |= put_entry(vault, &mailbox, password),
            None => changed |= vault.remove(&mailbox),
        }
        match smtp_password {
            Some(password) => changed |= put_entry(vault, &sending, password),
            None if fresh => changed |= vault.remove(&sending),
            None => {}
        }
        match client_secret {
            Some(secret) => changed |= put_entry(vault, &client, secret),
            None => changed |= vault.remove(&client),
        }
        (vault.contains(&mailbox) || vault.contains(&sending) || vault.contains(&client), changed)
    });
    match outcome {
        Ok(held) => Ok(held),
        // Nothing to write and nothing readable: the account has no entry to keep.
        Err(error) if !given && !error.while_saving() => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

/// Sets an entry unless it already holds this text; whether it changed.
fn put_entry(vault: &mut engine::vault::Vault, name: &str, secret: Secret) -> bool {
    if vault.get(name).is_some_and(|old| old == secret) {
        return false;
    }
    vault.set(name, secret);
    true
}

/// Takes every entry of account `id` out of the vault in `dir`: its passwords,
/// its client secret and its sign-in tokens, an attempt's included. A folder
/// with no vault, or one whose entries cannot be read at all (its key is lost),
/// has nothing to take out.
pub fn forget(dir: &Path, id: &str) -> Result<(), String> {
    let outcome = SharedVault::for_dir(dir).update(|vault| {
        let removed = vault.remove_prefix(&account_prefix(id));
        ((), removed > 0)
    });
    match outcome {
        Ok(()) => Ok(()),
        Err(error) if error.can_start_over() => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Keeps the client secret of a sign-in provider (`google`...) for the form to
/// offer to the next account; whether the vault was written. The caller does
/// this only for a provider that has no client id saved, so a secret already
/// there belongs to an id that is gone and is replaced; one that is the same is
/// not written again.
pub fn put_provider_secret(dir: &Path, provider: &str, secret: Secret) -> Result<bool, String> {
    let name = provider_client_secret_name(provider);
    SharedVault::for_dir(dir)
        .update_or_start_over(true, |vault| {
            let changed = put_entry(vault, &name, secret);
            (changed, changed)
        })
        .map_err(|error: VaultError| error.to_string())
}

/// The client secret kept for `provider`; `None` when there is none or the
/// vault cannot be opened (the form then asks for it).
pub fn provider_secret(dir: &Path, provider: &str) -> Option<Secret> {
    SharedVault::for_dir(dir).read(|vault| vault.get(&provider_client_secret_name(provider))).ok().flatten()
}

#[cfg(test)]
include!("vault_store.tests.rs");
