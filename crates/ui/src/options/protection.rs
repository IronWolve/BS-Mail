//! How the saved passwords are protected, as the Data page shows it: the way
//! in use and the three choices, each with one honest sentence about what it
//! gives and what it does not. The program says which way is in use; the page
//! only shows it and offers the others. Nothing here is a secret.

use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, paragraph};

/// How the saved passwords are protected now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protection {
    /// A random key in a file in the data folder: nothing to type, portable.
    KeyFile,
    /// A key made from a passphrase typed at start.
    Passphrase,
}

impl Protection {
    pub fn label(self) -> &'static str {
        match self {
            Self::KeyFile => "Key file in this folder",
            Self::Passphrase => "Passphrase, asked at start",
        }
    }
}

const KEY_FILE_SAYS: &str = "A random key is kept in a file in this folder. Nothing to type, and the folder can be \
    copied to another computer and still work. It keeps casual eyes out (a file shown on screen, a pasted file, a \
    scanner) but not someone who holds the whole folder.";
const PASSPHRASE_SAYS: &str = "The key is made from a passphrase you type each time the program starts. A stolen copy \
    of the folder is protected as far as the passphrase is strong. The program cannot recover a forgotten passphrase: \
    you would enter the passwords again. Mail is not lost; it is on the server.";
const KEYCHAIN_SAYS: &str = "Would keep the key in the system instead of the folder, so a copy of the folder alone \
    reveals nothing, but a copied folder would not open on another computer.";
pub const KEYCHAIN_UNAVAILABLE: &str = "not available in this version";

/// The "Protecting passwords" section: its heading, the way in use, and the
/// three choices.
pub fn rows(now: Protection, width: u16) -> Vec<Row> {
    let mut rows = vec![
        Row::text("Protecting passwords", Tone::Heading),
        Row::field("In use", Item::Text { text: now.label().to_string(), tone: Tone::Plain, cut: Cut::Right }),
        Row::blank(),
    ];
    // Key file.
    rows.push(choice("Key file", now == Protection::KeyFile, Control::UseKeyFile, "Use a key file"));
    rows.extend(paragraph(KEY_FILE_SAYS, width, Tone::Dim));
    rows.push(Row::blank());
    // Passphrase: in use, it offers to be changed.
    if now == Protection::Passphrase {
        rows.push(Row {
            label: Some("Passphrase".to_string()),
            items: vec![Item::text("in use", Tone::Plain), Item::button(Control::ChangePassphrase, "Change passphrase")],
        });
    } else {
        rows.push(choice("Passphrase", false, Control::UsePassphrase, "Use a passphrase..."));
    }
    rows.extend(paragraph(PASSPHRASE_SAYS, width, Tone::Dim));
    rows.push(Row::blank());
    // The system keychain: shown, not offered.
    rows.push(Row::field("System keychain", Item::text(KEYCHAIN_UNAVAILABLE, Tone::Dim)));
    rows.extend(paragraph(KEYCHAIN_SAYS, width, Tone::Dim));
    rows
}

/// One choice: a button, or "in use" for the way that is.
fn choice(label: &str, in_use: bool, control: Control, button: &str) -> Row {
    if in_use {
        return Row::field(label, Item::text("in use", Tone::Plain));
    }
    Row { label: Some(label.to_string()), items: vec![Item::button(control, button)] }
}

#[cfg(test)]
include!("protection.tests.rs");
