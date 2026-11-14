//! Removing an account from the Options screen, after the user confirmed.

use engine::MembersHandle;
use model::{AccountId, MailBackend};
use ui::AppView;

use crate::accounts::{self, Live, Place, checked_folder};

use super::answer::live_or_problem;

/// Stops the account `id`, deletes its folder and refreshes the lists.
///
/// The folder is checked first, so a request that cannot be carried out
/// changes nothing. The running account is let go before its folder is
/// deleted: its worker has stopped by then and cannot write into a folder
/// that is going away. If the account was showing, the screen forgets its mail
/// and the next account takes its place, as the running accounts replay what
/// they know about it.
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
        Ok(()) => view.options_info(&format!("Removed {address}.")),
        Err(reason) => view.options_problem(&format!(
            "{reason}; the account is stopped for now and comes back the next time the program starts"
        )),
    }
}

/// Takes the account out of the running ones now, and gives the screen what
/// the running accounts say about the change.
fn let_go(view: &mut AppView, backend: &mut Box<dyn MailBackend>, members: &MembersHandle, id: &str, was_showing: bool) {
    members.remove(&AccountId(id.to_string()));
    if was_showing {
        view.clear_mail();
    }
    for event in backend.poll_events() {
        view.apply_event(event);
    }
}

#[cfg(test)]
include!("remove.tests.rs");
