//! After a browser sign-in has worked, the client id it used is kept for the
//! next account of the same provider, so the form asks for it once.

use ui::AppView;

use crate::accounts::Plan;

use super::flow::SetupFlow;

impl SetupFlow {
    /// Saves the client id of a saved account that signs in through a browser,
    /// when its provider has none yet, and tells the form. Best effort: an
    /// account that is already saved and working is never held up by this, and
    /// the worst that happens is that the next account asks again.
    pub(super) fn remember_client(&self, plan: &Plan, view: &mut AppView) {
        let Ok(table) = plan.text.parse::<toml::Table>() else {
            return;
        };
        let text = |key: &str| table.get(key).and_then(toml::Value::as_str);
        if text("auth") != Some("oauth2") {
            return;
        }
        let (Some(provider), Some(client_id)) = (text("oauth_provider"), text("oauth_client_id")) else {
            return;
        };
        let store = self.live.clients();
        if let Ok(true) = store.remember(provider, client_id, text("oauth_client_secret")) {
            view.set_saved_clients(store.load());
        }
    }
}

#[cfg(test)]
include!("remember.tests.rs");
