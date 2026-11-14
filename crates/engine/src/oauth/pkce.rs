//! PKCE (RFC 7636): a random verifier kept here, and the S256 challenge
//! built from it that goes into the authorization link. The same module makes
//! the random `state` that ties the browser's return to this sign-in.

use model::Secret;
use ring::digest::{SHA256, digest};

use crate::send::{base64, random};

/// A verifier and its challenge.
#[derive(Debug, Clone)]
pub struct Pkce {
    verifier: Secret,
    challenge: String,
}

/// Base64 without the padding, with `-` and `_` for the URL-safe alphabet.
pub fn base64url(data: &[u8]) -> String {
    base64::encode(data)
        .trim_end_matches('=')
        .chars()
        .map(|c| match c {
            '+' => '-',
            '/' => '_',
            other => other,
        })
        .collect()
}

impl Pkce {
    /// A new pair from 32 random bytes.
    pub fn generate() -> Result<Self, String> {
        let bytes = random::random_bytes::<32>()?;
        Ok(Self::from_verifier(&base64url(&bytes)))
    }

    /// The pair for a verifier that is already chosen.
    pub fn from_verifier(verifier: &str) -> Self {
        let hash = digest(&SHA256, verifier.as_bytes());
        Self { verifier: Secret::new(verifier), challenge: base64url(hash.as_ref()) }
    }

    pub fn verifier(&self) -> &Secret {
        &self.verifier
    }

    pub fn challenge(&self) -> &str {
        &self.challenge
    }
}

/// A random `state` value: 32 random bytes, URL-safe base64.
pub fn random_state() -> Result<String, String> {
    Ok(base64url(&random::random_bytes::<32>()?))
}

#[cfg(test)]
include!("pkce.tests.rs");
