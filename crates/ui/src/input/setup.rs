//! The account form as part of the screen: opening it, its input, and how
//! the program and the form talk to each other.
//!
//! The form never touches a file or a server. When the user asks to connect
//! it checks what was typed and leaves the account file's text for the
//! program to pick up; the program tries the account and answers through
//! `setup_failed` or `finish_setup`. While the form is open, no mail is
//! drawn and no key reaches the screen beneath.

use model::{ConnectionState, Event};
use surface::{ButtonState, InputEvent, PointerButton, Rect};

use crate::app_view::AppView;
use crate::setup::draw::layout;
use crate::setup::input::{Outcome, click, handle_key, handle_text};
use crate::setup::SetupForm;

impl AppView {
    /// Shows the account form instead of the mail, with `note` under the
    /// title when there is something to say about where it will be saved.
    pub fn open_setup(&mut self) {
        self.setup = Some(SetupForm::new());
        self.setup_request = None;
        self.setup_online = false;
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
                let area = Rect::new(0, 0, grid.0, grid.1);
                match layout(area).and_then(|rects| rects.field_at(pos.col, pos.row)) {
                    Some(field) => click(form, field),
                    None => Outcome::Ignored,
                }
            }
            _ => Outcome::Ignored,
        };
        match outcome {
            Outcome::Ignored => false,
            Outcome::Changed => true,
            Outcome::Quit => {
                self.quit_requested = true;
                true
            }
            Outcome::Submit => {
                self.submit_setup();
                true
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
    pub(crate) fn note_setup_event(&mut self, event: &Event) {
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
