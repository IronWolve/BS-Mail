//! The browser step of signing in: show the link, wait for the user to finish
//! there, then go on to the mail server with the tokens that came back.

use ui::AppView;

use crate::accounts::Plan;
use crate::browser_login::BrowserPoll;

use super::attempt::{Attempt, Stage};
use super::flow::SetupFlow;

impl SetupFlow {
    /// Starts listening for the browser, shows the link and opens it in the
    /// user's browser at once; the page keeps Open and Copy for when nothing
    /// appears. The tokens will be kept in the vault.
    pub(super) fn start_browser(&mut self, plan: Plan, view: &mut AppView) {
        let config = plan.trial_config();
        match self.logins.start(&config) {
            Ok(login) => {
                self.token = plan.token_place();
                let url = login.url().to_string();
                view.setup_browser_link(&url);
                self.opener.open(&url);
                self.attempt = Some(Attempt { plan, stage: Stage::Browser(login) });
            }
            Err(reason) => view.setup_failed(&reason),
        }
    }

    /// Whether the link page is up and the program is waiting for the user to
    /// finish in the browser.
    pub(super) fn waiting_for_browser(&self) -> bool {
        matches!(&self.attempt, Some(Attempt { stage: Stage::Browser(_), .. }))
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
