//! The one place that hands out access tokens. Each call reads the token
//! file, refreshes when fewer than 60 seconds are left, writes the new tokens
//! back, and returns the access token. A process-wide lock covers the whole
//! check-refresh-write, so two connections that find the token ending do not
//! both refresh: the second one reads what the first wrote.
//!
//! A mail server can refuse a token the local clock still thinks is good (the
//! clock stepped back, or the data folder came from a machine whose clock was
//! behind). `after_refusal` is for that: it gives a token to try once more.
//!
//! New tokens are never dropped on the floor: a write that fails is tried
//! again, and if it keeps failing the tokens are kept in memory and used
//! instead of the old file (see `unsaved_tokens`).

use std::path::Path;
use std::time::Duration;

use model::Secret;
use tokio::sync::Mutex;

use super::error::TokenError;
use super::http::HttpLimits;
use super::previous_token;
use super::token_endpoint::{self, Tokens};
use super::token_file::{self, FileError, TokenRecord};
use super::unsaved_tokens;
use crate::account_config::AccountConfig;
use crate::account_oauth::OAuthSettings;
use crate::send::now_unix;
use crate::sign_in_advice::sign_in_first;

/// An access token with fewer seconds than this left is refreshed first.
pub const REFRESH_BELOW_SECONDS: i64 = 60;

/// How often a write of new tokens is tried, and the pause between tries.
const WRITE_TRIES: u32 = 3;
const WRITE_PAUSE: Duration = Duration::from_millis(100);

/// Held while the token file is read, refreshed and written.
pub(super) static REFRESH_LOCK: Mutex<()> = Mutex::const_new(());

/// Gives out access tokens for one account.
#[derive(Debug, Clone)]
pub struct TokenSource {
    settings: OAuthSettings,
    limits: HttpLimits,
}

/// An access token, and whether the provider issued it during this very call.
#[derive(Debug, Clone)]
pub struct Issued {
    pub token: Secret,
    /// A token that was just issued is not worth renewing when a server
    /// refuses it: a second one would be no different.
    pub renewed: bool,
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
        self.issue().await.map(|issued| issued.token)
    }

    /// Like `access_token`, and says whether the token is brand new.
    pub async fn issue(&self) -> Result<Issued, TokenError> {
        let path = self.place()?;
        let _held = REFRESH_LOCK.lock().await;
        let record = current(path).await?;
        if record.expires_at - now_unix() >= REFRESH_BELOW_SECONDS {
            return Ok(Issued { token: record.access_token, renewed: false });
        }
        let token = self.renew(path, &record).await?;
        Ok(Issued { token, renewed: true })
    }

    /// A token to try after a mail server refused `refused`: the one another
    /// connection has fetched since, if there is one, else a fresh one from
    /// the provider. Asking for it is the caller's decision, made once per
    /// sign-in, so a server that refuses every token costs one request.
    pub async fn after_refusal(&self, refused: &Secret) -> Result<Secret, TokenError> {
        let path = self.place()?;
        let _held = REFRESH_LOCK.lock().await;
        let record = current(path).await?;
        let has_time = record.expires_at - now_unix() >= REFRESH_BELOW_SECONDS;
        if has_time && record.access_token != *refused {
            return Ok(record.access_token);
        }
        self.renew(path, &record).await
    }

    fn place(&self) -> Result<&Path, TokenError> {
        let path = &self.settings.token_file;
        if path.as_os_str().is_empty() {
            return Err(TokenError::Permanent(
                "this account has no place set for its sign-in tokens (the account must be read from a file)".to_owned(),
            ));
        }
        Ok(path)
    }

    /// Trades the refresh token for new tokens and saves them. The caller
    /// holds the lock.
    async fn renew(&self, path: &Path, old: &TokenRecord) -> Result<Secret, TokenError> {
        let fresh = token_endpoint::refresh(&self.settings, &old.refresh_token, &self.limits).await?;
        let renewed = renewed_record(old, fresh);
        save(path, &renewed).await?;
        Ok(renewed.access_token)
    }
}

/// The tokens to work from: the ones kept after a failed save (saved now, or
/// the error says why not), else the token file.
async fn current(path: &Path) -> Result<TokenRecord, TokenError> {
    match unsaved_tokens::waiting(path) {
        Some(kept) => save(path, &kept).await.map(|()| kept),
        None => token_file::read(path).map_err(read_failure),
    }
}

/// Writes `record`, trying a few times. A write that never works leaves the
/// record in memory, so the next call saves it first and the old refresh
/// token is not used again; the error is transient because that call may work.
async fn save(path: &Path, record: &TokenRecord) -> Result<(), TokenError> {
    let mut failure = None;
    for attempt in 0..WRITE_TRIES {
        match token_file::write(path, record) {
            Ok(()) => {
                unsaved_tokens::forget(path);
                return Ok(());
            }
            Err(error) => failure = Some(error),
        }
        if attempt + 1 < WRITE_TRIES {
            tokio::time::sleep(WRITE_PAUSE).await;
        }
    }
    unsaved_tokens::keep(path, record.clone());
    let words = failure.map_or_else(|| "the file could not be written".to_owned(), file_words);
    Err(TokenError::Transient(format!(
        "could not save the new sign-in ({words}); it is kept until the file can be written"
    )))
}

/// Saves the tokens of a fresh sign-in, under the same lock the refresh uses.
/// The token file that was there is set aside first (`previous_token`), so
/// giving up can put it back; if the write fails it is put back at once.
pub async fn store_sign_in(settings: &OAuthSettings, tokens: Tokens) -> Result<(), TokenError> {
    let Some(refresh_token) = tokens.refresh_token else {
        return Err(TokenError::Permanent("the provider gave no lasting sign-in (no refresh token)".to_owned()));
    };
    let record = TokenRecord {
        access_token: tokens.access_token,
        refresh_token,
        expires_at: now_unix().saturating_add(i64::try_from(tokens.expires_in).unwrap_or(i64::MAX)),
    };
    let path = &settings.token_file;
    let _held = REFRESH_LOCK.lock().await;
    previous_token::keep(path).map_err(TokenError::Permanent)?;
    unsaved_tokens::forget(path);
    if let Err(error) = token_file::write(path, &record) {
        let _ = previous_token::restore(path);
        return Err(TokenError::Permanent(file_words(error)));
    }
    Ok(())
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
