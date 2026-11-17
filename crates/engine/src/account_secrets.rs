//! Where an account's passwords and client secret come from. An account file
//! written by an older version holds them in plain text (`password`,
//! `smtp_password`, `oauth_client_secret`) and that still works. A file that
//! says `secrets = "vault"` keeps them in the encrypted vault under
//! `accounts.<id>.password`, `accounts.<id>.smtp_password` and
//! `accounts.<id>.client_secret` (see `vault/names.rs`); a secret written in
//! the file wins over the vault's. An OAuth2 account's sign-in tokens are not
//! read here: the token source reads them from the vault when it needs one.

use model::Secret;

use crate::account_config::ConfigError;
use crate::vault::{START_OVER_ADVICE, VaultError, VaultView, client_secret_name, password_name, smtp_password_name};

/// The key that says the passwords are in the vault.
pub const MARKER_KEY: &str = "secrets";
/// What the marker says when they are.
pub const MARKER_VALUE: &str = "vault";

/// Said when the vault has no password for the account, or there is no vault.
pub const NOT_IN_VAULT: &str = "the password of this account is not in the vault; edit the account and enter it again";

/// The secrets the file gave in plain text, and the vault to fill in the rest
/// from when the file says so.
pub(crate) struct Given {
    pub password: Option<Secret>,
    pub smtp_password: Option<Secret>,
    /// The OAuth2 client secret; only an OAuth2 account has one.
    pub client_secret: Option<Secret>,
}

/// The mailbox password, the sending password and the client secret of
/// account `id`. A password account (`oauth` false) must end with a mailbox
/// password; the sending one and the client secret are optional.
pub(crate) fn resolve(
    id: &str,
    marker: Option<&str>,
    oauth: bool,
    given: Given,
    vault: &VaultView,
) -> Result<Given, ConfigError> {
    match marker {
        None => return Ok(given),
        Some(MARKER_VALUE) => {}
        Some(_) => return Err(ConfigError::Invalid(format!("`{MARKER_KEY}` must be \"{MARKER_VALUE}\" when it is set"))),
    }
    let typed = given.password.is_some();
    let client_secret = match given.client_secret {
        Some(secret) => Some(secret),
        None if oauth => from_vault(vault, &client_secret_name(id), oauth)?.filter(|secret| !secret.expose().is_empty()),
        None => None,
    };
    let password = match given.password {
        Some(password) => Some(password),
        None if oauth => None,
        None => Some(from_vault(vault, &password_name(id), oauth)?.ok_or_else(|| ConfigError::Vault(NOT_IN_VAULT.to_owned()))?),
    };
    // A password the file holds itself needs nothing from the vault, so a vault that cannot
    // be opened does not stop the account being read (it is how an account whose vault was
    // lost is edited and given its password again); the sending password, which only the
    // vault could have held, is then not known.
    let smtp_password = match given.smtp_password {
        Some(password) => Some(password),
        None => match from_vault(vault, &smtp_password_name(id), oauth) {
            Ok(found) => found,
            Err(_) if typed => None,
            Err(error) => return Err(error),
        },
    };
    Ok(Given { password, smtp_password, client_secret })
}

fn from_vault(vault: &VaultView, name: &str, oauth: bool) -> Result<Option<Secret>, ConfigError> {
    vault.get(name).map_err(|error| ConfigError::Vault(unavailable(&error, oauth, vault.needs_explicit_start_over())))
}

/// The sentence for a vault that could not be opened. An OAuth2 account has no
/// password to enter again: it signs in again, and the mail is safe. When only
/// the explicit "Start the vault over" in Options deals with the vault (a
/// passphrase or keychain vault, or an error no save deals with), the sentence
/// says so: entering the password again would fail the same way.
fn unavailable(error: &VaultError, oauth: bool, explicit_only: bool) -> String {
    if *error == VaultError::Locked {
        // Not lost: the passphrase has simply not been given in this run.
        return format!("the secrets of this account cannot be read yet: {error}; start the program again and enter it");
    }
    if *error == VaultError::KeychainUnavailable {
        // Not lost either, and nothing is offered to be thrown away: the key is in a keychain that
        // is out of reach for now, and the program asks it again the next time it starts.
        // The reason comes first: the footer clips a long sentence at its end.
        return format!("{error}, so the secrets of this account cannot be read now; the program asks again the next time it starts");
    }
    if oauth {
        let way = if explicit_only { format!("{}, then edit the account and sign in again", START_OVER_ADVICE) } else { "edit the account and sign in again".to_owned() };
        return format!(
            "the sign-in details of this account cannot be read: {error}; the mail is on the server and nothing \
             is lost; {way}"
        );
    }
    let way = if explicit_only { format!("{}, then edit the account and enter it again", START_OVER_ADVICE) } else { "edit the account and enter it again".to_owned() };
    format!("the password of this account cannot be read: {error}; {way}")
}

#[cfg(test)]
include!("account_secrets.tests.rs");
