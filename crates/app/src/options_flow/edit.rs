//! Opening the account form from the Options screen.

use ui::AppView;

use crate::accounts::{Live, prefill_of};

use super::answer::live_or_problem;

/// Opens the form empty, for a new account. Returns whether it opened.
pub(super) fn add_account(view: &mut AppView, live: Option<&Live>) -> bool {
    let Some(live) = live_or_problem(view, live) else {
        return false;
    };
    if let Some(why) = live.place.refuses_changes() {
        view.options_problem(why);
        return false;
    }
    view.open_setup();
    true
}

/// Opens the form filled with the account `id`. Returns whether it opened.
pub(super) fn edit_account(view: &mut AppView, live: Option<&Live>, id: &str) -> bool {
    let Some(live) = live_or_problem(view, live) else {
        return false;
    };
    let scan = live.place.scan();
    let Some(found) = scan.found.iter().find(|found| found.id == id) else {
        view.options_problem("That account is no longer there.");
        return false;
    };
    match prefill_of(found) {
        Ok(prefill) => {
            view.open_setup_with(prefill);
            true
        }
        Err(reason) => {
            view.options_problem(&reason);
            false
        }
    }
}

#[cfg(test)]
include!("edit.tests.rs");
