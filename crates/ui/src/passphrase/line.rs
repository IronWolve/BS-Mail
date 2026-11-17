//! One line of secret text being typed: a passphrase. It is kept in a buffer
//! whose room is taken once, so typing never moves the text to a new place and
//! leaves a copy behind, and the whole buffer is overwritten when the line is
//! cleared or dropped. It shows nothing in debug output, and the screen only
//! ever draws one dot for each character.

use std::fmt;

use model::{Secret, wipe_text};

/// The most bytes a passphrase may have.
pub const MAX_BYTES: usize = 1024;

pub struct SecretLine {
    text: String,
}

impl Default for SecretLine {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretLine {
    pub fn new() -> Self {
        Self { text: String::with_capacity(MAX_BYTES) }
    }

    /// How many characters have been typed.
    pub fn chars(&self) -> usize {
        self.text.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Adds typed or pasted text at the end. Control characters (a pasted
    /// newline included) are left out, and text that would not fit is cut
    /// off. Whether anything was added.
    pub fn push_str(&mut self, typed: &str) -> bool {
        let before = self.text.len();
        for ch in typed.chars().filter(|ch| !ch.is_control()) {
            if self.text.len() + ch.len_utf8() > MAX_BYTES {
                break;
            }
            self.text.push(ch);
        }
        self.text.len() != before
    }

    /// Takes the last character off; whether there was one.
    pub fn backspace(&mut self) -> bool {
        self.text.pop().is_some()
    }

    /// Overwrites everything typed.
    pub fn clear(&mut self) {
        wipe_text(&mut self.text);
    }

    /// Whether both lines hold the same text.
    pub fn same_as(&self, other: &SecretLine) -> bool {
        self.text == other.text
    }

    /// A copy as a `Secret`, which overwrites itself when dropped.
    pub fn to_secret(&self) -> Secret {
        Secret::new(self.text.clone())
    }
}

impl Drop for SecretLine {
    fn drop(&mut self) {
        wipe_text(&mut self.text);
    }
}

impl fmt::Debug for SecretLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretLine(<redacted>)")
    }
}

#[cfg(test)]
include!("line.tests.rs");
