//! Removing an account from the Options screen, after the user confirmed.

use std::time::{Duration, Instant};

use engine::MembersHandle;
use model::{AccountId, MailBackend};
use ui::AppView;

use crate::accounts::{self, Live, Place, checked_folder};

use super::answer::live_or_problem;

/// The longest the folder waits for the account's backend to finish stopping:
/// the time it gives sends under way, and then the time it gives its worker to
/// end.
const STOP_LIMIT: Duration = Duration::from_secs(20);
const STOP_STEP: Duration = Duration::from_millis(10);

/// Stops the account `id`, deletes its folder and refreshes the lists.
///
/// The folder is checked first, so a request that cannot be carried out
/// changes nothing. The running account is let go before its folder is
/// deleted, and the folder waits until it has stopped (see `let_go`): its
/// worker cannot write into a folder that is going away. If the account was
/// showing, the screen forgets its mail and the next account takes its place,
/// as the running accounts replay what they know about it.
pub(super) fn remove_account(view: &mut AppView, backend: &mut Box<dyn MailBackend>, live: Option<&Live>, id: &str) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    if let Some(why) = live.place.refuses_changes() {
        return view.options_problem(why);
    }
    let Place::Root(root) = &live.place else {
        return;
    };
    if let Err(reason) = checked_folder(root, id) {
        return view.options_problem(&reason);
    }
    let address = live
        .place
        .scan()
        .found
        .into_iter()
        .find(|found| found.id == id)
        .map_or_else(|| id.to_string(), |found| found.address);
    let was_showing = view.selection().account.as_ref().is_some_and(|shown| shown.0 == id);
    let_go(view, backend, &live.members, id, was_showing);
    let outcome = accounts::remove(root, id);
    view.set_accounts(live.summaries());
    match outcome {
        Ok(()) => match accounts::forget_secrets(&live.place.secrets_folder(), id) {
            Ok(()) => view.options_info(&format!("Removed {address}.")),
            Err(reason) => view.options_problem(&format!(
                "Removed {address}, but its secrets could not be taken out of the vault: {reason}"
            )),
        },
        Err(reason) => view.options_problem(&format!(
            "{reason}; the account is stopped for now and comes back the next time the program starts"
        )),
    }
}

/// Takes the account out of the running ones now, gives the screen what the
/// running accounts say about the change, and returns once the account's
/// backend has stopped. The backend stops on a thread of its own and may need
/// seconds for a send under way; the other accounts keep being served while
/// this waits. The wait is bounded, so a worker that is stuck cannot hold the
/// removal forever.
fn let_go(view: &mut AppView, backend: &mut Box<dyn MailBackend>, members: &MembersHandle, id: &str, was_showing: bool) {
    let account = AccountId(id.to_string());
    members.remove(&account);
    if was_showing {
        view.clear_mail();
    }
    show_what_is_new(view, backend);
    let until = Instant::now() + STOP_LIMIT;
    while members.winding_down(&account) && Instant::now() < until {
        std::thread::sleep(STOP_STEP);
        show_what_is_new(view, backend);
    }
}

fn show_what_is_new(view: &mut AppView, backend: &mut Box<dyn MailBackend>) {
    for event in backend.poll_events() {
        view.apply_event(event);
    }
}

#[cfg(test)]
include!("remove.tests.rs");
