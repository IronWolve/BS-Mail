//! The earlier sign-in, kept aside while a new one is tried.
//!
//! A new browser sign-in saves its tokens to the vault before the mail server
//! has accepted the account and before the account file is saved. So that
//! giving up (or a failed try, or a crash) can put things back, the first
//! save of an attempt copies the account's tokens to the `oauth_earlier_*`
//! entries and sets the `oauth_attempt` marker (`none` when there was no
//! earlier sign-in, `kept` when it waits); a token file of an older version
//! counts as the earlier sign-in and is removed once it is safe in the vault.
//! If the marker is there, the tokens are not yet the account's own.
//!
//! - `keep` runs just before the new tokens are saved. The first call of an
//!   attempt keeps; later ones leave the original aside.
//! - `restore` puts the earlier tokens back, or removes the new ones when
//!   there were none. It does nothing when no attempt left a marker.
//! - `discard` runs once the account is saved: the new tokens are its own.
//!
//! All three change the vault through its shared handle, so the copy, the
//! marker and the new tokens never overwrite another entry, and the secrets
//! stay in `Secret`s the whole way.

use model::Secret;

use super::token_file;
use super::token_place::TokenPlace;
use super::token_store::{TokenRecord, record_in, remove_record, set_record};
use crate::vault::{Vault, attempt_name, earlier_token_names};

const KEPT: &str = "kept";
const NONE: &str = "none";

/// Whether an attempt has replaced these tokens and not yet finished. A vault
/// that cannot be read holds no attempt.
pub fn is_pending(place: &TokenPlace) -> bool {
    place.vault().read(|vault| vault.contains(&attempt_name(place.id()))).unwrap_or(false)
}

/// Sets the account's tokens aside before a new sign-in replaces them. A
/// marker that is already there belongs to an earlier try of the same attempt
/// and is kept. This is the start of a sign-in the person asked for, which is
/// also the way out of a lost vault ("sign in again"): a vault whose key is
/// lost or that is damaged is set aside as `vault.unreadable` and a new one
/// started, as when a password is given again.
pub fn keep(place: &TokenPlace) -> Result<(), String> {
    let id = place.id();
    let older = (!place.legacy_file().as_os_str().is_empty()).then(|| token_file::read(place.legacy_file()).ok()).flatten();
    let from_legacy = place
        .vault()
        .update_or_start_over(true, |vault| aside(vault, id, older.as_ref()))
        .map_err(|error| format!("cannot keep the earlier sign-in of this account aside: {error}"))?;
    if from_legacy {
        // Its tokens wait in the vault now; the file must not keep a second copy.
        token_file::remove(place.legacy_file());
    }
    Ok(())
}

/// Copies the current tokens aside (the vault's, else `older`); whether the
/// older version's file was the source, and whether the vault changed.
fn aside(vault: &mut Vault, id: &str, older: Option<&TokenRecord>) -> (bool, bool) {
    if vault.contains(&attempt_name(id)) {
        return (false, false);
    }
    let in_vault = record_in(vault, id).ok().flatten();
    let from_legacy = in_vault.is_none() && older.is_some();
    match in_vault.as_ref().or(older) {
        Some(record) => {
            set_earlier(vault, id, record);
            vault.set(&attempt_name(id), Secret::new(KEPT));
        }
        None => vault.set(&attempt_name(id), Secret::new(NONE)),
    }
    (from_legacy, true)
}

/// Puts the earlier sign-in back over the account's tokens, or removes the new
/// tokens when there was no earlier one. Does nothing when no attempt left a
/// marker, or when the vault is one that cannot hold an attempt (lost or
/// damaged).
pub fn restore(place: &TokenPlace) -> Result<(), String> {
    let id = place.id();
    let done = place.vault().update(|vault| {
        let Some(mark) = vault.get(&attempt_name(id)) else {
            return ((), false);
        };
        remove_record(vault, id);
        if mark.expose() == KEPT
            && let Some(earlier) = earlier_record(vault, id)
        {
            set_record(vault, id, &earlier);
        }
        clear(vault, id);
        ((), true)
    });
    match done {
        Err(error) if !error.can_start_over() => {
            Err(format!("cannot put the earlier sign-in of this account back: {error}"))
        }
        _ => Ok(()),
    }
}

/// The new tokens are the account's own: the earlier ones are not needed.
pub fn discard(place: &TokenPlace) {
    let id = place.id();
    let _ = place.vault().update(|vault| ((), clear(vault, id)));
}

/// Takes the earlier tokens and the marker out of the vault; whether any were there.
fn clear(vault: &mut Vault, id: &str) -> bool {
    let names = earlier_token_names(id);
    let mut any = false;
    for name in [names.access, names.refresh, names.expiry, attempt_name(id)] {
        any |= vault.remove(&name);
    }
    any
}

fn set_earlier(vault: &mut Vault, id: &str, record: &TokenRecord) {
    let names = earlier_token_names(id);
    vault.set(&names.access, record.access_token.clone());
    vault.set(&names.refresh, record.refresh_token.clone());
    vault.set(&names.expiry, Secret::new(record.expires_at.to_string()));
}

fn earlier_record(vault: &Vault, id: &str) -> Option<TokenRecord> {
    let names = earlier_token_names(id);
    let expires_at = vault.get(&names.expiry)?.expose().trim().parse::<i64>().ok()?;
    Some(TokenRecord { access_token: vault.get(&names.access)?, refresh_token: vault.get(&names.refresh)?, expires_at })
}

#[cfg(test)]
include!("previous_token.tests.rs");
