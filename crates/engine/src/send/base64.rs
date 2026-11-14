//! Base64 (RFC 4648, standard alphabet with padding) for sign-in strings
//! and encoded words. Encoding only; nothing here reads base64.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The base64 text of `data`.
pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let group = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let bits = u32::from(group[0]) << 16 | u32::from(group[1]) << 8 | u32::from(group[2]);
        for step in 0..4 {
            if step <= chunk.len() {
                let index = usize::try_from((bits >> (18 - 6 * step)) & 0x3f).unwrap_or(0);
                out.push(char::from(ALPHABET[index]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
include!("base64.tests.rs");
