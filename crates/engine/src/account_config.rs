//! The account file the real backend reads: where the mail server is and how
//! to sign in.
//!
//! The password and the client secret are not kept in this file by the program
//! any more: they live in the encrypted vault (`vault/`), and the file says
//! `secrets = "vault"`. A file written by an older version, or by hand, may
//! still hold `password` (and `smtp_password`, `oauth_client_secret`) in plain
//! text, which keeps working until the program moves it into the vault at its
//! next start (see `account_secrets.rs`). A file that holds a password or
//! client secret is refused on Unix when other users can read it.
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
//! password = "app-password"  # older files only; the program writes
//!                            # secrets = "vault" instead
//! trust = "verify"           # verify | any                (default verify)
//! read_only = true           # true | false                (default true)
//! poll_seconds = 60          # seconds between checks for new mail
//!                            # (default 60; 0 = never; 1 to 14 count as 15)
//!
//! # Sending (all optional; without `smtp_host` a send is refused):
//! smtp_host = "smtp.example.org"
//! smtp_port = 587            # 587 for starttls, 465 for implicit, 25 for none
//! smtp_security = "starttls" # starttls | implicit | none  (default starttls)
//! smtp_username = "alex"     # default: the mailbox `username`
//! smtp_password = "app-password" # default: the mailbox `password`
//! smtp_trust = "verify"      # verify | any                (default verify)
//! save_sent = true           # true | false                (default true)
//!
//! signature = "Alex Rivera\nacme.example"  # optional; \n starts a new line
//! folders = { trash = "Papierkorb" }       # optional; see `account_folders.rs`
//! ```
//!
//! `signature` is the text the composer puts under the cursor of a new
//! message, after a `-- ` line, and above the quoted text of a reply or a
//! forward (see `account_signature.rs` for what is cleaned). The program only
//! reads it; the Options screen edits it.
//!
//! Signing in with OAuth2 instead of a password (`auth = "oauth2"`, no
//! `password` needed) is described in `account_oauth.rs`: the keys
//! `oauth_provider`, `oauth_client_id`, `oauth_client_secret`,
//! `oauth_auth_url`, `oauth_token_url`, `oauth_scopes` and
//! `oauth_redirect_host`. The tokens are kept in the vault, under the
//! account's id (`oauth/<id>.token` beside the account file was where older
//! versions kept them).
//!
//! `security = "none"` (no TLS at all) and `trust = "any"` (accept any
//! certificate, for a local bridge with a self-signed one) are allowed for
//! loopback hosts only, and so are `smtp_security = "none"` and
//! `smtp_trust = "any"`.
//!
//! `read_only = true` (the default) keeps the account strictly read-only:
//! mailboxes are opened for reading, nothing is ever sent that could change
//! the server, and flag changes, moves, deletes and sending mail are
//! refused. Only `read_only = false` lets the program mark mail, move it,
//! delete it, send mail and save a copy of what it sent.
//!
//! `poll_seconds` is how often the program asks the server whether the open
//! folder (and the Inbox) changed, so new mail shows up without pressing
//! Sync. It only ever reads. `0` turns the automatic check off; a value from
//! 1 to 14 is raised to 15, the shortest wait allowed, without an error.

use std::path::{Path, PathBuf};

use model::{Secret, wipe_text};
use serde::Deserialize;

pub use crate::account_error::ConfigError;
use crate::account_folders::FolderPaths;
use crate::account_oauth::{AuthKind, OAuthSettings, RawOAuth, token_file_in};
use crate::vault::SECRETS_FOLDER_NAME;
use crate::account_secrets::{self, Given};
pub use crate::account_signature::clean_signature;
use crate::account_smtp::RawSmtp;
pub use crate::account_security::{Security, Trust};
pub use crate::account_smtp::{SmtpAuth, SmtpSettings};
pub use crate::loopback_host::is_loopback_host;
use crate::vault::VaultView;

