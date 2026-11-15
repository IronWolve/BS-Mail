//! The flow's state, and starting a try.

use std::path::Path;

use engine::previous_token;
use model::MailBackend;
use ui::AppView;

use crate::accounts::{Live, Place, plan, remove_unsaved};
use crate::browser_login::{BrowserLoginStarter, EngineLogin};
use crate::link_opener::{LinkOpener, SystemOpener};

use super::attempt::Attempt;
use super::starter::{BackendStarter, EngineBackends};

pub struct SetupFlow {
    /// The accounts of this run: where a new one is saved and who it joins.
    pub(super) live: Live,
    /// The id of the account the form is changing; `None` for a new one.
    pub(super) edit: Option<String>,
    pub(super) attempt: Option<Attempt>,
    /// The id of an account folder that this flow's tries made. If nothing is
    /// saved in it, giving up removes it, and nothing else.
    pub(super) created: Option<String>,
    /// The token file a browser sign-in of this flow may have replaced. A new
    /// sign-in sets the earlier tokens aside before it writes (see
    /// `engine::previous_token`); giving up, a refused account or a failed
    /// save puts them back, and saving the account lets them go.
    pub(super) token: Option<std::path::PathBuf>,
    pub(super) logins: Box<dyn BrowserLoginStarter>,
    pub(super) opener: Box<dyn LinkOpener>,
    /// The page opened is the provider's registration page, not a sign-in
    /// link, so a failure to open it has its own words.
    pub(super) registering: bool,
    pub(super) backends: Box<dyn BackendStarter>,
}

impl SetupFlow {
    /// A flow for a form that adds an account, or changes the one `edit`
    /// names.
    pub fn new(live: Live, edit: Option<String>) -> Self {
        Self::with_parts(live, edit, Box::new(EngineLogin), Box::new(SystemOpener::new()), Box::new(EngineBackends))
    }

    /// A flow with its own browser sign-in, link opener and backend starter,
    /// for tests.
    pub fn with_parts(
        live: Live,
        edit: Option<String>,
        logins: Box<dyn BrowserLoginStarter>,
        opener: Box<dyn LinkOpener>,
        backends: Box<dyn BackendStarter>,
    ) -> Self {
        Self { live, edit, attempt: None, created: None, token: None, logins, opener, registering: false, backends }
    }

    /// Takes whatever the form has to say and answers it. Returns whether
    /// anything changed on screen. `backend` is the screen's own, which the
    /// account joins once it has signed in.
    pub fn step(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) -> bool {
        let mut changed = false;
        if let Some(text) = view.take_setup_request() {
            changed = true;
            self.try_account(&text, view);
        }
        changed |= self.answer_actions(view);
        changed |= self.watch_browser(view);
        changed |= self.watch_trial(view, backend);
        changed
    }

    /// Gives up: stops whatever is being tried, puts back the sign-in tokens
    /// an earlier account had, and removes the account folder this flow made,
    /// if nothing was saved in it.
    pub fn abandon(&mut self) {
        self.attempt = None;
        self.restore_token();
        self.remove_created();
    }

    /// Puts the account's earlier sign-in back (or removes the one a try made
    /// for an account that was never saved), if a try replaced it. The
    /// attempt, and with it the sign-in's own thread, is gone before this.
    pub(super) fn restore_token(&mut self) {
        if let Some(file) = self.token.take() {
            let _ = previous_token::restore(&file);
        }
    }

    fn try_account(&mut self, text: &str, view: &mut AppView) {
        // A new try ends any attempt still going, and starts from what was
        // there before the earlier one.
        self.attempt = None;
        self.restore_token();
        let planned = plan(&self.live.place, self.edit.as_deref(), &self.live.ids_in_use(), text);
        let plan = match planned {
            Ok(plan) => plan,
            Err(reason) => return view.setup_failed(&reason),
        };
        self.note_folder(&plan.id, plan.new_folder);
        if plan.config.oauth.is_some() {
            self.start_browser(plan, view);
        } else {
            self.start_trial(plan);
        }
    }

    /// Remembers that this flow is the one that makes the folder of `id`, and
    /// lets go of a folder an earlier try made for another id.
    fn note_folder(&mut self, id: &str, new_folder: bool) {
        if let Some(root) = self.root()
            && let Some(old) = self.created.take_if(|old| old != id)
        {
            let _ = remove_unsaved(&root, &old);
        }
        if self.created.is_none() && new_folder && self.root().is_some() {
            self.created = Some(id.to_string());
        }
    }

    /// Removes the folder this flow made, only when no account file is in it.
    pub(super) fn remove_created(&mut self) {
        if let (Some(root), Some(id)) = (self.root(), self.created.take()) {
            let _ = remove_unsaved(&root, &id);
        }
    }

    /// The accounts folder, when accounts are kept in one folder each.
    fn root(&self) -> Option<std::path::PathBuf> {
        match &self.live.place {
            Place::Root(root) => Some(Path::to_path_buf(root)),
            Place::File(_) => None,
        }
    }
}

impl Drop for SetupFlow {
    /// A form that goes away (the window closed, the program quitting) gives
    /// the try up the same way Escape does.
    fn drop(&mut self) {
        self.abandon();
    }
}

#[cfg(test)]
include!("flow.tests.rs");
