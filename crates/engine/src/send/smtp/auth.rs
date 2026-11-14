//! Signing in with a user name and password: AUTH PLAIN, or AUTH LOGIN for a
//! server that offers only that. Nothing is sent unless the connection is
//! under TLS, or is the plain loopback one the account file allows.

use super::error::{SmtpError, Step};
use super::link::Link;
use super::caps::ServerCaps;
use crate::account_config::Security;
use crate::account_smtp::SmtpSettings;
use crate::send::base64;

impl Link {
    /// Signs in when the server asks for it. A server that offers no sign-in
    /// at all (a trusted relay) is used as it is.
    pub async fn sign_in(&mut self, settings: &SmtpSettings, caps: &ServerCaps) -> Result<(), SmtpError> {
        if caps.auth.is_empty() {
            return Ok(());
        }
        if !self.secure && settings.security != Security::None {
            return Err(SmtpError::TlsFailed("the connection is not secure yet, so the password was not sent".to_owned()));
        }
        if caps.offers_auth("PLAIN") {
            self.auth_plain(settings).await
        } else if caps.offers_auth("LOGIN") {
            self.auth_login(settings).await
        } else {
            Err(SmtpError::SignInUnsupported(caps.auth.clone()))
        }
    }

    async fn auth_plain(&mut self, settings: &SmtpSettings) -> Result<(), SmtpError> {
        let mut blob = Vec::new();
        blob.push(0);
        blob.extend_from_slice(settings.username.as_bytes());
        blob.push(0);
        blob.extend_from_slice(settings.password.expose().as_bytes());
        let reply = self.ask(&format!("AUTH PLAIN {}", base64::encode(&blob)), Step::SignIn).await?;
        self.expect_signed_in(&reply)
    }

    async fn auth_login(&mut self, settings: &SmtpSettings) -> Result<(), SmtpError> {
        let asked = self.ask("AUTH LOGIN", Step::SignIn).await?;
        if asked.code != 334 {
            return Err(SmtpError::SignInRefused(self.said(&asked)));
        }
        let user = base64::encode(settings.username.as_bytes());
        let asked = self.ask(&user, Step::SignIn).await?;
        if asked.code != 334 {
            return Err(SmtpError::SignInRefused(self.said(&asked)));
        }
        let secret = base64::encode(settings.password.expose().as_bytes());
        let reply = self.ask(&secret, Step::SignIn).await?;
        self.expect_signed_in(&reply)
    }

    fn expect_signed_in(&self, reply: &super::reply::Reply) -> Result<(), SmtpError> {
        if reply.code == 235 { Ok(()) } else { Err(SmtpError::SignInRefused(self.said(reply))) }
    }
}
