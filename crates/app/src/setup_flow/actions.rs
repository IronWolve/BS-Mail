//! What the link page and a closed form ask for.

use ui::AppView;
use ui::setup::SetupAction;

use super::flow::SetupFlow;

const NO_BROWSER_FOR_REGISTRATION: &str =
    "no browser could be opened here; open your provider's app registration page yourself (docs/oauth-setup.md)";
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
                SetupAction::OpenLink(url) => {
                    self.registering = !self.waiting_for_browser();
                    self.opener.open(&url);
                }
                SetupAction::CancelSignIn | SetupAction::Abandon => self.abandon(),
            }
        }
        let Some(answer) = self.opener.poll() else {
            return changed;
        };
        // The browser's answer can come after the user has given up; the form
        // they are back on has nothing to say about it.
        let waiting = self.waiting_for_browser();
        let registering = std::mem::take(&mut self.registering);
        match (answer, waiting, registering) {
            (Ok(()), true, _) => view.setup_info(OPENED),
            (Err(reason), true, _) => view.setup_problem(&reason),
            (Err(_), false, true) => view.setup_problem(NO_BROWSER_FOR_REGISTRATION),
            _ => return changed,
        }
        true
    }
}
