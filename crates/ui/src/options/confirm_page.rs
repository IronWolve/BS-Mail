//! The question that comes before an account is removed: what will be deleted,
//! what will not be, and two buttons with "Keep it" as the safe one.

use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::summary::AccountSummary;

/// What a removal leaves alone, said plainly.
pub fn warning(address: &str) -> String {
    format!(
        "Remove {address}? Everything in its folder is deleted from this computer: its settings, \
         its saved sign-in and earlier copies of its settings. Mail on the server is not touched."
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
