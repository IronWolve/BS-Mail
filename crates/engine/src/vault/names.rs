//! The names secrets are kept under: `accounts.<id>.<what>`. An account id is
//! letters, digits, `-` and `_` only, so the prefix `accounts.<id>.` never
//! matches another account's entries. The one entry that is not an account's
//! is a provider's client secret, `oauth_clients.<provider>.client_secret`.

/// The mailbox password of account `id`.
pub fn password_name(id: &str) -> String {
    format!("{}password", account_prefix(id))
}

/// The separate sending password of account `id`, when it has one.
pub fn smtp_password_name(id: &str) -> String {
    format!("{}smtp_password", account_prefix(id))
}

/// The OAuth2 client secret of account `id`, when the provider gave one.
pub fn client_secret_name(id: &str) -> String {
    format!("{}client_secret", account_prefix(id))
}

/// The client secret kept once for a sign-in provider (`google`...), which the
/// form offers to every account of that provider.
pub fn provider_client_secret_name(provider: &str) -> String {
    format!("oauth_clients.{provider}.client_secret")
}

/// The three entries that hold one sign-in: the access token, the refresh
/// token and the Unix second at which the access token ends (a number, in
/// text). Kept apart, so each secret is a `Secret` of its own and no text is
/// ever built that holds two of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenNames {
    pub access: String,
    pub refresh: String,
    pub expiry: String,
}

/// Where account `id`'s sign-in is kept.
pub fn token_names(id: &str) -> TokenNames {
    let prefix = account_prefix(id);
    TokenNames {
        access: format!("{prefix}oauth_access_token"),
        refresh: format!("{prefix}oauth_refresh_token"),
        expiry: format!("{prefix}oauth_expiry"),
    }
}

/// Where the sign-in that was there before a new one is tried waits, until the
/// account is saved (or the try is given up and it is put back).
pub fn earlier_token_names(id: &str) -> TokenNames {
    let prefix = account_prefix(id);
    TokenNames {
        access: format!("{prefix}oauth_earlier_access_token"),
        refresh: format!("{prefix}oauth_earlier_refresh_token"),
        expiry: format!("{prefix}oauth_earlier_expiry"),
    }
}

/// The marker that a new sign-in is being tried: `none` when the account had
/// no sign-in before, `kept` when the earlier one waits under
/// `earlier_token_names`.
pub fn attempt_name(id: &str) -> String {
    format!("{}oauth_attempt", account_prefix(id))
}

/// Every entry of account `id` starts with this.
pub fn account_prefix(id: &str) -> String {
    format!("accounts.{id}.")
}

#[cfg(test)]
include!("names.tests.rs");
