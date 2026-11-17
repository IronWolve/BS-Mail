//! Writing what `migrate_find` found into the vault: one change under the
//! shared handle's lock and one save, flushed to disk, before any file is
//! touched.

use std::path::Path;

use engine::token_store::{record_in, set_record};
use engine::vault::{
    SharedVault, Vault, client_secret_name, password_name, provider_client_secret_name, smtp_password_name,
};
use model::Secret;

use super::migrate_find::{Candidate, Found};

/// Writes every secret of `found` into the vault in `secrets` and saves it. A
/// secret the vault already holds is left alone, and a vault that already holds
/// all of them is not written again. A file's own password and client secret
/// win over the vault's (it is how a secret put back by hand takes effect); a
/// sign-in the vault already holds wins over a token file (the vault's is the
/// newer, since every refresh since has gone there).
pub(super) fn save_to_vault(secrets: &Path, found: &Found) -> Result<(), String> {
    SharedVault::for_dir(secrets)
        .update(|vault| {
            let mut changed = false;
            for candidate in &found.accounts {
                changed |= put_account(vault, candidate);
            }
            if let Some(clients) = &found.clients {
                for (provider, secret) in &clients.secrets {
                    changed |= put(vault, &provider_client_secret_name(provider), secret);
                }
            }
            ((), changed)
        })
        .map_err(|error| error.to_string())
}

fn put_account(vault: &mut Vault, candidate: &Candidate) -> bool {
    let mut changed = false;
    if let Some(taken) = &candidate.taken {
        for (name, secret) in [
            (password_name(&candidate.id), &taken.password),
            (smtp_password_name(&candidate.id), &taken.smtp_password),
            (client_secret_name(&candidate.id), &taken.client_secret),
        ] {
            if let Some(secret) = secret {
                changed |= put(vault, &name, secret);
            }
        }
    }
    if let Some(old) = &candidate.token
        && !matches!(record_in(vault, &candidate.id), Ok(Some(_)))
    {
        set_record(vault, &candidate.id, &old.record);
        changed = true;
    }
    changed
}

/// Sets an entry unless it already holds this text; whether it changed.
fn put(vault: &mut Vault, name: &str, secret: &Secret) -> bool {
    if vault.get(name).is_some_and(|old| old == *secret) {
        return false;
    }
    vault.set(name, secret.clone());
    true
}

#[cfg(test)]
include!("migrate_vault.tests.rs");
