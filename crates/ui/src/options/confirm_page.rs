//! The question that comes before an account is removed: what will be deleted,
//! what will not be, and two buttons with "Keep it" as the safe one.

use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::summary::AccountSummary;

/// What a removal leaves alone, said plainly.
pub fn warning(address: &str) -> String {
    format!(
        "Remove {address}? Its folder and its saved sign-in are deleted from this computer. \
         Mail on the server is not touched. Earlier copies of the account file, named \
         account.toml.bak\u{2026}, are not touched."
    )
}

pub fn confirm_rows(account: &AccountSummary, width: u16) -> Vec<Row> {
    let mut rows = paragraph(&warning(&account.address), width, Tone::Warn);
    rows.push(Row::blank());
    rows.push(Row::field("Kept in", Item::Text { text: account.folder.clone(), tone: Tone::Plain, cut: Cut::Left }));
    rows.push(Row::blank());
    let buttons = vec![Item::button(Control::ConfirmRemove, "Remove"), Item::button(Control::KeepAccount, "Keep it")];
    rows.extend(button_rows(buttons, width));
    rows
}

#[cfg(test)]
include!("confirm_page.tests.rs");
