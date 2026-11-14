//! The account form as part of the screen: opening it, its input, and how
//! the program and the form talk to each other.
//!
//! The form never touches a file or a server. When the user asks to connect
//! it checks what was typed and leaves the account file's text for the
//! program to pick up; the program tries the account and answers through
//! `setup_failed` or `finish_setup`. While the form is open, no mail is
//! drawn and no key reaches the screen beneath.

use model::{ConnectionState, Event};
use surface::{ButtonState, InputEvent, PointerButton};

use crate::app_view::AppView;
use crate::setup::{AccountPrefill, SetupForm};
use crate::setup::input::{Outcome, click, handle_key, handle_text};
use crate::setup::layout::{area_of, layout};
use crate::setup::link_panel::PanelChoice;
use crate::setup::link_panel_input;
use crate::setup::SetupAction;

impl AppView {
    /// Shows the account form instead of the mail, with `note` under the
    /// title when there is something to say about where it will be saved.
    pub fn open_setup(&mut self) {
        self.setup = Some(SetupForm::new());
        self.setup_request = None;
        self.setup_online = false;
        self.setup_actions.clear();
    }

    /// Shows the account form filled with an existing account's details, for
    /// the user to change.
    pub fn open_setup_with(&mut self, prefill: AccountPrefill) {
        self.setup = Some(SetupForm::from_prefill(prefill));
        self.setup_request = None;
        self.setup_online = false;
        self.setup_actions.clear();
    }

    /// Whether the account form is open.
    pub fn is_setting_up(&self) -> bool {
        self.setup.is_some()
    }

    /// Whether letters go into a text box, so they are never read as the
    /// keys that close the program.
    pub fn takes_text(&self) -> bool {
        self.composer.is_some() || self.setup.is_some() || self.search_bar.is_some()
    }

    /// The account file text the user asked to try, once.
    pub fn take_setup_request(&mut self) -> Option<String> {
        self.setup_request.take()
    }

    /// Whether the account that was tried has signed in, once. The program
    /// then saves the file and calls `finish_setup`.
    pub fn take_setup_online(&mut self) -> bool {
        std::mem::take(&mut self.setup_online)
    }

    /// What the form asks the program to do for it (open the link, copy it,
    /// stop waiting), once each.
    pub fn take_setup_actions(&mut self) -> Vec<SetupAction> {
        std::mem::take(&mut self.setup_actions)
    }

    /// The program is waiting for the user to sign in through a browser: the
    /// link page shows `url` until the sign-in ends one way or the other.
    pub fn setup_browser_link(&mut self, url: &str) {
        if let Some(form) = self.setup.as_mut() {
            form.waiting = true;
            form.notice = None;
            form.info = None;
            form.link = Some(url.to_string());
        }
    }

    /// The browser step is over and the program goes on to the mail server.
    pub fn setup_browser_done(&mut self) {
        if let Some(form) = self.setup.as_mut() {
            form.link = None;
            form.info = Some("Signed in. Connecting to the mail server\u{2026}".to_string());
        }
    }

    /// Something to tell the user on the form or the link page that is not a
    /// problem, such as that the link was copied.
    pub fn setup_info(&mut self, text: &str) {
        if let Some(form) = self.setup.as_mut() {
            form.notice = None;
            form.info = Some(text.to_string());
        }
    }

    /// Something went wrong on the form or the link page that does not end
    /// the wait, such as no browser being found to open the link with.
    pub fn setup_problem(&mut self, text: &str) {
        if let Some(form) = self.setup.as_mut() {
            form.info = None;
            form.notice = Some(text.to_string());
        }
    }

    /// The account was saved and is working: the form closes and the mail
    /// that has arrived shows.
    pub fn finish_setup(&mut self) {
        self.setup = None;
    }

    /// The account could not be tried or saved; the form shows why and the
    /// user can change something and try again.
    pub fn setup_failed(&mut self, reason: &str) {
        if let Some(form) = self.setup.as_mut() {
            form.waiting = false;
            form.link = None;
            form.info = None;
            form.notice = Some(reason.to_string());
        }
    }

    /// Input while the form is open. Returns whether the screen changed.
    pub(crate) fn setup_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let Some(form) = self.setup.as_mut() else {
            return false;
        };
        let outcome = match event {
            InputEvent::Key(key) => handle_key(form, key),
            InputEvent::Text(text) | InputEvent::Paste(text) => handle_text(form, text),
            InputEvent::Button {
                button: PointerButton::Left,
                state: ButtonState::Pressed,
                pos,
                ..
            } => {
                if form.link.is_some() {
                    link_panel_input::click(grid, pos.col, pos.row)
                } else {
                    let area = area_of(grid);
                    match layout(area, form.signin).and_then(|rects| rects.field_at(pos.col, pos.row)) {
                        Some(field) => click(form, field),
                        None => Outcome::Ignored,
                    }
                }
            }
            _ => Outcome::Ignored,
        };
        match outcome {
            Outcome::Ignored => false,
            Outcome::Changed => true,
            Outcome::Close => {
                // A closed form is never saved: forget what was handed over
                // and tell the program to stop any attempt still going.
                self.setup = None;
                self.setup_request = None;
                self.setup_online = false;
                self.setup_actions.push(SetupAction::Abandon);
                true
            }
            Outcome::Submit => {
                self.submit_setup();
                true
            }
            Outcome::Panel(choice) => {
                self.setup_panel(choice);
                true
            }
        }
    }

    /// A button on the sign-in link page.
    fn setup_panel(&mut self, choice: PanelChoice) {
        let Some(form) = self.setup.as_mut() else {
            return;
        };
        let Some(link) = form.link.clone() else {
            return;
        };
        match choice {
            PanelChoice::Open => {
                form.info = Some("Opening your browser\u{2026}".to_string());
                self.setup_actions.push(SetupAction::OpenLink(link));
            }
            PanelChoice::Copy => {
                form.info = Some("The link is on the clipboard (if your terminal allows it).".to_string());
                form.copied = Some(link);
            }
            PanelChoice::Cancel => {
                form.link = None;
                form.waiting = false;
                form.info = None;
                self.setup_actions.push(SetupAction::CancelSignIn);
            }
        }
    }

    /// Checks what was typed and hands the account text over, or says what
    /// is wrong with it.
    fn submit_setup(&mut self) {
        let Some(form) = self.setup.as_mut() else {
            return;
        };
        match form.to_account_toml() {
            Err(problem) => form.notice = Some(problem),
            Ok(text) => {
                form.notice = None;
                form.waiting = true;
                self.setup_request = Some(text);
            }
        }
    }

    /// Reads what the account that was tried says: signed in, or why not.
    /// The program feeds the form the events of the account it is trying,
    /// and only those: what the accounts already running say is not about
    /// the form, so `apply_event` does not pass it on.
    pub fn note_setup_event(&mut self, event: &Event) {
        let Some(form) = self.setup.as_mut() else {
            return;
        };
        match event {
            Event::ConnectionState {
                state: ConnectionState::Online,
                ..
            } => self.setup_online = true,
            Event::OpFailed { reason, .. } if form.waiting => {
                form.waiting = false;
                form.notice = Some(reason.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
include!("setup.tests.rs");
