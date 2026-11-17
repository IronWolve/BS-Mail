//! The facts a page is built from, borrowed from the view for one frame or
//! one input: the accounts the program told of, the paths, the program's
//! identity, the settings now in force and what the mail snapshot knows about
//! connections.

use model::ConnectionState;
use model::config::Config;

use super::host::InfoLine;
use super::protection::{KeychainState, Protection};
use super::summary::{AccountSummary, OptionsPaths};
use crate::app_identity::AppIdentity;

pub struct OptionsView<'a> {
    /// `None` when the program has not said (mock mail).
    pub accounts: Option<&'a [AccountSummary]>,
    pub paths: &'a OptionsPaths,
    /// The program's name, tagline, version and home page, for the About page.
    pub identity: &'a AppIdentity,
    pub theme_names: &'a [String],
    pub config: &'a Config,
    /// The id of the account whose mail is showing.
    pub selected: Option<&'a str>,
    /// The accounts of the mail snapshot, for their connection state.
    pub connections: &'a [model::Account],
    /// The last problem the mail backend reported.
    pub problem: Option<&'a str>,
    pub line: Option<&'a InfoLine>,
    /// How the saved passwords are protected, when the program says.
    pub protection: Option<Protection>,
    /// What the program found out about the system keychain.
    pub keychain: KeychainState,
    /// Why the vault cannot be opened, when the explicit start-over is the way out.
    pub vault_trouble: Option<&'a str>,
}

impl OptionsView<'_> {
    /// The connection state of an account, matched by id.
    pub fn state_of(&self, id: &str) -> Option<ConnectionState> {
        self.connections.iter().find(|account| account.id.0 == id).map(|account| account.state)
    }

    /// The account the highlight is on, kept inside the list.
    pub fn highlighted(&self, cursor: usize) -> Option<&AccountSummary> {
        let list = self.accounts?;
        list.get(cursor.min(list.len().saturating_sub(1)))
    }

    /// Whether the account is the one whose mail is showing.
    pub fn is_selected(&self, account: &AccountSummary) -> bool {
        self.selected == Some(account.id.as_str())
    }
}

#[cfg(test)]
include!("view.tests.rs");
