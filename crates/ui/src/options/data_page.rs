//! The Data page: where things are kept, read-only, and a button to read the
//! settings again. The paths come from the program and are never stored here.

use super::control::Control;
use super::protection::{Protection, rows as protection_rows};
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::summary::OptionsPaths;

pub const UNKNOWN: &str = "not known";
const NOTE: &str = "Reload settings reads the settings file and the theme again.";

/// What is kept and where, said where the folder is shown.
pub const KEPT: &str = "Kept in the data folder and nowhere else: each account's file, with its passwords, its \
    browser sign-in tokens and any client secret kept encrypted in the secrets vault (the key to it is a file in \
    the same folder, so it keeps them out of casual view, such as a file shown on screen or pasted, but not from \
    someone who holds the whole folder: keep the folder private), your settings and window sizes, the addresses \
    your mail was from, to or copied to (to complete recipients as you type), and sent messages not yet saved on \
    the server. Mail itself is not kept on disk.";

/// What is kept, said for the way the passwords are protected now. With a
/// passphrase the key is not in the folder at all.
pub const KEPT_PASSPHRASE: &str = "Kept in the data folder and nowhere else: each account's file, with its passwords, its \
    browser sign-in tokens and any client secret kept encrypted in the secrets vault (the key to it is made from your \
    passphrase, which is not stored anywhere: a stolen copy of the folder is protected as far as the passphrase is \
    strong, and the program cannot recover a forgotten one), your settings and window sizes, the addresses your mail \
    was from, to or copied to (to complete recipients as you type), and sent messages not yet saved on the server. \
    Mail itself is not kept on disk.";

/// The text about what is kept, for the way the passwords are protected.
pub fn kept_text(protection: Option<Protection>) -> &'static str {
    match protection {
        Some(Protection::Passphrase) => KEPT_PASSPHRASE,
        Some(Protection::KeyFile) | None => KEPT,
    }
}

/// Who the program talks to.
pub const CONTACTED: &str = "Contacted: your mail servers and, for a browser sign-in, the provider's sign-in service. \
    Nothing is sent to the author of this program.";

/// How to erase it all.
pub const REMOVE: &str = "To remove everything, close the program and delete the data folder. Removing an account \
    here deletes its own folder and its entries in the vault but leaves the settings, any saved client ids (and \
    their secrets) and the address list.";

pub fn rows(paths: &OptionsPaths, width: u16) -> Vec<Row> {
    rows_for(paths, None, width)
}

/// The page with the section on how the passwords are protected, when the
/// program says (`Some`), and the text about what is kept for that way.
pub fn rows_for(paths: &OptionsPaths, protection: Option<Protection>, width: u16) -> Vec<Row> {
    let path = |text: &str| {
        let text = if text.is_empty() { UNKNOWN } else { text };
        Item::Text { text: text.to_string(), tone: Tone::Plain, cut: Cut::Left }
    };
    let mut rows = vec![
        Row::field("Data folder", path(&paths.data_dir)),
        Row::field("Settings file", path(&paths.config_file)),
        Row::field("Accounts folder", path(&paths.accounts_dir)),
        Row::blank(),
    ];
    rows.extend(button_rows(vec![Item::button(Control::Reload, "Reload settings")], width));
    rows.push(Row::blank());
    rows.extend(paragraph(NOTE, width, Tone::Dim));
    if let Some(now) = protection {
        rows.push(Row::blank());
        rows.extend(protection_rows(now, width));
    }
    for text in [kept_text(protection), CONTACTED, REMOVE] {
        rows.push(Row::blank());
        rows.extend(paragraph(text, width, Tone::Dim));
    }
    rows
}

#[cfg(test)]
include!("data_page.tests.rs");
