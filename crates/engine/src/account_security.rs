//! The two choices of an account file about the connection to the mail
//! server: how it is protected and how the server's certificate is judged.
//! `account_config` re-exports both.

use serde::Deserialize;

/// How the connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Security {
    /// TLS from the first byte (port 993).
    #[default]
    Implicit,
    /// A plain connection upgraded with STARTTLS before any sign-in.
    Starttls,
    /// No TLS. Loopback hosts only.
    None,
}

/// How the server's certificate is judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trust {
    /// Check it against the bundled root certificates.
    #[default]
    Verify,
    /// Accept any certificate. Loopback hosts only.
    Any,
}
