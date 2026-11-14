//! What the program does for the account form: tries the account the user
//! typed, and saves the file only once the server has let it in.
//!
//! The form asks; this answers. A bad password therefore never reaches the
//! disk, and the user can fix it and try again without an old attempt's
//! worker running on: starting a new backend drops the previous one.
//!
//! For an account that signs in through a browser there is one step before
//! that: the link is shown, the program waits for the user to finish in the
//! browser, and only then is the account tried with the tokens that came
//! back. The browser is opened only when the user presses Open.

use std::path::{Path, PathBuf};

use engine::{AccountConfig, RealBackend};
use model::MailBackend;
use ui::AppView;
use ui::setup::SetupAction;

use crate::account_writer::save_account;
use crate::browser_login::{BrowserLogin, BrowserLoginStarter, BrowserPoll, EngineLogin};
use crate::link_opener::{LinkOpener, SystemOpener};

const OPENED: &str = "Your browser should be opening. If nothing appears, press c to copy the link and open it yourself.";

/// A browser sign-in the user is working through.
struct Waiting {
    login: Box<dyn BrowserLogin>,
    /// The account being signed in, with the place its tokens are kept.
    config: AccountConfig,
    /// The account file text, kept until the account has signed in.
    text: String,
}

pub struct SetupFlow {
    /// Where the account file goes.
    target: PathBuf,
    /// The text of the account being tried, kept until it signs in.
    pending: Option<String>,
    browser: Option<Waiting>,
    starter: Box<dyn BrowserLoginStarter>,
    opener: Box<dyn LinkOpener>,
}

impl SetupFlow {
    pub fn new(target: PathBuf) -> Self {
        Self::with_parts(target, Box::new(EngineLogin), Box::new(SystemOpener::new()))
    }

    /// A flow with its own browser sign-in and link opener, for tests.
    pub fn with_parts(target: PathBuf, starter: Box<dyn BrowserLoginStarter>, opener: Box<dyn LinkOpener>) -> Self {
        Self { target, pending: None, browser: None, starter, opener }
    }

    /// Takes whatever the form has to say and answers it. Returns whether
    /// anything changed on screen.
    pub fn step(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) -> bool {
        let mut changed = false;
        if let Some(text) = view.take_setup_request() {
            changed = true;
            self.try_account(text, view, backend);
        }
        changed |= self.answer_actions(view);
        changed |= self.watch_browser(view, backend);
        if view.take_setup_online() {
            changed = true;
            self.save(view);
        }
        changed
    }

    fn try_account(&mut self, text: String, view: &mut AppView, backend: &mut Box<dyn MailBackend>) {
        // A new try ends any sign-in still waiting on the browser.
        self.browser = None;
        let config = match AccountConfig::from_toml_str(&text) {
            Ok(config) => config,
            Err(error) => return view.setup_failed(&error.to_string()),
        };
        if config.oauth.is_none() {
            return self.start_backend(config, text, backend);
        }
        let config = config.with_state_dir(&state_dir(&self.target));
        match self.starter.start(&config) {
            Ok(login) => {
                view.setup_browser_link(login.url());
                self.browser = Some(Waiting { login, config, text });
            }
            Err(reason) => view.setup_failed(&reason),
        }
    }

    fn start_backend(&mut self, config: AccountConfig, text: String, backend: &mut Box<dyn MailBackend>) {
        // The old backend is dropped as the new one takes its place.
        *backend = Box::new(RealBackend::start(config));
        self.pending = Some(text);
    }

    /// What the link page asked for: open the link, or stop waiting.
    fn answer_actions(&mut self, view: &mut AppView) -> bool {
        let mut changed = false;
        for action in view.take_setup_actions() {
            changed = true;
            match action {
                SetupAction::OpenLink(url) => self.opener.open(&url),
                SetupAction::CancelSignIn => self.browser = None,
            }
        }
        match self.opener.poll() {
            Some(Ok(())) => view.setup_info(OPENED),
            Some(Err(reason)) => view.setup_problem(&reason),
            None => return changed,
        }
        true
    }

    /// Looks at whether the user has finished in the browser.
    fn watch_browser(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) -> bool {
        let Some(waiting) = self.browser.as_mut() else {
            return false;
        };
        match waiting.login.poll() {
            BrowserPoll::Waiting => false,
            BrowserPoll::Done => {
                if let Some(Waiting { config, text, .. }) = self.browser.take() {
                    view.setup_browser_done();
                    self.start_backend(config, text, backend);
                }
                true
            }
            BrowserPoll::Failed(reason) => {
                self.browser = None;
                view.setup_failed(&reason);
                true
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

/// The folder the account file is in, where the sign-in tokens are kept too.
fn state_dir(target: &Path) -> PathBuf {
    target.parent().filter(|dir| !dir.as_os_str().is_empty()).map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[cfg(test)]
include!("setup_flow.tests.rs");
