//! Signing in: with a user name and password (AUTH PLAIN, or AUTH LOGIN for a
//! server that offers only that), or with an access token (AUTH XOAUTH2).
//! Nothing is sent unless the connection is under TLS, or is the plain
//! loopback one the account file allows.

use super::error::{SmtpError, Step};
use super::link::Link;
use super::caps::ServerCaps;
use crate::account_config::Security;
use crate::account_smtp::{SmtpAuth, SmtpSettings};
use crate::oauth::{TokenError, TokenSource, xoauth2_blob};
use crate::scrub::plain_blob;
use crate::send::base64;

impl Link {
    /// Signs in when the server asks for it. A server that offers no sign-in
    /// at all (a trusted relay) is used as it is.
    pub async fn sign_in(
        &mut self,
        settings: &SmtpSettings,
        caps: &ServerCaps,
        tokens: Option<&TokenSource>,
    ) -> Result<(), SmtpError> {
        if caps.auth.is_empty() {
            return Ok(());
        }
        let oauth = settings.auth == SmtpAuth::OAuth2;
        if !self.secure && settings.security != Security::None {
            let what = if oauth { "the access token" } else { "the password" };
            return Err(SmtpError::TlsFailed(format!("the connection is not secure yet, so {what} was not sent")));
        }
        if oauth {
            self.auth_xoauth2(settings, caps, tokens).await
        } else if caps.offers_auth("PLAIN") {
            self.auth_plain(settings).await
        } else if caps.offers_auth("LOGIN") {
            self.auth_login(settings).await
        } else {
            Err(SmtpError::SignInUnsupported(caps.auth.clone()))
        }
    }

    async fn auth_plain(&mut self, settings: &SmtpSettings) -> Result<(), SmtpError> {
        let coded = plain_blob(&settings.username, settings.password.expose());
        self.secrets.push(coded.clone());
        let reply = self.ask(&format!("AUTH PLAIN {coded}"), Step::SignIn).await?;
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

    /// AUTH XOAUTH2 with a fresh access token. A 334 answer is the server's
    /// error text: it is answered with an empty line, and what follows is the
    /// refusal.
    async fn auth_xoauth2(
        &mut self,
        settings: &SmtpSettings,
        caps: &ServerCaps,
        tokens: Option<&TokenSource>,
    ) -> Result<(), SmtpError> {
        if !caps.offers_auth("XOAUTH2") {
            return Err(SmtpError::SignInUnsupported(caps.auth.clone()));
        }
        let Some(tokens) = tokens else {
            return Err(SmtpError::SignInEnded("this account has no sign-in token".to_owned()));
        };
        let wait = self.limits.connect.saturating_mul(2);
        let token = match tokio::time::timeout(wait, tokens.access_token()).await {
            Ok(Ok(token)) => token,
            Ok(Err(TokenError::Permanent(text))) => return Err(SmtpError::SignInEnded(text)),
            Ok(Err(TokenError::Transient(text))) => return Err(SmtpError::TokenUnavailable(text)),
            Err(_) => return Err(SmtpError::TokenUnavailable("getting the access token took too long".to_owned())),
        };
        // Whatever the server says back must not repeat the token.
        self.secrets.push(token.expose().to_owned());
        let blob = xoauth2_blob(&settings.username, token.expose()).map_err(SmtpError::SignInEnded)?;
        let reply = self.ask(&format!("AUTH XOAUTH2 {blob}"), Step::SignIn).await?;
        let last = if reply.code == 334 { self.ask("", Step::SignIn).await? } else { reply };
        if last.code == 235 { Ok(()) } else { Err(SmtpError::OAuthRefused(self.said(&last))) }
    }

    fn expect_signed_in(&self, reply: &super::reply::Reply) -> Result<(), SmtpError> {
        if reply.code == 235 { Ok(()) } else { Err(SmtpError::SignInRefused(self.said(reply))) }
    }
}
