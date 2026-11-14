//! Signing in: with a user name and password (IMAP LOGIN), or with an access
//! token from the account's token source (AUTHENTICATE XOAUTH2). The
//! connection is already protected by then, or is a loopback one.

use std::time::Duration;

use async_imap::error::Error as ImapError;
use async_imap::{Authenticator, Client, Session};

use super::error::{ConnectError, server_words};
use super::stream::MailStream;
use crate::account_config::AccountConfig;
use crate::oauth::{HttpLimits, TokenError, TokenSource, xoauth2_raw};

/// Sends the credentials. A refusal is `AuthFailed` with the server's words
/// (`OAuthRefused` for an OAuth2 account); anything else is a connection
/// problem.
pub async fn sign_in(
    client: Client<MailStream>,
    config: &AccountConfig,
    limit: Duration,
) -> Result<Session<MailStream>, ConnectError> {
    if config.oauth.is_some() {
        return sign_in_with_token(client, config, limit).await;
    }
    let attempt = client.login(&config.username, config.password.expose());
    match tokio::time::timeout(limit, attempt).await {
        Ok(Ok(session)) => Ok(session),
        Ok(Err((ImapError::No(text) | ImapError::Bad(text), _))) => {
            Err(ConnectError::AuthFailed(server_words(&text)))
        }
        Ok(Err((other, _))) => Err(ConnectError::Unreachable(other.to_string())),
        Err(_) => Err(ConnectError::Unreachable("the login timed out".to_owned())),
    }
}

/// Answers the server's first challenge with the XOAUTH2 response and any
/// later one (the server's error text) with an empty line, after which the
/// server ends the exchange with NO. The IMAP library base64-encodes what is
/// returned here, so the response is given as it is, not already encoded.
struct TokenResponder {
    blob: Option<Vec<u8>>,
}

impl Authenticator for TokenResponder {
    type Response = Vec<u8>;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        self.blob.take().unwrap_or_default()
    }
}

async fn sign_in_with_token(
    client: Client<MailStream>,
    config: &AccountConfig,
    limit: Duration,
) -> Result<Session<MailStream>, ConnectError> {
    let token = access_token(config, limit).await?;
    let response = xoauth2_raw(&config.username, token.expose()).map_err(ConnectError::OAuthRefused)?;
    let responder = TokenResponder { blob: Some(response.into_bytes()) };
    match tokio::time::timeout(limit, client.authenticate("XOAUTH2", responder)).await {
        Ok(Ok(session)) => Ok(session),
        Ok(Err((ImapError::No(text) | ImapError::Bad(text), _))) => {
            // Whatever the server says back must not repeat the token.
            Err(ConnectError::OAuthRefused(server_words(&text).replace(token.expose(), "***")))
        }
        Ok(Err((other, _))) => Err(ConnectError::Unreachable(other.to_string())),
        Err(_) => Err(ConnectError::Unreachable("the login timed out".to_owned())),
    }
}

/// A fresh access token, with the failure sorted into "sign in again" and
/// "try again later".
async fn access_token(config: &AccountConfig, limit: Duration) -> Result<model::Secret, ConnectError> {
    let Some(source) = TokenSource::with_limits(config, HttpLimits::within(limit)) else {
        return Err(ConnectError::OAuthRefused("this account is not set up for OAuth2".to_owned()));
    };
    // The source has its own deadlines; this one covers the file and the lock.
    match tokio::time::timeout(limit.saturating_mul(2), source.access_token()).await {
        Ok(Ok(token)) => Ok(token),
        Ok(Err(TokenError::Permanent(text))) => Err(ConnectError::OAuthRefused(text)),
        Ok(Err(TokenError::Transient(text))) => Err(ConnectError::SignInTrouble(text)),
        Err(_) => Err(ConnectError::SignInTrouble("getting the access token took too long".to_owned())),
    }
}
