//! The About page: what the program is, under which terms it is used and where
//! its home page is, read-only. The name, tagline, version and home page come
//! from the program's identity and are never spelled here.

use super::data_page::UNKNOWN;
use super::rows::{Cut, Item, Row, Tone, paragraph};
use crate::app_identity::AppIdentity;

/// The terms the program is used under.
pub const LICENCE: &str = "Free for your own personal, non-commercial use. You may not copy or share it, change it, \
    sell it, or use it commercially without written permission. The LICENSE file that came with it has the terms.";

/// What the parts the program is made with are used under.
pub const THIRD_PARTY: &str =
    "The fonts and libraries it uses keep their own licenses, listed in the NOTICE file that came with it.";

/// The label of the home page line.
pub const HOME_PAGE: &str = "Home page";

pub fn rows(identity: &AppIdentity, width: u16) -> Vec<Row> {
    let mut rows = paragraph(&format!("{} {}", identity.name, identity.version), width, Tone::Plain);
    if !identity.tagline.is_empty() {
        rows.extend(paragraph(&identity.tagline, width, Tone::Dim));
    }
    for text in [LICENCE, THIRD_PARTY] {
        rows.push(Row::blank());
        rows.extend(paragraph(text, width, Tone::Dim));
    }
    rows.push(Row::blank());
    rows.push(Row::field(HOME_PAGE, link(&identity.homepage)));
    rows
}

/// The link as a read-only value, or `not known` when none was given.
fn link(homepage: &str) -> Item {
    let text = if homepage.trim().is_empty() { UNKNOWN } else { homepage.trim() };
    Item::Text { text: text.to_string(), tone: Tone::Plain, cut: Cut::Right }
}

#[cfg(test)]
include!("about_page.tests.rs");
