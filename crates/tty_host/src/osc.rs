//! Escape sequences that talk to the terminal itself: the clipboard and the
//! window title.

/// Puts `text` on the system clipboard through the terminal, for terminals
/// that allow it. Nothing says whether it worked.
pub fn clipboard_sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

/// Sets the terminal's window title. Control codes are taken out so a title
/// cannot end the sequence early and run commands in the terminal.
pub fn title_sequence(title: &str) -> String {
    let clean: String = title.chars().filter(|ch| !ch.is_control()).collect();
    format!("\x1b]0;{clean}\x07")
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let block = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let value = (u32::from(block[0]) << 16) | (u32::from(block[1]) << 8) | u32::from(block[2]);
        for position in 0..4 {
            if position <= chunk.len() {
                let index = (value >> (18 - 6 * position)) & 0x3f;
                out.push(char::from(ALPHABET[index as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
include!("osc.tests.rs");
