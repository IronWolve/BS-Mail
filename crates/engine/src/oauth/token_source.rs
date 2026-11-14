//! The one place that hands out access tokens. Each call reads the token
//! file, refreshes when fewer than 60 seconds are left, writes the new tokens
//! back, and returns the access token. A process-wide lock covers the whole
//! check-refresh-write, so two connections that find the token ending do not
//! both refresh: the second one reads what the first wrote.

use model::Secret;
use tokio::sync::Mutex;

use super::error::TokenError;
use super::http::HttpLimits;
use super::token_endpoint::{self, Tokens};
use super::token_file::{self, FileError, TokenRecord};
use crate::account_config::AccountConfig;
use crate::account_oauth::OAuthSettings;
use crate::send::now_unix;
use crate::sign_in_advice::sign_in_first;

/// An access token with fewer seconds than this left is refreshed first.
pub const REFRESH_BELOW_SECONDS: i64 = 60;

/// Held while the token file is read, refreshed and written.
static REFRESH_LOCK: Mutex<()> = Mutex::const_new(());

/// Gives out access tokens for one account.
#[derive(Debug, Clone)]
pub struct TokenSource {
    settings: OAuthSettings,
    limits: HttpLimits,
}

impl TokenSource {
    /// The source for an OAuth2 account; `None` for a password account.
    pub fn for_account(config: &AccountConfig) -> Option<Self> {
        Self::with_limits(config, HttpLimits::production())
    }

    /// Like `for_account`, with the waits of a request chosen by the caller.
    pub fn with_limits(config: &AccountConfig, limits: HttpLimits) -> Option<Self> {
        config.oauth.clone().map(|settings| Self { settings, limits })
    }

    /// An access token that is good for at least a minute more.
    pub async fn access_token(&self) -> Result<Secret, TokenError> {
        let path = &self.settings.token_file;
        if path.as_os_str().is_empty() {
            return Err(TokenError::Permanent(
                "this account has no place set for its sign-in tokens (the account must be read from a file)".to_owned(),
            ));
        }
        let _held = REFRESH_LOCK.lock().await;
        let record = token_file::read(path).map_err(read_failure)?;
        if record.expires_at - now_unix() >= REFRESH_BELOW_SECONDS {
            return Ok(record.access_token);
        }
        let fresh = token_endpoint::refresh(&self.settings, &record.refresh_token, &self.limits).await?;
        let renewed = renewed_record(&record, fresh);
        token_file::write(path, &renewed).map_err(|error| TokenError::Transient(file_words(error)))?;
        Ok(renewed.access_token)
    }
}

/// Saves the tokens of a fresh sign-in, under the same lock the refresh uses.
pub async fn store_sign_in(settings: &OAuthSettings, tokens: Tokens) -> Result<(), TokenError> {
    let Some(refresh_token) = tokens.refresh_token else {
        return Err(TokenError::Permanent("the provider gave no lasting sign-in (no refresh token)".to_owned()));
    };
    let record = TokenRecord {
        access_token: tokens.access_token,
        refresh_token,
        expires_at: now_unix().saturating_add(i64::try_from(tokens.expires_in).unwrap_or(i64::MAX)),
    };
    let _held = REFRESH_LOCK.lock().await;
    token_file::write(&settings.token_file, &record).map_err(|error| TokenError::Permanent(file_words(error)))
}

/// The record after a refresh: the new access token and lifetime, and the
/// old refresh token when the reply brought none.
fn renewed_record(old: &TokenRecord, fresh: Tokens) -> TokenRecord {
    TokenRecord {
        access_token: fresh.access_token,
        refresh_token: fresh.refresh_token.unwrap_or_else(|| old.refresh_token.clone()),
        expires_at: now_unix().saturating_add(i64::try_from(fresh.expires_in).unwrap_or(i64::MAX)),
    }
}

fn read_failure(error: FileError) -> TokenError {
    match error {
        FileError::Missing => TokenError::Permanent(format!("not signed in yet, {}", sign_in_first())),
        FileError::Failed(text) => TokenError::Permanent(text),
    }
}

fn file_words(error: FileError) -> String {
    match error {
        FileError::Missing => "the token file is missing".to_owned(),
        FileError::Failed(text) => text,
    }
}

#[cfg(test)]
include!("token_source.tests.rs");
