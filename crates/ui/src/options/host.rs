//! What the program has told the Options screen, and what the screen has
//! queued for the program. It lives in the view whether or not the screen is
//! open, because the program sets it up at start.

use super::action::OptionsAction;
use super::protection::{KeychainState, Protection};
use super::summary::{AccountSummary, OptionsPaths};

/// The one line at the bottom of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoLine {
    pub text: String,
    /// A problem is drawn in the error colour.
    pub problem: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OptionsHost {
    /// `None` until the program says (mock mail, tests); an empty list means
    /// no account is set up.
    pub accounts: Option<Vec<AccountSummary>>,
    pub paths: OptionsPaths,
    /// The themes the Theme choice goes through.
    pub theme_names: Vec<String>,
    /// What the screen asked for, until the program takes it.
    pub actions: Vec<OptionsAction>,
    pub line: Option<InfoLine>,
    /// How the saved passwords are protected; `None` when there is no vault
    /// to say (mock mail), and the Data page then offers nothing about it.
    pub protection: Option<Protection>,
    /// What the program found out about the system keychain.
    pub keychain: KeychainState,
}

impl OptionsHost {
    /// Whether the program says no account is set up.
    pub fn has_no_account(&self) -> bool {
        self.accounts.as_ref().is_some_and(Vec::is_empty)
    }

    /// How many accounts the program told of.
    pub fn account_count(&self) -> usize {
        self.accounts.as_ref().map_or(0, Vec::len)
    }
}

#[cfg(test)]
include!("host.tests.rs");
