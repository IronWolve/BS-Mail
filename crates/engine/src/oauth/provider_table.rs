//! The sign-in providers this program knows, as data only: the two endpoints,
//! the scopes, the host name the redirect uses, extra query parameters for
//! the authorization link, and the mail host names that select the entry.
//! Nothing outside this file (and its tests) names a provider.

/// One provider's entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderEntry {
    /// The value of `oauth_provider` in the account file.
    pub key: &'static str,
    pub auth_url: &'static str,
    pub token_url: &'static str,
    pub scopes: &'static [&'static str],
    /// The host name in the redirect address the provider is told about.
    pub redirect_host: &'static str,
    /// Extra query parameters for the authorization link.
    pub extra_params: &'static [(&'static str, &'static str)],
    /// Mail server names (IMAP or SMTP) that mean this provider.
    pub hosts: &'static [&'static str],
}

static TABLE: [ProviderEntry; 2] = [
    ProviderEntry {
        key: "google",
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
        token_url: "https://oauth2.googleapis.com/token",
        scopes: &["https://mail.google.com/"],
        redirect_host: "127.0.0.1",
        extra_params: &[("access_type", "offline"), ("prompt", "consent")],
        hosts: &["imap.gmail.com", "smtp.gmail.com", "imap.googlemail.com"],
    },
    ProviderEntry {
        key: "microsoft",
        auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
        token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        scopes: &[
            "https://outlook.office.com/IMAP.AccessAsUser.All",
            "https://outlook.office.com/SMTP.Send",
            "offline_access",
        ],
        redirect_host: "localhost",
        extra_params: &[],
        hosts: &[
            "outlook.office365.com",
            "smtp.office365.com",
            "imap-mail.outlook.com",
            "smtp-mail.outlook.com",
        ],
    },
];

/// Every entry.
pub fn entries() -> &'static [ProviderEntry] {
    &TABLE
}

/// The entry for an `oauth_provider` value.
pub fn by_key(key: &str) -> Option<&'static ProviderEntry> {
    entries().iter().find(|entry| entry.key.eq_ignore_ascii_case(key.trim()))
}

/// The entry whose host list holds this mail server name.
pub fn by_host(host: &str) -> Option<&'static ProviderEntry> {
    let bare = host.trim().trim_end_matches('.');
    entries().iter().find(|entry| entry.hosts.iter().any(|h| h.eq_ignore_ascii_case(bare)))
}

/// Every key, for a sentence that says what to set.
pub fn keys() -> String {
    entries().iter().map(|entry| entry.key).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
include!("provider_table.tests.rs");
