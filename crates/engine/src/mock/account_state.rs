//! What the screen is told about the fixture's account: at start-up, and when
//! the user switches to an account. A backend that serves several accounts
//! describes the account again on a switch; the mock does the same for the
//! fixture's account, and says plainly that it has no mail for the others.

use model::{AccountId, ConnectionState, Event};

use super::fixture_load::Fixture;

/// What is said when the user switches to a listed account the fixture has no
/// mail for, so the screen is not left empty without a word.
const NO_MAIL_FOR_ACCOUNT: &str = "the mock fixture has no mail for this account";

/// The fixture account's folders, connection, quota and sync status, in the
/// order the screen applies them.
pub(crate) fn account_state(data: &Fixture) -> Vec<Event> {
    let account = data.account.clone();
    vec![
        Event::FolderList {
            account: account.clone(),
            folders: data.folders.clone(),
        },
        Event::ConnectionState {
            account: account.clone(),
            state: selected_connection(data),
        },
        Event::Quota {
            account: account.clone(),
            quota: data.status.quota,
        },
        Event::SyncProgress {
            account,
            status: data.status.clone(),
        },
    ]
}

/// The user switched to `account`, which the screen has already emptied: the
/// fixture's account is described again. Another listed account is shown with
/// its connection and a notice; the notice comes last, because an account
/// coming online ends a failure.
pub(crate) fn select_account(data: &Fixture, account: &AccountId) -> Vec<Event> {
    let Some(listed) = data.accounts.iter().find(|entry| &entry.id == account) else {
        return vec![Event::OpFailed {
            account: Some(data.account.clone()),
            operation: "account".to_owned(),
            reason: format!("unknown account: {}", account.0),
        }];
    };
    if *account == data.account {
        return account_state(data);
    }
    vec![
        Event::ConnectionState {
            account: account.clone(),
            state: listed.state,
        },
        Event::OpFailed {
            account: Some(account.clone()),
            operation: "account".to_owned(),
            reason: NO_MAIL_FOR_ACCOUNT.to_owned(),
        },
    ]
}

fn selected_connection(data: &Fixture) -> ConnectionState {
    data.accounts
        .iter()
        .find(|account| account.selected)
        .map_or(ConnectionState::Offline, |account| account.state)
}
