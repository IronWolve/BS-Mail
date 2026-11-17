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
    /// A random key kept in the computer's own keychain: nothing to type, and
    /// the folder alone reveals nothing, but it opens only on this computer.
    Keychain,
}

impl Protection {
    pub fn label(self) -> &'static str {
        match self {
            Self::KeyFile => "Key file in this folder",
            Self::Passphrase => "Passphrase, asked at start",
            Self::Keychain => "System keychain",
        }
    }
}

/// What the program knows about whether this computer has a keychain the
/// program can use. The Data page offers the keychain only once the program
/// has looked and found one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeychainState {
    /// Not looked at yet; opening the Data page asks the program to look.
    #[default]
    Unknown,
    /// The program is looking.
    Checking,
    /// There is one and it answered.
    Available,
    /// There is none (or it did not answer): the program looked.
    Unavailable,
    /// This version of the program has no code for this system's keychain.
    NotBuilt,
}

const KEY_FILE_SAYS: &str = "A random key is kept in a file in this folder. Nothing to type, and the folder can be \
    copied to another computer and still work. It keeps casual eyes out (a file shown on screen, a pasted file, a \
    scanner) but not someone who holds the whole folder.";
const PASSPHRASE_SAYS: &str = "The key is made from a passphrase you type each time the program starts. A stolen copy \
    of the folder is protected as far as the passphrase is strong. The program cannot recover a forgotten passphrase: \
    you would enter the passwords again. Mail is not lost; it is on the server.";
const KEYCHAIN_SAYS: &str = "The key is kept in this computer's own keychain, not in the folder: nothing to type, and a \
    copy of the folder alone reveals nothing. A copy will not open on another computer or user account; the \
    passwords are entered again there. Mail is not lost; it is on the server.";
const KEYCHAIN_BACKUP: &str = "Backup incomplete: the vault key is outside the data folder, so a copy of this folder \
    cannot open its vault elsewhere. Deleting the folder leaves the key in the keychain; choosing another way here \
    removes it.";
pub const KEYCHAIN_UNAVAILABLE: &str = "no system keychain was found here";
pub const KEYCHAIN_NOT_BUILT: &str = "not available in this version";
pub const KEYCHAIN_CHECKING: &str = "checking...";

/// The "Protecting passwords" section: its heading, the way in use, and the
/// three choices.
pub fn rows(now: Protection, keychain: KeychainState, width: u16) -> Vec<Row> {
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
    // The system keychain: offered only where the program found one.
    rows.push(keychain_row(now, keychain));
    rows.extend(paragraph(KEYCHAIN_SAYS, width, Tone::Dim));
    if now == Protection::Keychain {
        rows.extend(paragraph(KEYCHAIN_BACKUP, width, Tone::Warn));
    }
    rows
}

/// The keychain's row: in use, a button when the program found a keychain,
/// otherwise read-only words that say why not.
fn keychain_row(now: Protection, keychain: KeychainState) -> Row {
    let label = "System keychain";
    if now == Protection::Keychain {
        return Row::field(label, Item::text("in use", Tone::Plain));
    }
    match keychain {
        KeychainState::Available => Row { label: Some(label.to_string()), items: vec![Item::button(Control::UseKeychain, "Use the system keychain...")] },
        KeychainState::Unavailable => Row::field(label, Item::text(KEYCHAIN_UNAVAILABLE, Tone::Dim)),
        KeychainState::NotBuilt => Row::field(label, Item::text(KEYCHAIN_NOT_BUILT, Tone::Dim)),
        KeychainState::Unknown | KeychainState::Checking => Row::field(label, Item::text(KEYCHAIN_CHECKING, Tone::Dim)),
    }
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
