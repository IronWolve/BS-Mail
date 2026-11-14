//! What the link page and a closed form ask for.

use ui::AppView;
use ui::setup::SetupAction;

use super::flow::SetupFlow;

const OPENED: &str = "Your browser should be opening. If nothing appears, press c to copy the link and open it yourself.";

impl SetupFlow {
    /// Opens the link, stops waiting, or gives the whole try up, as the form
    /// asked, then reports how the browser opener got on. Returns whether
    /// anything changed.
    pub(super) fn answer_actions(&mut self, view: &mut AppView) -> bool {
        let mut changed = false;
        for action in view.take_setup_actions() {
            changed = true;
            match action {
                SetupAction::OpenLink(url) => self.opener.open(&url),
                SetupAction::CancelSignIn | SetupAction::Abandon => self.abandon(),
            }
        }
        // The browser's answer can come after the user has given up; the form
        // they are back on has nothing to say about it.
        let waiting = self.waiting_for_browser();
        match self.opener.poll() {
            Some(Ok(())) if waiting => view.setup_info(OPENED),
            Some(Err(reason)) if waiting => view.setup_problem(&reason),
            _ => return changed,
        }
        true
    }
}
