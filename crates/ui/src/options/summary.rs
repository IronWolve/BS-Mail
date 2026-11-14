//! What the program tells the Options screen about accounts and places. The
//! screen only shows these; it never reads a file, and it stores no path.

/// One account, as the Accounts page shows it. All text is display text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountSummary {
    /// The id the program knows the account by; it also names the account's
    /// mail state in the snapshot.
    pub id: String,
    pub name: String,
    pub address: String,
    /// For example `imap.example.com:993, TLS`.
    pub incoming: String,
    /// For example `smtp.example.com:587, STARTTLS`, or `none`.
    pub outgoing: String,
    /// For example `password` or `web browser (microsoft)`.
    pub sign_in: String,
    /// Where this account's data is kept; shown as "Kept in".
    pub folder: String,
    /// False for a read-only account.
    pub changes_allowed: bool,
}

/// The places the Data page names. They are shown, never stored.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OptionsPaths {
    pub data_dir: String,
    pub config_file: String,
    pub accounts_dir: String,
}

#[cfg(test)]
include!("summary.tests.rs");
