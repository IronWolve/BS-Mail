//! What the program does for the account form: tries the account the user
//! typed, and saves the file only once the server has let it in.
//!
//! The form asks; this answers. A bad password therefore never reaches the
//! disk, and the user can fix it and try again without an old attempt's
//! worker running on: starting a new backend drops the previous one.

use std::path::PathBuf;

use engine::{AccountConfig, RealBackend};
use model::MailBackend;
use ui::AppView;

use crate::account_writer::save_account;

pub struct SetupFlow {
    /// Where the account file goes.
    target: PathBuf,
    /// The text of the account being tried, kept until it signs in.
    pending: Option<String>,
}

impl SetupFlow {
    pub fn new(target: PathBuf) -> Self {
        Self { target, pending: None }
    }

    /// Takes whatever the form has to say and answers it. Returns whether
    /// anything changed on screen.
    pub fn step(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) -> bool {
        let mut changed = false;
        if let Some(text) = view.take_setup_request() {
            changed = true;
            self.try_account(text, view, backend);
        }
        if view.take_setup_online() {
            changed = true;
            self.save(view);
        }
        changed
    }

    fn try_account(&mut self, text: String, view: &mut AppView, backend: &mut Box<dyn MailBackend>) {
        match AccountConfig::from_toml_str(&text) {
            Err(error) => view.setup_failed(&error.to_string()),
            Ok(config) => {
                // The old backend is dropped as the new one takes its place.
                *backend = Box::new(RealBackend::start(config));
                self.pending = Some(text);
            }
        }
    }

    fn save(&mut self, view: &mut AppView) {
        let Some(text) = self.pending.take() else {
            return;
        };
        match save_account(&self.target, &text) {
            Ok(_) => view.finish_setup(),
            Err(reason) => {
                self.pending = Some(text);
                view.setup_failed(&reason);
            }
        }
    }
}

#[cfg(test)]
include!("setup_flow.tests.rs");
