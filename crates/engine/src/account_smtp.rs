//! The sending half of the account file: where mail is submitted and how to
//! sign in there. Every option is optional; `smtp_host` has no default, and
//! without it a send is refused.
//!
//! The same loopback-only rules as for the mailbox connection apply:
//! `smtp_security = "none"` (no TLS at all) and `smtp_trust = "any"` (accept
//! any certificate) are allowed for loopback hosts only.
//!
//! An OAuth2 account signs in to the submission server with the same token
//! source as the mailbox (SASL XOAUTH2), unless the file sets `smtp_password`
//! itself: then that password is used as for any account.

use model::Secret;

use crate::account_config::{ConfigError, Security, Trust, is_loopback_host};

/// How the submission server is signed in to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpAuth {
    /// AUTH PLAIN or LOGIN with `username` and `password`.
    Password,
    /// AUTH XOAUTH2 with `username` and an access token from the account's
    /// token source.
    OAuth2,
}

/// How to submit mail for one account, checked and ready to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpSettings {
    /// The submission server. `None`: this account cannot send.
    pub host: Option<String>,
    /// 587 for `starttls`, 465 for `implicit`, 25 for `none`, unless set.
    pub port: u16,
    /// `starttls` unless set.
    pub security: Security,
    /// The mailbox user name unless set.
    pub username: String,
    /// The mailbox password unless set; empty when `auth` is `OAuth2`. Never
    /// shown in debug output.
    pub password: Secret,
    /// A password sign-in, or a token sign-in for an OAuth2 account that sets
    /// no `smtp_password`.
    pub auth: SmtpAuth,
    pub trust: Trust,
    /// Append what was sent to the Sent folder. On unless set.
    pub save_sent: bool,
}

/// The option values as the file spelled them, before defaults. Its debug
/// output never shows the password.
#[derive(Default)]
pub(crate) struct RawSmtp {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub security: Option<Security>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub trust: Option<Trust>,
    pub save_sent: Option<bool>,
}

impl std::fmt::Debug for RawSmtp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawSmtp")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("security", &self.security)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .field("trust", &self.trust)
            .field("save_sent", &self.save_sent)
            .finish()
    }
}

impl SmtpSettings {
    /// Fills in the defaults; the mailbox sign-in stands in for the missing
    /// sending one.
    pub(crate) fn resolve(
        raw: RawSmtp,
        mailbox_user: &str,
        mailbox_password: &Secret,
        mailbox_oauth: bool,
    ) -> Self {
        let auth = if mailbox_oauth && raw.password.is_none() { SmtpAuth::OAuth2 } else { SmtpAuth::Password };
        let security = raw.security.unwrap_or(Security::Starttls);
        let default_port = match security {
            Security::Starttls => 587,
            Security::Implicit => 465,
            Security::None => 25,
        };
        Self {
            host: raw.host.map(|host| host.trim().to_owned()),
            port: raw.port.unwrap_or(default_port),
            security,
            username: raw.username.unwrap_or_else(|| mailbox_user.to_owned()),
            password: raw
                .password
                .map_or_else(|| mailbox_password.clone(), Secret::new),
            auth,
            trust: raw.trust.unwrap_or_default(),
            save_sent: raw.save_sent.unwrap_or(true),
        }
    }

    /// Refuses values that cannot work or that would send a password or
    /// mail over a connection that is not protected, to a host that is not
    /// this machine.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |text: &str| Err(ConfigError::Invalid(text.to_owned()));
        if self.port == 0 {
            return invalid("`smtp_port` must not be 0");
        }
        if self.username.is_empty() {
            return invalid("`smtp_username` is empty");
        }
        if self.auth == SmtpAuth::Password && self.password.expose().is_empty() {
            return invalid("`smtp_password` is empty");
        }
        if self.username.contains('\0') || self.password.expose().contains('\0') {
            return invalid("`smtp_username` and `smtp_password` must not contain a NUL character");
        }
        let Some(host) = self.host.as_deref() else {
            return Ok(());
        };
        if host.is_empty() || host.chars().any(|c| c.is_whitespace() || c.is_control() || c == '/') {
            return invalid("`smtp_host` must be a host name or address");
        }
        self.check_loopback_only(host)
    }

    fn check_loopback_only(&self, host: &str) -> Result<(), ConfigError> {
        if is_loopback_host(host) {
            return Ok(());
        }
        if self.security == Security::None {
            return Err(ConfigError::Invalid(format!(
                "`smtp_security = \"none\"` is only allowed for a loopback host (localhost, \
                 127.x.x.x, ::1); \"{host}\" is not one. Use \"starttls\" or \"implicit\"."
            )));
        }
        if self.trust == Trust::Any {
            return Err(ConfigError::Invalid(format!(
                "`smtp_trust = \"any\"` is only allowed for a loopback host (localhost, \
                 127.x.x.x, ::1); \"{host}\" is not one. Use `smtp_trust = \"verify\"`."
            )));
        }
        Ok(())
    }
}
