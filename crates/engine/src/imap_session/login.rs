//! Signing in: with a user name and password (IMAP LOGIN, or AUTHENTICATE
//! PLAIN where the server forbids LOGIN), or with an access token from the
//! account's token source (AUTHENTICATE XOAUTH2). The connection is already
//! protected by then, or is a loopback one.

use std::time::Duration;

use async_imap::error::Error as ImapError;
use async_imap::{Authenticator, Client, Session};

use super::error::{ConnectError, server_words_without};
use super::login_modes::{self, Method};
use super::stream::MailStream;
use crate::account_config::AccountConfig;
use crate::oauth::{HttpLimits, TokenError, TokenSource, xoauth2_raw};
use crate::scrub::plain_blob;

/// The sentence for a server that allows no password sign-in on this
/// connection.
const NO_PASSWORD_SIGN_IN: &str = "the server does not accept a password here (it says LOGINDISABLED and \
     offers no AUTH=PLAIN), so the password was not sent; use a protected connection (STARTTLS or TLS) \
     or change the server's setting";

/// Sends the credentials. A refusal is `AuthFailed` with the server's words
/// (`OAuthRefused` for an OAuth2 account); anything else is a connection
/// problem.
pub async fn sign_in(
    mut client: Client<MailStream>,
    config: &AccountConfig,
    limit: Duration,
) -> Result<Session<MailStream>, ConnectError> {
    if config.oauth.is_some() {
        return sign_in_with_token(client, config, limit).await;
    }
    let (user, password) = (config.username.as_str(), config.password.expose());
    let modes = login_modes::ask(&mut client, limit).await?;
    let blob = plain_blob(user, password);
    let secrets = [password, blob.as_str()];
    let result = match modes.method_for(user, password) {
        Method::Login => tokio::time::timeout(limit, client.login(user, password)).await,
        Method::Plain => {
            let responder = OneAnswer { blob: Some(plain_answer(user, password)) };
            tokio::time::timeout(limit, client.authenticate("PLAIN", responder)).await
        }
        Method::Refuse => return Err(ConnectError::PasswordNotAccepted(NO_PASSWORD_SIGN_IN.to_owned())),
    };
    match result {
        Ok(Ok(session)) => Ok(session),
        Ok(Err((ImapError::No(text) | ImapError::Bad(text), _))) => {
            Err(ConnectError::AuthFailed(server_words_without(&text, &secrets)))
        }
        Ok(Err((other, _))) => Err(ConnectError::Unreachable(server_words_without(&other.to_string(), &secrets))),
        Err(_) => Err(ConnectError::Unreachable("the login timed out".to_owned())),
    }
}

/// The bytes of a PLAIN answer: no authorization name, the user, the password.
fn plain_answer(user: &str, password: &str) -> Vec<u8> {
    let mut answer = vec![0u8];
    answer.extend_from_slice(user.as_bytes());
    answer.push(0);
    answer.extend_from_slice(password.as_bytes());
    answer
}

/// Answers the server's first challenge with the sign-in response and any
/// later one (the server's error text) with an empty line, after which the
/// server ends the exchange with NO. The IMAP library base64-encodes what is
/// returned here, so the response is given as it is, not already encoded.
struct OneAnswer {
    blob: Option<Vec<u8>>,
}

impl Authenticator for OneAnswer {
    type Response = Vec<u8>;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        self.blob.take().unwrap_or_default()
    }
}

/// Signs in with the account's access token. When the server refuses a token
/// that was not just issued (the local clock may be wrong about how long it
/// has left), one fresh token is asked for and the sign-in is tried once more
/// on the same connection, before the refusal is called final.
async fn sign_in_with_token(
    client: Client<MailStream>,
    config: &AccountConfig,
    limit: Duration,
) -> Result<Session<MailStream>, ConnectError> {
    let Some(source) = TokenSource::with_limits(config, HttpLimits::within(limit)) else {
        return Err(ConnectError::OAuthRefused("this account is not set up for OAuth2".to_owned()));
    };
    let issued = within(limit, source.issue()).await?;
    let (first, client) = match authenticate(client, config, &issued.token, limit).await {
        Ok(session) => return Ok(session),
        Err(Refused { error, client: Some(client) }) if !issued.renewed => (error, client),
        Err(refused) => return Err(refused.error),
    };
    let fresh = within(limit, source.after_refusal(&issued.token)).await?;
    if fresh == issued.token {
        return Err(first);
    }
    authenticate(client, config, &fresh, limit).await.map_err(|refused| refused.error)
}

/// A sign-in that failed. The connection comes back when the server refused
/// the token and is still there to be asked again.
struct Refused {
    error: ConnectError,
    client: Option<Client<MailStream>>,
}

impl Refused {
    fn plain(error: ConnectError) -> Self {
        Self { error, client: None }
    }
}

/// One AUTHENTICATE XOAUTH2 with `token`.
async fn authenticate(
    client: Client<MailStream>,
    config: &AccountConfig,
    token: &model::Secret,
    limit: Duration,
) -> Result<Session<MailStream>, Refused> {
    let response = xoauth2_raw(&config.username, token.expose()).map_err(|text| Refused::plain(ConnectError::OAuthRefused(text)))?;
    // Whatever the server says back must not repeat the token, or the sign-in
    // line that holds it, which is the response in base64.
    let secrets = [token.expose(), response.as_str()];
    let responder = OneAnswer { blob: Some(response.clone().into_bytes()) };
    match tokio::time::timeout(limit, client.authenticate("XOAUTH2", responder)).await {
        Ok(Ok(session)) => Ok(session),
        Ok(Err((ImapError::No(text) | ImapError::Bad(text), client))) => Err(Refused {
            error: ConnectError::OAuthRefused(server_words_without(&text, &secrets)),
            client: Some(client),
        }),
        Ok(Err((other, _))) => {
            Err(Refused::plain(ConnectError::Unreachable(server_words_without(&other.to_string(), &secrets))))
        }
        Err(_) => Err(Refused::plain(ConnectError::Unreachable("the login timed out".to_owned()))),
    }
}

/// A token step, with the failure sorted into "sign in again" and "try again
/// later". The source has its own deadlines; this one covers the file and the
/// lock.
async fn within<T>(
    limit: Duration,
    work: impl std::future::Future<Output = Result<T, TokenError>>,
) -> Result<T, ConnectError> {
    match tokio::time::timeout(limit.saturating_mul(2), work).await {
        Ok(Ok(found)) => Ok(found),
        Ok(Err(TokenError::Permanent(text))) => Err(ConnectError::OAuthRefused(text)),
        Ok(Err(TokenError::Transient(text))) => Err(ConnectError::SignInTrouble(text)),
        Err(_) => Err(ConnectError::SignInTrouble("getting the access token took too long".to_owned())),
    }
}

#[cfg(test)]
include!("login.tests.rs");
