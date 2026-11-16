//! The Data page: where things are kept, read-only, and a button to read the
//! settings again. The paths come from the program and are never stored here.

use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::summary::OptionsPaths;

pub const UNKNOWN: &str = "not known";
const NOTE: &str = "Reload settings reads the settings file and the theme again.";

/// What is kept and where, said where the folder is shown.
pub const KEPT: &str = "Kept in the data folder and nowhere else: each account's file with its password or sign-in \
    token (plain text, limited to your user where the system allows it), your settings and window sizes, the \
    addresses your mail was from, to or copied to (to complete recipients as you type), and sent messages not \
    yet saved on the server. Mail itself is not kept on disk.";

/// Who the program talks to.
pub const CONTACTED: &str = "Contacted: your mail servers and, for a browser sign-in, the provider's sign-in service. \
    Nothing is sent to the author of this program.";

/// How to erase it all.
pub const REMOVE: &str = "To remove everything, close the program and delete the data folder. Removing an account \
    here deletes its own folder but leaves the settings, any saved client ids and the address list.";

pub fn rows(paths: &OptionsPaths, width: u16) -> Vec<Row> {
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
    for text in [KEPT, CONTACTED, REMOVE] {
        rows.push(Row::blank());
        rows.extend(paragraph(text, width, Tone::Dim));
    }
    rows
}

#[cfg(test)]
include!("data_page.tests.rs");
