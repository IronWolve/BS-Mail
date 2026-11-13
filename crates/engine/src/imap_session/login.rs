//! Signing in with a user name and password (IMAP LOGIN). The connection is
//! already protected by then, or is a loopback one.

use std::time::Duration;

use async_imap::error::Error as ImapError;
use async_imap::{Client, Session};

use super::error::ConnectError;
use super::stream::MailStream;
use crate::account_config::AccountConfig;

/// Sends the credentials. A refusal is `AuthFailed` with the server's words;
/// anything else is a connection problem.
pub async fn sign_in(
    client: Client<MailStream>,
    config: &AccountConfig,
    limit: Duration,
) -> Result<Session<MailStream>, ConnectError> {
    let attempt = client.login(&config.username, config.password.expose());
    match tokio::time::timeout(limit, attempt).await {
        Ok(Ok(session)) => Ok(session),
        Ok(Err((ImapError::No(text) | ImapError::Bad(text), _))) => {
            Err(ConnectError::AuthFailed(text))
        }
        Ok(Err((other, _))) => Err(ConnectError::Unreachable(other.to_string())),
        Err(_) => Err(ConnectError::Unreachable("the login timed out".to_owned())),
    }
}
