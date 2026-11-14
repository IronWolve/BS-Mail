//! Talking to the provider's token endpoint: trading the authorization code
//! for tokens, and trading the refresh token for a new access token.
//!
//! An `error` reply of `invalid_grant`, `invalid_client` or
//! `unauthorized_client` is permanent (the person must sign in again). Every
//! other error reply, and any network trouble, is transient.

use model::Secret;

use super::authorize_url::form_body;
use super::error::{TokenError, provider_words};
use super::http::{HttpError, HttpLimits, Url, post_form};
use super::json::{self, Value};
use crate::account_oauth::OAuthSettings;

/// The longest lifetime of an access token that is believed: a year.
const MAX_LIFETIME_SECONDS: u64 = 365 * 24 * 3600;

/// What the endpoint handed back.
#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: Secret,
    /// Absent in many refresh replies: the old one then stays in use.
    pub refresh_token: Option<Secret>,
    pub expires_in: u64,
}

/// Trades an authorization code (and the PKCE verifier) for tokens. The
/// provider must have given a refresh token, or the sign-in would not last.
pub async fn exchange_code(
    settings: &OAuthSettings,
    code: &Secret,
    verifier: &Secret,
    redirect_uri: &str,
    limits: &HttpLimits,
) -> Result<Tokens, TokenError> {
    let mut pairs = vec![
        ("grant_type", "authorization_code"),
        ("code", code.expose()),
        ("redirect_uri", redirect_uri),
        ("client_id", settings.client_id.as_str()),
        ("code_verifier", verifier.expose()),
    ];
    if let Some(secret) = &settings.client_secret {
        pairs.push(("client_secret", secret.expose()));
    }
    let secrets = [code.expose(), verifier.expose()];
    let tokens = request(settings, &form_body(&pairs), &all_secrets(settings, &secrets), limits).await?;
    if tokens.refresh_token.is_none() {
        return Err(TokenError::Permanent(
            "the provider gave no lasting sign-in (its reply held no refresh token), so the program \
             could not stay signed in; check that the client may ask for offline access and try again"
                .to_owned(),
        ));
    }
    Ok(tokens)
}

/// Trades the refresh token for a new access token.
pub async fn refresh(
    settings: &OAuthSettings,
    refresh_token: &Secret,
    limits: &HttpLimits,
) -> Result<Tokens, TokenError> {
    let mut pairs = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.expose()),
        ("client_id", settings.client_id.as_str()),
    ];
    if let Some(secret) = &settings.client_secret {
        pairs.push(("client_secret", secret.expose()));
    }
    let secrets = [refresh_token.expose()];
    request(settings, &form_body(&pairs), &all_secrets(settings, &secrets), limits).await
}

/// The request's own secrets plus the client secret, for scrubbing what the
/// provider says back.
fn all_secrets<'a>(settings: &'a OAuthSettings, own: &[&'a str]) -> Vec<&'a str> {
    let mut all = own.to_vec();
    if let Some(secret) = &settings.client_secret {
        all.push(secret.expose());
    }
    all
}

async fn request(
    settings: &OAuthSettings,
    form: &str,
    secrets: &[&str],
    limits: &HttpLimits,
) -> Result<Tokens, TokenError> {
    let url = Url::parse(&settings.token_url)
        .map_err(|why| TokenError::Permanent(format!("the token address is not usable: {why}")))?;
    let reply = post_form(&url, form, limits).await.map_err(|error| match error {
        HttpError::NotAllowed(_) => TokenError::Permanent(error.to_string()),
        other => TokenError::Transient(other.to_string()),
    })?;
    let text = String::from_utf8_lossy(&reply.body);
    let parsed = json::parse(&text);
    if let Ok(value) = &parsed {
        if let Some(error) = value.get("error").and_then(Value::as_text) {
            return Err(error_reply(error, value, secrets));
        }
    }
    if !(200..300).contains(&reply.status) {
        return Err(TokenError::Transient(format!(
            "the sign-in service answered with HTTP {} and no explanation",
            reply.status
        )));
    }
    let value = parsed.map_err(|_| TokenError::Transient("the sign-in service's reply is not readable JSON".to_owned()))?;
    tokens_of(&value)
}

/// An `error` reply, classified and phrased from the provider's own words.
fn error_reply(error: &str, value: &Value, secrets: &[&str]) -> TokenError {
    let description = value.get("error_description").and_then(Value::as_text);
    let words = provider_words(error, description, secrets);
    let permanent = matches!(error, "invalid_grant" | "invalid_client" | "unauthorized_client");
    if permanent {
        TokenError::Permanent(format!("the provider ended or refused the sign-in: {words}"))
    } else {
        TokenError::Transient(format!("the provider's sign-in service reported: {words}"))
    }
}

fn tokens_of(value: &Value) -> Result<Tokens, TokenError> {
    let bad = |what: &str| TokenError::Transient(format!("the sign-in service's reply {what}"));
    let access = value.get("access_token").and_then(Value::as_text).filter(|t| !t.is_empty());
    let access = access.ok_or_else(|| bad("holds no access token"))?;
    let kind = value.get("token_type").and_then(Value::as_text);
    if !kind.is_some_and(|k| k.eq_ignore_ascii_case("bearer")) {
        return Err(bad("does not describe a bearer token"));
    }
    let lifetime = value.get("expires_in").and_then(Value::as_seconds).ok_or_else(|| bad("holds no usable lifetime"))?;
    let refresh = value.get("refresh_token").and_then(Value::as_text).filter(|t| !t.is_empty());
    Ok(Tokens {
        access_token: Secret::new(access),
        refresh_token: refresh.map(Secret::new),
        expires_in: lifetime.min(MAX_LIFETIME_SECONDS),
    })
}

#[cfg(test)]
include!("token_endpoint.tests.rs");
