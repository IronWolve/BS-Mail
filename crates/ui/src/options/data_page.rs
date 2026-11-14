//! The Data page: where things are kept, read-only, and a button to read the
//! settings again. The paths come from the program and are never stored here.

use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::summary::OptionsPaths;

pub const UNKNOWN: &str = "not known";
const NOTE: &str = "Reload settings reads the settings file and the theme again.";

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
    rows
}

#[cfg(test)]
include!("data_page.tests.rs");
