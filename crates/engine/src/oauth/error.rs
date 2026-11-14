//! What can go wrong while getting or using tokens, as one plain sentence.
//! Nothing secret is ever part of one: text that came from the provider is
//! reduced to its `error` and `error_description` words, cleaned of control
//! codes, scrubbed of the secrets this program sent, and cut short.

use std::fmt;

use crate::text_clean::single_line;

/// The most characters of the provider's own words that are ever shown.
pub const PROVIDER_WORDS_MAX: usize = 200;

/// Why an access token could not be had.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    /// Trying again cannot help: the person has to sign in again (no token
    /// file, a revoked or expired sign-in, a refused client).
    Permanent(String),
    /// Trouble that may pass: no network, a provider that is down.
    Transient(String),
}

impl TokenError {
    /// The sentence, without the kind.
    pub fn text(&self) -> &str {
        match self {
            Self::Permanent(text) | Self::Transient(text) => text,
        }
    }

    pub fn is_permanent(&self) -> bool {
        matches!(self, Self::Permanent(_))
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text())
    }
}

impl std::error::Error for TokenError {}

/// At most `limit` characters of `text`.
pub fn cut(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// The provider's own error words: `error`, then `error_description` when it
/// said one. Control codes are removed, every secret in `secrets` is replaced
/// by `***`, and the whole is cut to `PROVIDER_WORDS_MAX` characters.
pub fn provider_words(error: &str, description: Option<&str>, secrets: &[&str]) -> String {
    let mut words = single_line(error).trim().to_owned();
    if let Some(more) = description.map(|d| single_line(d).trim().to_owned()) {
        if !more.is_empty() {
            words = format!("{words}: {more}");
        }
    }
    for secret in secrets.iter().filter(|s| !s.is_empty()) {
        words = words.replace(secret, "***");
    }
    cut(&words, PROVIDER_WORDS_MAX)
}

#[cfg(test)]
include!("error.tests.rs");
