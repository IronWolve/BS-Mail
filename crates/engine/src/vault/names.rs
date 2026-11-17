//! The names secrets are kept under: `accounts.<id>.<what>`. An account id is
//! letters, digits, `-` and `_` only, so the prefix `accounts.<id>.` never
//! matches another account's entries.

/// The mailbox password of account `id`.
pub fn password_name(id: &str) -> String {
    format!("{}password", account_prefix(id))
}

/// The separate sending password of account `id`, when it has one.
pub fn smtp_password_name(id: &str) -> String {
    format!("{}smtp_password", account_prefix(id))
}

/// Every entry of account `id` starts with this.
pub fn account_prefix(id: &str) -> String {
    format!("accounts.{id}.")
}

#[cfg(test)]
include!("names.tests.rs");
