//! Where an account's passwords come from. An account file written by an
//! older version holds them in plain text (`password`, `smtp_password`) and
//! that still works. A file that says `secrets = "vault"` keeps them in the
//! encrypted vault under `accounts.<id>.password` and
//! `accounts.<id>.smtp_password` (see `vault/names.rs`); a password written in
//! the file wins over the vault's.

use model::Secret;

use crate::account_config::ConfigError;
use crate::vault::{VaultError, VaultView, password_name, smtp_password_name};

/// The key that says the passwords are in the vault.
pub const MARKER_KEY: &str = "secrets";
/// What the marker says when they are.
pub const MARKER_VALUE: &str = "vault";

/// Said when the vault has no password for the account, or there is no vault.
pub const NOT_IN_VAULT: &str = "the password of this account is not in the vault; edit the account and enter it again";

/// The passwords the file gave in plain text, and the vault to fill in the rest
/// from when the file says so.
pub(crate) struct Given {
    pub password: Option<Secret>,
    pub smtp_password: Option<Secret>,
}

/// The mailbox password and the sending password of account `id`. A password
/// account (`oauth` false) must end with a mailbox password; the sending one
/// is optional.
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
    let password = match given.password {
        Some(password) => Some(password),
        None if oauth => None,
        None => Some(from_vault(vault, &password_name(id))?.ok_or_else(|| ConfigError::Vault(NOT_IN_VAULT.to_owned()))?),
    };
    // A password the file holds itself needs nothing from the vault, so a vault that cannot
    // be opened does not stop the account being read (it is how an account whose vault was
    // lost is edited and given its password again); the sending password, which only the
    // vault could have held, is then not known.
    let smtp_password = match given.smtp_password {
        Some(password) => Some(password),
        None => match from_vault(vault, &smtp_password_name(id)) {
            Ok(found) => found,
            Err(_) if typed => None,
            Err(error) => return Err(error),
        },
    };
    Ok(Given { password, smtp_password })
}

fn from_vault(vault: &VaultView, name: &str) -> Result<Option<Secret>, ConfigError> {
    vault.get(name).map_err(|error| ConfigError::Vault(unavailable(&error)))
}

/// The sentence for a vault that could not be opened.
fn unavailable(error: &VaultError) -> String {
    format!("the password of this account cannot be read: {error}; edit the account and enter it again")
}

#[cfg(test)]
include!("account_secrets.tests.rs");
