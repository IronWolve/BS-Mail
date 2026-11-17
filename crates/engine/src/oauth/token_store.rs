//! A sign-in's tokens in the vault: the access token, the refresh token and
//! the Unix second the access token ends, as three entries under the account's
//! id (`vault/names.rs`). Every change goes through the one shared handle on
//! the vault (`vault/shared.rs`), which reads the vault from disk under its
//! lock, so a refresh and a password save at the same moment never lose each
//! other's entries; the lock is never held while the network is used.
//!
//! A token file that an older version wrote (`token_file.rs`) is still read
//! when the vault has no entries for the account, and is removed as soon as the
//! tokens have been saved to the vault; nothing here ever writes one.

use model::Secret;

use super::token_file;
use super::token_place::TokenPlace;
use crate::sign_in_advice::sign_in_again;
use crate::vault::{Vault, VaultError, token_names};

/// The tokens of one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRecord {
    pub access_token: Secret,
    pub refresh_token: Secret,
    /// Unix seconds at which the access token stops working.
    pub expires_at: i64,
}

/// Why the tokens could not be read or saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// There are none: the account has not been signed in, or its sign-in was
    /// lost with the vault.
    Missing,
    /// Another problem, as one sentence.
    Failed(String),
}

/// The record in `vault` for account `id`: `Ok(None)` when none of its three
/// entries is there, and an error when it is only partly there or its expiry
/// is not a number (a vault edited by hand, or cut short).
pub fn record_in(vault: &Vault, id: &str) -> Result<Option<TokenRecord>, StoreError> {
    let names = token_names(id);
    let (access, refresh, expiry) = (vault.get(&names.access), vault.get(&names.refresh), vault.get(&names.expiry));
    let (Some(access), Some(refresh), Some(expiry)) = (access.as_ref(), refresh.as_ref(), expiry.as_ref()) else {
        if access.is_none() && refresh.is_none() && expiry.is_none() {
            return Ok(None);
        }
        return Err(StoreError::Failed(format!("the saved sign-in of this account is incomplete; {}", sign_in_again())));
    };
    let Ok(expires_at) = expiry.expose().trim().parse::<i64>() else {
        return Err(StoreError::Failed(format!("the saved sign-in of this account is not valid; {}", sign_in_again())));
    };
    if access.expose().is_empty() || refresh.expose().is_empty() {
        return Err(StoreError::Failed(format!("the saved sign-in of this account is empty; {}", sign_in_again())));
    }
    Ok(Some(TokenRecord { access_token: access.clone(), refresh_token: refresh.clone(), expires_at }))
}

/// Puts `record` in `vault` as account `id`'s sign-in (not saved yet).
pub fn set_record(vault: &mut Vault, id: &str, record: &TokenRecord) {
    let names = token_names(id);
    vault.set(&names.access, record.access_token.clone());
    vault.set(&names.refresh, record.refresh_token.clone());
    vault.set(&names.expiry, Secret::new(record.expires_at.to_string()));
}

/// Takes account `id`'s sign-in out of `vault` (not saved yet); whether any
/// entry was there.
pub fn remove_record(vault: &mut Vault, id: &str) -> bool {
    let names = token_names(id);
    [names.access, names.refresh, names.expiry].iter().fold(false, |any, name| vault.remove(name) | any)
}

/// The sentence for a vault that could not be read.
fn unreadable(error: &VaultError) -> StoreError {
    StoreError::Failed(format!(
        "the saved sign-in of this account cannot be read: {error}; the mail is on the server and nothing is lost; {}",
        sign_in_again()
    ))
}

/// The tokens of the account at `place`: the vault's, else the token file an
/// older version left (also when the vault cannot be opened, so an account whose
/// vault was lost keeps working from its file as far as its access token lasts;
/// a refresh then cannot be saved and says so). `Missing` when there are none
/// anywhere.
pub fn read(place: &TokenPlace) -> Result<TokenRecord, StoreError> {
    let old_file = || match place.legacy_file() {
        file if file.as_os_str().is_empty() => Err(StoreError::Missing),
        file => token_file::read(file),
    };
    match place.vault().read(|vault| record_in(vault, place.id())) {
        Ok(Ok(Some(record))) => Ok(record),
        Ok(Ok(None)) => old_file(),
        Ok(Err(error)) => Err(error),
        Err(vault_error) => old_file().map_err(|_| unreadable(&vault_error)),
    }
}

/// Saves `record` as the tokens of the account at `place`: the vault is
/// written first and flushed, and only then is a token file of an older
/// version, if there is one, removed.
pub fn write(place: &TokenPlace, record: &TokenRecord) -> Result<(), StoreError> {
    place
        .vault()
        .update(|vault| (set_record(vault, place.id(), record), true))
        .map_err(|error| StoreError::Failed(error.to_string()))?;
    token_file::remove(place.legacy_file());
    Ok(())
}

#[cfg(test)]
include!("token_store.tests.rs");