pub use model::{DEFAULT_POLL_SECONDS, MIN_POLL_SECONDS};

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
    /// Empty for an OAuth2 account. Never shown in debug output.
    pub password: Secret,
    pub trust: Trust,
    /// When true, nothing the program does can change the mailbox on the
    /// server. This is the default; changes must be switched on by the owner.
    pub read_only: bool,
    /// How to submit mail; see `SmtpSettings`.
    pub smtp: SmtpSettings,
    /// Seconds between automatic checks for new mail; 0 means never. A
    /// non-zero value is at least `MIN_POLL_SECONDS`.
    pub poll_seconds: u32,
    /// How an OAuth2 account signs in; `None` for a password account.
    pub oauth: Option<OAuthSettings>,
    /// The text the composer puts under a new message, lines joined with a
    /// newline; empty for none. See `account_signature`.
    pub signature: String,
    /// The folders the file names as Sent, Drafts, Trash, Archive and Spam,
    /// which win over the guessed roles. See `account_folders`.
    pub folders: FolderPaths,
    /// The folder this account keeps what it must not lose in: the copies of
    /// sent mail not yet saved on the server (and where an older version kept
    /// its sign-in tokens). `None` for text parsed in memory, which has no
    /// folder of its own.
    pub state_dir: Option<PathBuf>,
}

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
    #[serde(default, deserialize_with = "secret_field")]
    password: Option<Secret>,
    /// `"vault"`: the passwords the file does not hold are in the vault.
    secrets: Option<String>,
    #[serde(default)]
    auth: AuthKind,
    oauth_provider: Option<String>,
    oauth_client_id: Option<String>,
    #[serde(default, deserialize_with = "secret_field")]
    oauth_client_secret: Option<Secret>,
    oauth_auth_url: Option<String>,
    oauth_token_url: Option<String>,
    oauth_scopes: Option<Vec<String>>,
    oauth_redirect_host: Option<String>,
    #[serde(default)]
    trust: Trust,
    #[serde(default = "read_only_by_default")]
    read_only: bool,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_security: Option<Security>,
    smtp_username: Option<String>,
    #[serde(default, deserialize_with = "secret_field")]
    smtp_password: Option<Secret>,
    smtp_trust: Option<Trust>,
    save_sent: Option<bool>,
    poll_seconds: Option<u32>,
    signature: Option<String>,
    #[serde(default)]
    folders: FolderPaths,
}

/// Reads an optional text value of the file straight into a `Secret`, so the
/// password is never a plain `String` that outlives its parse.
fn secret_field<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Secret>, D::Error> {
    Option::<String>::deserialize(deserializer).map(|text| text.map(Secret::new))
}

/// The wait between checks the file asks for, as it is used: absent means the
/// default, 0 means never, and anything shorter than the minimum is raised.
fn resolve_poll_seconds(asked: Option<u32>) -> u32 {
    match asked {
        None => DEFAULT_POLL_SECONDS,
        Some(0) => 0,
        Some(seconds) => seconds.max(MIN_POLL_SECONDS),
    }
}

/// Accounts are read-only unless the file says otherwise.
fn read_only_by_default() -> bool {
    true
}

/// The OAuth2 settings of an account that asks for them; `None` for a
/// password account.
fn resolve_oauth(raw: &mut RawAccount) -> Result<Option<OAuthSettings>, ConfigError> {
    if raw.auth != AuthKind::Oauth2 {
        return Ok(None);
    }
    let asked = RawOAuth {
        provider: raw.oauth_provider.take(),
        client_id: raw.oauth_client_id.take(),
        client_secret: raw.oauth_client_secret.take(),
        auth_url: raw.oauth_auth_url.take(),
        token_url: raw.oauth_token_url.take(),
        scopes: raw.oauth_scopes.take(),
        redirect_host: raw.oauth_redirect_host.take(),
    };
    OAuthSettings::resolve(asked, raw.imap_host.trim()).map(Some)
}

/// The mailbox password: whatever the file or the vault says for a password
/// account (which must say something), nothing for an OAuth2 account.
fn mailbox_password(given: Option<Secret>, oauth: bool) -> Result<Secret, ConfigError> {
    if oauth {
        return Ok(Secret::new(""));
    }
    given.ok_or_else(|| {
        ConfigError::Parse("missing field `password` (it is only optional when auth = \"oauth2\")".to_owned())
    })
}

