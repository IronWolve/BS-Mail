//! The dialog on the Options screen that changes how passwords are protected:
//! a new passphrase typed twice (to protect with one, or to change it), or the
//! question before going back to a key file. It never holds a passphrase
//! longer than it is open: the two lines are overwritten when the dialog is
//! dropped, and it shows only dots. What it decided goes back as a
//! `DialogStep`; the program does the switching.

use std::fmt;

use model::Secret;

use super::frame::Focus;
use super::layout::{Block, ButtonId, FieldId, Prompt, Tone};
use super::line::SecretLine;

const SET: &str = "A stolen copy of this folder is protected as far as this passphrase is strong. The program cannot \
    recover a forgotten passphrase: you would enter the passwords again. Mail is not lost; it stays on the server.";
const CHANGE: &str = "Choose a new passphrase for the saved passwords. The old one stops working at once. The program \
    cannot recover a forgotten passphrase.";
const KEY_FILE: &str = "The saved passwords will be protected by a key file kept in this folder instead. Nothing is \
    typed at start, and the folder can be copied to another computer, but anyone who holds the whole folder can read \
    them. The passphrase is no longer needed.";
const KEY_FILE_FROM_KEYCHAIN: &str = "The saved passwords will be protected by a key file kept in this folder instead, and \
    the key is removed from the system keychain. Nothing is typed at start, and the folder can be copied to another \
    computer and opened there, but anyone who holds the whole folder can read them.";
const KEYCHAIN: &str = "The key to the saved passwords will be kept in this computer's own keychain instead of in this \
    folder or a passphrase. Nothing is typed at start, and a copy of the folder alone reveals nothing. A copy will \
    not open on another computer or user account: the passwords are entered again there. Mail is not lost; it \
    stays on the server.";

/// What the dialog is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    /// Protect the passwords with a passphrase (from a key file).
    Set,
    /// A new passphrase in place of the one in use.
    Change,
    /// Give the passphrase up for a key file: only a question.
    KeyFile,
    /// Give the system keychain up for a key file: only a question.
    KeyFileFromKeychain,
    /// Keep the key in the system keychain: only a question.
    Keychain,
}

impl DialogKind {
    /// A question with two buttons, no text to type.
    pub(super) fn is_question(self) -> bool {
        matches!(self, Self::KeyFile | Self::KeyFileFromKeychain | Self::Keychain)
    }
}

/// What the program is to do after an input.
#[derive(Debug, PartialEq, Eq)]
pub enum DialogStep {
    Nothing,
    /// The dialog changed; draw it again.
    Redraw,
    /// Close it, changing nothing.
    Cancel,
    /// Protect with this new passphrase (it was typed twice and passed the rule).
    UsePassphrase(Secret),
    /// Use this new passphrase in place of the old one.
    ChangePassphrase(Secret),
    /// Switch to a key file.
    UseKeyFile,
    /// Switch to the system keychain.
    UseKeychain,
}

pub struct PassphraseDialog {
    pub(super) kind: DialogKind,
    pub(super) first: SecretLine,
    pub(super) second: SecretLine,
    pub(super) focus: Focus,
    pub(super) problem: Option<String>,
    /// The letter of a command key just handled, whose text is still to come.
    pub(super) paired: Option<char>,
}

impl fmt::Debug for PassphraseDialog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PassphraseDialog").field("kind", &self.kind).finish_non_exhaustive()
    }
}

impl PassphraseDialog {
    pub fn new(kind: DialogKind) -> Self {
        let focus = match kind {
            DialogKind::Set | DialogKind::Change => Focus { field: Some(FieldId::First), button: None },
            DialogKind::KeyFile | DialogKind::KeyFileFromKeychain | DialogKind::Keychain => {
                Focus { field: None, button: Some(ButtonId::Keep) }
            }
        };
        Self { kind, first: SecretLine::new(), second: SecretLine::new(), focus, problem: None, paired: None }
    }

    pub fn kind(&self) -> DialogKind {
        self.kind
    }

    /// How many characters each box holds (the screen shows this many dots).
    pub fn typed_chars(&self) -> (usize, usize) {
        (self.first.chars(), self.second.chars())
    }

    /// The reason the passphrase was not accepted, if one is showing.
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    /// What is on the box now.
    pub(crate) fn prompt(&self) -> Prompt {
        let problem = self.problem.clone().map(|text| (text, Tone::Problem));
        match self.kind {
            DialogKind::Set | DialogKind::Change => Prompt {
                title: if self.kind == DialogKind::Set { " Protect passwords with a passphrase " } else { " Change the passphrase " },
                hint: " Tab next   Enter next or save   Esc cancel ",
                blocks: vec![
                    Block::Blank,
                    Block::Text((if self.kind == DialogKind::Set { SET } else { CHANGE }).to_string(), Tone::Plain),
                    Block::Text("At least 8 characters.".to_string(), Tone::Dim),
                    Block::Blank,
                    Block::Field(FieldId::First, "New passphrase"),
                    Block::Field(FieldId::Second, "Again"),
                    Block::Notice(2, problem),
                    Block::Blank,
                    Block::Buttons(vec![(ButtonId::Save, "Save"), (ButtonId::Cancel, "Cancel")]),
                ],
            },
            DialogKind::KeyFile => question(
                " Use a key file instead ",
                " y switch   Esc keep the passphrase ",
                KEY_FILE,
                ("Switch to a key file", "Keep the passphrase"),
            ),
            DialogKind::KeyFileFromKeychain => question(
                " Use a key file instead ",
                " y switch   Esc keep the keychain ",
                KEY_FILE_FROM_KEYCHAIN,
                ("Switch to a key file", "Keep the keychain"),
            ),
            DialogKind::Keychain => question(
                " Use the system keychain ",
                " y switch   Esc keep it as it is ",
                KEYCHAIN,
                ("Use the system keychain", "Keep it as it is"),
            ),
        }
    }
}

/// A question with a text and two buttons, the first of which does it.
fn question(title: &'static str, hint: &'static str, text: &str, buttons: (&'static str, &'static str)) -> Prompt {
    Prompt {
        title,
        hint,
        blocks: vec![
            Block::Blank,
            Block::Text(text.to_string(), Tone::Plain),
            Block::Blank,
            Block::Buttons(vec![(ButtonId::Switch, buttons.0), (ButtonId::Keep, buttons.1)]),
        ],
    }
}

#[cfg(test)]
include!("dialog.tests.rs");
