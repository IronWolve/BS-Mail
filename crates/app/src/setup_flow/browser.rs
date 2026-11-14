//! The browser step of signing in: show the link, wait for the user to finish
//! there, then go on to the mail server with the tokens that came back.

use ui::AppView;

use crate::accounts::Plan;
use crate::browser_login::BrowserPoll;

use super::attempt::{Attempt, Stage};
use super::flow::SetupFlow;

impl SetupFlow {
    /// Starts listening for the browser and shows the link. The tokens will
    /// be kept in the account's own folder.
    pub(super) fn start_browser(&mut self, plan: Plan, view: &mut AppView) {
        let config = plan.config.clone().with_state_dir(&plan.folder);
        match self.logins.start(&config) {
            Ok(login) => {
                view.setup_browser_link(login.url());
                self.attempt = Some(Attempt { plan, stage: Stage::Browser(login) });
            }
            Err(reason) => view.setup_failed(&reason),
        }
    }

    /// Looks at whether the user has finished in the browser.
    pub(super) fn watch_browser(&mut self, view: &mut AppView) -> bool {
        let Some(Attempt { stage: Stage::Browser(login), .. }) = self.attempt.as_mut() else {
            return false;
        };
        match login.poll() {
            BrowserPoll::Waiting => false,
            BrowserPoll::Done => {
                if let Some(Attempt { plan, .. }) = self.attempt.take() {
                    view.setup_browser_done();
                    self.start_trial(plan);
                }
                true
            }
            BrowserPoll::Failed(reason) => {
                self.attempt = None;
                view.setup_failed(&reason);
                true
            }
        }
    }
}