impl AccountConfig {
    /// Reads and checks the account file at `path`, with no vault: a file
    /// that says its password is in the vault is refused with the sentence for
    /// a password that is not there. An OAuth2 account keeps its tokens in the
    /// vault in the `secrets` folder beside the file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        Self::from_file_with_vault(path, &VaultView::none())
    }

    /// As `from_file`, with the vault a `secrets = "vault"` file's passwords
    /// are read from.
    pub fn from_file_with_vault(path: &Path, vault: &VaultView) -> Result<Self, ConfigError> {
        check_private(path)?;
        let mut text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.to_string()))?;
        let folder = path.parent().unwrap_or_else(|| Path::new(""));
        let parsed = Self::from_toml_str_with_vault(&text, vault);
        // The file text holds the password and the client secret.
        wipe_text(&mut text);
        // The tokens go to the vault the secrets were read from; with none, to
        // the `secrets` folder beside the file (the `--account` convention).
        let secrets = vault.dir().map_or_else(|| folder.join(SECRETS_FOLDER_NAME), Path::to_path_buf);
        parsed.map(|config| config.with_state_dir(folder).with_secrets_dir(&secrets))
    }

    /// Gives the account its folder: any account keeps the copies of sent mail
    /// it has not yet saved in `<dir>/sent_copies`, and an OAuth2 account's
    /// token file of an older version, `<dir>/oauth/<id>.token`, is read from
    /// there and moved into the vault. For text parsed in memory, which has no
    /// folder of its own.
    pub fn with_state_dir(mut self, dir: &Path) -> Self {
        self.state_dir = Some(dir.to_path_buf());
        let file = token_file_in(dir, &self.id);
        if let Some(oauth) = self.oauth.as_mut() {
            oauth.token_place.set_id(&self.id);
            oauth.token_place.set_legacy_file(&file);
        }
        self
    }

    /// Gives an OAuth2 account the folder of the vault its sign-in tokens are
    /// kept in (the `secrets` folder of the data folder). Nothing for a
    /// password account.
    pub fn with_secrets_dir(mut self, dir: &Path) -> Self {
        if let Some(oauth) = self.oauth.as_mut() {
            oauth.token_place.set_id(&self.id);
            oauth.token_place.set_vault_dir(dir);
        }
        self
    }

    /// Parses and checks account file text, with no vault.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        Self::from_toml_str_with_vault(text, &VaultView::none())
    }

    /// Parses and checks account file text; a password the text does not hold
    /// is read from `vault` when the text says `secrets = "vault"`.
    pub fn from_toml_str_with_vault(text: &str, vault: &VaultView) -> Result<Self, ConfigError> {
        let mut raw: RawAccount = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        let default_port = match raw.security {
            Security::Implicit => 993,
            Security::Starttls | Security::None => 143,
        };
        let mut oauth = resolve_oauth(&mut raw)?;
        let given = Given {
            password: raw.password.take(),
            smtp_password: raw.smtp_password.take(),
            client_secret: oauth.as_mut().and_then(|settings| settings.client_secret.take()),
        };
        let given = account_secrets::resolve(&raw.id, raw.secrets.as_deref(), oauth.is_some(), given, vault)?;
        if let Some(settings) = oauth.as_mut() {
            settings.client_secret = given.client_secret;
        }
        raw.smtp_password = given.smtp_password;
        let password = mailbox_password(given.password, oauth.is_some())?;
        let smtp = SmtpSettings::resolve(
            RawSmtp {
                host: raw.smtp_host,
                port: raw.smtp_port,
                security: raw.smtp_security,
                username: raw.smtp_username,
                password: raw.smtp_password,
                trust: raw.smtp_trust,
                save_sent: raw.save_sent,
            },
            &raw.username,
            &password,
            oauth.is_some(),
        );
        let config = Self {
            name: raw.name.unwrap_or_else(|| raw.address.clone()),
            id: raw.id,
            address: raw.address,
            imap_host: raw.imap_host.trim().to_owned(),
            imap_port: raw.imap_port.unwrap_or(default_port),
            security: raw.security,
            username: raw.username,
            password,
            trust: raw.trust,
            read_only: raw.read_only,
            smtp,
            poll_seconds: resolve_poll_seconds(raw.poll_seconds),
            oauth,
            signature: clean_signature(raw.signature.as_deref()),
            folders: raw.folders.checked()?,
            state_dir: None,
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
        if self.oauth.is_none() && self.password.expose().is_empty() {
            return invalid("`password` is empty".to_owned());
        }
        self.check_loopback_only()?;
        self.smtp.validate()
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
