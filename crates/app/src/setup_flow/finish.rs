//! A signed-in account: save it, join it to the running ones, show it.

use engine::{Member, previous_token};
use model::{AccountId, Command, MailBackend};
use ui::AppView;

use super::attempt::{Attempt, Stage};
use super::flow::SetupFlow;
use crate::accounts::save;

impl SetupFlow {
    /// The account being tried has signed in. Saves its file; then hands the
    /// connected backend to the running accounts with what it said so far,
    /// closes the form, refreshes the account list and shows the account.
    /// A file that cannot be saved leaves the form open with the reason and
    /// the account is let go.
    pub(super) fn complete(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) {
        let Some(Attempt { plan, stage: Stage::Trial { backend: trial, kept } }) = self.attempt.take() else {
            return;
        };
        if let Err(reason) = save(&plan) {
            self.restore_token();
            return view.setup_failed(&reason);
        }
        // The account file is saved: the new sign-in is the account's own.
        if let Some(file) = self.token.take() {
            previous_token::discard(&file);
        }
        self.created = None;
        self.remember_client(&plan, view);
        let id = AccountId(plan.id.clone());
        let member = Member::new(id.clone(), plan.config.name.clone(), plan.config.address.clone(), trial);
        if plan.edit {
            self.live.members.replace(member, kept);
        } else {
            self.live.members.add(member, kept);
        }
        view.finish_setup();
        view.set_accounts(self.live.summaries());
        show(view, backend, id);
        if view.is_in_options() {
            let what = if plan.edit { "Saved" } else { "Added" };
            view.options_info(&format!("{what} {}.", plan.config.address));
        }
    }
}

/// Makes `account` the one whose mail shows, with nothing left over from what
/// was showing: the screen forgets its mail and asks the running accounts to
/// make this one the active account, which they answer from what they know.
pub(super) fn show(view: &mut AppView, backend: &mut Box<dyn MailBackend>, account: AccountId) {
    view.clear_mail();
    for event in backend.handle(Command::SelectAccount { account }) {
        view.apply_event(event);
    }
}
