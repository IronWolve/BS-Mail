//! The one rule about a new passphrase that the screen and the vault both
//! apply, so the dialog's plain reason and the engine's refusal agree.

/// The fewest characters a new passphrase may have.
pub const MIN_PASSPHRASE_CHARS: usize = 8;

/// Why a new passphrase of `chars` characters, typed a second time as
/// `again_matches`, is not accepted; `None` when it is. The sentence is what
/// the dialog shows, and it never quotes the passphrase.
pub fn passphrase_problem(chars: usize, again_matches: bool) -> Option<&'static str> {
    if chars < MIN_PASSPHRASE_CHARS {
        return Some("The passphrase must have at least 8 characters.");
    }
    if !again_matches {
        return Some("The two passphrases are not the same.");
    }
    None
}

#[cfg(test)]
include!("passphrase.tests.rs");
