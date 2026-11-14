//! The account file the real backend reads: where the mail server is and how
//! to sign in.
//!
//! INTERIM, DEV-ONLY: the password sits in this file as plain text. That is a
//! stand-in until the encrypted vault exists; the file is refused on Unix when
//! other users can read it, and nothing here is meant for a shipped build.
//!
//! The file is flat TOML:
//!
//! ```toml
//! id = "main"
//! name = "Alex Rivera"
//! address = "alex@example.org"
//! imap_host = "imap.example.org"
//! imap_port = 993            # optional: 993 for implicit TLS, else 143
//! security = "implicit"      # implicit | starttls | none  (default implicit)
//! username = "alex"
//! password = "app-password"
//! trust = "verify"           # verify | any                (default verify)
//! read_only = true           # true | false                (default true)
//! ```
//!
//! `security = "none"` (no TLS at all) and `trust = "any"` (accept any
//! certificate, for a local bridge with a self-signed one) are allowed for
//! loopback hosts only.
//!
//! `read_only = true` (the default) keeps the account strictly read-only:
//! mailboxes are opened for reading, nothing is ever sent that could change
//! the server, and flag changes, moves and deletes are refused. Only
//! `read_only = false` lets the program mark mail, move it and delete it.

use std::fmt;
use std::net::IpAddr;
use std::path::Path;

use model::Secret;
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

/// One mail account, checked and ready to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    pub address: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub security: Security,
    pub username: String,
    pub password: Secret,
    pub trust: Trust,
    /// When true, nothing the program does can change the mailbox on the
    /// server. This is the default; changes must be switched on by the owner.
    pub read_only: bool,
}

/// Why an account file was refused.
#[derive(Debug)]
pub enum ConfigError {
    /// The file could not be read.
    Read(String),
    /// The text is not valid TOML or has a wrong or missing field.
    Parse(String),
    /// A value is present but not allowed.
    Invalid(String),
    /// Other users can read the file (Unix mode bits).
    Permissions { path: String, mode: u32 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(text) => write!(f, "cannot read the account file: {text}"),
            Self::Parse(text) => write!(f, "the account file is not valid: {text}"),
            Self::Invalid(text) => write!(f, "the account file is not allowed: {text}"),
            Self::Permissions { path, mode } => write!(
                f,
                "the account file {path} holds a password and is readable by other users \
                 (mode {mode:o}); run: chmod 600 {path}"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Deserialize)]
struct RawAccount {
    id: String,
    name: Option<String>,
    address: String,
    imap_host: String,
    imap_port: Option<u16>,
    #[serde(default)]
    security: Security,
    username: String,
    password: String,
    #[serde(default)]
    trust: Trust,
    #[serde(default = "read_only_by_default")]
    read_only: bool,
}

/// Accounts are read-only unless the file says otherwise.
fn read_only_by_default() -> bool {
    true
}

impl AccountConfig {
    /// Reads and checks the account file at `path`.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        check_private(path)?;
        let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.to_string()))?;
        Self::from_toml_str(&text)
    }

    /// Parses and checks account file text.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        let raw: RawAccount = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        let default_port = match raw.security {
            Security::Implicit => 993,
            Security::Starttls | Security::None => 143,
        };
        let config = Self {
            name: raw.name.unwrap_or_else(|| raw.address.clone()),
            id: raw.id,
            address: raw.address,
            imap_host: raw.imap_host.trim().to_owned(),
            imap_port: raw.imap_port.unwrap_or(default_port),
            security: raw.security,
            username: raw.username,
            password: Secret::new(raw.password),
            trust: raw.trust,
            read_only: raw.read_only,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |text: String| Err(ConfigError::Invalid(text));
        let id_ok = !self.id.is_empty()
            && self.id.len() <= 64
            && self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !id_ok {
            return invalid("`id` must be 1 to 64 letters, digits, `-` or `_`".to_owned());
        }
        let host_ok = !self.imap_host.is_empty()
            && !self.imap_host.chars().any(|c| c.is_whitespace() || c == '/');
        if !host_ok {
            return invalid("`imap_host` must be a host name or address".to_owned());
        }
        if self.imap_port == 0 {
            return invalid("`imap_port` must not be 0".to_owned());
        }
        if self.username.is_empty() {
            return invalid("`username` is empty".to_owned());
        }
        if self.password.expose().is_empty() {
            return invalid("`password` is empty".to_owned());
        }
        self.check_loopback_only()
    }

    fn check_loopback_only(&self) -> Result<(), ConfigError> {
        if is_loopback_host(&self.imap_host) {
            return Ok(());
        }
        let host = &self.imap_host;
        if self.security == Security::None {
            return Err(ConfigError::Invalid(format!(
                "`security = \"none\"` is only allowed for a loopback host (localhost, \
                 127.x.x.x, ::1); \"{host}\" is not one. Use \"implicit\" or \"starttls\"."
            )));
        }
        if self.trust == Trust::Any {
            return Err(ConfigError::Invalid(format!(
                "`trust = \"any\"` is only allowed for a loopback host (localhost, \
                 127.x.x.x, ::1); \"{host}\" is not one. Use `trust = \"verify\"`."
            )));
        }
        Ok(())
    }
}

/// True for `localhost` and for loopback addresses, with or without brackets.
pub fn is_loopback_host(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if bare.trim_end_matches('.').eq_ignore_ascii_case("localhost") {
        return true;
    }
    match bare.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => v4.is_loopback(),
        Ok(IpAddr::V6(v6)) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
        Err(_) => false,
    }
}

#[cfg(unix)]
fn check_private(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path).map_err(|e| ConfigError::Read(e.to_string()))?;
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(ConfigError::Permissions {
            path: path.display().to_string(),
            mode,
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_private(_path: &Path) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(test)]
include!("account_config.tests.rs");
