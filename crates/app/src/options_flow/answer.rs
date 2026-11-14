//! Each request of the Options screen and the one answer it gets.

use model::MailBackend;
use ui::{AppView, OptionsAction};

use crate::accounts::{Live, MOCK_RUN};

use super::edit::{add_account, edit_account};
use super::remove::remove_account;

/// Said for a setting the program does not save yet.
pub const NOT_BUILT: &str = "Saving this setting is not built yet.";

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

/// Answers `actions` in order.
pub fn answer(
    view: &mut AppView,
    backend: &mut Box<dyn MailBackend>,
    live: Option<&Live>,
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
            OptionsAction::SetTheme(_)
            | OptionsAction::SetIcons(_)
            | OptionsAction::SetFontSize(_)
            | OptionsAction::SetLineSpacing(_)
            | OptionsAction::ReloadSettings => view.options_problem(NOT_BUILT),
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
