//! The Accounts page: the list of accounts with the highlighted one's details
//! below, and the buttons for it. While the removal question shows, the page
//! is the question (`confirm_page.rs`).

use model::ConnectionState;

use super::confirm_page::confirm_rows;
use super::control::Control;
use super::rows::{Cut, Item, Row, Tone, button_rows, paragraph};
use super::state::OptionsState;
use super::summary::AccountSummary;
use super::view::OptionsView;

pub const NO_ACCOUNT: &str = "No account is set up.";
pub const MOCK_MAIL: &str = "This run shows mock mail, not an account.";

/// The rows of the page for `width` cells.
pub fn rows(view: &OptionsView<'_>, state: &OptionsState, width: u16) -> Vec<Row> {
    let Some(list) = view.accounts else {
        return empty_rows(MOCK_MAIL, width);
    };
    if list.is_empty() {
        return empty_rows(NO_ACCOUNT, width);
    }
    if state.confirming
        && let Some(account) = view.highlighted(state.cursor)
    {
        return confirm_rows(account, width);
    }
    list_rows(view, list, state, width)
}

fn empty_rows(text: &str, width: u16) -> Vec<Row> {
    let mut rows = paragraph(text, width, Tone::Plain);
    rows.push(Row::blank());
    rows.extend(button_rows(vec![Item::button(Control::AddAccount, "Add account")], width));
    rows
}

fn list_rows(view: &OptionsView<'_>, list: &[AccountSummary], state: &OptionsState, width: u16) -> Vec<Row> {
    let mut rows: Vec<Row> = list
        .iter()
        .enumerate()
        .map(|(index, account)| {
            Row::item(Item::Account {
                index,
                address: account.address.clone(),
                state: view.state_of(&account.id),
                selected: view.is_selected(account),
            })
        })
        .collect();
    let Some(account) = view.highlighted(state.cursor) else {
        return rows;
    };
    rows.push(Row::blank());
    rows.extend(detail_rows(view, account));
    rows.push(Row::blank());
    rows.extend(button_rows(buttons(view.is_selected(account)), width));
    rows
}

fn detail_rows(view: &OptionsView<'_>, account: &AccountSummary) -> Vec<Row> {
    let text = |value: &str| Item::text(value, Tone::Plain);
    let changes = if account.changes_allowed { "on" } else { "off, read only" };
    let mut rows = vec![
        Row::field("Name", text(&account.name)),
        Row::field("Address", text(&account.address)),
        Row::field("Incoming server", text(&account.incoming)),
        Row::field("Outgoing server", text(&account.outgoing)),
        Row::field("Sign in", text(&account.sign_in)),
        Row::field("Mail changes", text(changes)),
        Row::field("Kept in", Item::Text { text: account.folder.clone(), tone: Tone::Plain, cut: Cut::Left }),
    ];
    if let (true, Some(problem)) = (view.is_selected(account), view.problem) {
        rows.push(Row::field("Problem", Item::text(problem, Tone::Warn)));
    }
    rows
}

fn buttons(selected: bool) -> Vec<Item> {
    let mut buttons = Vec::new();
    if !selected {
        buttons.push(Item::button(Control::SwitchAccount, "Switch to this account"));
    }
    buttons.push(Item::button(Control::EditAccount, "Edit account"));
    buttons.push(Item::button(Control::RemoveAccount, "Remove account"));
    buttons.push(Item::button(Control::AddAccount, "Add account"));
    buttons
}

/// The word that goes with a connection state in the list.
pub fn state_word(state: Option<ConnectionState>) -> &'static str {
    match state {
        Some(ConnectionState::Online) => "connected",
        Some(ConnectionState::Connecting) => "connecting",
        Some(ConnectionState::Offline) => "offline",
        None => "",
    }
}

#[cfg(test)]
include!("accounts_page.tests.rs");
