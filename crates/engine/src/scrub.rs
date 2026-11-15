//! Taking secrets out of words that came from a server. A server that echoes
//! a command back would otherwise put the password in a line shown on screen.

use crate::send::base64;

/// `text` with every copy of each secret, and of its base64 form, replaced.
/// Nothing is replaced for an empty secret.
pub fn scrub(text: &str, secrets: &[&str]) -> String {
    let mut clean = text.to_owned();
    for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
        let coded = base64::encode(secret.as_bytes());
        clean = clean.replace(secret, "***").replace(&coded, "***");
    }
    clean
}

/// The base64 of the PLAIN sign-in blob for `user` and `secret`, which is
/// what goes over the wire after `AUTH PLAIN`.
pub fn plain_blob(user: &str, secret: &str) -> String {
    let mut blob = vec![0u8];
    blob.extend_from_slice(user.as_bytes());
    blob.push(0);
    blob.extend_from_slice(secret.as_bytes());
    base64::encode(&blob)
}

#[cfg(test)]
include!("scrub.tests.rs");
