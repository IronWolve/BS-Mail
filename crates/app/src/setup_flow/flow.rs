//! The flow's state, and starting a try.

use std::path::Path;

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
    pub(super) logins: Box<dyn BrowserLoginStarter>,
    pub(super) opener: Box<dyn LinkOpener>,
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
        Self { live, edit, attempt: None, created: None, logins, opener, backends }
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

    /// Gives up: stops whatever is being tried and removes the account folder
    /// this flow made, if nothing was saved in it.
    pub fn abandon(&mut self) {
        self.attempt = None;
        self.remove_created();
    }

    fn try_account(&mut self, text: &str, view: &mut AppView) {
        // A new try ends any attempt still going.
        self.attempt = None;
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

#[cfg(test)]
include!("flow.tests.rs");
