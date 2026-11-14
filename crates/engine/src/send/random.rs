//! Random bytes from the operating system, through the TLS provider's
//! generator, so message ids cannot be guessed.

use rustls::crypto::ring::default_provider;

/// `N` random bytes, or why there are none.
pub fn random_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut bytes = [0u8; N];
    default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| "the system's random number source is not available".to_owned())?;
    Ok(bytes)
}

/// `N` random bytes as lower-case hex, `2 * N` characters.
pub fn random_hex<const N: usize>() -> Result<String, String> {
    let bytes = random_bytes::<N>()?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
include!("random.tests.rs");
