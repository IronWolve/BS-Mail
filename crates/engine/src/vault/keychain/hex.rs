//! Lower-case hexadecimal, for the names entries are kept under and for the
//! text a key is stored as (a keychain keeps text, and every one of them can
//! hold hex).

const DIGITS: &[u8; 16] = b"0123456789abcdef";

/// `bytes` as lower-case hex.
pub(super) fn encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    text
}

fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// The bytes `text` spells; `None` for an odd length or any other character.
pub(super) fn decode(text: &str) -> Option<Vec<u8>> {
    let raw = text.as_bytes();
    if raw.len() % 2 != 0 {
        return None;
    }
    raw.chunks(2).map(|pair| Some(digit(pair[0])? << 4 | digit(pair[1])?)).collect()
}

#[cfg(test)]
include!("hex.tests.rs");
