//! Each request of the Options screen and the one answer it gets.

use model::MailBackend;
use ui::{AppView, OptionsAction};

use crate::accounts::{Live, MOCK_RUN};

use super::edit::{add_account, edit_account};
use super::background::Background;
use super::protect::{change_passphrase, check_keychain, start_vault_over, use_key_file, use_keychain, use_passphrase};
use super::remove::remove_account;

/// Which account form, if any, the answers opened. The caller makes the flow
/// that answers it, and needs to know whether the form changes an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// No form was opened.
    Nothing,
    /// A form for a new account.
    New,
    /// A form that changes the account with this id.
    Editing(String),
}

/// Answers the accounts' requests in `actions` in order. Requests about
/// settings are `Settings::answer`'s and are passed over here. `jobs` is where
/// the requests that can wait on the system keychain are done, apart from the
/// screen.
pub fn answer(
    view: &mut AppView,
    backend: &mut Box<dyn MailBackend>,
    live: Option<&Live>,
    jobs: &mut Background,
    actions: Vec<OptionsAction>,
) -> Opened {
    let mut opened = Opened::Nothing;
    for action in actions {
        match action {
            OptionsAction::AddAccount => {
                if add_account(view, live) {
                    opened = Opened::New;
                }
            }
            OptionsAction::EditAccount(id) => {
                if edit_account(view, live, &id) {
                    opened = Opened::Editing(id);
                }
            }
            OptionsAction::RemoveAccount(id) => remove_account(view, backend, live, &id),
            OptionsAction::UsePassphrase(secret) => use_passphrase(view, live, jobs, &secret),
            OptionsAction::ChangePassphrase(secret) => change_passphrase(view, live, &secret),
            OptionsAction::UseKeyFile => use_key_file(view, live, jobs),
            OptionsAction::UseKeychain => use_keychain(view, live, jobs),
            OptionsAction::StartVaultOver => start_vault_over(view, live),
            OptionsAction::CheckKeychain => check_keychain(view, live, jobs),
            OptionsAction::SetTheme(_)
            | OptionsAction::SetIcons(_)
            | OptionsAction::SetFontSize(_)
            | OptionsAction::SetLineSpacing(_)
            | OptionsAction::SetLayout(_)
            | OptionsAction::ReloadSettings => {}
        }
    }
    opened
}

/// The accounts of this run, or the problem to show when mock mail has none.
pub(super) fn live_or_problem<'a>(view: &mut AppView, live: Option<&'a Live>) -> Option<&'a Live> {
    if live.is_none() {
        view.options_problem(MOCK_RUN);
    }
    live
}

#[cfg(test)]
include!("answer.tests.rs");
