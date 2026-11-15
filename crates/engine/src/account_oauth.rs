//! The OAuth2 half of the account file: which provider signs the person in,
//! the client id, and where the tokens are kept (plan section 9.1a).
//!
//! ```toml
//! auth = "oauth2"                  # password (default) | oauth2
//! oauth_provider = "..."           # a key of the provider table; guessed from
//!                                  # `imap_host` when omitted
//! oauth_client_id = "..."          # required
//! oauth_client_secret = "..."      # optional
//!
//! # For a provider that is not in the table:
//! oauth_auth_url = "https://..."   # the authorization page
//! oauth_token_url = "https://..."  # the token endpoint
//! oauth_scopes = ["..."]           # what to ask for
//! oauth_redirect_host = "127.0.0.1"   # 127.0.0.1 | localhost
//! ```
//!
//! A custom value overrides the table's value for that one key. `password`
//! is not needed (and not used) for an OAuth2 account.

use std::path::{Path, PathBuf};

use model::Secret;
use serde::Deserialize;

use crate::account_config::{ConfigError, is_loopback_host};
use crate::oauth::Url;
use crate::oauth::provider_table::{self, ProviderEntry};

/// How the account signs in to the mail server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthKind {
    /// A user name and password (or app password).
    #[default]
    Password,
    /// An access token from the provider's sign-in, over SASL XOAUTH2.
    Oauth2,
}

/// The redirect host names the program accepts.
// The listener binds IPv4 loopback only, so an IPv6 name could not be reached.
const REDIRECT_HOSTS: [&str; 2] = ["127.0.0.1", "localhost"];

/// The values as the file spelled them. Its debug output never shows the
/// client secret.
#[derive(Default)]
pub(crate) struct RawOAuth {
    pub provider: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub auth_url: Option<String>,
    pub token_url: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub redirect_host: Option<String>,
}

impl std::fmt::Debug for RawOAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawOAuth")
            .field("provider", &self.provider)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret.as_ref().map(|_| "***"))
            .field("auth_url", &self.auth_url)
            .field("token_url", &self.token_url)
            .field("scopes", &self.scopes)
            .field("redirect_host", &self.redirect_host)
            .finish()
    }
}

/// Everything the sign-in needs about the provider and the client, checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthSettings {
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    /// `127.0.0.1` or `localhost`: the host in the redirect address.
    pub redirect_host: String,
    /// Extra query parameters for the authorization link.
    pub extra_params: Vec<(String, String)>,
    pub client_id: String,
    /// Never shown in debug output.
    pub client_secret: Option<Secret>,
    /// Where the tokens live. Empty until the account is read from a file or
    /// given a folder with `AccountConfig::with_state_dir`.
    pub token_file: PathBuf,
}

/// `<dir>/oauth/<id>.token`.
pub fn token_file_in(dir: &Path, id: &str) -> PathBuf {
    dir.join("oauth").join(format!("{id}.token"))
}

fn invalid(text: String) -> ConfigError {
    ConfigError::Invalid(text)
}

impl OAuthSettings {
    /// Fills in the provider's values, applies the file's own, and checks the
    /// result. `imap_host` selects the provider when the file names none.
    pub(crate) fn resolve(raw: RawOAuth, imap_host: &str) -> Result<Self, ConfigError> {
        let entry = chosen_entry(raw.provider.as_deref(), imap_host)?;
        let auth_url = pick(raw.auth_url, entry.map(|e| e.auth_url), "oauth_auth_url", imap_host)?;
        let token_url = pick(raw.token_url, entry.map(|e| e.token_url), "oauth_token_url", imap_host)?;
        let scopes = match (raw.scopes, entry) {
            (Some(own), _) => own,
            (None, Some(e)) => e.scopes.iter().map(|s| (*s).to_owned()).collect(),
            (None, None) => return Err(missing_provider("oauth_scopes", imap_host)),
        };
        let client_id = raw.client_id.map(|id| id.trim().to_owned()).unwrap_or_default();
        let redirect = raw
            .redirect_host
            .or_else(|| entry.map(|e| e.redirect_host.to_owned()))
            .unwrap_or_else(|| "127.0.0.1".to_owned());
        let settings = Self {
            auth_url,
            token_url,
            scopes,
            redirect_host: redirect.trim().to_ascii_lowercase(),
            extra_params: entry
                .map(|e| e.extra_params.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect())
                .unwrap_or_default(),
            client_id,
            client_secret: raw.client_secret.map(Secret::new),
            token_file: PathBuf::new(),
        };
        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.client_id.is_empty() {
            return Err(invalid("`oauth_client_id` is required when auth = \"oauth2\"".to_owned()));
        }
        if self.client_id.chars().any(char::is_control) {
            return Err(invalid("`oauth_client_id` must not hold control characters".to_owned()));
        }
        if self.client_secret.as_ref().is_some_and(|s| s.expose().is_empty()) {
            return Err(invalid("`oauth_client_secret` is empty; leave it out or give the secret".to_owned()));
        }
        check_address("oauth_auth_url", &self.auth_url, false)?;
        check_address("oauth_token_url", &self.token_url, true)?;
        if self.scopes.is_empty() {
            return Err(invalid("`oauth_scopes` must list at least one scope".to_owned()));
        }
        if self.scopes.iter().any(|s| s.is_empty() || s.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"')) {
            return Err(invalid("each of `oauth_scopes` must be one word with no blanks".to_owned()));
        }
        if !REDIRECT_HOSTS.contains(&self.redirect_host.as_str()) {
            return Err(invalid(format!(
                "`oauth_redirect_host` must be one of {}; \"{}\" is not",
                REDIRECT_HOSTS.join(", "),
                self.redirect_host
            )));
        }
        Ok(())
    }
}

/// The provider named by the file, or the one the mail host belongs to.
fn chosen_entry(named: Option<&str>, imap_host: &str) -> Result<Option<&'static ProviderEntry>, ConfigError> {
    match named {
        Some(key) => provider_table::by_key(key).map(Some).ok_or_else(|| {
            invalid(format!("`oauth_provider` must be one of: {}; \"{}\" is not", provider_table::keys(), key.trim()))
        }),
        None => Ok(provider_table::by_host(imap_host)),
    }
}

/// The file's value, else the provider's, else a sentence that says what to set.
fn pick(
    own: Option<String>,
    from_table: Option<&'static str>,
    key: &str,
    imap_host: &str,
) -> Result<String, ConfigError> {
    own.map(|text| text.trim().to_owned())
        .or_else(|| from_table.map(str::to_owned))
        .ok_or_else(|| missing_provider(key, imap_host))
}

fn missing_provider(key: &str, imap_host: &str) -> ConfigError {
    let host = format!(" for the mail server \"{imap_host}\"");
    invalid(format!(
        "no sign-in provider is known{host} and `{key}` is not set: set `oauth_provider` to one of {}, \
         or set `oauth_auth_url`, `oauth_token_url` and `oauth_scopes`",
        provider_table::keys()
    ))
}

/// `https` only, except (for the token endpoint) plain `http` to this machine.
fn check_address(key: &str, text: &str, loopback_http_ok: bool) -> Result<(), ConfigError> {
    let url = Url::parse(text).map_err(|why| invalid(format!("`{key}` is not a usable address: {why}")))?;
    if url.https || (loopback_http_ok && is_loopback_host(&url.host)) {
        return Ok(());
    }
    let extra = if loopback_http_ok { ", except plain http to this machine" } else { "" };
    Err(invalid(format!("`{key}` must be an https:// address{extra}")))
}

#[cfg(test)]
include!("account_oauth.tests.rs");
